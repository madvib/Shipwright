---
name: ship-mcp-reference
stable-id: ship-mcp-reference
description: Use when calling Ship MCP tools — workspace management, session lifecycle, job coordination, targets, capabilities, events. Complete tool reference with parameters.
tags: [ship, mcp, reference]
authors: [ship]
---

# Ship MCP Tools

The Ship MCP server (`ship mcp serve`) exposes platform tools to agents via the Model Context Protocol. Never access the database directly -- always use these tools or the `ship` CLI.

## Three-Stage Workflow

1. **Planning** -- `open_project` then read resources (`ship://project_info`, `ship://targets`)
2. **Workspace** -- `create_workspace` / `activate_workspace` then `set_agent`
3. **Session** -- `start_session`, work, `log_progress`, `end_session`

## Tool Domains

### Project
- `open_project` -- register the active project (call once per MCP session)

### Notes and ADRs
- `create_note` -- project-scoped note with optional branch association
- `update_note` -- replace note content by filename
- `create_adr` -- architecture decision record with context and alternatives

### Workspaces
- `create_workspace` -- new workspace with git worktree (imperative/declarative/service)
- `activate_workspace` -- mark a workspace active for the current session
- `complete_workspace` -- write handoff.md and optionally prune the worktree
- `list_workspaces` -- list all workspaces, filter by status
- `list_stale_worktrees` -- find worktrees idle beyond a threshold
- `set_agent` -- set or clear the active agent profile

### Sessions
- `start_session` -- begin a session for the current workspace
- `end_session` -- end the session with a summary
- `log_progress` -- record a progress checkpoint (requires active session)

### Jobs
- `create_job` -- create a new job in the queue with kind, scope, acceptance criteria
- `update_job` -- update status, assignment, priority, or blocking dependencies
- `list_jobs` -- list jobs, filter by status or branch
- `append_job_log` -- log progress, warnings, or file ownership to a job
- `claim_file` -- claim exclusive file ownership for a job (first-wins)
- `get_file_owner` -- look up which job owns a file

### Skills
- `list_skills` -- list installed skills, filter by substring

### Targets and Capabilities
- `create_target` -- create a milestone or surface target
- `list_targets` -- list targets, filter by kind
- `get_target` -- get a target with its capability progress board
- `update_target` -- update target metadata or body markdown
- `create_capability` -- add a capability to a target
- `update_capability` -- update capability fields (status, phase, scope)
- `mark_capability_actual` -- mark delivered with evidence (test, commit, behavior)
- `list_capabilities` -- list capabilities, filter by target, milestone, or status
- `delete_capability` -- remove a capability by id

### Events
- `list_events` -- query the append-only event log with time, actor, entity, action filters

### Compiler
- `provider_matrix` -- show supported providers and their generated config files

## Resources

Read-only `ship://` URIs for bulk data retrieval. Use resources for read-heavy workflows instead of calling list tools repeatedly.

Key resources: `ship://project_info`, `ship://workspaces`, `ship://sessions`, `ship://jobs`, `ship://targets`, `ship://skills`, `ship://events`, `ship://notes`, `ship://adrs`, `ship://specs`, `ship://modes`, `ship://providers`, `ship://log`.

See the reference docs for full parameter tables, resource URI templates, and workflow patterns.
