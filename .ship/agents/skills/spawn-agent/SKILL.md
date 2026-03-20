---
name: spawn-agent
description: Dispatch a job to a specialist agent in a git worktree. Creates the worktree, compiles the agent config, writes the job spec, and gives the human a ready-to-paste launch command.
tags: [commander, orchestration, worktree, dispatch]
authors: [ship]
---

# Spawn Agent

Dispatch a job to a specialist agent. Follow this sequence exactly.

## Prerequisites

- `claude` authenticated: `claude auth login` once per machine
- `ship` and `claude` on `$PATH`
- Job exists in the queue with description, scope, and profile hint

## Sequence

### 1. Read the job

```
list_jobs()  — find the job by ID, get full payload
```

CLI fallback: `ship job list`

> Never use raw `sqlite3` to access Ship data. The schema evolves. Always use MCP tools or `ship` CLI.

### 2. Resolve the worktree path

Default: `~/dev/ship-worktrees/<job-id>`

Check `~/.ship/config.toml` for override:
```toml
[worktrees]
dir = "~/dev/my-worktrees"
```

### 3. Create the worktree

```bash
git worktree add ~/dev/ship-worktrees/<job-id> -b job/<job-id>
```

Resuming a stalled job (branch exists):
```bash
git worktree add ~/dev/ship-worktrees/<job-id> job/<job-id>
```

### 4. Compile the agent config

```bash
cd ~/dev/ship-worktrees/<job-id> && ship use <profile>
```

This writes `CLAUDE.md`, `.mcp.json`, and permission files into the worktree. The agent reads them automatically at session start.

Profile from job payload (`preset_hint`), or:

| Work type | Profile |
|-----------|---------|
| Rust runtime / DB / platform | `rust-runtime` |
| Rust compiler / CLI | `rust-compiler` |
| Web / React / Studio | `web-lane` |
| Cloudflare Workers / infra | `cloudflare` |
| Auth / Better Auth | `better-auth` |
| Default / mixed | `default` |

### 5. Write the job spec

```bash
cat > ~/dev/ship-worktrees/<job-id>/job-spec.md << 'EOF'
# Job <JOB_ID> — <title>

Read this file first. It is your complete context.

## Mode
autonomous
# autonomous — begin immediately, log questions via append_job_log
# interactive — present your plan to the human, wait for approval before executing

## What
<description>

## Scope
File scope: <file_scope>
Profile: <profile>

## Acceptance Criteria
<acceptance_criteria checklist>

## Constraints
- Stay within declared file scope
- Log touched files: `append_job_log(id, "touched: path/to/file")`

## Completion Contract
When acceptance criteria are met, do all three in order:
1. Commit touched files: `git add <files> && git commit -m "complete: <title>"`
2. Write handoff.md to this directory (what you did, decisions made, anything incomplete)
3. `update_job(id="<JOB_ID>", status="complete")`

All three are required. Commander uses all three as completion signals.

## Context
Branch: job/<job-id>
Worktree: ~/dev/ship-worktrees/<job-id>
Profile: <profile>
Ship MCP is active in this directory.
EOF
```

The compiled `CLAUDE.md` includes a rule telling the agent to read `job-spec.md` immediately and begin without waiting for instruction.

### 6. Update job status

```
update_job(id="<job-id>", status="running", claimed_by="<your-provider-id>")
```

### 7. Give the human the launch command

Terminal auto-launch is unreliable across platforms. Give the human a clean, ready-to-paste command instead.

**Standard launch:**
```
cd ~/dev/ship-worktrees/<job-id> && claude .
```

**If the profile uses `default_mode = "bypassPermissions"`:**
```
cd ~/dev/ship-worktrees/<job-id> && claude . --dangerously-skip-permissions
```

Format this as a single copy block. The agent will read `job-spec.md` automatically and start — no further input needed from the human.

Tell the human:
```
Dispatched [<job-id>] <title>
→ worktree: ~/dev/ship-worktrees/<job-id>
→ profile: <profile>
→ launch:

  cd ~/dev/ship-worktrees/<job-id> && claude .

Paste in a new terminal tab. The agent will start automatically.
```

## Stale Worktree Cleanup

After gate passes:
```bash
git worktree remove ~/dev/ship-worktrees/<job-id>
git branch -d job/<job-id>
```

Or use `list_stale_worktrees()` to find idle worktrees > 24h.

## Agent Teams

Claude Code agent teams (`CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1`) stack multiple agents in the **same directory** — they share the working tree. This is not a replacement for worktrees.

Use agent teams for: research, review, debate, investigation where file isolation is not needed.
Use worktrees for: parallel implementation, long-running jobs, agents that need their own compiled profile.

## Troubleshooting

**Agent starts but has no MCP tools:** `.mcp.json` wasn't generated. Run `ship use <profile>` again in the worktree, then restart the Claude session.

**`ship mcp serve` not found:** The MCP server is part of the `ship` binary. Confirm with `ship mcp serve --help`. If missing, reinstall: `cargo install --path apps/cli`.

**Auth missing in spawned terminal:** Run `claude auth login` in the new terminal. With Claude Code Max, auth is per-machine OAuth — no API key needed.
