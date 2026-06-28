use crate::app_state::AppState;
use crate::countdown::events::AppEvent;
use crate::overlay::dto::GroupDto;
use crate::overlay::model::{Layout, OverlayConfig};
use std::sync::Arc;
use tauri::{State, command};

#[command]
pub async fn group_create(state: State<'_, Arc<AppState>>, name: String) -> Result<u64, String> {
    state
        .overlay_service
        .create_group(name)
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn group_list(state: State<'_, Arc<AppState>>) -> Result<Vec<GroupDto>, String> {
    Ok(state
        .overlay_service
        .list_groups()
        .await
        .into_iter()
        .map(GroupDto::from)
        .collect())
}

#[command]
pub async fn group_update(
    state: State<'_, Arc<AppState>>,
    id: u64,
    name: String,
    members: Vec<u64>,
    layout: Layout,
    hide_idle: bool,
) -> Result<(), String> {
    state
        .overlay_service
        .update_group(id, name, members, layout, hide_idle)
        .await
        .map_err(|e| e.to_string())?;
    let _ = state.event_bus.send(AppEvent::Reload);
    Ok(())
}

#[command]
pub async fn group_delete(state: State<'_, Arc<AppState>>, id: u64) -> Result<(), String> {
    state
        .overlay_service
        .delete_group(id)
        .await
        .map_err(|e| e.to_string())?;
    let _ = state.event_bus.send(AppEvent::Reload);
    Ok(())
}

#[command]
pub async fn set_overlay_config(
    state: State<'_, Arc<AppState>>,
    id: u64,
    config: OverlayConfig,
) -> Result<(), String> {
    state.overlay_service.set_config(id, config).await;
    let _ = state.event_bus.send(AppEvent::Reload);
    Ok(())
}
