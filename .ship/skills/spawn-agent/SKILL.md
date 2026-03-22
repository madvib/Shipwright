---
name: spawn-agent
description: Dispatch a job to a specialist agent in a git worktree. Creates the worktree, compiles the agent config, writes the job spec, and gives the human a ready-to-paste launch command.
tags: [commander, orchestration, worktree, dispatch]
authors: [ship]
---

# Spawn Agent

Dispatch a job to a specialist agent in a git worktree. Idempotent — safe to re-run.

## Available scripts

- **`scripts/dispatch.sh`** — Creates worktree, writes job spec, compiles agent, opens terminal. One command does everything.

## Environment variables

Ship dispatch respects these env vars for user preferences. Set them in your shell profile.

| Variable | Values | Default | Purpose |
|----------|--------|---------|---------|
| `SHIP_DEFAULT_TERMINAL` | `wt`, `iterm`, `tmux`, `gnome`, `vscode`, `manual` | auto-detect | Which terminal to open new tabs in |
| `SHIP_DISPATCH_CONFIRM` | `1` | unset (no confirm) | Show spec summary and ask y/n before launching agent |
| `SHIP_WORKTREE_DIR` | path | `~/dev/ship-worktrees` | Default base directory for worktrees |

Auto-detection checks: `$WT_SESSION` -> wt, `$TMUX` -> tmux, `$TERM_PROGRAM` -> iterm/vscode/apple-terminal, `gnome-terminal` on PATH -> gnome. Set `SHIP_DEFAULT_TERMINAL` to override.

## Quick dispatch (preferred)

Write a job spec file, then dispatch:

```bash
bash scripts/dispatch.sh --slug jsonc-config --agent rust-compiler --spec /path/to/spec.md
```

This is idempotent. Running it again skips existing worktrees, only updates the spec if changed, and re-compiles the agent config.

Options:
- `--slug <name>` — Worktree and branch name (required)
- `--agent <agent>` — Ship agent profile (required)
- `--spec <file>` — Path to job-spec.md (required)
- `--base <branch>` — Branch to fork from (default: current branch)
- `--dir <path>` — Worktree base directory (overrides `SHIP_WORKTREE_DIR`)
- `--no-open` — Skip terminal auto-open, print launch command instead
- `--confirm` — Show spec and ask y/n before launching (or set `SHIP_DISPATCH_CONFIRM=1`)
- `--dry-run` — Show what would happen

## Agent selection

| Work type | Agent |
|-----------|---------|
| Rust runtime / DB / platform | `rust-runtime` |
| Rust compiler / CLI | `rust-compiler` |
| Web / React / Studio | `web-lane` |
| Cloudflare Workers / infra | `cloudflare` |
| Auth / Better Auth | `better-auth` |
| Default / mixed | `default` |

## Stale Worktree Cleanup

After gate passes:
```bash
git worktree remove ~/dev/ship-worktrees/<slug>
git branch -d job/<slug>
```

Or use `list_stale_worktrees()` to find idle worktrees > 24h.
