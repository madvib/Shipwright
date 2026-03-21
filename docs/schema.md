# Ship Database Schema

Ship stores platform state in `~/.ship/platform.db` (SQLite). This document describes what lives there, what each field means, and the rules for accessing it.

**Golden rule: never access the database directly.** Use the MCP tools or `ship` CLI. The schema evolves across releases. Direct SQL queries will break on upgrades and bypass event emission, invariant checks, and migration logic.

```
# Wrong
sqlite3 ~/.ship/platform.db "SELECT * FROM jobs"

# Right
list_jobs()                          # via MCP
ship job list                        # via CLI
```

---

## Entities

### Workspace

A workspace is a unit of parallel work. It corresponds to a git worktree (for imperative/declarative kinds) or a standing service branch.

| Field | Type | Description |
|-------|------|-------------|
| `id` | text | UUID (nanoid) — not branch-derived |
| `branch` | text | Git branch name (also used as the workspace identifier in API calls) |
| `name` | text | Human-readable name |
| `workspace_type` | enum | `Declarative` — long-lived, tracks a capability area; `Imperative` — short-lived, delivers a specific job; `Service` — standing service, no worktree |
| `status` | enum | `Active`, `Idle`, `Archived` |
| `active_agent` | text | Currently active agent profile id |
| `created_at` | timestamp | |
| `updated_at` | timestamp | |

**Workspace kinds:**
- `Imperative` — created for a specific job, pruned after the gate passes. Most jobs use this.
- `Declarative` — long-running, tracks a surface or capability area (e.g. a lane).
- `Service` — no worktree, represents a standing service or external system.

---

### WorkspaceSession

A session records one agent visit to a workspace. Sessions are append-only — ending a session creates a new record, not an update.

| Field | Type | Description |
|-------|------|-------------|
| `id` | text | UUID |
| `workspace_id` | text | Parent workspace id |
| `branch` | text | Workspace branch |
| `goal` | text | Session goal (optional) |
| `primary_provider` | text | Provider that ran this session (e.g. `claude`, `codex`) |
| `agent_id` | text | Agent profile id |
| `started_at` | timestamp | |
| `ended_at` | timestamp | Null if session is active |
| `summary` | text | End-of-session summary written by the agent |

Only one session can be active per workspace at a time. Call `end_session` before starting a new one on the same branch.

---

### Job

Jobs are the coordination layer across agents. The job queue is how parallel agents signal work to each other without calling each other directly.

| Field | Type | Description |
|-------|------|-------------|
| `id` | text | Nanoid |
| `kind` | text | `feature`, `fix`, `infra`, `test`, `review`, `migration`, `human-action`, etc. |
| `status` | text | `pending`, `running`, `complete`, `failed` |
| `description` | text | What needs to be done |
| `branch` | text | Associated git branch (optional) |
| `assigned_to` | text | Agent id or workspace this job is assigned to |
| `requesting_workspace` | text | Workspace that created this job |
| `priority` | integer | Scheduling priority — higher runs first (default 0) |
| `blocked_by` | text | Job id that must complete before this one can start |
| `touched_files` | json | List of file paths this job has modified |
| `file_scope` | json | Paths the agent is authorized to touch — checked at gate |
| `acceptance_criteria` | json | Checklist items for the gate |
| `capability_id` | text | Capability this job delivers (optional) |
| `preset_hint` | text | Profile to compile in the worktree |
| `payload` | json | Arbitrary job metadata |
| `created_at` | timestamp | |
| `updated_at` | timestamp | |

**File ownership**: `touched_files` and `claim_file` prevent two running jobs from modifying the same file. The gate only commits files listed in `touched_files`. If two jobs want the same file, serialize them.

**Status transitions**: `pending` → `running` → `complete` or `failed`. Only one agent should claim a job (transition from `pending` to `running`). This is the atomic claim protocol.

---

### JobLog

Append-only log entries attached to a job.

| Field | Type | Description |
|-------|------|-------------|
| `id` | integer | Auto-increment |
| `job_id` | text | Parent job id |
| `session_id` | text | Session that wrote this entry (optional) |
| `workspace_id` | text | Workspace context (optional) |
| `message` | text | Log message. Messages prefixed `touched: <path>` also register file ownership. |
| `created_at` | timestamp | |

Use `append_job_log` to write entries. Entries cannot be updated or deleted.

---

### Note

Notes are cross-session records for decisions, blockers, cross-lane signals, and summaries.

| Field | Type | Description |
|-------|------|-------------|
| `id` | text | UUID |
| `title` | text | Note title |
| `content` | text | Markdown content |
| `branch` | text | Associated git branch (optional) |
| `tags` | json | Tag list (optional) |
| `created_at` | timestamp | |
| `updated_at` | timestamp | |

Notes are written via `create_note` / `update_note`. They are project-scoped by default.

---

### ADR (Architecture Decision Record)

ADRs record architectural decisions with context, alternatives, and consequences.

