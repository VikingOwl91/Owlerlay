# 007 — Preview per countdown

**Type:** Feature (frontend-design)  
**Reported by:** Tester  
**Priority:** Low  
**Affects:** Countdown detail / appearance panel

## Problem

When customizing a countdown's appearance (font, colors, progress bar,
icon), the user can't see what it will look like in OBS without
switching context. A preview embedded next to the appearance controls
lets them iterate instantly.

## Current state

- `CountdownDetail.svelte` shows controls (start/pause/reset) and
  embeds `AppearancePanel.svelte` for design settings.
- `AppearancePanel` has rich controls (icon, colors, font, time format,
  progress bar, etc.) but no visual preview.

## Scope

### Frontend

1. Reuse the shared `PreviewTile.svelte` from issue 006 (same
   component, single member instead of first-group-member).
2. Place the preview **above** the appearance controls in
   `CountdownDetail.svelte` so editing a setting shows the result
   immediately (glance up, see the change).
3. The preview reads the persisted `OverlayConfig` from the hydrated
   cache (issue 002) — if hydration didn't happen for this countdown,
   the existing `Default` falls back gracefully without loading state.
4. Add a side panel layout option: a side-by-side configuration form
   on the left, live preview on the right, on wide enough viewports
   (`>1100px`); stacked on narrow ones.

### No new backend endpoint.

The single-countdown overlay endpoint
(`/overlay?countdown=<id>`, originally proposed in the first draft of
this spec) is **not** needed for this issue — the Svelte preview is
self-contained. If a real "OBS-wants-this-URL" use case surfaces
later, that endpoint can land independently.

## Design decisions

- **Svelte mock, not iframe.** Same reasoning as 006: no CSP work,
  no per-keystroke SSE churn, native tokens, refreshes in step with
  the form, and the OBS "Open in browser" link serves as the
  exact-parity escape hatch when genuinely needed.
- **Preview placement above controls.** Edit a colour, glance up,
  see the result. The original full-width readout remains below
  the controls for the big at-a-glance view.
- **Single shared preview component.** 006 and 007 ship the same
  `PreviewTile.svelte` — duplication is forbidden because the two
  surfaces *must* render identically when they show the same
  countdown.

## Dependencies

- Issue 002 (persisted overlay config) so the preview reflects saved
  settings.
- Issue 005 (icon-label + layout presets) for the rich preview. The
  preview can ship before 005 in a degraded shape (single-row layout
  only) if needed for sequencing.

## Out of scope

- Editing appearance directly in the preview (click-to-style).
- Full-screen preview mode.
- A server-rendered single-countdown endpoint (out; the Svelte mock
  is sufficient for the control-UI preview loop).

## Verification

- Open a countdown detail → preview shows the countdown as it would
  appear in OBS, using the persisted `OverlayConfig`.
- Change font, colour, icon, progress-bar colour, or layout preset →
  preview updates without any navigation.
- Same configuration across two restarts renders identically (relies
  on 002).
- The preview matches the live OBS overlay URL "as closely as the
  Svelte render can get" — the SSR render test
  (`tests/overlay_render.rs`) is the source of truth for OBS
  fidelity; regression must fail there before the preview can drift.
