# 006 — Preview per group

**Type:** Feature (frontend-design)  
**Reported by:** Tester  
**Priority:** Low  
**Affects:** Group panel UI

## Problem

When configuring a group, the user has no preview of what the OBS overlay
will look like. They have to switch to OBS (or open the overlay URL in a
browser) to see the result. A preview embedded in the group panel
dramatically speeds up configuration.

## Current state

- `GroupPanel.svelte` shows: name, layout toggle (row/column), hide-idle
  toggle, member checkboxes, and an OBS URL with copy button.
- The overlay is rendered server-side by Axum at `/overlay?group={id}`
  using Jinja2 templates.
- No preview exists anywhere in the frontend.

## Scope

1. New shared `src/shared/preview/PreviewTile.svelte` component
   consumed by both `GroupPanel.svelte` (this issue) and
   `CountdownDetail.svelte` (007). It mimics the OBS countdown DOM —
   icon + label + digit-styled timer + optional progress bar — using
   the live `OverlayConfig` fields (with `iconLabel`, the layout
   preset, and the progress bar toggle wired through). Live `duration`
   comes from the existing `countdownStore.liveRemaining` for the
   selected countdown, or the per-item `remaining` snapshot otherwise;
   the tile does NOT subscribe to the SSE tick (see "Static mock"
   below).
2. Wire `PreviewTile.svelte` into `GroupPanel.svelte` above the URL
   row (single tile showing the group's first member, since a group
   preview that loops every member would either be tiny or scrollable).
3. Style the preview container to feel like an OBS source window —
   fixed-aspect-ratio dark card with a small "Preview" label and
   a "Open in browser" link alongside (the link still uses
   `OVERLAY_SERVER_ORIGIN + /overlay?group=<id>`, unchanged).
4. The preview must be **sourced from the persisted config** — relies
   on issue 002 to round-trip `OverlayConfig` across the relevant
   loads. If 002 isn't in, the preview always shows defaults.

## Design decisions

- **Static mock over iframe.** iframe was the originally proposed path
  but considered against a Svelte-native mock: the iframe approach is
  pixel-identical to OBS but hits the overlay server, depends on
  Tauri's CSP allowing `http://localhost:7420`, requires per-member
  layout handling for the OBS source, and won't reflect preview-time
  changes live (the SSE-driven reload races with edits). A Svelte
  component is cheaper, lets us use the same Owlerlay tokens as the
  rest of the control UI, and refreshes in step with the controls.
  Pixel parity with OBS is "good enough" rather than "exact", which
  is acceptable because the user editing widget appearance is
  reviewing intent, not chasing rendering differences. The "Open in
  browser" link is the escape hatch for exact parity when needed.
- **No SSE subscription.** The preview is a snapshot, not a live
  readout; that's the desktop UI's job. Keeps the Svelte tree cheap
  and avoids spurious re-renders during typing.
- **Render test stays the source of truth for OBS fidelity.** This
  issue **does not** weaken `tests/overlay_render.rs` — any drift
  between the Svelte preview and the OBS render must show up there
  first, then the preview follows.

## Dependencies

- Issue 002 (persisted overlay config) so the preview reflects
  saved settings across app restarts.
- Issue 005 (icon-label + layout presets) so the preview exposes
  the new fields. The preview can ship before 005 lands, but in
  that window it can only honour the existing single-row layout.

## Out of scope

- Editing overlay config from the group preview (that's per-countdown
  in `AppearancePanel`).
- Full-screen preview mode.
- Per-member previews in the group view (one preview per member
  would balloon the panel).

## Verification

- Open a group panel → preview renders the group's first member as
  the OBS countdown would, using the persisted `OverlayConfig`.
- Change layout preset → preview updates immediately.
- Change icon, color, font size → preview reflects the change without
  navigation.
- "Open in browser" link opens `http://localhost:7420/overlay?group=<id>`
  in a new tab/window — confirms the actual OBS endpoint is reachable.
- Manual check: open the same OBS URL in OBS → it matches the preview
  "as closely as the Svelte render can get" (see Design decisions).
- Render-test (`tests/overlay_render.rs`) regression check passes
  whenever a preview-affecting field is touched.
