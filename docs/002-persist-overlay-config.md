# 002 — Persist overlay config (countdown design)

**Type:** Bug  
**Reported by:** Tester  
**Priority:** High  
**Affects:** Per-countdown appearance / overlay config

## Problem

Three related symptoms that all trace to "the frontend has no way to read
back what it wrote":

1. **Design resets on view-switch** — switching from a countdown to a group
   and back loses the countdown's appearance settings.
2. **Design lost on restart** — overlay config is in-memory only on both
   backend and frontend.
3. (Implicit) Any future widget that wants to *display* the styling (e.g.
   the previews in 006/007) currently has no backend source of truth to
   read from.

## Root cause

### Restart loss
`OverlayConfig` lives in `OverlayService.configs: Mutex<HashMap<u64,
OverlayConfig>>`, which starts empty on boot. No save/load path exists.

### View-switch loss
`AppearancePanel.svelte` stores settings in a **component-local** `$state`
object (`overlaySettings: Record<number, OverlaySettings>`). This survives
within a session as long as the component stays mounted, but:
- The panel is only rendered when `subject === "countdown"`.
- Switching to a group unmounts the panel → component state is destroyed.
- Switching back remounts with fresh defaults.

The frontend never loads config back from the backend — `set_overlay_config`
is write-only (UI → backend), there is no `get_overlay_config` call on mount.

**Fix is unified:** adding the persistence + the `get_overlay_config` IPC +
having `AppearancePanel` re-hydrate from the backend on mount fixes BOTH
restart loss and view-switch loss in one change. After the fix, the
component-local dict is just the in-flight editing buffer; the source of
truth is the backend (which is itself backed by `overlays.json` on disk).

## Relevant code

| File | What |
|---|---|
| `src-tauri/src/overlay/model.rs` L81-128 | `OverlayConfig` struct (16 fields, derives `Default`) |
| `src-tauri/src/overlay/service.rs` L96-107 | In-memory `set_config` / `get_config` (must become persistent) |
| `src-tauri/src/overlay/commands.rs` L57-66 | `set_overlay_config` Tauri command (must save after mutation) |
| `src-tauri/src/app_state.rs` L70 | `OverlayService::new()` (must accept persisted groups + configs) |
| `src-tauri/src/lib.rs` L43-56 | `setup` hook (must `overlay::store::load` alongside the countdown store) |
| `src-tauri/src/countdown/store.rs` | Reference impl — atomic write, `.json.corrupt` quarantine, `spawn_blocking` |
| `src-tauri/src/settings.rs` | `write_atomic`, `local_data_file` helpers |
| `src/features/countdown/components/AppearancePanel.svelte` L139-185 | Component-local `$state` (must hydrate from backend) |
| `src/features/countdown/api/countdown.client.ts` | Where the new `getOverlayConfig` wrapper lives |
| `src/features/countdown/state/countdown.store.ts` / `group.store.ts` | Where the warm-up fetch on `AppShell.onMount` is wired |

## Scope

### Backend
1. Add `src-tauri/src/overlay/store.rs` mirroring `countdown/store.rs`:
   - `load(handle) -> (Vec<GroupDto>, HashMap<u64, OverlayConfig>)` from
     `<app_local_data_dir>/overlays.json` via `settings::local_data_file`;
     same `countdowns.json.corrupt` quarantine on parse failure.
   - `save(handle, &groups, &configs)` — fire-and-forget
     `spawn_blocking` + atomic write.
   - On-disk shape: a single `OverlaysDto { groups, configs }` so
     `next_id` derivation stays a one-stop change.
2. New `OverlayService::from_groups_and_configs(dto, next_id) -> Self`
   that mirrors `CountdownService::from_dtos` (derives `next_id` as
   `max(group.id)+1`).
3. Wire `overlay::store::save` into every mutating overlay command
   (`group_create`, `group_update`, `group_delete`, `set_overlay_config`)
   — same fire-and-forget pattern as the countdown commands.
4. Wire `overlay::store::load` into the `setup` hook in `lib.rs` and
   pass the loaded groups + configs through a new `AppState::new`
   parameter.
5. Add `get_overlay_config(id) -> OverlayConfig` Tauri command (exposes
   the existing `OverlayService::get_config`); register it in
   `lib.rs::generate_handler!`.

### Frontend
6. Add `getOverlayConfig(id: number): Promise<OverlayConfig>` to the
   countdown client, mirroring the existing `invokeCommand` pattern.
7. In `AppShell.svelte`'s `onMount`, after `loadGroups()` resolves, walk
   every group member and call `getOverlayConfig` for each so the cache
   is warm before any panel mounts. Cache lives alongside the countdown
   store (or a thin new `overlayStore` — design choice while
   implementing, no separate spec needed).
8. In `AppearancePanel.svelte`, replace the empty-local-state fallback
   with: `$effect` on `id` → call `getOverlayConfig(id)` → seed local
   state from the response → only fall back to `DEFAULT_SETTINGS` when
   the backend has nothing stored (still safe because `OverlayConfig:
   Default` fills in any missing field).

## Design decision

Should overlay configs live in the same file as groups (`groups.json` →
`overlays.json` containing both) or their own file? **Same file** —
they're both overlay-module data and always small. Mirrors the
"one store per widget" rule in AGENTS.md.

## Out of scope

- Wiring the remaining `OverlayConfig` fields not yet exposed in the UI
  (progress bars, dividers, borders, etc.) — that's roadmap item 1.
- Lifting the local dict into a Svelte store as an alternative to
  backend hydration — the backend round-trip is required anyway for
  restart, so doing both is wasted work.

## Verification

- Configure a countdown's appearance, switch to a group, switch back →
  settings preserved.
- Configure appearance, restart app → settings restored.
- Two countdowns with different configs → each restores independently.
