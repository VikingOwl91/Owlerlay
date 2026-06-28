# CLAUDE.md

> This file configures Claude Code's behaviour in this repository.
> The canonical collaboration rules live in [AGENTS.md](./AGENTS.md).
> Claude must read and comply with AGENTS.md on every session.

---

## Role

**Hybrid model** — Claude builds collaboratively with the owner (see the full
working agreement in [AGENTS.md](./AGENTS.md)).

- **Do** write and modify code (frontend, Rust, tests).
- **Do** propose a brief plan before non-trivial work; the owner decides who
  implements (they may claim parts they want to learn).
- **Do** read the current on-disk source before answering or reviewing — never
  rely on memory from earlier turns.
- **Do** edit documentation whenever it's helpful.

---

## Project Overview

**Owlerlay** is a Tauri-based **OBS overlay control center** for streamers
(countdowns today, more on the roadmap — see AGENTS.md). Built with:

| Layer | Tech |
|-------|------|
| Frontend | Svelte 5, TypeScript (strict), Vite 7, PicoCSS |
| Backend | Rust (Edition 2024), Tauri 2 |
| Package manager | pnpm |

### Key paths

| Path | Purpose |
|------|---------|
| `src/` | Svelte/TS frontend |
| `src-tauri/src/` | Rust Tauri commands & state |
| `src-tauri/src/lib.rs` | Tauri builder & command registration |
| `src-tauri/src/countdown/` | Countdown feature (commands, logic) |
| `src-tauri/src/app_state.rs` | Shared app state |
| `src-tauri/tauri.conf.json` | Tauri configuration |

---

## Common Commands

```bash
pnpm install              # install JS deps
pnpm dev                  # Vite dev server (http://localhost:1420)
pnpm tauri dev            # full app in dev mode
pnpm build                # tsc + Vite build
pnpm tauri build          # desktop bundle
pnpm check                # svelte-check + tsc
cargo test --manifest-path src-tauri/Cargo.toml   # Rust tests
```

---

## Coding Conventions (summary — see AGENTS.md for full details)

- **TypeScript**: 2-space indent, strict mode, `camelCase` vars/fns, `PascalCase` types.
- **Rust**: 4-space indent (`rustfmt`), `snake_case` fns/modules, Edition 2024.
- Tauri commands: keep small and side-effect focused; register in `lib.rs`.
- Commit style: Conventional Commits (`feat:`, `fix:`, etc.).
