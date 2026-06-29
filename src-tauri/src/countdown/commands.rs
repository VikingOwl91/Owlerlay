use crate::app_state::AppState;
use crate::app_state::ClockAnchor;
use crate::countdown::dto::CountdownSnapshotDto;
use crate::countdown::errors::CountdownError;
use crate::countdown::events::{
    AppEvent, CountdownTickPayload, finished_events, state_change_events,
};
use crate::countdown::service::CountdownSnapshot;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State, command};
use tokio::time::Instant;

/// Single source of truth for snapshot → DTO mapping (incl. the clock-anchor
/// `Instant` → epoch-ms conversion), shared by the list and single-snapshot
/// paths so they can't drift.
fn snap_to_dto(s: CountdownSnapshot, clock_anchor: &ClockAnchor) -> CountdownSnapshotDto {
    CountdownSnapshotDto {
        id: s.id,
        label: s.label,
        duration: s.duration.as_millis(),
        initial_duration: s.initial_duration.as_millis(),
        state: s.state,
        start_epoch_ms: s.start_instant.map(|i| clock_anchor.instant_to_epoch_ms(i)),
        target_epoch_ms: s
            .target_instant
            .map(|i| clock_anchor.instant_to_epoch_ms(i)),
    }
}

pub(crate) async fn build_snapshot_dtos(
    state: &AppState,
) -> Result<Vec<CountdownSnapshotDto>, CountdownError> {
    let snapshots = state.countdown_service.list_countdown().await?;
    Ok(snapshots
        .into_iter()
        .map(|s| snap_to_dto(s, &state.clock_anchor))
        .collect())
}

fn emit_changed(app: &AppHandle, state: &AppState, snapshots: Vec<CountdownSnapshotDto>) {
    let _ = crate::countdown::store::save(app, &snapshots);
    let _ = app.emit("countdown_changed", &snapshots);
    let _ = state.event_bus.send(AppEvent::Changed(snapshots));
}

/// Notify only the desktop UI (Tauri IPC), without touching the event bus.
fn notify_desktop(app: &AppHandle, snapshots: &[CountdownSnapshotDto]) {
    let _ = app.emit("countdown_changed", snapshots);
}

/// Notify the desktop and patch overlays in place for a single countdown whose
/// run-state changed (start/pause/resume/reset). Overlays update visibility
/// (and, on reset, the restored value) via `countdown-state`/`countdown-tick`
/// instead of a full page reload that would flash the OBS source. The broadcast
/// `State` event is also how SSE-only clients (the phone remote) learn that the
/// timer paused/resumed — overlays ignore `State` for any non-idle state, so
/// pausing still keeps their current frame.
fn emit_state_change(
    app: &AppHandle,
    state: &AppState,
    snapshots: &[CountdownSnapshotDto],
    id: u64,
) {
    let _ = crate::countdown::store::save(app, snapshots);
    notify_desktop(app, snapshots);
    if let Some(snap) = snapshots.iter().find(|s| s.id == id) {
        for event in state_change_events(snap) {
            let _ = state.event_bus.send(event);
        }
    }
}

// Shared timer-control logic, called by both the Tauri IPC commands (below) and
// the LAN remote HTTP routes (`server::remote`). Centralising the mutate+emit
// here is what keeps the desktop panel, OBS overlays, AND the phone remote in
// sync no matter which transport drove the change. All four broadcast a `State`
// event (via emit_state_change) so the SSE-only remote tracks every transition;
// overlays only act on `State` for idle visibility, so they never flash.

pub(crate) async fn do_start(
    app: &AppHandle,
    state: &AppState,
    id: u64,
) -> Result<(), CountdownError> {
    state.countdown_service.start(id, Instant::now()).await?;
    let snapshots = build_snapshot_dtos(state).await?;
    emit_state_change(app, state, &snapshots, id);
    Ok(())
}

pub(crate) async fn do_reset(
    app: &AppHandle,
    state: &AppState,
    id: u64,
) -> Result<(), CountdownError> {
    state.countdown_service.reset(id).await?;
    let snapshots = build_snapshot_dtos(state).await?;
    emit_state_change(app, state, &snapshots, id);
    Ok(())
}

