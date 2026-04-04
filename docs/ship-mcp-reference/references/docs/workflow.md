---
group: MCP Tools
order: 4
title: Workflow Patterns
description: The three-mode workflow -- planning, orchestration, and gate -- using workspaces, sessions, and jobs.
---

# Workflow Patterns

Three operational modes use the MCP tools: planning (understand the work), orchestration (dispatch and monitor), and gate (verify completion).

## Planning Mode

Read project state, define goals, break work into capabilities.

### Typical sequence

1. `open_project` -- register the project path
2. Read resources for context: `ship://project_info`, `ship://targets`, `ship://jobs`, `ship://workspaces`
3. `create_adr` -- record architecture decisions

### What planning produces

- ADRs documenting trade-offs and decisions
- Understanding of existing targets and job state from resources

## Orchestration Mode

Create workspaces, dispatch jobs, monitor progress, handle file conflicts.

### Workspace setup

```
create_workspace(name="auth-flow", kind="imperative", base_branch="main")
activate_workspace(branch="auth-flow")
set_agent(id="web-specialist")
```

Workspace kinds determine worktree lifecycle:

| Kind | Worktree | On completion |
|------|----------|---------------|
| imperative | created | pruned |
| declarative | created | kept |
| service | none | n/a |

### Job dispatch

```
create_job(
  kind="feature",
  description="Implement OAuth provider",
  branch="auth-flow",
  file_scope=["apps/web/src/auth/"],
  acceptance_criteria=["OAuth flow completes", "Tokens stored"],
  capability_id="<cap-id>"
)
```

Jobs move through: `pending` -> `running` -> `complete` or `failed`.

### Monitoring

- `list_jobs(status="running")` -- see active work
- Read `ship://events` -- watch for recent state changes across all workspaces
- Read `ship://workspaces/{branch}/session` -- check session state

### Job dependencies

Jobs can block on other jobs:

```
create_job(kind="feature", description="Build login UI", blocked_by="<auth-job-id>")
```

The blocked job stays `pending` until the blocking job completes. Higher `priority` values schedule first.

## Gate Mode

Verify completed work against acceptance criteria. Either pass (capability becomes actual) or fail (job gets feedback).

### Pass sequence

```
update_job(id="<job-id>", status="completed")
complete_workspace(workspace_id="auth-flow", summary="OAuth flow implemented and tested")
```

`complete_workspace` writes a `handoff.md` in the worktree root and optionally prunes the worktree (default for imperative workspaces).

### Fail sequence

```
update_job(id="<job-id>", status="failed")
```

The job stays in the queue with failure context. A new job or a retry can pick up the remaining work.

## Session Lifecycle

Sessions track individual agent visits within a workspace. One active session per workspace.

```
start_session(goal="Implement OAuth login")
  ... work ...
  log_progress(note="Created auth provider config")
  log_progress(note="Login UI renders, testing OAuth flow")
end_session(summary="OAuth flow working", files_changed=12, model="claude-opus-4-20250514")
```

Session records are immutable once ended. They include: start/end timestamps, goal, progress notes, summary, file count, model used, and optional gate result.

`end_session` accepts `gate_result` ("pass" or "fail") for sessions that represent a gate check.

## Event Log

Every mutation (workspace creation, session start/end, job updates) emits an event to the append-only log. Agents emit custom domain events via the `event` tool and read the log via the `ship://events` resource:

- `ship://events` -- 100 most recent events
- `ship://events/20` -- last 20 events

The event log is the audit trail for understanding what happened across all workspaces and agents.

## End-to-End Example

1. **Plan:** Read `ship://targets` and `ship://jobs`, create ADR for decisions
2. **Dispatch:** Create imperative workspace `auth-flow`, create job for the feature
3. **Work:** Start session, implement the feature, log progress at checkpoints
4. **Gate:** Run tests, update job to completed
5. **Complete:** End session with summary, complete workspace (prunes worktree)

Each step uses the tools from the corresponding mode. The event log captures the full history.
