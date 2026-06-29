//! Restore round-trip: persisted DTOs must rebuild into the same live state,
//! with running timers re-anchored against the boot wall-clock (and any that
//! expired during downtime coming back Finished), and `next_id` derived as
//! `max(id)+1` so a freshly created countdown can't collide with a restored one.

use owlerlay_lib::countdown::dto::CountdownSnapshotDto;
use owlerlay_lib::countdown::model::CountdownState;
use owlerlay_lib::countdown::service::{CountdownService, MAX_COUNTDOWNS};
use tokio::time::Duration;

const NOW_EPOCH_MS: u128 = 1_700_000_000_000;
const INITIAL_MS: u128 = 60_000;

fn dto(
    id: u64,
    state: CountdownState,
    duration: u128,
    target_epoch_ms: Option<u128>,
) -> CountdownSnapshotDto {
    CountdownSnapshotDto {
        id,
        label: format!("c{id}"),
        duration,
        initial_duration: INITIAL_MS,
        state,
        start_epoch_ms: None,
        target_epoch_ms,
    }
}

#[tokio::test]
async fn restore_round_trips_every_state() {
    let dtos = vec![
        dto(10, CountdownState::Idle, INITIAL_MS, None),
        // Running, 50s left → stays Running.
        dto(
            11,
            CountdownState::Running,
            50_000,
            Some(NOW_EPOCH_MS + 50_000),
        ),
        // Running, target already elapsed during downtime → Finished.
        dto(
            12,
            CountdownState::Running,
            5_000,
            Some(NOW_EPOCH_MS - 1_000),
        ),
        dto(13, CountdownState::Paused, 7_000, None),
        dto(14, CountdownState::Finished, 0, None),
    ];

    let service = CountdownService::from_dtos(dtos, NOW_EPOCH_MS);
    let snaps = service.list_countdown().await.expect("list");
    let by_id = |id: u64| snaps.iter().find(|s| s.id == id).expect("snapshot present");

    // Idle keeps its configured length.
    assert_eq!(by_id(10).state, CountdownState::Idle);
    assert_eq!(by_id(10).duration, Duration::from_millis(INITIAL_MS as u64));

    // Running re-anchored: ~50s left (allow a small window for the elapsed
    // microseconds between from_dtos and list).
    assert_eq!(by_id(11).state, CountdownState::Running);
    let left = by_id(11).duration;
    assert!(
        left <= Duration::from_secs(50) && left >= Duration::from_millis(49_000),
        "running remaining out of range: {left:?}"
    );

    // Expired-during-downtime Running came back Finished.
    assert_eq!(by_id(12).state, CountdownState::Finished);
    assert_eq!(by_id(12).duration, Duration::ZERO);

    // Paused keeps its stored remaining exactly.
    assert_eq!(by_id(13).state, CountdownState::Paused);
    assert_eq!(by_id(13).duration, Duration::from_secs(7));

    assert_eq!(by_id(14).state, CountdownState::Finished);
}

#[tokio::test]
async fn next_id_avoids_collision_with_restored_ids() {
    let dtos = vec![dto(3, CountdownState::Idle, INITIAL_MS, None)];
    let service = CountdownService::from_dtos(dtos, NOW_EPOCH_MS);

    let new_id = service
        .create_countdown("fresh".into(), Duration::from_secs(10))
        .await
        .expect("create");
    assert_eq!(new_id, 4, "next_id must be max(restored id)+1");
}

#[tokio::test]
async fn running_remaining_is_clamped_to_initial() {
    // A backward clock change (or a hand-edited far-future target) makes the
    // raw target-vs-now delta exceed the configured length; restore must clamp
    // it so the overlay can't show more time left than the countdown started
    // with — and must never panic on the overflowing instant.
    let dtos = vec![dto(
        1,
        CountdownState::Running,
        INITIAL_MS,
        Some(NOW_EPOCH_MS + 10 * INITIAL_MS), // 10x the configured length out
    )];
    let service = CountdownService::from_dtos(dtos, NOW_EPOCH_MS);
    let snaps = service.list_countdown().await.expect("list");
    let s = &snaps[0];
    assert_eq!(s.state, CountdownState::Running);
    assert!(
        s.duration <= Duration::from_millis(INITIAL_MS as u64),
        "remaining must be clamped to initial, got {:?}",
        s.duration
    );
}

