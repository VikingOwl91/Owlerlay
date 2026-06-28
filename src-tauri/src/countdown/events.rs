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
