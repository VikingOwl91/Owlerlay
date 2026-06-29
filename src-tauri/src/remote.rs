//! Tauri commands backing the desktop "Phone remote" settings panel: read the
//! current state (incl. the QR URL), flip the enable flag, and rotate the token.

use std::net::UdpSocket;
use std::sync::Arc;

use serde::Serialize;
use tauri::{AppHandle, State};

use crate::app_state::AppState;
use crate::server::PORT;
use crate::settings;

#[derive(Serialize)]
pub struct RemoteSettings {
    /// The persisted desired state (what the toggle shows).
    pub enabled: bool,
    /// `http://<lan-ip>:<port>/remote?t=<token>` — only when the server is
    /// actually bound to the LAN (i.e. enabled *and* restarted).
    pub url: Option<String>,
    pub token: Option<String>,
    /// Persisted flag differs from what the running server bound — a restart is
    /// needed for the change to take effect.
    pub restart_required: bool,
}

/// Primary LAN IP via the UDP "connect" trick: no packets are sent, the socket
/// just resolves which local interface would route to the target.
fn lan_ip() -> Option<std::net::IpAddr> {
    let sock = UdpSocket::bind("0.0.0.0:0").ok()?;
    sock.connect("8.8.8.8:80").ok()?;
    sock.local_addr().ok().map(|a| a.ip())
}

async fn current_settings(state: &AppState, app: &AppHandle) -> RemoteSettings {
    let persisted = settings::load_config(app).remote_enabled;
    let running = state.remote_enabled;
    let (url, token) = if running {
        let token = state.remote_token.read().await.clone();
        let url = lan_ip().map(|ip| format!("http://{ip}:{PORT}/remote?t={token}"));
        (url, Some(token))
    } else {
        (None, None)
    };
    RemoteSettings {
        enabled: persisted,
        url,
        token,
        restart_required: persisted != running,
    }
}

#[tauri::command]
pub async fn remote_get_settings(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
) -> Result<RemoteSettings, String> {
    Ok(current_settings(&state, &app).await)
}

#[tauri::command]
pub fn remote_set_enabled(app: AppHandle, enabled: bool) -> Result<(), String> {
    settings::set_enabled(&app, enabled).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remote_regenerate_token(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
) -> Result<RemoteSettings, String> {
    let token = settings::regenerate_token(&app).map_err(|e| e.to_string())?;
    *state.remote_token.write().await = token;
    Ok(current_settings(&state, &app).await)
}
