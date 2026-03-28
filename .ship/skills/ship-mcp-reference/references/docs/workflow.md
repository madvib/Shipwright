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
3. `create_target` -- define a milestone or surface if none exists
4. `create_capability` -- break the target into verifiable slices with acceptance criteria
5. `create_note` or `create_adr` -- record decisions

### What planning produces

- Targets with goals and body_markdown describing strategy
- Capabilities with acceptance criteria, file scope, and phase assignments
- Notes or ADRs documenting trade-offs

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

### File ownership

When parallel agents work on the same codebase, file claims prevent conflicts:

```
claim_file(job_id="<job-id>", path="apps/web/src/auth/provider.ts")
```

This is atomic and first-wins. Check ownership before claiming with `get_file_owner`.

### Monitoring

- `list_jobs(status="running")` -- see active work
- `list_events(since="1h")` -- watch for state changes
- `append_job_log(job_id, message)` -- record progress within a job
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
mark_capability_actual(id="<cap-id>", evidence="test_oauth_flow passes, commit a1b2c3")
update_job(id="<job-id>", status="complete")
complete_workspace(workspace_id="auth-flow", summary="OAuth flow implemented and tested")
```

`complete_workspace` writes a `handoff.md` in the worktree root and optionally prunes the worktree (default for imperative workspaces).

### Fail sequence

```
update_job(id="<job-id>", status="failed")
append_job_log(job_id="<job-id>", message="Tests fail: token refresh not implemented", level="error")
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

Every mutation (workspace creation, session start/end, job updates, capability transitions) emits an event to the append-only log. Query with `list_events`:

- `list_events(since="24h")` -- last day of activity
- `list_events(entity="workspace", action="create")` -- workspace creations
- `list_events(actor="web-specialist")` -- activity by a specific agent

The event log is the audit trail for understanding what happened across all workspaces and agents.

## End-to-End Example

1. **Plan:** Read targets, create capabilities for "User Auth" surface
2. **Dispatch:** Create imperative workspace `auth-flow`, create job for OAuth capability
3. **Work:** Start session, implement OAuth, log progress at checkpoints
4. **Gate:** Run tests, mark capability actual with test name as evidence
5. **Complete:** End session with summary, complete workspace (prunes worktree)

Each step uses the tools from the corresponding mode. The event log captures the full history.
