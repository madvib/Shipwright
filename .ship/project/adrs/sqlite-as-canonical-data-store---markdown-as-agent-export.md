+++
id = "176e3fbc-efd8-4618-b4b0-07fc052d0a66"
title = "SQLite as canonical data store — markdown as agent export"
status = "accepted"
date = "2026-03-02"
tags = []
+++

## Decision

# SQLite as canonical data store — markdown as agent export

## Context

Shipwright initially used markdown files with TOML frontmatter as the source of truth for all entities (features, specs, issues, ADRs, releases). This worked for the CLI and MCP tools but breaks down in three ways:

1. **Rich UI** — interactive checklists, completion %, relational views (feature hub, release dashboard) require structured rows, not markdown parsing
2. **Cloud sync** — replicating structured rows via Rivet Actors is straightforward; syncing arbitrary markdown files with conflict resolution is not
3. **Relational queries** — "show all features blocking this release" or "which todos are incomplete across features in v0.1.0-alpha" require joins, not filesystem walks

## Decision

SQLite is the canonical store for all structured Shipwright entities. Markdown is a generated export format consumed by agents, not a source of truth.

**What moves to SQLite:**
- Features (with FeatureTodo and FeatureAcceptanceCriteria as separate tables)
- Specs, Issues, ADRs, Releases
- Workspaces (type, status, branch, feature_id, context_hash, last_activated_at)
- Events (was NDJSON — deprecated)
- Agent config resolution cache

**What stays in git (committed, human-authored):**
- `ship.toml` — project config
- `agents/` — skills, rules, permissions, mcp.toml
- `project/vision.md` — free-form north star document

**DB location:** `~/.ship/projects/{projectId}.db` — never in the repo, never committed

**Markdown generation:** On branch checkout (and on-demand via `ship git sync`), Shipwright generates `CLAUDE.md`, `.mcp.json`, and provider-specific context files from SQLite. Agents consume these. The generated files are gitignored.

**Migration:** Existing `.ship/` markdown files are imported to SQLite on first `ship` invocation after upgrade. The markdown files become stale artifacts; a migration command cleans them up.

## Consequences

**Positive:**
- Rich UI (feature hub, release dashboard, interactive todos) becomes straightforward
- Cloud sync (v0.2.0 Rivet layer) slots in without schema changes — sync is transport, not storage
- Agent context generation is faster (SQL query vs filesystem walk + markdown parse)
- Relational integrity: orphaned references caught at write time

**Negative:**
- Agents cannot read project state by directly grepping `.ship/` markdown — they must use MCP tools
- Migration required for existing projects (one-time, automated)
- Git history of features/specs/issues is lost (mitigated: events table provides write history)

