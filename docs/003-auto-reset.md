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

1. Add `auto_reset_on_finish: bool` field to `Countdown` struct
   (`countdown/model.rs`). Default: `false`.
2. Add `auto_reset_on_finish` to `CountdownSnapshotDto` (`countdown/dto.rs`)
   with `#[serde(default)]` for backward compatibility with existing
   `countdowns.json` files.
3. Extend `countdown_create` to take the new flag, and add a separate
   `set_auto_reset_on_finish(id, bool)` Tauri command for live updates.
4. In the ticker (`commands.rs::spawn_ticker`, the
   `newly_finished` branch): when a countdown just transitioned
   Running → Finished **and** `auto_reset_on_finish == true`, call
   `countdown_service.reset(id)` after the existing snapshot has been
   taken, and emit a `Changed` event so the store persists the new Idle
   state. The zero-tick `countdown-tick` for OBS still fires first
   (no flicker), then the next tick carries the restored full
   `initial_duration`.
5. Persist on toggle (fire-and-forget save) and on every auto-reset
   transition.
6. The field rides along in snapshots and DTOs; no separate SSE event
   needed because `Changed` already broadcasts the full state list.

### Frontend

7. Add a "Behavior" section to `CountdownDetail.svelte` with an
   "Auto-reset when finished" toggle. Wire to
   `set_auto_reset_on_finish`.
8. Visual indicator on `Roost.svelte` (sidebar) — small loop/repeat
   glyph next to the countdown name when the flag is set, so the
   user knows at a glance.

## Design decisions

- **Reset to Idle, not auto-start.** Auto-reset ends in `Idle`, same
  shape as the user pressing Reset by hand. A future "auto-start"
  flag can layer on top if the user wants a loop.
- **In-place finish, not a reload.** This matters because OBS
  browser-sources flash on reload; the existing `finished_events`
  contract (`Tick { remaining_ms: 0 }` + `State(Finished)` instead of
  `Changed`) is what keeps that path no-flash. Auto-reset still
  emits a `Changed` for the Idle transition (it's a structural change
  visible in the snapshot list), but the OBS overlay handles the
  zero → restored transition gracefully because the next tick carries
  the full duration and the `state-change-events` helper already
  pushes a restored-value Tick for any state moving to Idle
  (`countdown/events.rs::state_change_events`).
- **Trigger only on natural finish.** A manually-reset timer is
  already Idle; pausing a finished timer keeps it Finished (the
  inverse of running). Auto-reset applies only to the in-tick
  Running → Finished transition.

## Out of scope

- Auto-start (automatically begin counting again after reset).
- Repeat count / max iterations.
- Auto-reset for countdowns that finished *during* downtime — those
  come back as `Finished` from `from_dtos` already; the auto-reset
  semantics apply only to in-session transitions. This is
  intentional and matches the existing re-anchor behavior.

## Verification

- Create countdown with auto-reset on, start it, let it expire →
  state returns to Idle automatically.
- Restart app → `auto_reset_on_finish: true` persisted and honored.
- OBS overlay updates correctly on auto-reset — zero tick first
  (no flash), then the next tick carries the restored value.
- Auto-reset off → countdown stays Finished as before (no regression).
- Manual reset still works the way it always did.
- Rust tests:
  `tests/countdown_finish.rs` extended to assert the auto-reset path
  emits Tick(0) + State(Finished) followed by Changed with the Idle
  snapshot (verify ordering, no double-fire, no `Reload` event for OBS).
  `tests/countdown_model.rs` extended to assert
  `auto_reset_on_finish` round-trips through restore.
  `tests/countdown_persist.rs` extended to confirm `#[serde(default)]`
  on DTO means an existing on-disk `countdowns.json` (without the
  field) still loads and defaults to `false`.
