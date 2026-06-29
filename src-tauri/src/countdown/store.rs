//! Persistent countdown storage.
//!
//! Countdowns live in their own file (`<app_local_data_dir>/countdowns.json`),
//! separate from the trackable remote `config.json` — this is user data, not
//! config. The on-disk shape is just `Vec<CountdownSnapshotDto>` (the same DTO
//! the IPC/SSE paths already serialize), so there's no extra format to maintain.
//! `next_id` isn't stored: it's derived as `max(id)+1` on restore.

use std::io;
use std::path::PathBuf;

use tauri::{AppHandle, Manager};

use crate::countdown::dto::CountdownSnapshotDto;

fn store_path(handle: &AppHandle) -> io::Result<PathBuf> {
    let dir = handle
        .path()
        .app_local_data_dir()
        .map_err(|e| io::Error::other(e.to_string()))?;
    Ok(dir.join("countdowns.json"))
}

/// Load persisted countdowns, falling back to an empty list on any
/// missing-file/parse error — a corrupt store must never brick startup.
pub fn load(handle: &AppHandle) -> Vec<CountdownSnapshotDto> {
    let Ok(path) = store_path(handle) else {
        return Vec::new();
    };
    match std::fs::read_to_string(&path) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

/// Persist the current countdowns. Best-effort: callers ignore the result so a
/// transient IO error never blocks a timer action.
// ponytail: plain write (not atomic), matching settings.rs. A crash mid-write
// corrupts the file, but load() falls back to empty. Upgrade to temp+rename if
// that ever bites.
pub fn save(handle: &AppHandle, snapshots: &[CountdownSnapshotDto]) -> io::Result<()> {
    let path = store_path(handle)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(snapshots).map_err(io::Error::other)?;
    std::fs::write(&path, json)
}
