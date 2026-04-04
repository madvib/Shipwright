---
name: ship-mcp-reference
stable-id: ship-mcp-reference
description: Use when calling Ship MCP tools — workspace management, session lifecycle, job coordination, events. Complete tool reference with parameters.
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

### ADRs
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
- `get_session` -- get the active session for a workspace branch
- `list_sessions` -- list session history for a branch

### Jobs
- `create_job` -- create a new job in the queue with kind, scope, acceptance criteria
- `update_job` -- advance a job to the next status
- `list_jobs` -- list jobs, filter by status
- `get_job` -- get a single job record by id

### Skills
- `list_skills` -- list installed skills, filter by substring

### Events
- `event` -- emit a domain event (write-only; agents cannot read the event store)

### Skills (extended)
- `get_skill_vars` -- get merged variable state for a skill
- `set_skill_var` -- set a skill variable value
- `list_skill_vars` -- list skills with configurable variables

### Session Files
- `write_session_file` -- write a file to .ship-session/
- `read_session_file` -- read a file from .ship-session/
- `list_session_files` -- list files in .ship-session/

### Mesh
- `mesh_send` -- send a directed message to another agent
- `mesh_broadcast` -- broadcast a message to all agents
- `mesh_discover` -- discover agents on the mesh
- `mesh_status` -- update this agent's mesh status

### Dispatch
- `dispatch_agent` -- spawn an agent in a git worktree
- `list_agents` -- list running agents
- `stop_agent` -- stop a running agent
- `steer_agent` -- inject a message into a running agent

## Resources

Read-only `ship://` URIs for bulk data retrieval. Use resources for read-heavy workflows instead of calling list tools repeatedly.

Key resources: `ship://project_info`, `ship://workspaces`, `ship://sessions`, `ship://jobs`, `ship://targets`, `ship://skills`, `ship://events`, `ship://notes`, `ship://adrs`, `ship://specs`, `ship://modes`, `ship://providers`, `ship://log`.

See the reference docs for full parameter tables, resource URI templates, and workflow patterns.
