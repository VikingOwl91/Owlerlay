extern crate core;

mod app_state;
pub mod countdown;
pub mod overlay;
mod remote;
mod server;
pub mod settings;

use std::sync::Arc;

use crate::app_state::AppState;
use crate::countdown::commands::{
    countdown_create, countdown_delete, countdown_list, countdown_pause, countdown_reset,
    countdown_resume, countdown_snapshot, countdown_start, spawn_ticker,
};
use crate::overlay::commands::{
    group_create, group_delete, group_list, group_update, set_overlay_config,
};
use crate::remote::{remote_get_settings, remote_regenerate_token, remote_set_enabled};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Build state here (not before the builder) so we have an AppHandle
            // and the resolved config/token paths up front.
            let handle = app.handle().clone();
            let config = settings::load_config(&handle);
            let token = settings::load_or_create_token(&handle);
            app.manage(Arc::new(AppState::new(
                handle,
                config.remote_enabled,
                token,
            )));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            countdown_create,
            countdown_list,
            countdown_delete,
            countdown_start,
            countdown_reset,
            countdown_resume,
            countdown_pause,
            countdown_snapshot,
            set_overlay_config,
            group_create,
            group_list,
            group_update,
            group_delete,
            remote_get_settings,
            remote_set_enabled,
            remote_regenerate_token,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::Ready = event {
                let axum_state = app_handle.state::<Arc<AppState>>().inner().clone();
                tauri::async_runtime::spawn(async move {
                    server::start(axum_state).await;
                });
                spawn_ticker(app_handle.clone());
            }
        });
}
