//! Persistent countdown storage.
//!
//! Countdowns live in their own file (`<app_local_data_dir>/countdowns.json`),
//! separate from the trackable remote `config.json` — this is user data, not
//! config. The on-disk shape is just `Vec<CountdownSnapshotDto>` (the same DTO
//! the IPC/SSE paths already serialize), so there's no extra format to maintain.
//! `next_id` isn't stored: it's derived as `max(id)+1` on restore.

use std::io;

use tauri::AppHandle;

use crate::countdown::dto::CountdownSnapshotDto;

fn store_path(handle: &AppHandle) -> io::Result<std::path::PathBuf> {
    crate::settings::local_data_file(handle, "countdowns.json")
}

/// Load persisted countdowns, falling back to an empty list so a corrupt store
/// never bricks startup. A *missing* file is the normal first-run case; a file
/// that's present but unparseable is moved aside to `countdowns.json.corrupt`
/// before returning empty, so the next save can't silently overwrite (and
/// permanently destroy) the user's timers — they stay recoverable by hand.
pub fn load(handle: &AppHandle) -> Vec<CountdownSnapshotDto> {
    let Ok(path) = store_path(handle) else {
        return Vec::new();
    };
    let Ok(contents) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    match serde_json::from_str(&contents) {
        Ok(parsed) => parsed,
        Err(_) => {
            // ponytail: single .corrupt slot; a later corruption overwrites it.
            let _ = std::fs::rename(&path, path.with_extension("json.corrupt"));
            Vec::new()
        }
    }
}

/// Persist the current countdowns. Fire-and-forget: the (tiny) serialize runs
/// on the caller, then the blocking disk write (atomic — see
/// [`crate::settings::write_atomic`]) is handed to a blocking thread so it never
/// stalls the async runtime that drives the 100ms ticker and the LAN-remote SSE
/// stream. Errors are dropped — every call site is best-effort.
pub fn save(handle: &AppHandle, snapshots: &[CountdownSnapshotDto]) {
    let Ok(path) = store_path(handle) else {
        return;
    };
    let Ok(json) = serde_json::to_string_pretty(snapshots) else {
        return;
    };
    tauri::async_runtime::spawn_blocking(move || {
        let _ = crate::settings::write_atomic(&path, &json, None);
    });
}
