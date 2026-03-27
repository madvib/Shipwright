---
name: spawn-agent
stable-id: spawn-agent
description: Dispatch a job to a specialist agent in a git worktree. Creates the worktree, compiles the agent config, writes the job spec, and gives the human a ready-to-paste launch command.
tags: [commander, orchestration, worktree, dispatch]
authors: [ship]
---

# Spawn Agent

Dispatch a job to a specialist agent in a git worktree. Idempotent — safe to re-run.

## Available scripts

- **`scripts/dispatch.sh`** — Creates worktree, writes job spec, compiles agent, opens terminal. One command does everything.

## Quick dispatch (preferred)

Write a job spec file, then dispatch:

```bash
bash scripts/dispatch.sh --slug jsonc-config --agent rust-compiler --spec /path/to/spec.md
```

Options:
- `--slug <name>` — Worktree and branch name (required)
- `--agent <agent>` — Ship agent profile (required)
- `--spec <file>` — Path to job-spec.md (required)
- `--base <branch>` — Branch to fork from (default: current branch)
- `--dir <path>` — Worktree base directory (overrides configured default)
- `--no-open` — Skip terminal auto-open, print launch command instead
- `--confirm` — Show spec and ask y/n before launching
- `--dry-run` — Show what would happen

## User preferences

{% if terminal == "auto" %}
Terminal auto-detected from environment: `$WT_SESSION` → wt, `$TMUX` → tmux, `$TERM_PROGRAM` → iterm/vscode/apple-terminal, `gnome-terminal` on PATH → gnome.
{% else %}
Terminal: **{{ terminal }}** (configured via `ship vars set spawn-agent terminal`).
{% endif %}

Worktree base: **{{ worktree_dir }}**

{% if confirm_on_dispatch %}
Dispatch confirmation is **on** — you will be shown the job spec and prompted before launch.
{% else %}
Dispatch confirmation is **off** — agents launch immediately.
{% endif %}

To change any of these:
```bash
ship vars set spawn-agent terminal <value>        # auto, wt, iterm, tmux, gnome, vscode, manual
ship vars set spawn-agent worktree_dir <path>
ship vars set spawn-agent confirm_on_dispatch true
```

## Agent selection

| Work type | Agent |
|-----------|---------|
| Rust runtime / DB / platform | `rust-runtime` |
| Rust compiler / CLI | `rust-compiler` |
| Web / React / Studio | `web-lane` |
| Cloudflare Workers / infra | `cloudflare` |
| Auth / Better Auth | `better-auth` |
| Default / mixed | `default` |

## Stale worktree cleanup

After gate passes:
```bash
git worktree remove {{ worktree_dir }}/<slug>
git branch -d job/<slug>
```
