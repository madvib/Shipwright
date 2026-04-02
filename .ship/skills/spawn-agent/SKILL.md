---
name: dispatch
stable-id: spawn-agent
description: Dispatch a job to a specialist agent in a git worktree. Write the spec, create the worktree, launch the agent.
tags: [commander, dispatch, worktree]
authors: [ship]
---

# Dispatch

Launch a specialist agent in a git worktree to execute a job. One command does everything.

## Usage

```bash
bash scripts/dispatch.sh \
  --slug <name> \
  --agent <agent> \
  --spec <path-to-spec>
```

| Flag | What |
|------|------|
| `--slug <name>` | Worktree dir and branch name (required) |
| `--agent <agent>` | Ship agent profile (required) |
| `--spec <file>` | Path to job spec (required) |
| `--base <branch>` | Branch to fork from (default: current branch) |
| `--no-open` | Print launch command instead of opening terminal |
| `--dry-run` | Show what would happen |

## Workflow

1. Write the job spec to `.ship-session/job-spec-<slug>.md`
2. Run dispatch: `bash scripts/dispatch.sh --slug <slug> --agent <agent> --spec <path>`
3. Dispatch creates worktree, compiles agent config, opens terminal
4. Agent picks up `.ship-session/job-spec.md` on start and works autonomously

## Job Spec Template

```markdown
# Job Spec: <title>

**Branch:** job/<slug>
**Agent:** <agent>
**Mode:** autonomous

## Goal

<one-paragraph outcome>

## File scope

<directories/files the agent may modify>

## What to change

<specific instructions>

## Architectural context
- Active ADRs: <relevant ADRs or "none">
- Key constraints: <from CLAUDE.md or "none">

## Acceptance criteria

<numbered verifiable outcomes>

---
> If you notice a bug or problem outside your file scope, log it via
> `mcp__ship__log_progress` and continue your work.
```

## Test/Impl Split

For feature work, dispatch as two sequential jobs:

```bash
# Job 1: tests only
bash scripts/dispatch.sh --slug auth-tests --agent test-writer \
  --spec .ship-session/job-spec-auth-tests.md

# Job 2: implementation (after tests complete and gate passes)
bash scripts/dispatch.sh --slug auth-impl --agent rust-runtime \
  --spec .ship-session/job-spec-auth-impl.md
```

The test spec scopes to test files only. The impl spec references the test files as its contract and must not modify them.

{% if runtime.agents %}
## Available Agents

{% for a in runtime.agents %}- **{{ a.id }}**{% if a.description %} — {{ a.description }}{% endif %}
{% endfor %}
{% endif %}

## Environment

Dispatch verifies before launching:
1. `ship` CLI is on PATH
2. `ship use <agent>` succeeds in the worktree
3. `.mcp.json` exists with `ship mcp serve`

If any check fails, the agent is not launched.

{% if terminal == "auto" %}
Terminal: auto-detected (`$TMUX` → tmux, `$WT_SESSION` → wt, `$TERM_PROGRAM` → iterm/vscode).
{% else %}
Terminal: **{{ terminal }}**
{% endif %}

Worktree base: **{{ worktree_dir }}**

```bash
ship vars set dispatch terminal <auto|tmux|wt|iterm|vscode|manual>
ship vars set dispatch worktree_dir <path>
```

## Cleanup

After gate passes:
```bash
git worktree remove {{ worktree_dir }}/<slug>
git branch -d job/<slug>
```
