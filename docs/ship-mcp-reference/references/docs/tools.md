---
group: MCP Tools
order: 2
title: Tool Reference
description: Every MCP tool with parameters, grouped by domain. Stable and unstable tools marked.
---

# Tool Reference

Stable tools are always compiled in. Unstable tools require the `unstable` feature flag.

## Project (stable)

**open_project** -- `path` (string, required). Set the active project.

**set_agent** -- `id` (string, optional). Activate an agent profile or clear it.

## Studio Sync (StudioServer only)

These tools are registered on the StudioServer, not the full ShipServer.

**pull_agents** -- no params. Returns all local agents with resolved skills, rules, MCP configs.

**list_local_agents** -- no params. Returns agent profile IDs in `.ship/agents/`.

**push_bundle** -- `bundle` (string, required). Write a TransferBundle JSON string to `.ship/`.

**list_project_skills** -- `query` (optional). Returns all skills in `.ship/skills/` with full resolved content as PullSkill JSON.

**write_skill_file** -- `skill_id` (string, required), `file_path` (string, required, relative within skill dir), `content` (string, required).

**delete_skill_file** -- `skill_id` (string, required), `file_path` (string, required). Refuses to delete SKILL.md.

## Workspaces (stable)

**create_workspace** -- `name` (string, required), `kind` (string, required: imperative/declarative/service), `branch` (optional, derived from name), `base_branch` (optional, default: main), `file_scope` (optional), `preset_id` (optional).

**activate_workspace** -- `branch` (string, required), `agent_id` (optional).

**complete_workspace** -- `workspace_id` (string, required), `summary` (string, required, written to handoff.md), `prune_worktree` (boolean, optional, default: true for imperative).

**list_workspaces** -- `status` (optional: active/idle/archived).

**list_stale_worktrees** -- `idle_hours` (integer, optional, default: 24).

## Sessions (stable)

**start_session** -- `branch` (optional, resolves from git), `goal` (optional), `agent_id` (optional), `provider_id` (optional).

**end_session** -- `branch` (optional), `summary` (optional), `files_changed` (integer, optional), `model` (optional), `gate_result` (optional: pass/fail), `updated_workspace_ids` (string[], optional).

**log_progress** -- `note` (string, required), `branch` (optional).

**get_session** -- `branch` (optional). Get the active session for a workspace branch.

**list_sessions** -- `branch` (optional). List session history for a branch. Returns all branches if omitted.

## Skills (stable)

**list_skills** -- `query` (optional substring filter).

**get_skill_vars** -- `skill_id` (string, required). Returns merged variable state.

**set_skill_var** -- `skill_id` (string, required), `key` (string, required), `value_json` (string, required, JSON-encoded).

**list_skill_vars** -- `skill_id` (optional). Lists skills with configurable variables.

## Events (stable)

**event** -- `event_type` (string, required, namespaced with dot e.g. "deployment.completed"), `payload` (JSON, required), `elevated` (boolean, optional, default false). Emit a domain event. Reserved prefixes (actor.*, session.*, skill.*, workspace.*, gate.*, job.*, config.*, project.*) are rejected. Actor and workspace are injected from connection context. Agents cannot read the event store via tools -- use `ship://events` resource for read access.

## ADRs (unstable)

**create_adr** -- `title` (string, required), `decision` (string, required).

## Jobs (stable)

**create_job** -- `kind` (string, required), `description` (string, required), `branch` (optional), `assigned_to` (optional), `requesting_workspace` (optional), `priority` (integer, optional, higher runs first), `blocked_by` (optional, job id), `touched_files` (string[], optional), `file_scope` (string[], optional), `acceptance_criteria` (string[], optional), `capability_id` (optional), `symlink_name` (optional).

**update_job** -- `id` (required), optional: `status` (pending/running/complete/failed), `assigned_to`, `priority`, `blocked_by`, `touched_files`.

**list_jobs** -- optional: `status`.

**get_job** -- `job_id` (string, required). Returns job JSON or error.

## Session Files (stable)

**write_session_file** -- `filename` (string, required), `content` (string, required). Write a file to `.ship-session/`. Fires a resource update notification.

**read_session_file** -- `filename` (string, required). Read a file from `.ship-session/`. Returns text content or base64 for binary files.

**list_session_files** -- no params. List all files in `.ship-session/` with metadata (name, path, type, size).

## Mesh (stable)

**mesh_send** -- `target` (string, required), `message` (string, required). Send a directed message to another agent on the mesh.

**mesh_broadcast** -- `message` (string, required), `capability` (optional). Broadcast a message to all agents, optionally filtered by capability.

**mesh_discover** -- `capability` (optional), `status` (optional). Discover agents on the mesh.

**mesh_status** -- `status` (string, required). Update this agent's status on the mesh (active, busy, idle).

## Dispatch (unstable)

**dispatch_agent** -- Spawn an agent: creates a git worktree, compiles provider config, and launches the agent process.

**list_agents** -- no params. List all running agents managed by the dispatch service.

**stop_agent** -- `agent_id` (string, required). Stop a running agent by id.

**steer_agent** -- `agent_id` (string, required), `message` (string, required). Inject a message into a running agent's stdin.
