//! Persistent countdown storage.
//!
//! Countdowns live in their own file (`<app_local_data_dir>/countdowns.json`),
//! separate from the trackable remote `config.json` — this is user data, not
//! config. The on-disk shape is just `Vec<CountdownSnapshotDto>` (the same DTO
//! the IPC/SSE paths already serialize), so there's no extra format to maintain.
//! `next_id` isn't stored: it's derived as `max(id)+1` on restore.

use std::io;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

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

/// Write `json` to `path` atomically: a *unique* sibling temp file then a rename
/// (atomic on the same filesystem), so a concurrent save (desktop IPC + LAN
/// remote) or a crash mid-write can never leave a half-written file that
/// `load()` discards. The per-write sequence keeps the two writers off each
/// other's temp file; on failure the temp is cleaned up so it can't accumulate.
fn write_atomic(path: &Path, json: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    static SAVE_SEQ: AtomicU64 = AtomicU64::new(0);
    let seq = SAVE_SEQ.fetch_add(1, Ordering::Relaxed);
    let tmp = path.with_extension(format!("json.tmp.{seq}"));
    match std::fs::write(&tmp, json).and_then(|()| std::fs::rename(&tmp, path)) {
        Ok(()) => Ok(()),
        Err(e) => {
            let _ = std::fs::remove_file(&tmp);
            Err(e)
        }
    }
}
