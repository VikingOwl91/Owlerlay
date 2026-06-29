use crate::countdown::dto::CountdownSnapshotDto;
use crate::countdown::events::AppEvent;
use crate::countdown::service::CountdownService;
use crate::overlay::service::OverlayService;
use tauri::AppHandle;
use tokio::sync::{RwLock, broadcast};

#[derive(Clone, Debug)]
pub struct ClockAnchor {
    pub boot_instant: tokio::time::Instant,
    pub boot_epoch_ms: u128,
}

impl ClockAnchor {
    pub fn new() -> Self {
        Self {
            boot_instant: tokio::time::Instant::now(),
            boot_epoch_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
        }
    }

    pub fn instant_to_epoch_ms(&self, instant: tokio::time::Instant) -> u128 {
        if let Some(delta) = instant.checked_duration_since(self.boot_instant) {
            self.boot_epoch_ms + delta.as_millis()
        } else {
            let delta = self.boot_instant.duration_since(instant).as_millis();
            self.boot_epoch_ms.saturating_sub(delta)
        }
    }
}

#[derive(Debug)]
pub struct AppState {
    pub clock_anchor: ClockAnchor,
    pub countdown_service: CountdownService,
    pub overlay_service: OverlayService,
    pub event_bus: broadcast::Sender<AppEvent>,
    /// Handle to the Tauri app, so HTTP control routes can emit the same
    /// desktop-facing events the IPC commands do (keeps the panel in sync).
    pub app_handle: AppHandle,
    /// Resolved once at startup from the persisted config; drives the bind
    /// address and gates the remote routes.
    pub remote_enabled: bool,
    /// Capability secret checked by the remote routes. Behind a lock so it can
    /// be rotated at runtime (regenerate = revoke).
    pub remote_token: RwLock<String>,
}

impl AppState {
    pub fn new(
        app_handle: AppHandle,
        remote_enabled: bool,
        remote_token: String,
        persisted: Vec<CountdownSnapshotDto>,
    ) -> Self {
        let (event_bus, _) = broadcast::channel(64);
        // Anchor first, then restore countdowns against its boot wall-clock so
        // running timers keep counting across the restart.
        let clock_anchor = ClockAnchor::new();
        let countdown_service = CountdownService::from_dtos(persisted, clock_anchor.boot_epoch_ms);
        Self {
            clock_anchor,
            countdown_service,
            overlay_service: OverlayService::new(),
            event_bus,
            app_handle,
            remote_enabled,
            remote_token: RwLock::new(remote_token),
        }
    }
}
