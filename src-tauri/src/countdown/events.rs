use crate::countdown::dto::CountdownSnapshotDto;
use crate::countdown::model::CountdownState;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct CountdownTickPayload {
    pub id: u64,
    pub label: String,
    pub remaining_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CountdownStatePayload {
    pub id: u64,
    pub state: CountdownState,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum AppEvent {
    Tick(CountdownTickPayload),
    /// A countdown's run-state changed (start/pause/resume/reset/finish) without
    /// changing page structure: overlays patch visibility in place instead of
    /// reloading, and SSE-only clients (the phone remote) learn the new state.
    State(CountdownStatePayload),
    Changed(Vec<CountdownSnapshotDto>),
    /// A group or overlay-config change: connected overlays should reload to
    /// pick up the freshly server-rendered page.
    Reload,
}

/// Final updates for countdowns that just reached zero.
///
/// For OBS overlays, reaching zero is visually only "timer → 0", which a
/// [`AppEvent::Tick`] with `remaining_ms: 0` conveys in place (emitting
/// [`AppEvent::Changed`] would reload the whole page and flash on stream). The
/// accompanying [`AppEvent::State`] is what tells SSE-only clients like the
/// phone remote that the run-state is now `Finished` — overlays ignore it for
/// any non-idle state, so it costs them nothing.
pub fn finished_events(
    newly_finished: &[u64],
    snapshots: &[CountdownSnapshotDto],
) -> Vec<AppEvent> {
    newly_finished
        .iter()
        .flat_map(|id| {
            let label = snapshots
                .iter()
                .find(|s| s.id == *id)
                .map(|s| s.label.clone())
                .unwrap_or_default();
            [
                AppEvent::Tick(CountdownTickPayload {
                    id: *id,
                    label,
                    remaining_ms: 0,
                }),
                AppEvent::State(CountdownStatePayload {
                    id: *id,
                    state: CountdownState::Finished,
                }),
            ]
        })
        .collect()
}

/// Overlay updates for a countdown whose run-state changed without a structural
/// change (start, reset). Always a [`AppEvent::State`] so overlays patch
/// `hide_idle` visibility in place. A reset back to [`CountdownState::Idle`]
/// also needs a [`AppEvent::Tick`] with the restored value, because an idle
/// countdown never ticks and would otherwise keep its stale frozen frame.
pub fn state_change_events(snap: &CountdownSnapshotDto) -> Vec<AppEvent> {
    let mut events = vec![AppEvent::State(CountdownStatePayload {
        id: snap.id,
        state: snap.state,
    })];
    if snap.state == CountdownState::Idle {
        events.push(AppEvent::Tick(CountdownTickPayload {
            id: snap.id,
            label: snap.label.clone(),
            remaining_ms: snap.duration as u64,
        }));
    }
    events
}
