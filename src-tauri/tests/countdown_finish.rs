//! A finished countdown must update overlays in place, not reload them.
//! Regression guard for the OBS "flash on finish" bug: `finished_tick_events`
//! must emit `Tick { remaining_ms: 0 }` (which the SSE route turns into a
//! `countdown-tick`), never `Changed` (which turns into a full-page `reload`).

use owlerlay_lib::countdown::dto::CountdownSnapshotDto;
use owlerlay_lib::countdown::events::{AppEvent, finished_tick_events};
use owlerlay_lib::countdown::model::CountdownState;

fn finished(id: u64, label: &str) -> CountdownSnapshotDto {
    CountdownSnapshotDto {
        id,
        label: label.into(),
        duration: 0,
        initial_duration: 60_000,
        state: CountdownState::Finished,
        start_epoch_ms: None,
        target_epoch_ms: None,
    }
}

#[test]
fn finish_emits_zero_ticks_not_reload() {
    let snaps = vec![finished(1, "Intro"), finished(2, "BRB")];
    let events = finished_tick_events(&[1, 2], &snaps);

    assert_eq!(events.len(), 2);
    for event in &events {
        match event {
            AppEvent::Tick(p) => {
                assert_eq!(p.remaining_ms, 0, "finished tick must land on zero");
            }
            other => panic!("expected Tick (in-place update), got {other:?} (would reload/flash)"),
        }
    }
    // Labels are carried through for the desktop event payload.
    match &events[0] {
        AppEvent::Tick(p) => {
            assert_eq!(p.id, 1);
            assert_eq!(p.label, "Intro");
        }
        _ => unreachable!(),
    }
}

#[test]
fn unknown_finished_id_falls_back_to_empty_label() {
    // A finished id with no matching snapshot still yields a zero tick.
    let events = finished_tick_events(&[9], &[]);
    match events.as_slice() {
        [AppEvent::Tick(p)] => {
            assert_eq!(p.id, 9);
            assert_eq!(p.remaining_ms, 0);
            assert_eq!(p.label, "");
        }
        _ => panic!("expected a single zero tick"),
    }
}
