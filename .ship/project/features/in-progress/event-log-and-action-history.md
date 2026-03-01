+++
id = "jGtadbvi"
title = "Event Log and Action History"
created = "2026-02-28T15:56:07Z"
updated = "2026-02-28T15:56:07Z"
branch = ""
release_id = "v0.1.0-alpha"
spec_id = ""
adr_ids = []
tags = []

[agent]
model = "claude"
max_cost_per_session = 10.0
mcp_servers = []
skills = []
+++

## Why

Every action in Shipwright — creating an issue, moving a feature, triggering a hook — should leave a trace. The event log is the audit trail that answers "what happened and when" without querying multiple tables. It also provides the foundation for cloud sync in later versions: the event stream is the replication unit. Human-readable log views derive from it.

## Acceptance Criteria

- [ ] All CRUD operations append typed events to SQLite `events` table
- [ ] Events have: `seq`, `entity_id` (UUID), `entity_type`, `action`, `payload` (JSON), `created_at`
- [ ] `ship event list` shows recent events in human-readable format
- [ ] MCP: `list_events` with `since` cursor parameter for incremental reads
- [ ] `ingest_events` MCP tool detects out-of-band file edits and emits synthetic events
- [ ] NDJSON export for portability (`events.ndjson` — gitignored, local-only)
- [ ] UI: activity log / history view derived from event stream

## Delivery Todos

- [ ] Confirm event schema in `state_db.rs` covers all entity types
- [ ] Typed `ShipEvent` enum for Tauri (IssuesChanged, FeaturesChanged, etc.)
- [ ] `ship event list` CLI command
- [ ] MCP `list_events` + `ingest_events` tools (already implemented — verify)
- [ ] UI activity log view
- [ ] NDJSON export command

## Notes

Events are append-only. The `events` table is SQLite (project DB). NDJSON is export-only — it is not the primary store. Event model must stay compatible with future global aggregation across projects. `entity_id` is the short ID (8-char nanoid) of the affected document.
