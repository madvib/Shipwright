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
- Job exists in the queue with description, scope, and agent hint

## Sequence

### 1. Read the job

```
list_jobs()  — find the job by ID, get full payload
```

CLI fallback: `ship job list`

> Never use raw `sqlite3` to access Ship data. The schema evolves. Always use MCP tools or `ship` CLI.

### 2. Name the worktree

Derive a short, human-readable slug from the job title. This is used for the branch name and worktree directory — the job ID stays in the queue for tracking.

```
Job title: "DB Consolidation + Dead Code Purge"  →  slug: "db-consolidation"
Job title: "Fix ship install/use dep resolution"  →  slug: "fix-dep-resolution"
Job title: "Events Rewrite"                       →  slug: "events-rewrite"
```

Rules:
- Lowercase, hyphen-separated, 2-4 words max
- No job IDs in the name — humans read directories
- If a slug collides with an existing branch, append a short disambiguator (e.g. `events-rewrite-2`)

### 3. Resolve the worktree path

Default: `~/dev/ship-worktrees/<slug>`

Check `~/.ship/config.toml` for override:
```toml
[worktrees]
dir = "~/dev/my-worktrees"
```

### 4. Create the worktree

```bash
git worktree add ~/dev/ship-worktrees/<slug> -b job/<slug>
```

Resuming a stalled job (branch exists):
```bash
git worktree add ~/dev/ship-worktrees/<slug> job/<slug>
```

### 5. Compile the agent config

```bash
cd ~/dev/ship-worktrees/<slug> && ship use <agent>
```

This writes `CLAUDE.md`, `.mcp.json`, and permission files into the worktree. The agent reads them automatically at session start.

Agent from job payload (`preset_hint`), or:

| Work type | Agent |
|-----------|---------|
| Rust runtime / DB / platform | `rust-runtime` |
| Rust compiler / CLI | `rust-compiler` |
| Web / React / Studio | `web-lane` |
| Cloudflare Workers / infra | `cloudflare` |
| Auth / Better Auth | `better-auth` |
| Default / mixed | `default` |

### 6. Write the job spec

```bash
cat > ~/dev/ship-worktrees/<slug>/job-spec.md << 'EOF'
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
Agent: <agent>

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
Branch: job/<slug>
Worktree: ~/dev/ship-worktrees/<slug>
Job ID: <JOB_ID> (use this for MCP calls)
Agent: <agent>
Ship MCP is active in this directory.
EOF
```

The compiled `CLAUDE.md` includes a rule telling the agent to read `job-spec.md` immediately and begin without waiting for instruction.

### 7. Update job status

```
update_job(id="<job-id>", status="running", claimed_by="<your-provider-id>")
```

### 8. Give the human the launch command

Terminal auto-launch is unreliable across platforms. Give the human a clean, ready-to-paste command instead.

**Standard launch:**
```
cd ~/dev/ship-worktrees/<slug> && claude .
```

**If the agent uses `default_mode = "bypassPermissions"`:**
```
cd ~/dev/ship-worktrees/<slug> && claude . --dangerously-skip-permissions
```

Format this as a single copy block. The agent will read `job-spec.md` automatically and start — no further input needed from the human.

Tell the human:
```
Dispatched [<JOB_ID>] <title>
→ worktree: ~/dev/ship-worktrees/<slug>
→ agent: <agent>
→ launch:

  cd ~/dev/ship-worktrees/<slug> && claude .

Paste in a new terminal tab. The agent will start automatically.
```

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