| Field | Type | Description |
|-------|------|-------------|
| `id` | text | UUID |
| `title` | text | Short, specific title — verb + noun |
| `status` | text | `accepted`, `deprecated`, `superseded` |
| `decision` | text | Full decision text: context → decision → alternatives → consequences |
| `created_at` | timestamp | |
| `updated_at` | timestamp | |

Use the `write-adr` skill to structure ADR content before calling `create_adr`. An ADR with no alternatives is not useful — it documents what happened, not why.

---

### Target

Targets are the north-star layer. A target is either a milestone (time-boxed goal, e.g. v0.1.0) or a surface (product area, e.g. compiler).

| Field | Type | Description |
|-------|------|-------------|
| `id` | text | Nanoid |
| `kind` | text | `milestone` or `surface` |
| `title` | text | Short title |
| `description` | text | Longer description (optional) |
| `goal` | text | One-line north star goal (optional) |
| `status` | text | `active`, `planned`, `complete`, `frozen` |
| `created_at` | timestamp | |

---

### Capability

Capabilities belong to targets. They represent specific features or behaviors, tracked as aspirational (not yet delivered) or actual (delivered with evidence).

| Field | Type | Description |
|-------|------|-------------|
| `id` | text | Nanoid |
| `target_id` | text | Surface this capability belongs to |
| `milestone_id` | text | Milestone this capability is required for (optional) |
| `title` | text | Capability title |
| `status` | text | `aspirational` or `actual` |
| `evidence` | text | Proof of delivery: test name, commit hash, observable behavior. Required to mark actual. |
| `created_at` | timestamp | |
| `updated_at` | timestamp | |

**Never mark a capability actual without evidence.** The gate protocol enforces this — the commander runs the gate; the agent cannot self-report done.

---

### Event

Events are the audit log. Every platform state change emits an event.

| Field | Type | Description |
|-------|------|-------------|
| `seq` | integer | Monotonic sequence number |
| `timestamp` | timestamp | |
| `actor` | text | Agent or user that caused the event |
| `entity` | text | Entity type: `workspace`, `session`, `note`, `adr`, `job`, `capability`, etc. |
| `action` | text | What happened: `create`, `update`, `delete`, `start`, `stop`, etc. |
| `subject` | text | Entity id or name |
| `details` | text | Additional context (optional) |

Events are **append-only** — they are never updated or deleted. Query via `list_events`.

---

## ship.toml Manifest

The project manifest at `.ship/ship.toml` declares package identity, dependencies, and exports.

### [module]

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | **yes** | Namespaced package name. Must match `^[a-z0-9._/@-]+$`. Examples: `github.com/owner/repo`, `@scope/name`. |
| `version` | string | **yes** | Semver version. Optional `v` prefix is stripped before validation. |
| `description` | string | no | Human-readable package description. |
| `license` | string | no | SPDX license expression, e.g. `MIT`, `MIT OR Apache-2.0`. |
| `authors` | string[] | no | Package authors, e.g. `["Alice <alice@example.com>"]`. |

### [dependencies]

Key-value map where the key is a package path and the value is either a version string (shorthand) or a full table.

```toml
# Shorthand — version constraint only
"github.com/owner/skills" = "^1.0.0"

# Full form — version + permission grants
"github.com/owner/tools" = { version = "~2.1.0", grant = ["Bash", "Read"] }
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `version` | string | **yes** | Semver range, branch name, or 40-char commit SHA. |
| `grant` | string[] | no | Tool permissions granted to this dependency's skills. |

### [exports]

Declares which skills and agents this package makes available to consumers.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `skills` | string[] | no | Skill directory paths relative to `.ship/` (each must contain `SKILL.md`). |
| `agents` | string[] | no | Agent TOML paths relative to `.ship/` (each must end in `.toml`). |

---

## What lives where

| Data | Location | Access |
|------|----------|--------|
| Platform state (workspaces, sessions, jobs, notes, ADRs, events) | `~/.ship/platform.db` | MCP tools or `ship` CLI only |
| Agent profiles | `.ship/agents/profiles/*.toml` | Edit directly; recompile with `ship use` |
| Skills | `.ship/agents/skills/*/SKILL.md` | `ship skill add/remove` or edit directly |
| MCP server config | `.ship/agents/mcp.toml` | `ship mcp add/remove` |
| Permission presets | `.ship/agents/permissions.toml` | Edit directly |
| Project manifest | `.ship/ship.toml` | Edit directly; run `ship install` after changing deps |
| Compiled artifacts | `CLAUDE.md`, `.mcp.json`, `.cursor/`, etc. | Generated by `ship use` — do not commit, do not edit |

---

## Access rules

1. **MCP tools write state, not files.** Agents write job status, session progress, notes, and capability evidence through MCP tools — not by editing `.ship/` files directly.

2. **No direct sqlite3.** The schema is internal and will change between releases. MCP tools and the `ship` CLI are the stable interface.

3. **Events are append-only.** Never delete or update event records.

4. **Capabilities require evidence.** `mark_capability_actual` requires a non-empty `evidence` string. The gate protocol is the only valid path to marking a capability actual.

5. **File ownership is respected.** Before modifying a file, check `get_file_owner`. If claimed by a running job, wait or coordinate.
