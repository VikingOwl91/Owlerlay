//! Persistent countdown storage.
//!
//! Countdowns live in their own file (`<app_local_data_dir>/countdowns.json`),
//! separate from the trackable remote `config.json` — this is user data, not
//! config. The on-disk shape is just `Vec<CountdownSnapshotDto>` (the same DTO
//! the IPC/SSE paths already serialize), so there's no extra format to maintain.
//! `next_id` isn't stored: it's derived as `max(id)+1` on restore.

use std::io;
use std::path::Path;

use tauri::AppHandle;

use crate::countdown::dto::CountdownSnapshotDto;

fn store_path(handle: &AppHandle) -> io::Result<std::path::PathBuf> {
    crate::settings::local_data_file(handle, "countdowns.json")
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

/// Persist the current countdowns. Fire-and-forget: the (tiny) serialize runs
/// on the caller, then the blocking disk write is handed to a blocking thread so
/// it never stalls the async runtime that drives the 100ms ticker and the
/// LAN-remote SSE stream. Errors are dropped — every call site is best-effort.
pub fn save(handle: &AppHandle, snapshots: &[CountdownSnapshotDto]) {
    let Ok(path) = store_path(handle) else {
        return;
    };
    let Ok(json) = serde_json::to_string_pretty(snapshots) else {
        return;
    };
    tauri::async_runtime::spawn_blocking(move || {
        let _ = write_atomic(&path, &json);
    });
}

/// Write `json` to `path` atomically: a sibling temp file then a rename (atomic
/// on the same filesystem), so a concurrent save (desktop IPC + LAN remote) or a
/// crash mid-write can never leave a half-written file that `load()` discards.
// ponytail: both writers share one .tmp path; the rare overlap self-heals via
// load()'s empty fallback. Give the temp a unique suffix if that ever bites.
fn write_atomic(path: &Path, json: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, json)?;
    std::fs::rename(&tmp, path)
}
