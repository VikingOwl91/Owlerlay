# 002 — Persist overlay config (countdown design)

**Type:** Bug  
**Reported by:** Tester  
**Priority:** High  
**Affects:** Per-countdown appearance / overlay config

## Problem

Two related symptoms:

1. **Design resets on view-switch** — switching from a countdown to a group
   and back loses the countdown's appearance settings.
2. **Design lost on restart** — overlay config is in-memory only on both
   backend and frontend.

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

## Relevant code

| File | What |
|---|---|
| `src-tauri/src/overlay/model.rs` L81-128 | `OverlayConfig` struct (16 fields) |
| `src-tauri/src/overlay/service.rs` L96-107 | In-memory `set_config` / `get_config` |
| `src-tauri/src/overlay/commands.rs` L57-66 | `set_overlay_config` Tauri command |
| `src/features/countdown/components/AppearancePanel.svelte` L139-185 | Component-local `$state`, defaults |

## Scope

### Backend
1. Add persistence for `OverlayConfig` — save a `HashMap<u64, OverlayConfig>`
   to `overlay_configs.json` (or co-locate in `groups.json` alongside groups).
2. Wire save into `set_overlay_config` command.
3. Wire load into `AppState::new` → seed `OverlayService.configs`.
4. Add a `get_overlay_config` Tauri command that returns the config for a
   given countdown ID (or all configs in bulk).

### Frontend
5. `AppearancePanel` must call `get_overlay_config(id)` on mount and seed
   its local state from the backend response.
6. Alternatively, lift `overlaySettings` into a Svelte store so it survives
   component unmount — but backend persistence is still needed for restart.

## Design decision

Should overlay configs live in the same file as groups (`groups.json` →
`overlays.json` containing both) or their own file? Recommend **same file**
since they're both overlay-module data and always small.

## Out of scope

- Wiring the remaining `OverlayConfig` fields not yet exposed in the UI
  (progress bars, dividers, borders, etc.) — that's roadmap item 1.

## Verification

- Configure a countdown's appearance, switch to a group, switch back →
  settings preserved.
- Configure appearance, restart app → settings restored.
- Two countdowns with different configs → each restores independently.
