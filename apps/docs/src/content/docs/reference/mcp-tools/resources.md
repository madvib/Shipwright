---
title: "Resources"
description: "All ship:// URIs for read-only access to project state."
sidebar:
  label: "Resources"
  order: 3
---
The full MCP server (not Studio) exposes read-only resources via the `ship://` URI scheme. Resources return project state without side effects. Use them for bulk reads instead of calling list tools repeatedly.

Resources are registered in `resources.rs` and resolved in `resource_resolver.rs`.

## Static Resources

No parameters needed. Returns collection data.

| URI | Returns |
|-----|---------|
| `ship://project_info` | Active agent, installed skills, project path, config state |
| `ship://specs` | List of spec filenames in the specs directory |
| `ship://adrs` | All ADRs: id, status, title |
| `ship://notes` | All notes: id, title |
| `ship://skills` | All effective skills: id, name |
| `ship://workspaces` | All workspaces as JSON |
| `ship://sessions` | Recent sessions (up to 50) as JSON |
| `ship://modes` | Active agent mode and all configured modes |
| `ship://providers` | All configured providers as JSON |
| `ship://log` | Raw project log markdown |
| `ship://events` | 100 most recent events (id, timestamp, actor, entity, action, subject) |
| `ship://jobs` | All jobs: id, status, kind |
| `ship://targets` | All targets: id, status, title, kind |

## Resource Templates

Parameterized URIs for fetching specific items.

### Documents

| URI Template | Returns | Format |
|-------------|---------|--------|
| `ship://specs/{id}` | Single spec by filename (no extension) | text/markdown |
| `ship://adrs/{id}` | Single ADR with title, status, date, context, decision | text/markdown |
| `ship://notes/{id}` | Single note with title and content | text/markdown |
| `ship://skills/{id}` | Single skill's resolved content | text/markdown |

### Workspaces and Sessions

| URI Template | Returns | Format |
|-------------|---------|--------|
| `ship://workspaces/{branch}` | Single workspace object | application/json |
| `ship://workspaces/{branch}/provider-matrix` | Provider matrix for a workspace | application/json |
| `ship://workspaces/{branch}/session` | Active session for a workspace | application/json |
| `ship://sessions/{workspace}` | Sessions for a workspace (up to 50) | application/json |

### Providers

| URI Template | Returns | Format |
|-------------|---------|--------|
| `ship://providers/{id}/models` | Available models for a provider | application/json |

### Events

| URI Template | Returns | Format |
|-------------|---------|--------|
| `ship://events/{limit}` | N most recent events | text/plain |

### Workflow

| URI Template | Returns | Format |
|-------------|---------|--------|
| `ship://jobs/{id}` | Single job object | application/json |
| `ship://targets/{id}` | Single target object | application/json |
| `ship://capabilities/{id}` | Single capability object | application/json |

## Resolution Logic

The resource resolver (`resource_resolver.rs`) matches URIs using prefix stripping, not a router. Resolution order:

1. Exact match on static URIs (`ship://project_info`, `ship://skills`, etc.)
2. Prefix match on parameterized URIs (`ship://specs/`, `ship://adrs/`, etc.)
3. Workspace URIs (checked for nested paths like `/provider-matrix` and `/session`)
4. Workflow URIs (`ship://jobs/`, `ship://targets/`, `ship://capabilities/`)

If no URI matches, the resolver returns `None` and the server returns a "resource not found" error.

## Resource Notifications

When tools modify state (push_bundle, set_skill_var, write_skill_file, delete_skill_file), the server calls `notify_resource_list_changed()` on the connected peer. This tells MCP clients to re-fetch resource lists.

{% aside type="tip" %}
Resources are only available on the full ShipServer. The StudioServer does not expose resources -- it uses tools exclusively.
{% /aside %}

## Usage Patterns

**Bulk read at session start.** Read `ship://project_info`, `ship://targets`, and `ship://jobs` to understand project state before planning.

**Check before mutate.** Read `ship://workspaces/{branch}/session` to check for an active session before calling `start_session`.

**Monitor activity.** Read `ship://events` or `ship://events/20` to see recent changes across all workspaces.

**Inspect specific items.** Use `ship://targets/{id}` or `ship://jobs/{id}` to get full JSON details for a single record.
