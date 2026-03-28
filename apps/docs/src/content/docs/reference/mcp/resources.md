---
title: "Resources"
description: "MCP resources reference -- the ship:// URI scheme for read-only access to project state."
sidebar:
  label: "Resources"
  order: 3
---
The Ship MCP server exposes read-only resources via the `ship://` URI scheme. Resources return project state without side effects -- use them for read-heavy workflows instead of calling list tools repeatedly.

## Static Resources

These URIs return collections. No parameters needed.

### ship://project_info

Project metadata and active configuration. Returns the compiled project context: active agent, installed skills, project path, and configuration state.

### ship://specs

List all spec documents in the project's specs directory. Returns a list of spec filenames (without extension).

### ship://adrs

List all Architecture Decision Records. Returns id, status, and title for each ADR.

### ship://notes

List all project notes. Returns id and title for each note.

### ship://skills

List all installed skills for the active project. Returns skill id and name for each entry.

### ship://workspaces

List all workspaces for the active project. Returns full workspace objects as JSON, including branch, kind, status, and metadata.

### ship://sessions

List recent workspace sessions (up to 50). Returns session objects as JSON with workspace, start time, end time, and summary.

### ship://modes

Show the active agent mode and all configured modes. Returns the active mode id and the full mode configuration map.

### ship://providers

List all configured providers. Returns provider objects as JSON.

### ship://log

Read the project log file. Returns the raw markdown content, or "No log entries yet." if empty.

### ship://events

Read the 100 most recent events from the append-only event log. Each event includes id, timestamp, actor, entity type, action, subject, and optional details.

### ship://jobs

List all jobs in the queue. Returns id, status, and kind for each job.

### ship://targets

List all targets (milestones and surfaces). Returns id, status, title, and kind for each target.

## Resource Templates

These URIs accept parameters for fetching specific items. Replace `{placeholder}` with the actual value.

### ship://specs/{id}

Fetch a single spec document by id. Returns the raw markdown content.

| Param | Type | Description |
|-------|------|-------------|
| `id` | string | Spec filename without extension |

### ship://adrs/{id}

Fetch a single ADR by id. Returns formatted markdown with title, status, date, context, and decision sections.

| Param | Type | Description |
|-------|------|-------------|
| `id` | string | ADR id (nanoid) |

### ship://notes/{id}

Fetch a single note by id. Returns title and content.

| Param | Type | Description |
|-------|------|-------------|
| `id` | string | Note id (nanoid) |

### ship://skills/{id}

Fetch a single skill's resolved content. Returns the compiled SKILL.md with variables resolved.

| Param | Type | Description |
|-------|------|-------------|
| `id` | string | Skill id |

### ship://workspaces/{branch}

Fetch a single workspace by branch name. Returns the full workspace object as JSON.

| Param | Type | Description |
|-------|------|-------------|
| `branch` | string | Workspace branch/id |

### ship://workspaces/{branch}/provider-matrix

Show the provider matrix for a specific workspace -- which providers are configured and what config files each generates.

| Param | Type | Description |
|-------|------|-------------|
| `branch` | string | Workspace branch/id |

### ship://workspaces/{branch}/session

Fetch the active session for a workspace. Returns the session object as JSON, or a message if no session is active.

| Param | Type | Description |
|-------|------|-------------|
| `branch` | string | Workspace branch/id |

### ship://sessions/{workspace}

List sessions for a specific workspace (up to 50). Returns session objects as JSON.

| Param | Type | Description |
|-------|------|-------------|
| `workspace` | string | Workspace branch/id |

### ship://providers/{id}/models

List available models for a specific provider.

| Param | Type | Description |
|-------|------|-------------|
| `id` | string | Provider id (e.g. `claude`, `codex`) |

### ship://events/{limit}

Read a specific number of recent events from the event log.

| Param | Type | Description |
|-------|------|-------------|
| `limit` | integer | Number of events to return |

### ship://jobs/{id}

Fetch a single job by id. Returns the full job object as JSON.

| Param | Type | Description |
|-------|------|-------------|
| `id` | string | Job id |

### ship://targets/{id}

Fetch a single target by id. Returns the full target object as JSON.

| Param | Type | Description |
|-------|------|-------------|
| `id` | string | Target id |

### ship://capabilities/{id}

Fetch a single capability by id. Returns the full capability object as JSON.

| Param | Type | Description |
|-------|------|-------------|
| `id` | string | Capability id |

## Usage Patterns

**Bulk read at session start.** Read `ship://project_info`, `ship://targets`, and `ship://jobs` to understand the project state before planning work.

**Check specific state.** Use templated URIs like `ship://workspaces/{branch}/session` to check whether a session is active before calling `start_session`.

**Monitor progress.** Read `ship://events` or `ship://events/20` to see recent activity across all workspaces and agents.

**Review decisions.** Read `ship://adrs` for the list, then `ship://adrs/{id}` for the full decision text.
