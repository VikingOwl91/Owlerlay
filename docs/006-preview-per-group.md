# 006 — Preview per group

**Type:** Feature (frontend-design)  
**Reported by:** Tester  
**Priority:** Low  
**Affects:** Group panel UI

## Problem

When configuring a group, the user has no preview of what the OBS overlay
will look like. They have to switch to OBS (or open the overlay URL in a
browser) to see the result. A live preview embedded in the group panel would
dramatically speed up configuration.

## Current state

- `GroupPanel.svelte` shows: name, layout toggle (row/column), hide-idle
  toggle, member checkboxes, and an OBS URL with copy button.
- The overlay is rendered server-side by Axum at `/overlay?group={id}`
  using Jinja2 templates.
- No preview exists anywhere in the frontend.

## Scope

1. Embed an `<iframe>` (or `<webview>`) in `GroupPanel.svelte` pointing
   at `http://localhost:7420/overlay?group={id}`.
2. The iframe should:
   - Auto-refresh when the group config changes (already happens via SSE
     `reload` event in the overlay).
   - Be scaled down to fit the panel (CSS `transform: scale()` with a
     fixed aspect ratio container).
   - Have a "Open in browser" link alongside.
3. Style the preview container to look like a mini OBS source window
   (dark background, subtle border, maybe a small "PREVIEW" label).

## Design decisions

- **iframe vs re-render:** iframe is simplest — reuses the exact server-side
  render pipeline, so the preview is pixel-identical to what OBS sees.
  Downside: depends on the overlay server running (it always is in dev/prod).
- **Tauri webview security:** iframe to localhost should work within Tauri's
  CSP. May need to add `http://localhost:7420` to `tauri.conf.json` CSP
  if not already allowed.

## Out of scope

- Editing overlay config from the group preview (that's per-countdown in
  `AppearancePanel`).
- Preview for individual countdowns (separate issue: 008).

## Verification

- Open a group panel → preview shows the overlay as OBS would see it.
- Change group layout (row ↔ column) → preview updates live.
- Start/stop a countdown in the group → preview reflects state change.
- hide-idle enabled, all countdowns idle → preview shows empty/minimal.
