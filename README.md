# Owlerlay 🦉

An **OBS overlay control center** for streamers. Owlerlay is a desktop app that
manages on-screen widgets — countdowns today, more on the way — and serves them
to OBS browser sources over a local HTTP/SSE server with live updates.

## Stack

| Layer | Tech |
|-------|------|
| Frontend | Svelte 5 (runes), TypeScript (strict), Vite 7, PicoCSS |
| Backend | Rust (Edition 2024), Tauri 2 |
| Overlay server | Axum on `:7420`, Jinja2 templates (`minijinja`), Server-Sent Events |
| Package manager | pnpm |

## How it works

1. The Tauri app manages countdowns (create / start / pause / resume / reset) and
   **overlay groups** — named sets of countdowns rendered together.
2. A local Axum server renders each group as an OBS browser-source page:
   `http://localhost:7420/overlay?group=<id>`.
3. The page subscribes to `/sse/group/<id>` and updates timers and progress bars
   live, with no polling.

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
| `src-tauri/src/countdown/` | Countdown state machine, service, Tauri commands |
| `src-tauri/src/overlay/` | Overlay groups + per-countdown config, commands |
| `src-tauri/src/server/` | Axum overlay/SSE server |
| `src-tauri/templates/overlay/` | Jinja2 overlay templates |

## Roadmap

See [AGENTS.md](./AGENTS.md) — polish countdown styling, LAN remote control,
alarms, Twitch integration, alerts, and a companion avatar.

## Contributing

Conventional Commits (`feat:`, `fix:`, …). Collaboration rules and conventions
live in [AGENTS.md](./AGENTS.md).
