# 001 — Persist groups across restarts

**Type:** Bug  
**Reported by:** Tester  
**Priority:** High  
**Affects:** Groups (overlay module)

## Problem

Groups are stored in-memory only (`OverlayService::new()` creates empty
`HashMap`s). On app restart, all groups are lost — the user has to recreate
them from scratch.

## Root cause

No persistence layer exists for the overlay module. Countdown persistence
was added in v0.4.0 (`countdown/store.rs`, `countdowns.json`) but groups
(`overlay/service.rs`) were not included.

## Relevant code

| File | What |
|---|---|
| `src-tauri/src/overlay/service.rs` | In-memory `HashMap<u64, Group>` + CRUD |
| `src-tauri/src/overlay/model.rs` L67-74 | `Group` struct (id, name, members, layout, hide_idle) |
| `src-tauri/src/overlay/dto.rs` | `GroupDto` — already has `Serialize`/`Deserialize` |
| `src-tauri/src/overlay/commands.rs` | Tauri commands: create, list, update, delete |
| `src-tauri/src/countdown/store.rs` | Reference impl for persistence (atomic write, load/save) |
| `src-tauri/src/settings.rs` | `write_atomic`, `local_data_file` helpers |

## Scope

1. Add `overlay/store.rs` — `load` and `save` functions following the
   countdown store pattern (`local_data_file(handle, "groups.json")`).
2. Wire `save` into every mutating group command (create, update, delete) —
   same fire-and-forget pattern as countdown store.
3. Wire `load` into `AppState::new` in `lib.rs` → pass persisted groups
   into `OverlayService`.
4. Add `OverlayService::from_dtos(groups)` (or equivalent restore constructor)
   that populates the `HashMap` and derives `next_id` as `max(id) + 1`.
5. Handle corrupt/missing file gracefully (fall back to empty, rename `.corrupt`).

## Out of scope

- Persisting `OverlayConfig` (separate issue: 002).
- Migrating existing group data (there is none to migrate).

## Verification

- Create groups, restart app → groups survive.
- Delete a group, restart → deletion persisted.
- Corrupt `groups.json` manually → app starts with empty groups, corrupt file
  renamed.
- `cargo test` — add round-trip tests in `tests/`.
