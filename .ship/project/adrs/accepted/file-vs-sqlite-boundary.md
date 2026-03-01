+++
id = "KRYQsL3P"
title = "File vs SQLite boundary"
date = "2026-02-28"
spec_id = ""
supersedes_id = ""
tags = []
+++

## Context

Shipwright produces two kinds of data: documents that humans author (specs, features, ADRs, notes) and runtime state that the system manages (workspace sessions, branch cache, UI preferences). Early implementations mixed these — some runtime state lived in files, some documents were stored with metadata in SQLite. This created confusion about where the truth lived and made the data model hard to reason about.

## Decision

Clear rule for what lives in git-tracked files vs local SQLite.

**Rule:** If a human would ever want to read, diff, commit, or share it — it's a file. If it's ephemeral, derived, concurrent, or machine-generated churn — it's SQLite.

**Files (git-tracked markdown + TOML):**
- All documents: vision, notes, ADRs, releases, features, specs, issues
- project.jsonc / project config
- Modes, skills, prompts (file-based for community contribution — copy a file, run `ship skill add <url>`)
- Templates (colocated TEMPLATE.md per document directory)
- MCP server definitions (community contributions, API integration)

**SQLite — ~/.ship/shipwright.db (global):**
- Project registry
- Global mode state
- Entitlements / auth cache
- Global MCP server library (managed via UI)
- Available models cache (24h TTL)
- Global MCP connection state

**SQLite — .ship/ship.db (project, gitignored):**
- Active mode state
- Worktree registry
- Agent sessions
- Branch context cache
- UI preferences (colors, themes — set in GUI, not config files)
- Orchestration locks

**Boundary violations to avoid:**
- Do not store document content or metadata in SQLite (files are the truth)
- Do not store user-facing config in SQLite (it must be hand-editable)
- Do not store UI preferences in files (they are personal, not team-shared)

## Consequences

### Positive
- Clear single source of truth for every piece of data
- Documents are grep-able, diffable, committable without special tooling
- SQLite runtime state can be safely deleted and rebuilt from files
- No sync conflicts between file and DB representations

### Negative
- Two storage layers to maintain and reason about
- Workspace and session state is lost if ship.db is deleted (acceptable — it's derived)
- Agents reading files directly bypass any validation Shipwright would apply at write time
