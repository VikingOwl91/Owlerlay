use crate::countdown::events::AppEvent;
use crate::countdown::service::CountdownService;
use crate::overlay::service::OverlayService;
use tokio::sync::broadcast;

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
}

impl AppState {
    pub fn new() -> Self {
        let (event_bus, _) = broadcast::channel(64);
        Self {
            clock_anchor: ClockAnchor::new(),
            countdown_service: CountdownService::new(),
            overlay_service: OverlayService::new(),
            event_bus,
        }
    }
}
