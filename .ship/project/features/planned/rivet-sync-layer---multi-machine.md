+++
id = "DXXSxrhs"
title = "Multi-machine sync layer"
status = "planned"
created = "2026-03-02T17:11:58.922652252Z"
updated = "2026-03-02T17:11:58.922652252Z"
release_id = "v0.2.0"
adr_ids = []
tags = []

[agent]
mcp_servers = []
skills = []
+++

## Why

A developer working across two machines today uses git as a clunky sync mechanism. This is the tier 2 unlock: one Rivet Actor per project holds the canonical SQLite state. Any machine syncs on open and on write. Self-hostable means no data leaves the user's network. Ship Cloud is the managed option for users who don't want to run infrastructure.

## Acceptance Criteria

- [ ] Two machines with the same project stay in sync without git push/pull
- [ ] Works fully offline — syncs when connection restores
- [ ] Self-hosted Rivet works identically to Ship Cloud (same protocol)
- [ ] Tier 1 (local solo, no account) still works with zero changes
- [ ] Sync conflict model documented: last-write-wins for single-user, per-field for teams

## Delivery Todos

- [ ] Rivet Actor definition: one per project, SQLite storage, HTTP + WebSocket
- [ ] Push changeset on every local mutation (queued when offline)
- [ ] Pull on app open: fetch changes since last sync timestamp
- [ ] Ship Cloud: managed Rivet deployment, auth via Ship account
- [ ] Self-hosted: ship rivet up command, docker-compose for local Rivet
- [ ] UI: sync status indicator (synced / syncing / offline)
- [ ] Account system: Ship Cloud auth (GitHub OAuth to start)
- [ ] Pricing: free tier (local solo) vs paid (Ship Cloud sync)

## Notes

Architecture: ~/.ship/projects/{projectId}.db is always the local read/write store. Rivet Actor is the cloud replica. Sync is push/pull over HTTP, WebSocket for real-time updates. No CRDTs needed for single-user. Teams get per-field timestamps for conflict resolution.
