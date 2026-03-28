---
title: "Workflow Patterns"
description: "The three-mode workflow -- Planning, Orchestration, and Gate -- and how workspaces, sessions, and jobs fit together."
sidebar:
  label: "Workflow Patterns"
  order: 4
---
Ship's workflow is built on three building blocks (targets, capabilities, jobs) and three operational modes (planner, orchestrator, gate). This guide explains how they fit together.

## The Three Building Blocks

### Target

The north star for a phase of work. Describes the desired end state in plain language:

> "Ship a working auth flow with GitHub OAuth and session management by end of March."

Targets don't change often. Every capability and job is evaluated against the active target. Two kinds exist:

- **Milestones** are time-bounded (v0.1, v0.2). They have due dates and represent shipping checkpoints.
- **Surfaces** are evergreen capability domains (Compiler, Studio, Registry). They accumulate capabilities over time.

### Capability

A verifiable slice of a target. Each capability has:
- A title (e.g. "GitHub OAuth login flow")
- Acceptance criteria -- a checklist you can actually verify
- A phase -- which milestone it belongs to
- A scope -- which files/directories an agent may touch

Capabilities start **aspirational** and become **actual** once a gate passes with evidence. Use `mark_capability_actual` with a test name, commit hash, or observable behavior as proof.

### Job

The execution record for a unit of work. Tied to a capability, assigned to an agent or human. Moves through: `pending` then `running` then `complete` (or `failed` / `blocked`).

Jobs carry file scope declarations and ownership claims. When two jobs need the same file, `claim_file` enforces first-wins arbitration.

## The Three Modes

### Planner Mode

Active at session start or when a new goal is given. The planner reads the active target, asks focused questions if needed, and builds the capability map.

```
Capability 1: GitHub OAuth provider setup
  Agent: better-auth
  Scope: apps/web/src/auth/
  Done when: OAuth flow completes in dev, tokens stored correctly

Capability 2: Login/logout UI
  Agent: web-lane
  Scope: apps/web/src/components/auth/
  Done when: login button visible, session persists on refresh
```

MCP tools used in planning:
- `open_project` -- register the project
- `create_target` / `get_target` -- define or review the north star
- `create_capability` -- break the target into verifiable slices
- `create_note` / `create_adr` -- record planning decisions

### Orchestrator Mode

Active when jobs are running. The orchestrator monitors progress, surfaces blockers, and routes work between agents. It does not do specialist work -- it routes.

Each job runs in its own git worktree -- an isolated copy of the repo on a dedicated branch.

```
create_workspace(name="auth-flow", kind="imperative", base_branch="main")
create_job(kind="feature", description="Implement GitHub OAuth",
           branch="auth-flow", file_scope=["apps/web/src/auth/"],
           acceptance_criteria=["OAuth flow completes", "Tokens stored correctly"])
```

MCP tools used in orchestration:
- `create_workspace` / `activate_workspace` -- set up isolated work environments
- `create_job` / `update_job` -- dispatch and track work
- `list_jobs` -- monitor queue state
- `append_job_log` -- record progress within a job
- `claim_file` / `get_file_owner` -- prevent file conflicts across parallel agents
- `list_events` -- watch for state changes across the system

### Gate Mode

Active when an agent marks a job done. The gate reviews the work against the capability's acceptance criteria.

- **Pass**: capability becomes actual, worktree pruned, job marked complete.
- **Fail**: job blocked, specific failures surfaced. Only failures that need a human decision bubble up.

```
mark_capability_actual(id="cap_abc123",
                       evidence="test_github_oauth_flow passes, commit a1b2c3d")
update_job(id="job_xyz789", status="complete")
complete_workspace(workspace_id="auth-flow",
                   summary="GitHub OAuth flow implemented and tested")
```

## Session Lifecycle

Sessions track agent activity within a workspace. The lifecycle is:

1. `start_session` -- begin work, declare the goal
2. Work -- call tools, edit files, run tests
3. `log_progress` -- record meaningful milestones during work
4. `end_session` -- summarize what was accomplished

One active session per workspace at a time. Sessions produce immutable records that feed the event log.

## Job Coordination

### File Ownership

When multiple agents work in parallel, file ownership prevents conflicts:

```
claim_file(job_id="job_abc", path="apps/web/src/auth/provider.ts")
```

This is atomic and first-wins. If another job already owns the file, the claim returns an error. Use `get_file_owner` to check before claiming.

The shorthand via job logs also works:
```
append_job_log(job_id="job_abc", message="touched: apps/web/src/auth/provider.ts")
```

### Job Dependencies

Jobs can declare dependencies using `blocked_by`:

```
create_job(kind="feature", description="Build login UI",
           blocked_by="job_abc")
```

The blocked job stays in `pending` until the blocking job completes.

### Priority

Higher priority numbers run first. Use priority to express scheduling intent when multiple jobs compete for the same agent.

## Putting It Together

A typical workflow for a new feature:

1. **Plan**: Read the target, break it into capabilities, confirm the approach
2. **Dispatch**: Create workspaces and jobs for each capability
3. **Monitor**: Watch job progress via logs and events
4. **Gate**: Review completed work against acceptance criteria
5. **Ship**: Mark capabilities actual with evidence, complete workspaces

The event log (`list_events`) provides a complete audit trail of every state change across all workspaces, sessions, and jobs.
