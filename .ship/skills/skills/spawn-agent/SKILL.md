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

Auto-detection checks: `$WT_SESSION` → wt, `$TMUX` → tmux, `$TERM_PROGRAM` → iterm/vscode/apple-terminal, `gnome-terminal` on PATH → gnome. Set `SHIP_DEFAULT_TERMINAL` to override.

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

### Confirm flow

With `--confirm` or `SHIP_DISPATCH_CONFIRM=1`, the script provisions the worktree and writes the spec, then pauses:

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Job: jsonc-config
  Agent: rust-compiler
  Title: JSONC Config Format Migration
  Mode: autonomous
  Acceptance criteria: 18
  Scope:
    - crates/core/compiler/src/manifest.rs
    - apps/ship-studio-cli/src/init.rs
    ...
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Launch agent? [y/N/e(dit spec)]
```

- **y** — compiles agent and opens terminal
- **n** — aborts, worktree preserved for later
- **e** — prints spec path so you can edit, then re-run

## Batch dispatch

Dispatch multiple jobs from a directory of spec files:

```bash
for spec in .ship-session/specs/*.md; do
    slug=$(basename "$spec" .md)
    agent=$(head -20 "$spec" | grep -A1 '## Agent' | tail -1 | tr -d ' ')
    bash scripts/dispatch.sh --slug "$slug" --agent "${agent:-default}" --spec "$spec"
done
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

## Manual sequence (reference)

If the script doesn't fit your situation, the steps are:

### 1. Read the job
```
list_jobs()  — find the job, get full payload
```

### 2. Name the worktree
Derive a slug from the job title: lowercase, hyphen-separated, 2-4 words.

### 3. Create the worktree
```bash
git worktree add -b job/<slug> ~/dev/ship-worktrees/<slug> <base-branch>
```

### 4. Write the job spec
```bash
mkdir -p ~/dev/ship-worktrees/<slug>/.ship-session
cp <spec-file> ~/dev/ship-worktrees/<slug>/.ship-session/job-spec.md
```

### 5. Compile the agent config
```bash
cd ~/dev/ship-worktrees/<slug> && ship use <agent>
```

### 6. Update job status
```
update_job(id="<job-id>", status="running", claimed_by="<your-provider-id>")
```

### 7. Launch
```
cd ~/dev/ship-worktrees/<slug> && claude .
```

The agent reads `.ship-session/job-spec.md` automatically and starts.

## Stale Worktree Cleanup

After gate passes:
```bash
git worktree remove ~/dev/ship-worktrees/<slug>
git branch -d job/<slug>
```

Or use `list_stale_worktrees()` to find idle worktrees > 24h.

## Agent Teams

Claude Code agent teams (`CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1`) stack multiple agents in the **same directory** — they share the working tree. This is not a replacement for worktrees.

Use agent teams for: research, review, debate, investigation where file isolation is not needed.
Use worktrees for: parallel implementation, long-running jobs, agents that need their own compiled agent.

## Troubleshooting

**Agent starts but has no MCP tools:** `.mcp.json` wasn't generated. Run `ship use <agent>` again in the worktree, then restart the Claude session.

**`claude .` opens but has no MCP tools:** The `.mcp.json` wasn't generated. Run `ship use <agent>` again in the worktree, then restart the Claude session in that directory.

**Agent can't see job queue:** MCP server not running or wrong project path. Confirm `ship mcp serve` is on `$PATH` (`which ship`) and `.mcp.json` is configured to run `ship mcp serve`.

**`ship mcp serve` not found:** The MCP server is part of the `ship` binary. Confirm with `ship mcp serve --help`. If missing, reinstall: `cargo install --path apps/cli`.

**Auth missing in spawned terminal:** Run `claude auth login` in the new terminal. With Claude Code Max, auth is per-machine OAuth — no API key needed.
