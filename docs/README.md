# Owlerlay — Issue Tracker

Scoped issues from user testing (2026-06-29), refined during planning
on the same day. The existing per-issue specs document the agreed design
decisions; this README is the dependency graph + order of attack.

## Bugs

| # | Title | Priority | Status |
|---|---|---|---|
| [001](001-persist-groups.md) | Persist groups across restarts | High | Open |
| [002](002-persist-overlay-config.md) | Persist overlay config (countdown design; also fixes the view-switch bug) | High | Open |

## Features

| # | Title | Priority | Status | Tag |
|---|---|---|---|---|
| [003](003-auto-reset.md) | Auto-reset on countdown finish | Medium | Open | backend |
| [004](004-dashboard.md) | Dashboard overview in control hub (tiles w/ inline transport) | Medium | Open | frontend-design |
| [005](005-countdown-icon-label.md) | Icons + labels per countdown (4 presets + custom flexbox) | Medium | Open | frontend-design |
| [006](006-preview-per-group.md) | Per-group preview (static Svelte mock from OverlayConfig) | Low | Open | frontend-design |
| [007](007-preview-per-countdown.md) | Per-countdown preview (static Svelte mock) | Low | Open | frontend-design |

## Dependency graph

```
001 (persist groups)
 └── 004 (dashboard — groups shown grouped; needs persisted groups)
  
002 (persist overlay config; adds get_overlay_config IPC)
 ├── 005 (icon + label + layout — needs persisted config to span restarts)
 ├── 006 (preview per group — reads persisted config)
 └── 007 (preview per countdown — reads persisted config)
 • 002 also fixes the view-switch bug (AppearancePanel hydrates from
   backend on mount) — see the spec for the path.

003 (auto-reset) — independent; small Rust-only change
```

## Implementation order (each step independently mergeable)

1. **001** + **002** — persistence layer + new `get_overlay_config` IPC
   + frontend hydration. This step alone fixes all three reported bugs
   (groups lost on restart, design lost on switch, design lost on restart).
2. **003** — auto-reset on finish. Self-contained backend feature with
   a small frontend toggle; no dependency on the other items.
3. **004** — Dashboard panel (uses the now-persisted groups + persisted
   per-member configs for the tile state).
4. **005** — icon-label layout presets. New fields on `OverlayConfig`,
   Jinja2 template changes, frontend preset picker, render tests in
   `tests/overlay_render.rs`.
5. **006** + **007** — preview tiles. New shared `PreviewTile.svelte`
   consumed by both `GroupPanel` and `CountdownDetail`.

## Cross-cutting rules pulled into this plan

- **Persistence shape:** one `overlays.json` file holding both
  `groups` and `configs` (mirrors the single-store-per-widget rule in
  AGENTS.md and the countdown store precedent).
- **Disk location:** `<app_local_data_dir>/overlays.json`, written
  via `settings::write_atomic`, with the same `.json.corrupt`
  quarantine the countdown store uses.
- **Auto-reset semantics:** natural finish only, transitions Finished
  → Idle in place (no OBS reload). Verifies the existing
  `finished_events` invariants still hold.
- **Layout presets:** `OverlayLayout` enum with 4 named choices plus a
  `Custom` arm that unlocks the free-form flex-direction / wrap /
  justify-content / align-items fields. Each preset must render
  under strict-undefined (new render test).
