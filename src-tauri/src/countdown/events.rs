use crate::countdown::dto::CountdownSnapshotDto;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct CountdownTickPayload {
    pub id: u64,
    pub label: String,
    pub remaining_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum AppEvent {
    Tick(CountdownTickPayload),
    Changed(Vec<CountdownSnapshotDto>),
    /// A group or overlay-config change: connected overlays should reload to
    /// pick up the freshly server-rendered page.
    Reload,
}

/// Final overlay updates for countdowns that just reached zero.
///
/// Reaching zero is visually only "timer → 0", which a [`AppEvent::Tick`] with
/// `remaining_ms: 0` conveys in place. Emitting [`AppEvent::Changed`] here
/// instead would make every connected OBS browser source reload the whole page
/// — a visible flash on stream each time a timer finishes.
pub fn finished_tick_events(
    newly_finished: &[u64],
    snapshots: &[CountdownSnapshotDto],
) -> Vec<AppEvent> {
    newly_finished
        .iter()
        .map(|id| {
            let label = snapshots
                .iter()
                .find(|s| s.id == *id)
                .map(|s| s.label.clone())
                .unwrap_or_default();
            AppEvent::Tick(CountdownTickPayload {
                id: *id,
                label,
                remaining_ms: 0,
            })
        })
        .collect()
}
