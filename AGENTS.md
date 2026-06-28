# Repository Guidelines

## Vision

**Owlerlay** is a Tauri-based **OBS overlay control center** for streamers. It is
a desktop app that manages on-screen widgets (countdowns today, more later) and
serves them to OBS browser-sources over a local HTTP/SSE server. The countdown
feature is just the first of several planned widgets.

**Goals:**
- Control center for OBS browser-source overlays, with live updates.
- Low resource consumption, cross-platform.
- Extensible toward a plugin-style feature set.

---

## Architecture

A Tauri 2 desktop app: Svelte/TypeScript frontend + Rust backend, plus a local
Axum web server that renders and serves overlays to OBS.

**Frontend — `src/`** (Svelte 5, TypeScript strict, Vite 7, PicoCSS)
- Organized by feature. The countdown feature lives in
  `src/features/countdown/`, split into `api/` (Tauri command wrappers),
  `model/` (types + DTO mappers), `state/` (Svelte stores), and `components/`.
- Cross-cutting code in `src/shared/` (e.g. the generic `invoke` wrapper,
  duration helpers).
- App shell / routing in `src/app/shell/AppShell.svelte`.

**Backend — `src-tauri/src/`** (Rust Edition 2024, Tauri 2)
- Tauri commands in `countdown/commands.rs`, registered in `lib.rs`.
- State machine in `countdown/model.rs` (Idle→Running→Paused→Finished) and
  `countdown/service.rs` (in-memory store, 100ms ticker).
- Shared state in `app_state.rs` (clock anchor, countdown service, broadcast
  event bus, overlay configs).

**Overlay server — `src-tauri/src/server/`**
- Axum HTTP server bound to `:7420`, started at app launch.
- Renders Jinja2 templates from `src-tauri/templates/overlay/` (via `minijinja`).
- Pushes live updates to OBS over Server-Sent Events (SSE). JavaScript in the
  templates is kept minimal — just an `EventSource` listener. (Note: earlier docs
  mentioned HTMX; the implementation uses plain SSE, not HTMX.)

**Data flow:**
UI `invoke()` → Tauri command → `CountdownService` mutates state → broadcast
event bus → (Tauri events back to the UI) **and** (SSE out to OBS overlays).

---

## Roadmap

Ordered, near-term first:

1. **Polish countdown** — wire the rich overlay options the templates already
   support (progress bars, font size, dividers, hide-idle) through
   `set_overlay_config` and the UI; clean up dead code; harden the overlay server.
2. **LAN remote control (web UI)** — control timers from a phone/tablet on the
   same network via a page the backend serves. The Axum server already binds the
   LAN and broadcasts state over SSE; needs only POST control routes calling the
   existing `CountdownService` plus a small mobile-friendly page. Decide HTTP
   write-access scoping/auth when scoping (today the server is read-only,
   CORS allows any origin).
3. **Alarms / scheduled events** — time-of-day triggers, not just countdowns.
4. **Twitch integration** — react to Twitch events (subs/follows/points) to drive
   overlays and timers.
5. **Alerts / notifications** — on-screen alert overlays for OBS (e.g. new follower).
6. **Companion (avatar)** — scope TBD, next-up not now. Basis is the proof at
   `~/Dev/3D_Models/AndreIsohedronCephalon/`: a Three.js audio-reactive 3D avatar
   with in-browser Kokoro TTS. Becomes a responsive companion for voice/alert
   animations.

---

## Working agreement (Hybrid)

This was a learning-only repo; it is now built collaboratively in a **Hybrid**
model.

- The assistant **may** write and modify any code (frontend, Rust, tests).
- For non-trivial work, the assistant proposes a brief plan first; the owner
  decides who implements ("you do it" / "I'll do it" — the owner may claim parts
  they want to learn). When unspecified, default to: assistant plans, owner picks.
- The assistant must read the current, on-disk source before answering technical
  questions or reviewing — never rely on memory from earlier turns.
- The assistant may edit documentation any time it's helpful.

---

## Project Structure & Module Organization
- `src/`: Svelte/TS frontend (feature folders + `src/shared/`, `src/app/`).
- `src-tauri/src/`: Rust entry points, Tauri commands, state, overlay server.
- `src-tauri/templates/overlay/`: Jinja2 overlay templates.
- `src-tauri/icons/`: app icons for desktop bundles.
- `src-tauri/tauri.conf.json`: Tauri app/build configuration.
- `dist/` and `src-tauri/target/`: build outputs (generated; do not edit).

## Build, Test, and Development Commands
- `pnpm install`: install JavaScript dependencies.
- `pnpm dev`: Vite web dev server at `http://localhost:1420`.
- `pnpm tauri dev`: run the full app in development mode.
- `pnpm build`: type-check (`tsc`) and build frontend into `dist/`.
- `pnpm check`: `svelte-check` + `tsc`.
- `pnpm format` / `pnpm format:check`: Prettier write / verify.
- `pnpm tauri build`: create desktop bundles.
- `pnpm preview`: preview the frontend bundle.
- `cargo test --manifest-path src-tauri/Cargo.toml`: run Rust tests.

## Coding Style & Naming Conventions
- TypeScript/Svelte: formatted by Prettier (`prettier-plugin-svelte`,
  2-space) — run `pnpm format` before committing. `strict` mode; prefer explicit
  types at API boundaries. `camelCase` vars/functions, `PascalCase` types.
- Rust: Edition 2024, `rustfmt` defaults (4-space), `snake_case` functions/modules.
- Keep Tauri commands small and side-effect focused; register them in `lib.rs`.
- Name files by feature.

## Testing Guidelines
No JS test framework is configured yet. For new features:
- add Rust tests in `src-tauri/tests/` (integration tests against each module's
  public API — keep them out of `src-tauri/src/`, no `#[cfg(test)]` blocks inside
  source files);
- add frontend tests only for non-trivial UI/state logic (Vitest preferred when
  introduced);
- include manual verification steps in PRs (OS, command run, expected behavior).

## Commit & Pull Request Guidelines
Use Conventional Commits:
- `feat: add tray menu action`
- `fix: handle empty greet input`

PRs should include a short problem/solution summary, linked issues when relevant,
screenshots/recordings for UI changes, and the exact verification commands run.

## Known cleanup backlog
Tracked, not yet done (fix opportunistically):
- About page is a one-line stub (`src/app/shell/AppShell.svelte`).
- Overlay server origin is centralized in `src/shared/overlay/origin.ts` but
  still hardcoded to `localhost:7420` — make it env/settings-configurable (needed
  for LAN remote control, roadmap item 2).
- Overlay config gap: `set_overlay_config` stores only icon/colors/HH:MM while
  `OverlayConfig` (and the templates) support progress bars, fonts, dividers,
  borders, etc. — wiring these to the UI is roadmap item 1.
- No sanitization of overlay config strings into Jinja2 (low risk, local-only).

Cleared in the overlay-migration pass: dead `view.ts`/`sse.rs` removed, unused
crates (chrono/toml/tracing) dropped, `minijinja` render no longer panics
(returns errors + strict undefined), README rewritten, frontend on Svelte 5
runes, Prettier adopted project-wide (2-space, `prettier-plugin-svelte`).
