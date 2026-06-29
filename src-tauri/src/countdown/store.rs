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
pub fn save(handle: &AppHandle, snapshots: &[CountdownSnapshotDto]) -> io::Result<()> {
    let path = store_path(handle)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(snapshots).map_err(io::Error::other)?;
    // Atomic write: the desktop IPC and the LAN remote can both reach save()
    // concurrently, and a crash can land mid-write. Write a sibling temp file
    // then rename (atomic on the same filesystem) so the real file is always a
    // complete previous-or-new version, never a half-written one load() discards.
    // ponytail: both writers share one .tmp path; the rare overlap self-heals via
    // load()'s empty fallback. Give the temp a unique suffix if that ever bites.
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, json)?;
    std::fs::rename(&tmp, &path)
}
