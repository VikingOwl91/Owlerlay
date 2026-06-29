//! A finished countdown must update overlays in place, not reload them.
//! Regression guard for the OBS "flash on finish" bug: `finished_events` must
//! emit `Tick { remaining_ms: 0 }` (which the SSE route turns into a
//! `countdown-tick`) plus a `State(Finished)` (so SSE-only clients like the
//! phone remote learn the timer ended), never `Changed` (a full-page `reload`).

use owlerlay_lib::countdown::dto::CountdownSnapshotDto;
use owlerlay_lib::countdown::events::{AppEvent, finished_events, state_change_events};
use owlerlay_lib::countdown::model::CountdownState;

fn snap(id: u64, label: &str, state: CountdownState, duration: u128) -> CountdownSnapshotDto {
    CountdownSnapshotDto {
        id,
        label: label.into(),
        duration,
        initial_duration: 60_000,
        state,
        start_epoch_ms: None,
        target_epoch_ms: None,
    }
}

fn finished(id: u64, label: &str) -> CountdownSnapshotDto {
    snap(id, label, CountdownState::Finished, 0)
}

#[test]
fn finish_emits_zero_tick_plus_finished_state_not_reload() {
    let snaps = vec![finished(1, "Intro"), finished(2, "BRB")];
    let events = finished_events(&[1, 2], &snaps);

    // Each finished id yields exactly a zero Tick + a Finished State, never a
    // Changed/Reload (which would flash the OBS source).
    assert_eq!(events.len(), 4);
    for event in &events {
        match event {
            AppEvent::Tick(p) => assert_eq!(p.remaining_ms, 0, "finished tick must land on zero"),
            AppEvent::State(p) => {
                assert_eq!(p.state, CountdownState::Finished, "state must be Finished")
            }
            other => panic!("expected Tick/State (in-place), got {other:?} (would reload/flash)"),
        }
    }
    // First id: zero tick carrying the label, then its Finished state.
    match (&events[0], &events[1]) {
        (AppEvent::Tick(t), AppEvent::State(s)) => {
            assert_eq!(t.id, 1);
            assert_eq!(t.label, "Intro");
            assert_eq!(s.id, 1);
        }
        other => panic!("expected Tick then State for id 1, got {other:?}"),
    }
}

#[test]
fn unknown_finished_id_falls_back_to_empty_label() {
    // A finished id with no matching snapshot still yields a zero tick + state.
    let events = finished_events(&[9], &[]);
    match events.as_slice() {
        [AppEvent::Tick(t), AppEvent::State(s)] => {
            assert_eq!(t.id, 9);
            assert_eq!(t.remaining_ms, 0);
            assert_eq!(t.label, "");
            assert_eq!(s.id, 9);
            assert_eq!(s.state, CountdownState::Finished);
        }
        _ => panic!("expected a zero tick + Finished state"),
    }
}

#[test]
fn start_emits_only_a_state_event() {
    // Starting (-> Running) just un-hides via countdown-state; the ticker drives
    // the value, so no tick and definitely no reload.
    let events = state_change_events(&snap(1, "Intro", CountdownState::Running, 60_000));
    match events.as_slice() {
        [AppEvent::State(p)] => {
            assert_eq!(p.id, 1);
            assert_eq!(p.state, CountdownState::Running);
        }
        other => panic!("expected one State event, got {other:?}"),
    }
}

#[test]
fn reset_emits_state_plus_restored_value_tick() {
    // Resetting (-> Idle) won't tick on its own, so it must also push a tick
    // with the restored value (here the full 60s), never a reload.
    let events = state_change_events(&snap(1, "Intro", CountdownState::Idle, 60_000));
    match events.as_slice() {
        [AppEvent::State(s), AppEvent::Tick(t)] => {
            assert_eq!(s.state, CountdownState::Idle);
            assert_eq!(t.id, 1);
            assert_eq!(t.remaining_ms, 60_000, "tick must carry the restored value");
        }
        other => panic!("expected State + restored-value Tick, got {other:?}"),
    }
}
