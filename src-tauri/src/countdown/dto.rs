use crate::countdown::model::CountdownState;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct CountdownSnapshotDto {
    pub id: u64,
    pub label: String,
    /// Remaining time, in milliseconds.
    pub duration: u128,
    /// Configured countdown length, in milliseconds (used for progress percent).
    pub initial_duration: u128,
    pub state: CountdownState,
    pub start_epoch_ms: Option<u128>,
    pub target_epoch_ms: Option<u128>,
}
