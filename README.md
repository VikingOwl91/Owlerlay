# Owlerlay 🦉

An **OBS overlay control center** for streamers. Owlerlay is a desktop app that
manages on-screen widgets — countdowns today, more on the way — and serves them
to OBS browser sources over a local HTTP/SSE server with live, flicker-free
updates.

## Stack

| Layer | Tech |
|-------|------|
| Frontend | Svelte 5 (runes), TypeScript (strict), Vite 7 |
| Styling | Bespoke CSS — the "Night-Owl Control Room" theme (design tokens, no UI framework); self-hosted variable fonts via `@fontsource` |
| Backend | Rust (Edition 2024), Tauri 2 |
| Overlay server | Axum on `:7420`, Jinja2 templates (`minijinja`), Server-Sent Events |
| Package manager | pnpm |

## How it works

1. The Tauri app manages countdowns (create / start / pause / resume / reset) and
   **overlay groups** — named sets of countdowns rendered together as one OBS
   browser source.
2. A local Axum server renders each group as a page:
   `http://localhost:7420/overlay?group=<id>`.
3. The page subscribes to `/sse/group/<id>` and patches timers and progress bars
   in place — no polling, and no source reloads on start/pause/resume/finish, so
   the overlay never flickers on stream.

## Overlay widgets

Each countdown is styled independently from the control center's **Appearance**
panel, and the chosen look is baked into the served page:

- **Time format** — Auto (strips leading zero units: `05:03`, `1:05:03`,
  `2d 03:14:50`), or fixed `DD:HH:MM:SS` / `HH:MM:SS` / `MM:SS` / Seconds.
- **Typography** — five bundled, self-hosted fonts (Spline Sans Mono, Hanken
  Grotesk, Bricolage Grotesque, Quicksand, Fraunces) that render identically on
  any OBS machine, with a live preview in the picker. Tabular digits, so the
  timer never jitters between ticks.
- **Progress bar** — optional capsule bar locked to the timer's width, with
  configurable fill / track / divider colours.
- **Container** — text colour, font size, icon + icon size, background
  (transparent or solid), border, corner radius, backdrop blur, drop shadow.
- **Groups** — flex row/column layout and "hide idle countdowns".

Styling is served with `Cache-Control: no-store`, so a config change re-renders
the page instantly without a manual cache-bypassing reload.

## Development

```bash
pnpm install            # install JS deps
pnpm tauri dev          # run the full app in dev mode
pnpm dev                # frontend only (http://localhost:1420)
pnpm build              # tsc + Vite build
pnpm check              # svelte-check + tsc

# Rust
cargo test  --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo fmt   --manifest-path src-tauri/Cargo.toml
```

## Layout

| Path | Purpose |
|------|---------|
| `src/features/countdown/` | Countdown UI (api / model / state / components) |
| `src/features/overlay/` | Overlay-group manager UI |
| `src/app/` | App shell + the Night-Owl design tokens (`styles/`) |
| `src-tauri/src/countdown/` | Countdown state machine, service, Tauri commands |
| `src-tauri/src/overlay/` | Overlay groups + per-countdown config, commands |
| `src-tauri/src/server/` | Axum overlay/SSE server |
| `src-tauri/templates/overlay/` | Jinja2 overlay templates |
| `public/fonts/` | Self-hosted widget fonts (served at `/static/fonts/`) |

## Roadmap

Polish countdown styling is **done**. Next up: **LAN remote control** (drive
timers from a phone on the same network), then alarms, Twitch integration,
alerts, and a companion avatar. The full ordered roadmap and collaboration rules
live in [AGENTS.md](./AGENTS.md).

## Contributing

Conventional Commits (`feat:`, `fix:`, …). Collaboration rules and conventions
live in [AGENTS.md](./AGENTS.md).
