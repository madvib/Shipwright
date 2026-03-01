+++
id = "UMuREHqq"
title = "Capability Registry (Skills/Servers)"
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

Developers shouldn't need to know the exact npm package name or config syntax for popular MCP servers, or hunt GitHub for community skills. Shipwright embeds a curated catalog of official MCP servers and community skills that users can browse, install, and reference in feature `[agent]` blocks. The catalog is the discovery layer; `agents/mcp.toml` and `agents/skills/` are the installation layer.

## Acceptance Criteria

- [ ] Embedded static catalog: ~10 official MCP servers (`@modelcontextprotocol/*`) + ~6 community skills
- [ ] `ship catalog list` shows available entries by kind (Skill | McpServer)
- [ ] `ship catalog search <query>` filters by name/description
- [ ] MCP: `list_catalog`, `list_catalog_by_kind`, `search_catalog` tools
- [ ] Skills installed to `agents/skills/<id>/` (directory format: `index.md` + `skill.toml`)
- [ ] MCP servers registered in `agents/mcp.toml`
- [ ] UI: catalog browser with install action

## Delivery Todos

- [ ] Confirm `catalog.rs` has all 10 MCP servers + 6 community skills embedded
- [ ] `ship catalog` CLI subcommand (list, search, install)
- [ ] Skill install: copy directory template, populate from catalog entry
- [ ] MCP server install: append entry to `agents/mcp.toml`
- [ ] UI catalog browser component

## Notes

The catalog is embedded in the binary (static list) — no network call required. This is intentional: offline-first, no CDN dependency. A live marketplace is a V1+ concern. The embedded list should be curated and small (high signal-to-noise) rather than exhaustive.
