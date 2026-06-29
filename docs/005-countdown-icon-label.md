# 005 — Icons + labels per countdown (configurable layout)

**Type:** Feature (frontend-design)  
**Reported by:** Tester  
**Priority:** Medium  
**Affects:** Countdown detail, sidebar rows, overlay templates

## Problem

Countdowns currently display only their name. Users want:
- A **label** distinct from the countdown name (e.g. name = "Stream Timer",
  label = "⏱ Time Remaining" shown in overlay).
- An **icon** per countdown (emoji or image).
- **Configurable layout** of icon, label, and timer in the overlay — e.g.:
  - Icon left (two rows height), label top-right, timer bottom-right
  - Flipped (icon right)
  - Icon + label top row, timer below
  - Timer top, icon + label below
  - etc.

## Current state

- `Countdown` struct has `name: String` only. No label or icon fields.
- `OverlayConfig` has `icon: String` (emoji icon for the overlay) but no
  separate label field and no layout variant for icon/label/timer arrangement.
- The overlay template (`countdown.css.j2` / `browsersource.html.j2`)
  renders: icon (if set) + timer. No label element.

## Scope

### Data model

1. Add `label: Option<String>` to `Countdown` (backend + DTO + frontend type).
   When `None`, overlay falls back to countdown name. Persisted.
2. The existing `OverlayConfig.icon` already covers the icon per countdown.
   Ensure it's persisted (depends on 002).

### Layout variants

3. Add a `WidgetLayout` enum to `OverlayConfig`:
   ```
   IconLeftStacked    — icon left (spans 2 rows), label top-right, timer bottom-right
   IconRightStacked   — mirror of above
   IconLabelTop       — icon + label on top row, timer below
   TimerTop           — timer on top row, icon + label below
   Inline             — icon | label | timer in a single row
   ```
4. Wire `WidgetLayout` into the Jinja2 overlay template — use CSS grid/flex
   classes per variant.
5. Persist as part of `OverlayConfig` (depends on 002).

### Frontend

6. Add label field to `CountdownDetail.svelte` (editable text input).
7. Add layout picker to `AppearancePanel.svelte` — visual selector showing
   each layout variant as a mini-diagram.
8. Show icon + label in sidebar rows (`Roost.svelte`) for at-a-glance
   identification.

## Design decisions

- **Label vs name:** Name is the internal identifier (sidebar, references).
  Label is the display string for the overlay. They are independent.
- **Layout enum vs freeform CSS:** Enum keeps the overlay template
  manageable and guarantees tested layouts. Freeform CSS is out of scope.
- **Icon type:** Keep as string (emoji). Image icons are future scope
  (persistence rule: store by path reference).

## Out of scope

- Image/file icons (use emoji for now).
- Custom CSS per countdown.
- Label/icon in the OBS overlay URL (already driven by config).

## Verification

- Set label ≠ name → overlay shows label, sidebar shows name.
- Pick each layout variant → overlay renders correctly.
- Restart app → label and layout persisted (requires 002).
