+++
id = "qdqSyJXN"
title = "Shipwright MCP Server"
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

The MCP server is how AI agents interact with Shipwright — reading project state, creating issues, moving work, and triggering actions without leaving the conversation. It is the most important integration surface: it turns Shipwright from a file manager into an active participant in the agent workflow. Every CRUD operation available in the CLI should be available via MCP.

## Acceptance Criteria

- [ ] MCP server runs as a stdio process (`ship-mcp` binary)
- [ ] Covers all entity types: issues, features, specs, ADRs, notes, releases, skills, events
- [ ] `get_project_info` — single call for full project overview
- [ ] `list_issues`, `create_issue`, `move_issue`, `update_issue`, `delete_issue`
- [ ] `list_features`, `get_feature`, `create_feature`, `update_feature`
- [ ] `list_specs`, `get_spec`, `create_spec`, `update_spec`
- [ ] `list_adrs`, `get_adr`, `create_adr`, `generate_adr`
- [ ] `list_notes`, `get_note`, `create_note`, `update_note`
- [ ] `list_releases`, `get_release`, `create_release`, `update_release`
- [ ] `list_skills`, `get_skill`, `create_skill`, `update_skill`, `delete_skill`
- [ ] `git_feature_sync`, `git_hooks_install`, `git_config_get/set`
- [ ] AI generation tools: `generate_issue_description`, `generate_adr`, `brainstorm_issues`
- [ ] Project auto-detection from `SHIP_DIR` env var or CWD walk

## Delivery Todos

- [ ] Audit MCP tools for completeness against CLI surface
- [ ] Verify `create_feature` no longer double-wraps frontmatter (bug was fixed — confirm)
- [ ] `get_project_info` returns fully resolved linked graph
- [ ] MCP server added to `.ship/agents/mcp.toml` for self-referential dogfooding
- [ ] Error messages from MCP tools are actionable (not raw Rust panics)

## Notes

The MCP server operates over stdio — not HTTP. No port, no authentication, no network exposure in alpha. The `ship-mcp` binary is installed alongside `ship` at `~/.cargo/bin/ship-mcp`. Claude Code connects via the `.mcp.json` config written by the post-checkout hook.