#[tokio::test]
async fn next_id_does_not_overflow_on_max_id() {
    // A corrupt/hand-edited store with id == u64::MAX must not panic on restore.
    let dtos = vec![dto(u64::MAX, CountdownState::Idle, INITIAL_MS, None)];
    let service = CountdownService::from_dtos(dtos, NOW_EPOCH_MS);
    assert_eq!(service.list_countdown().await.expect("list").len(), 1);
}

#[tokio::test]
async fn restore_caps_at_max_countdowns() {
    // A corrupt/hand-edited store with more than the create limit must not load
    // more timers than the app expects.
    let dtos: Vec<_> = (0..MAX_COUNTDOWNS as u64 + 5)
        .map(|id| dto(id, CountdownState::Idle, INITIAL_MS, None))
        .collect();
    let service = CountdownService::from_dtos(dtos, NOW_EPOCH_MS);
    assert_eq!(
        service.list_countdown().await.expect("list").len(),
        MAX_COUNTDOWNS
    );
}

#[tokio::test]
async fn restore_dedups_duplicate_ids() {
    // Two entries sharing an id must collapse to one deterministically (first
    // wins), never silently lose a timer in HashMap order.
    let mut a = dto(7, CountdownState::Idle, INITIAL_MS, None);
    a.label = "first".into();
    let mut b = dto(7, CountdownState::Paused, 7_000, None);
    b.label = "second".into();
    let service = CountdownService::from_dtos(vec![a, b], NOW_EPOCH_MS);
    let snaps = service.list_countdown().await.expect("list");
    assert_eq!(snaps.len(), 1);
    assert_eq!(snaps[0].label, "first");
    assert_eq!(snaps[0].state, CountdownState::Idle);
}

#[tokio::test]
async fn restore_clamps_paused_to_initial() {
    // A corrupt Paused entry holding more than its configured length must be
    // clamped, same invariant the Running arm enforces.
    let dtos = vec![dto(1, CountdownState::Paused, INITIAL_MS * 5, None)];
    let service = CountdownService::from_dtos(dtos, NOW_EPOCH_MS);
    let s = &service.list_countdown().await.expect("list")[0];
    assert_eq!(s.state, CountdownState::Paused);
    assert!(
        s.duration <= Duration::from_millis(INITIAL_MS as u64),
        "paused remaining must be clamped to initial, got {:?}",
        s.duration
    );
}

#[tokio::test]
async fn restore_drops_empty_label_entries() {
    // create_countdown rejects empty labels; restore must not resurrect one.
    let mut blank = dto(1, CountdownState::Idle, INITIAL_MS, None);
    blank.label = String::new();
    let good = dto(2, CountdownState::Idle, INITIAL_MS, None);
    let service = CountdownService::from_dtos(vec![blank, good], NOW_EPOCH_MS);
    let snaps = service.list_countdown().await.expect("list");
    assert_eq!(snaps.len(), 1);
    assert_eq!(snaps[0].id, 2);
}

#[tokio::test]
async fn running_without_target_falls_back_to_stored_remaining() {
    // A Running entry that lost its target_epoch_ms must resume from the stored
    // remaining, not silently boot Finished.
    let dtos = vec![dto(1, CountdownState::Running, 30_000, None)];
    let service = CountdownService::from_dtos(dtos, NOW_EPOCH_MS);
    let s = &service.list_countdown().await.expect("list")[0];
    assert_eq!(s.state, CountdownState::Running);
    assert!(
        s.duration > Duration::from_secs(28) && s.duration <= Duration::from_secs(30),
        "expected ~30s remaining, got {:?}",
        s.duration
    );
}

#[tokio::test]
async fn empty_store_restores_empty_service() {
    let service = CountdownService::from_dtos(Vec::new(), NOW_EPOCH_MS);
    assert!(service.list_countdown().await.expect("list").is_empty());

    let new_id = service
        .create_countdown("first".into(), Duration::from_secs(10))
        .await
        .expect("create");
    assert_eq!(new_id, 0, "empty store starts ids at 0");
}
