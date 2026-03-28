---
title: "Tool Reference"
description: "Every MCP tool with parameters, grouped by domain. Stable and unstable tools marked."
sidebar:
  label: "Tool Reference"
  order: 2
---
Stable tools are always compiled in. Unstable tools require the `unstable` feature flag.

## Project (stable)

**open_project** -- `path` (string, required). Set the active project.

**set_agent** -- `id` (string, optional). Activate an agent profile or clear it.

## Studio Sync (stable)

**pull_agents** -- no params. Returns all local agents with resolved skills, rules, MCP configs.

**list_local_agents** -- no params. Returns agent profile IDs in `.ship/agents/`.

**push_bundle** -- `bundle` (string, required). Write a TransferBundle JSON string to `.ship/`.

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

## Skills (stable)

**list_skills** -- `query` (optional substring filter).

**list_project_skills** -- `query` (optional). Returns all skills in `.ship/skills/` with full resolved content as PullSkill JSON.

**write_skill_file** -- `skill_id` (string, required), `file_path` (string, required, relative within skill dir), `content` (string, required).

**delete_skill_file** -- `skill_id` (string, required), `file_path` (string, required). Refuses to delete SKILL.md.

**get_skill_vars** -- `skill_id` (string, required). Returns merged variable state.

**set_skill_var** -- `skill_id` (string, required), `key` (string, required), `value_json` (string, required, JSON-encoded).

**list_skill_vars** -- `skill_id` (optional). Lists skills with configurable variables.

## Events (stable)

**list_events** -- `since` (optional: ISO 8601 or relative like `1h`, `24h`, `7d`), `actor` (optional, substring), `entity` (optional: workspace/session/note/adr/etc.), `action` (optional: create/update/delete/start/stop), `limit` (optional, default 50, max 200).

## Notes and ADRs (unstable)

**create_note** -- `title` (string, required), `content` (optional), `branch` (optional).

**update_note** -- `id` (string, required, note filename), `content` (string, required), `scope` (optional: project/user).

**create_adr** -- `title` (string, required), `decision` (string, required).

## Targets and Capabilities (unstable)

**create_target** -- `kind` (string, required: milestone/surface), `title` (string, required), `description` (optional), `goal` (optional), `status` (optional: active/planned/complete/frozen), `phase` (optional), `due_date` (optional, ISO 8601), `body_markdown` (optional), `file_scope` (string[], optional).

**update_target** -- `id` (required), all other target fields optional (patch-style).

**list_targets** -- `kind` (optional: milestone/surface).

**get_target** -- `id` (string, required). Returns target with capability progress board.

**create_capability** -- `target_id` (string, required), `title` (string, required), `milestone_id` (optional), `phase` (optional), `acceptance_criteria` (string[], optional), `file_scope` (string[], optional), `assigned_to` (optional), `priority` (integer, optional).

**update_capability** -- `id` (required), optional: `title`, `status` (aspirational/in_progress/actual), `phase`, `acceptance_criteria`, `file_scope`, `assigned_to`, `priority`.

**delete_capability** -- `id` (string, required).

**mark_capability_actual** -- `id` (string, required), `evidence` (string, required: test name, commit hash, or behavior).

**list_capabilities** -- optional filters: `target_id`, `milestone_id`, `status`, `phase`.

## Jobs (unstable)

**create_job** -- `kind` (string, required), `description` (string, required), `branch` (optional), `assigned_to` (optional), `requesting_workspace` (optional), `priority` (integer, optional, higher runs first), `blocked_by` (optional, job id), `touched_files` (string[], optional), `file_scope` (string[], optional), `acceptance_criteria` (string[], optional), `capability_id` (optional), `symlink_name` (optional).

**update_job** -- `id` (required), optional: `status` (pending/running/complete/failed), `assigned_to`, `priority`, `blocked_by`, `touched_files`.

**list_jobs** -- optional: `branch`, `status`.

**append_job_log** -- `job_id` (string, required), `message` (string, required), `level` (optional: info/warn/error).

**claim_file** -- `job_id` (string, required), `path` (string, required). Atomic, first-wins.

**get_file_owner** -- `path` (string, required). Returns owning job id or "unclaimed".