pub(crate) async fn do_pause(
    app: &AppHandle,
    state: &AppState,
    id: u64,
) -> Result<(), CountdownError> {
    state.countdown_service.pause(id, Instant::now()).await?;
    let snapshots = build_snapshot_dtos(state).await?;
    emit_state_change(app, state, &snapshots, id);
    Ok(())
}

pub(crate) async fn do_resume(
    app: &AppHandle,
    state: &AppState,
    id: u64,
) -> Result<(), CountdownError> {
    state.countdown_service.resume(id, Instant::now()).await?;
    let snapshots = build_snapshot_dtos(state).await?;
    emit_state_change(app, state, &snapshots, id);
    Ok(())
}

#[command]
pub async fn countdown_create(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    label: String,
    duration: u64,
) -> Result<u64, String> {
    let duration = tokio::time::Duration::from_millis(duration);
    let id = state
        .countdown_service
        .create_countdown(label, duration)
        .await
        .map_err(|e: CountdownError| e.to_string())?;
    let snapshots = build_snapshot_dtos(&state)
        .await
        .map_err(|e| e.to_string())?;
    emit_changed(&app, &state, snapshots);
    Ok(id)
}

#[command]
pub async fn countdown_list(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<CountdownSnapshotDto>, String> {
    build_snapshot_dtos(&state).await.map_err(|e| e.to_string())
}

#[command]
pub async fn countdown_delete(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    id: u64,
) -> Result<(), String> {
    state
        .countdown_service
        .delete_countdown(id)
        .await
        .map_err(|e: CountdownError| e.to_string())?;
    let snapshots = build_snapshot_dtos(&state)
        .await
        .map_err(|e| e.to_string())?;
    emit_changed(&app, &state, snapshots);
    Ok(())
}

#[command]
pub async fn countdown_start(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    id: u64,
) -> Result<(), String> {
    do_start(&app, &state, id).await.map_err(|e| e.to_string())
}

#[command]
pub async fn countdown_reset(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    id: u64,
) -> Result<(), String> {
    do_reset(&app, &state, id).await.map_err(|e| e.to_string())
}

#[command]
pub async fn countdown_pause(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    id: u64,
) -> Result<(), String> {
    do_pause(&app, &state, id).await.map_err(|e| e.to_string())
}

#[command]
pub async fn countdown_resume(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    id: u64,
) -> Result<(), String> {
    do_resume(&app, &state, id).await.map_err(|e| e.to_string())
}

#[command]
pub async fn countdown_snapshot(
    state: State<'_, Arc<AppState>>,
    id: u64,
) -> Result<CountdownSnapshotDto, String> {
    let s = state
        .countdown_service
        .snapshot(id, Instant::now())
        .await
        .map_err(|e: CountdownError| e.to_string())?;
    Ok(snap_to_dto(s, &state.clock_anchor))
}

pub(crate) fn spawn_ticker(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));
        loop {
            interval.tick().await;
            let state = app.state::<Arc<AppState>>();
            let now = tokio::time::Instant::now();
            let result = state.countdown_service.tick(now).await;

            for (id, label, remaining) in
                result.still_running as Vec<(u64, String, tokio::time::Duration)>
            {
                let payload = CountdownTickPayload {
                    id,
                    label,
                    remaining_ms: remaining.as_millis() as u64,
                };
                let _ = app.emit("countdown_tick", &payload);
                let _ = state.event_bus.send(AppEvent::Tick(payload));
            }

            if !result.newly_finished.is_empty()
                && let Ok(snapshots) = build_snapshot_dtos(&state).await
            {
                // Persist the natural finish: this is the one state transition
                // not driven by a user action, so without this the store keeps
                // a stale `Running` entry until the next mutation.
                let _ = crate::countdown::store::save(&app, &snapshots);
                // Desktop app: refresh the list so finished countdowns show
                // their new state.
                let _ = app.emit("countdown_changed", &snapshots);
                // Overlays: push a final tick (remaining 0) per finished
                // countdown instead of a reload, so OBS browser sources land on
                // zero in place without flashing the source on stream.
                for event in finished_events(&result.newly_finished, &snapshots) {
                    let _ = state.event_bus.send(event);
                }
            }
        }
    });
}
