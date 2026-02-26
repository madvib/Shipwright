+++
id = "9127ea10-fe06-4c9c-aabf-76bb83449641"
title = "File vs SQLite boundary"
status = "accepted"
date = "2026-02-26"
tags = []
+++

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
