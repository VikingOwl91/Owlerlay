use crate::app_state::AppState;
use crate::countdown::dto::CountdownSnapshotDto;
use crate::countdown::errors::CountdownError;
use crate::countdown::events::{AppEvent, CountdownTickPayload, finished_tick_events};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State, command};
use tokio::time::Instant;

pub(crate) async fn build_snapshot_dtos(
    state: &AppState,
) -> Result<Vec<CountdownSnapshotDto>, CountdownError> {
    let snapshots = state.countdown_service.list_countdown().await?;
    Ok(snapshots
        .into_iter()
        .map(|s| {
            let start = s
                .start_instant
                .map(|i| state.clock_anchor.instant_to_epoch_ms(i));
            let target = s
                .target_instant
                .map(|i| state.clock_anchor.instant_to_epoch_ms(i));
            CountdownSnapshotDto {
                id: s.id,
                label: s.label,
                duration: s.duration.as_millis(),
                initial_duration: s.initial_duration.as_millis(),
                state: s.state,
                start_epoch_ms: start,
                target_epoch_ms: target,
            }
        })
        .collect())
}

fn emit_changed(app: &AppHandle, state: &AppState, snapshots: Vec<CountdownSnapshotDto>) {
    let _ = app.emit("countdown_changed", &snapshots);
    let _ = state.event_bus.send(AppEvent::Changed(snapshots));
}

/// Notify only the desktop UI of a state change, leaving overlays untouched.
///
/// Used for transitions that don't change what an overlay renders — pause and
/// resume merely stop/restart the ticking timer, so the overlay keeps its
/// current frame. Emitting [`AppEvent::Changed`] here would make connected OBS
/// browser sources reload the whole page and flash on stream.
fn notify_desktop(app: &AppHandle, snapshots: &[CountdownSnapshotDto]) {
    let _ = app.emit("countdown_changed", snapshots);
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
    state
        .countdown_service
        .start(id, Instant::now())
        .await
        .map_err(|e: CountdownError| e.to_string())?;
    let snapshots = build_snapshot_dtos(&state)
        .await
        .map_err(|e| e.to_string())?;
    emit_changed(&app, &state, snapshots);
    Ok(())
}

#[command]
pub async fn countdown_reset(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    id: u64,
) -> Result<(), String> {
    state
        .countdown_service
        .reset(id)
        .await
        .map_err(|e: CountdownError| e.to_string())?;
    let snapshots = build_snapshot_dtos(&state)
        .await
        .map_err(|e| e.to_string())?;
    emit_changed(&app, &state, snapshots);
    Ok(())
}

#[command]
pub async fn countdown_pause(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    id: u64,
) -> Result<(), String> {
    state
        .countdown_service
        .pause(id, Instant::now())
        .await
        .map_err(|e: CountdownError| e.to_string())?;
    let snapshots = build_snapshot_dtos(&state)
        .await
        .map_err(|e| e.to_string())?;
    notify_desktop(&app, &snapshots);
    Ok(())
}

#[command]
pub async fn countdown_resume(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    id: u64,
) -> Result<(), String> {
    state
        .countdown_service
        .resume(id, Instant::now())
        .await
        .map_err(|e: CountdownError| e.to_string())?;
    let snapshots = build_snapshot_dtos(&state)
        .await
        .map_err(|e| e.to_string())?;
    notify_desktop(&app, &snapshots);
    Ok(())
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
    let start = s
        .start_instant
        .map(|i| state.clock_anchor.instant_to_epoch_ms(i));
    let target = s
        .target_instant
        .map(|i| state.clock_anchor.instant_to_epoch_ms(i));
    Ok(CountdownSnapshotDto {
        id: s.id,
        label: s.label,
        duration: s.duration.as_millis(),
        initial_duration: s.initial_duration.as_millis(),
        state: s.state,
        start_epoch_ms: start,
        target_epoch_ms: target,
    })
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
                // Desktop app: refresh the list so finished countdowns show
                // their new state.
                let _ = app.emit("countdown_changed", &snapshots);
                // Overlays: push a final tick (remaining 0) per finished
                // countdown instead of a reload, so OBS browser sources land on
                // zero in place without flashing the source on stream.
                for event in finished_tick_events(&result.newly_finished, &snapshots) {
                    let _ = state.event_bus.send(event);
                }
            }
        }
    });
}
