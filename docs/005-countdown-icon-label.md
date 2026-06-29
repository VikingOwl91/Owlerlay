# 005 — Icons + labels per countdown (configurable layout)

**Type:** Feature (frontend-design)  
**Reported by:** Tester  
**Priority:** Medium  
**Affects:** Countdown detail, sidebar rows, overlay templates

## Problem

Countdowns currently display only their name in the overlay. Users want:
- A **label** distinct from the countdown name (e.g. name = "Stream Timer",
  label = "⏱ Time Remaining" shown in overlay).
- An **icon** per countdown.
- **Configurable layout** of icon, label, and timer — at minimum:
  - Icon left (two rows height), label top-right, timer bottom-right
  - Flipped (icon right, label/timer stack on the left)
  - Icon + label top row, timer below
  - Timer top row, icon + label below
  - Plus a "custom" mode that unlocks the underlying flex properties
    (direction, wrap, justify, align) for anything beyond the presets.

## Current state

- `Countdown` struct has only `label`. The existing
  `OverlayConfig.icon: String` carries an icon (already a single field,
  emoji-via-string).
- The overlay template (`countdown.css.j2` / `countdown.html.j2`) renders
  icon + timer in a single flex row; no label element, no layout variant.

## Terminology (to set the record straight)

- **Countdown label** = the `Countdown::label` field already on the model
  (the name the streamer types in the create form).
- **Overlay icon_label** = the new `OverlayConfig` field that drives the
  on-stream label string shown to viewers. May be empty / different from
  the countdown label. This is what 005 adds.

## Scope

### Backend (data model)

1. Add `icon_label: String` and `overlay_layout: OverlayLayout` to
   `OverlayConfig`. Default `icon_label = ""`; default `OverlayLayout =
   IconLeftTwoRow`.
2. Add the four free-form flex fields, valid only when
   `OverlayLayout::Custom` is selected:
   - `flex_direction: String` — default `"row"`.
   - `flex_wrap: String` — default `"nowrap"`.
   - `align_items: String` — default `"center"`.
   - `justify_content: String` — default `"center"`.
3. Update `Default for OverlayConfig` so missing fields fall back to
   sane defaults; `#[serde(default)]` on the struct (already present)
   handles partial payloads.

### Backend (overlay layout enum)

```rust
// src-tauri/src/overlay/model.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OverlayLayout {
    IconLeftTwoRow,    // icon left (spans 2 rows), label top-right, timer bottom-right
    IconRightTwoRow,   // mirror
    IconLabelTop,      // icon + label on top row, timer below
    TimerTop,          // timer on top row, icon + label below
    Custom,            // free-form (uses the four fields above)
}
impl Default for OverlayLayout { fn default() -> Self { OverlayLayout::IconLeftTwoRow } }
```

### Overlay templates

4. Update `countdown.html.j2` to emit `<img class="cd-icon">`,
   `<span class="cd-label">`, and `<span class="cd-timer">` inside
   whichever wrapper structure the chosen `OverlayLayout` prescribes.
   When `Custom`, the wrapper gets inline `style="flex-direction: ...;
   flex-wrap: ...; align-items: ...; justify-content: ..."` from the
   four free-form fields.
5. Update `countdown.css.j2` to scope `#countdown-<id> { ... }` with
   the layout-variant class (`layout-icon-left`, etc.); the per-id
   rules stay sc — they only set the visual properties, not the
   layout.
6. **Strict-undefined render test.** Extend `tests/overlay_render.rs`
   so the test `C` struct adds all the new fields, then renders **all
   four presets + Custom** under `UndefinedBehavior::Strict` (the same
   guard the test already uses, which caught the prior
   stale-template-vs-route drift). Without this, the next "add a
   field" change can ship broken again.

### Frontend

7. Add `iconLabel` and `overlayLayout` (+ the four custom fields) to
   the TS `OverlayConfig` mirror, the JS-side composition in
   `AppearancePanel.svelte`, and the resulting
   `set_overlay_config` payload.
8. New "Layout" segment in `AppearancePanel.svelte`:
   - A visual preset picker (4 cards showing a mini-diagram of each
     preset, not a dropdown).
   - When `Custom` is selected, reveal four free-form flex controls.
9. Read-only display in `Roost.svelte` — the sidebar strip already
   shows the pip + name + time; the per-countdown icon + icon_label
   are nice-to-have here but not required by the spec ("at-a-glance
   identification" is already satisfied by the pip and label).

## Design decisions

- **Layout enum + one Custom escape hatch.** 4 named presets cover the
  cases the user spelled out, and `Custom` gives power users full
  flex control without forcing every preset to expand into its own
  enum variant forever. This matches the user's choice: "4 presets +
  free-form".
- **All layout fields persisted** on `OverlayConfig`. They live next
  to the existing styling fields and round-trip through the same
  overlay store (issue 002).
- **Icon stays a string** (filename for the bundled icons in
  `public/icons/` or an emoji fallback). Image-file icons are
  deferred; the "store media by path reference" persistence rule
  from AGENTS.md will apply when they land.
- **No `label` field on `Countdown`.** The internal `label` already
  exists; 005 only adds the *display* label (`icon_label`) on the
  styling side. The internal label and the overlay display label can
  drift independently — that's the point.

## Out of scope

- Image/file icons (store media by path reference when landed).
- Per-widget free-form CSS (covered by the `Custom` flex fields).
- Per-widget Tailwind/utility-class overrides.

## Out of scope

- Image/file icons (use emoji / bundled SVG until the per-path
  reference scheme lands — see AGENTS.md persistence rules).
- Layout layouts larger than the 4 + 1 chosen here (e.g. grid
  arrangements). The Custom flex fields are the ceiling.
- A live in-overlay `Design` button or any inline-OBS editing.

## Verification

- Set `icon_label` (overlay) different from the countdown's internal
  `label` → sidebar shows the internal label, overlay shows
  `icon_label`.
- Pick each layout variant → render test asserts the wrapper class
  matches and the page emits the expected icon/label/timer element
  order. Manual OBS browser-source test confirms the visual.
- Restart app → `icon_label`, `overlay_layout`, and the four
  Custom fields are persisted (requires 002).
- Frontend preset picker is reachable from
  `CountdownDetail.svelte` → `AppearancePanel.svelte` and persists
  on change without losing the rest of the styling.
