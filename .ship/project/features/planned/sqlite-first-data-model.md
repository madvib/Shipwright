+++
id = "beUJ4VtG"
title = "SQLite-first data model"
status = "planned"
created = "2026-03-02T17:11:10.062517276Z"
updated = "2026-03-02T17:11:10.062517276Z"
release_id = "v0.1.0-alpha"
adr_ids = []
tags = []

[agent]
mcp_servers = []
skills = []
+++

## Why

Markdown as source of truth breaks down for rich UI, cloud sync, and relational queries. SQLite becomes the canonical store. Markdown is a generated export for agents — a role it already plays well via CLAUDE.md generation.

## Acceptance Criteria

- [ ] Features, specs, releases, workspaces stored as structured SQLite rows
- [ ] FeatureTodo and FeatureAcceptanceCriteria as separate tables (not markdown checkboxes)
- [ ] WorkspaceType enum: feature | refactor | experiment | hotfix
- [ ] WorkspaceStatus enum: planned | active | idle | review | merged | archived
- [ ] Narrative content (Why, Notes) stored as text columns, not markdown files
- [ ] Events stay as SQLite rows — NDJSON sync role removed
- [ ] DB lives at ~/.ship/projects/{projectId}.db (not in repo)
- [ ] Markdown export regenerated for agents on demand (existing pattern)
- [ ] Git commits: ship.toml, agents/ (rules, skills, permissions, mcp.toml), vision.md only

## Delivery Todos

- [ ] Design and write SQLite migrations for new schema
- [ ] FeatureTodo table: id, feature_id, text, completed, completed_at, ord
- [ ] FeatureAcceptanceCriteria table: id, feature_id, text, met, ord
- [ ] WorkspaceType + WorkspaceStatus enums in Rust
- [ ] Workspace table: id, type, status, branch, worktree_path, feature_id, release_id, last_activated_at, context_hash
- [ ] Migrate narrative fields (description, notes) to text columns on Feature
- [ ] Update agent export to generate markdown from SQLite (not read from .md files)
- [ ] Update MCP tools to read/write SQLite
- [ ] Update Tauri commands to use new schema
- [ ] Migration path for existing .ship markdown files → SQLite

## Notes

Cloud sync (Rivet) slots in later without schema changes — sync layer is transport, not storage. Design the schema now as if sync already exists.
