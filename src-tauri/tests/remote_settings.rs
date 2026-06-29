//! Capability token + remote-config invariants. The token-file IO and the
//! enable flag go through Tauri's path API (needs a running app), so these
//! cover the parts testable without one: token shape and config (de)serialization.

use owlerlay_lib::settings::{RemoteConfig, generate_token};

#[test]
fn token_is_256_bits_of_hex_and_unique() {
    let a = generate_token();
    let b = generate_token();
    assert_eq!(a.len(), 64, "32 random bytes -> 64 hex chars");
    assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
    assert_ne!(a, b, "two freshly minted tokens must not collide");
}

#[test]
fn config_defaults_to_disabled_and_round_trips() {
    // A missing/empty config must read as "remote off" (safe default).
    let default: RemoteConfig = serde_json::from_str("{}").unwrap();
    assert!(!default.remote_enabled);

    let json = serde_json::to_string(&RemoteConfig {
        remote_enabled: true,
    })
    .unwrap();
    let back: RemoteConfig = serde_json::from_str(&json).unwrap();
    assert!(back.remote_enabled);
}
