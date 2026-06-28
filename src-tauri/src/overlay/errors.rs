use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum OverlayError {
    #[error("Group id not found")]
    GroupNotFound,
    #[error("Max groups reached")]
    MaxGroupsReached,
    #[error("Group name must not be empty")]
    EmptyName,
}
