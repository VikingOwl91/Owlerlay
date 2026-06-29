# 007 — Preview per countdown

**Type:** Feature (frontend-design)  
**Reported by:** Tester  
**Priority:** Low  
**Affects:** Countdown detail / appearance panel

## Problem

When customizing a countdown's appearance (font, colors, progress bar, icon),
the user can't see what it will look like in OBS without switching context.
A live preview in the countdown detail view would let them iterate on
appearance settings instantly.

## Current state

- `CountdownDetail.svelte` shows controls (start/pause/reset) and embeds
  `AppearancePanel.svelte` for design settings.
- `AppearancePanel` has rich controls (icon, colors, font, time format,
  progress bar, etc.) but no visual preview.
- No single-countdown overlay endpoint exists — the server only renders
  groups at `/overlay?group={id}`.

## Scope

### Backend

1. Add a single-countdown overlay endpoint: `/overlay?countdown={id}`
   (or `/overlay/countdown/{id}`). Renders just that one countdown using
   the same template, as if it were the sole member of a virtual group.

### Frontend

2. Embed a scaled `<iframe>` in `CountdownDetail.svelte` (or inside
   `AppearancePanel`) pointing at the single-countdown endpoint.
3. The iframe auto-updates via SSE (ticks + config reload) — same
   mechanism as the group overlay.
4. Position the preview above the appearance controls so the user sees
   the effect of each change immediately.
5. Style to match the group preview (006) — mini OBS window look.

## Design decisions

- **Single-countdown endpoint:** Lightweight — reuse the existing group
  render logic but with a one-element members list. The route handler
  can construct a virtual `CountdownView` from just the one countdown.
- **Preview placement:** Above appearance controls is most natural —
  change a setting, glance up to see the result.

## Dependencies

- Benefits from 002 (persisted overlay config) so the preview survives
  navigation.
- Benefits from 005 (icon + label + layout) for a richer preview.

## Out of scope

- Editing appearance directly in the preview (click-to-style).
- Full-screen preview mode.

## Verification

- Open a countdown detail → preview shows the countdown as OBS would.
- Change font/color/icon → preview updates live.
- Start/pause/reset → preview reflects state.
- Compare preview to actual OBS browser source → pixel-identical.
