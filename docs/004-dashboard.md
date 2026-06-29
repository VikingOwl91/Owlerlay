# 004 — Dashboard overview in control hub

**Type:** Feature (frontend-design)  
**Reported by:** Tester  
**Priority:** Medium  
**Affects:** App shell / main stage area

## Problem

There is no overview screen. When no countdown or group is selected, the
main area shows a nearly empty placeholder with just a "New countdown" CTA.
Users want a dashboard that shows all widgets at a glance, grouped by group,
so they can see the state of their entire stream setup without clicking
through each item.

## Current behavior

`Stage.svelte` renders content based on a `Subject` discriminator:
- `null` / no selection → empty placeholder
- `"countdown"` → `CountdownDetail`
- `"group"` → `GroupPanel`
- `"create"` → inline create form
- `"settings"` → `RemoteSettings`

## Scope

1. Add a `"dashboard"` `Subject` variant in `src/app/shell/types.ts` and
   route it through `AppShell` and `Stage`.
2. New `src/app/shell/Dashboard.svelte` rendered when no specific
   countdown or group is selected.
3. Sections:
   - **Live** (top) — every Running countdown as a tile, regardless of
     group membership. Highest visibility because it's what the streamer
     is watching right now.
   - **Groups** — one card per group, each containing a tile per member
     countdown. Members show the same state pip, label, and live
     remaining as the rest of the app.
   - **Ungrouped** — one tile per countdown not in any group's
     `members` list.
4. **Each tile is interactive** (per design decision):
   - "Open" / title-click → switches `subject` to that countdown
     (`countdown`) or group (`group`).
   - Inline transport buttons per countdown tile: Start, Pause, Reset.
     These call the existing `countdownStore.startSelected/...`
     actions against the tile's id (re-using the same paths the
     detail panel uses — single source of truth).
   - Tile-color follows the existing state palette
     (`--st-idle / --st-running / --st-paused / --st-finished`) so the
     dashboard reads as a glance-able status board.
5. Subscribes to `countdownStore.items` and `groupStore.items` for
   live updates; no polling.

## Design direction (frontend-design skill)

- Tile shape is **clearly distinct** from the rail strips on the left:
  the rail strips have no affordances, the dashboard tiles do.
  Visually heavier (border, small shadow, more padding).
- Type uses the existing tokens (`--font-display` for headings,
  `--font-mono` for the timer digits, `--eye` accent for the live
  section header).
- One memorable quality: the "Live" section at the top, with a soft
  pulse on each running tile's pip, makes the dashboard feel like a
  live console. No decorative gradients — strictly the established
  twilight-and-tawny palette.
- Honest empty states ("No live countdowns right now",
  "No groups yet — create one to bundle widgets",
  "Create your first countdown" CTA), no fabricated values.

## Out of scope

- Drag-and-drop reordering (tiles or groups).
- Per-tile config (overlay settings belong on the countdown detail;
  the dashboard is a status surface, not an editor).
- Preview rendering (separate issues: 006, 007).

## Verification

- Open app with multiple countdowns in multiple groups → dashboard shows
  all, correctly grouped.
- Start a countdown → dashboard row updates live.
- Click a countdown/group → navigates to detail.
- No countdowns → dashboard shows empty state with create CTA.
