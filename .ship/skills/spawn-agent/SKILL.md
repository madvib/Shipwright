---
name: spawn-agent
stable-id: spawn-agent
description: Dispatch a job to a specialist agent in a git worktree. Creates the worktree, compiles the agent config, writes the job spec, and gives the human a ready-to-paste launch command.
tags: [commander, orchestration, worktree, dispatch]
authors: [ship]
---

# Spawn Agent

Dispatch a job to a specialist agent in a git worktree. Idempotent — safe to re-run.

## Quick dispatch (preferred)

The `scripts/dispatch.sh` script does everything: creates worktree, writes job spec, compiles agent, opens terminal.

```bash
bash scripts/dispatch.sh \
  --slug jsonc-config \
  --agent rust-compiler \
  --spec /path/to/spec.md
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

{% if runtime.agents %}
## Available agents

{% for a in runtime.agents %}- **{{ a.id }}**{% if a.description %} — {{ a.description }}{% endif %}

{% endfor %}
{% endif %}
## Environment setup (mandatory)

Before launching any agent, dispatch verifies:

1. `SHIP_GLOBAL_DIR` is set to `$HOME/.ship`
2. `ship` CLI is on PATH — hard error if not found
3. `ship use <agent>` runs in the worktree and exits 0
4. `.mcp.json` exists in the worktree
5. `.mcp.json` contains `ship mcp serve` args

**If any check fails, dispatch stops and surfaces the error. The agent is not launched.**

An agent without MCP cannot access the Ship runtime. This is not recoverable after the fact.

## Test/impl separation for feature jobs

When the spec describes new behaviour, spawn **two sequential jobs**:

| Job | Slug | Input | Constraint |
|-----|------|-------|------------|
| 1 — tests | `<slug>-tests` | Spec + interface definition only | No implementation files in scope. Writes failing tests. |
| 2 — impl | `<slug>-impl` | Tests as spec | `blocked_by: <slug>-tests`. Makes tests pass. Never writes tests. |

Single-agent feature jobs are permitted only with `single-agent: true` in the spec and a noted reason.

The impl spec must reference the test job and list the test files as its authoritative contract.

## Job spec template

```markdown
# Job Spec: <title>

**Branch:** <branch>
**Agent:** <agent>
**Mode:** autonomous

## Goal

<one-paragraph description of the outcome>

## File scope

<list the files or directories the agent is allowed to modify>

## What to change

<specific instructions>

## Architectural context
- Active ADRs: <list relevant ADR IDs, or "none">
- Key constraints: <from CLAUDE.md or active ADRs, or "none">

## Acceptance criteria

<numbered list of verifiable outcomes>

---
> If you notice a bug or problem outside your file scope: append to the job log via
> `mcp__ship__append_job_log`, describe it specifically, continue your work.
> Never silently leave a noticed problem.
```

## Configuration

{% if terminal == "auto" %}
Terminal: auto-detected from environment (`$WT_SESSION` → wt, `$TMUX` → tmux, `$TERM_PROGRAM` → iterm/vscode, `gnome-terminal` on PATH → gnome).
{% else %}
Terminal: **{{ terminal }}**
{% endif %}

Worktree base: **{{ worktree_dir }}**

{% if confirm_on_dispatch %}
Dispatch confirmation is **on** — you will be shown the job spec and prompted before launch.
{% endif %}

```bash
ship vars set spawn-agent terminal <auto|wt|iterm|tmux|gnome|vscode|manual>
ship vars set spawn-agent worktree_dir <path>
ship vars set spawn-agent confirm_on_dispatch true
```

## Stale worktree cleanup

After gate passes:
```bash
git worktree remove {{ worktree_dir }}/<slug>
git branch -d job/<slug>
```
