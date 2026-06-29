//! Persistent remote-control settings.
//!
//! Two files, deliberately split so the config can be tracked (e.g. chezmoi)
//! without ever leaking the secret:
//!   - **config** (`<app_config_dir>/config.json`): `remote_enabled` only — safe
//!     to commit to a dotfiles repo.
//!   - **token** (`<app_local_data_dir>/remote_token`): the capability secret,
//!     kept out of the tracked config.

use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

/// The trackable slice of config. Anything secret stays in the token file.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RemoteConfig {
    #[serde(default)]
    pub remote_enabled: bool,
}

/// Resolve a file in the app local-data dir (`<app_local_data_dir>/<name>`).
/// The single place the data-dir idiom lives — shared by `token_path` here and
/// `countdown::store`, so a future relocation of user data is a one-spot change.
pub(crate) fn local_data_file(handle: &AppHandle, name: &str) -> io::Result<PathBuf> {
    let dir = handle
        .path()
        .app_local_data_dir()
        .map_err(|e| io::Error::other(e.to_string()))?;
    Ok(dir.join(name))
}

fn config_path(handle: &AppHandle) -> io::Result<PathBuf> {
    let dir = handle
        .path()
        .app_config_dir()
        .map_err(|e| io::Error::other(e.to_string()))?;
    Ok(dir.join("config.json"))
}

fn token_path(handle: &AppHandle) -> io::Result<PathBuf> {
    local_data_file(handle, "remote_token")
}

/// Load the config, falling back to defaults (remote disabled) on any
/// missing-file/parse error — a corrupt config must never brick startup.
pub fn load_config(handle: &AppHandle) -> RemoteConfig {
    let Ok(path) = config_path(handle) else {
        return RemoteConfig::default();
    };
    match std::fs::read_to_string(&path) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => RemoteConfig::default(),
    }
}

/// Persist the `remote_enabled` flag. Takes effect on next launch (the bind
/// address is chosen once at startup).
pub fn set_enabled(handle: &AppHandle, enabled: bool) -> io::Result<()> {
    let path = config_path(handle)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let cfg = RemoteConfig {
        remote_enabled: enabled,
    };
    let json = serde_json::to_string_pretty(&cfg).map_err(io::Error::other)?;
    std::fs::write(&path, json)
}

/// Read the persisted token, generating + writing one if absent. On any IO
/// error resolving/writing the path, returns a fresh in-memory token so the
/// remote still works for this session (it just won't survive a restart).
pub fn load_or_create_token(handle: &AppHandle) -> String {
    let Ok(path) = token_path(handle) else {
        return generate_token();
    };
    if let Ok(existing) = std::fs::read_to_string(&path) {
        let trimmed = existing.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    let token = generate_token();
    let _ = write_token(&path, &token);
    token
}

/// Mint a new token, overwrite the secret file, and return it (revokes every
/// previously-issued URL).
pub fn regenerate_token(handle: &AppHandle) -> io::Result<String> {
    let path = token_path(handle)?;
    let token = generate_token();
    write_token(&path, &token)?;
    Ok(token)
}

fn write_token(path: &PathBuf, token: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, token)?;
    // The secret must not be readable by other local accounts.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

/// 256 bits of randomness, hex-encoded (64 chars).
pub fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).expect("OS RNG must be available");
    let mut s = String::with_capacity(64);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}
