# Owlerlay — Issue Tracker

Scoped issues from user testing (2026-06-29).

## Bugs

| # | Title | Priority | Status |
|---|---|---|---|
| [001](001-persist-groups.md) | Persist groups across restarts | High | Open |
| [002](002-persist-overlay-config.md) | Persist overlay config (countdown design) | High | Open |

## Features

| # | Title | Priority | Status | Tag |
|---|---|---|---|---|
| [003](003-auto-reset.md) | Auto-reset on countdown finish | Medium | Open | backend |
| [004](004-dashboard.md) | Dashboard overview in control hub | Medium | Open | frontend-design |
| [005](005-countdown-icon-label.md) | Icons + labels per countdown | Medium | Open | frontend-design |
| [006](006-preview-per-group.md) | Preview per group | Low | Open | frontend-design |
| [007](007-preview-per-countdown.md) | Preview per countdown | Low | Open | frontend-design |

## Dependency graph

```
002 (persist overlay config)
 ├── 005 (icon + label — needs persisted config)
 ├── 006 (group preview — better with persisted config)
 └── 007 (countdown preview — better with persisted config)

001 (persist groups)
 └── 004 (dashboard — needs groups to group by)
 
003 (auto-reset) — independent
```

## Suggested order

1. **001** + **002** — fix the persistence bugs first (blocking, most visible)
2. **003** — auto-reset (self-contained backend feature, quick win)
3. **004** — dashboard (leverages persisted groups from 001)
4. **005** — icon + label + layout (leverages persisted config from 002)
5. **006** + **007** — previews (polish, low priority)
