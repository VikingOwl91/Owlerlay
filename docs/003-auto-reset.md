# 003 — Auto-reset on countdown finish

**Type:** Feature  
**Reported by:** Tester  
**Priority:** Medium  
**Affects:** Countdown state machine, persistence, UI

## Problem

When a countdown finishes, it stays in `Finished` state until the user
manually resets it. Streamers want a "loop" mode where a countdown
automatically resets to Idle (ready for the next start) when it reaches zero.

## Current behavior

State machine: `Running → (expires) → Finished`. The ticker detects
`newly_finished`, emits events, persists — but never auto-resets.
`Finished` state shows only Reset and Delete buttons in the UI.

## Scope

### Backend

1. Add `auto_reset: bool` field to `Countdown` struct (`countdown/model.rs`).
   Default: `false`.
2. Add `auto_reset` to `CountdownSnapshotDto` (`countdown/dto.rs`) with
   `#[serde(default)]` for backward compatibility with existing
   `countdowns.json` files.
3. Add a `set_auto_reset(id, bool)` Tauri command (or extend `update`).
4. In the ticker (`commands.rs` ~L227-267): when a countdown is
   `newly_finished` and `auto_reset == true`, call `reset()` on it
   immediately instead of leaving it `Finished`.
5. Persist on toggle (fire-and-forget save).
6. Include `auto_reset` in SSE snapshot / Tauri events so the overlay
   and frontend stay in sync.

### Frontend

7. Add a toggle in `CountdownDetail.svelte` — "Auto-reset when finished"
   checkbox/switch.
8. Wire the toggle to the new Tauri command.
9. Visual indicator: show a loop/repeat icon on the sidebar row when
   `auto_reset` is enabled so the user knows at a glance.

## Design decisions

- **Reset vs. restart:** Auto-reset should go to `Idle`, not auto-start
  a new run. The user can combine this with a future "auto-start" if
  desired. This keeps behavior predictable.
- **Event emission:** The auto-reset should emit the same events as a
  manual `reset()` so the overlay and UI update correctly.

## Out of scope

- Auto-start (automatically begin counting again after reset).
- Repeat count / max iterations.

## Verification

- Create countdown with auto-reset on, start it, let it expire →
  state returns to Idle automatically.
- Restart app → `auto_reset: true` persisted and honored.
- OBS overlay updates correctly on auto-reset (SSE `reset` event).
- Auto-reset off → countdown stays Finished as before (no regression).
