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

1. Add a `"dashboard"` subject (or use `null` as the dashboard state).
2. Create a `Dashboard.svelte` component rendered when no specific item
   is selected.
3. Dashboard content:
   - **Grouped sections** — one card/section per group, showing the group
     name, layout info, and its member countdowns with live state + time.
   - **Ungrouped section** — countdowns not assigned to any group.
   - Each countdown row: state pip (Idle/Running/Paused/Finished), name,
     live remaining time.
   - Click a countdown → navigate to its detail view.
   - Click a group header → navigate to group panel.
4. Wire into `Stage.svelte` as the default view.

## Design direction

- Card-based layout. Each group is a card with its members listed inside.
- Live-updating — subscribe to the existing countdown store's tick updates.
- Compact but informative — this is a monitoring view, not a control panel.
- Match existing Pico CSS + app aesthetic.

## Out of scope

- Drag-and-drop reordering.
- Quick-actions (start/pause) directly from the dashboard (keep it read-only
  for now — click through to detail for controls).
- Preview rendering (separate issue: 006, 008).

## Verification

- Open app with multiple countdowns in multiple groups → dashboard shows
  all, correctly grouped.
- Start a countdown → dashboard row updates live.
- Click a countdown/group → navigates to detail.
- No countdowns → dashboard shows empty state with create CTA.
