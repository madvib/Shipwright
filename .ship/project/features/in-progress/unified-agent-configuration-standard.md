+++
id = "FqLNtSDx"
title = "Unified Agent Configuration Standard"
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

Agent configuration (skills, MCP servers, model, cost limits, permissions) is currently split across feature frontmatter `[agent]` blocks, mode files, and project defaults with no single schema tying them together. This makes validation impossible and the resolution order unclear. `AgentConfig` is the canonical struct that every configuration surface conforms to — modes are named `AgentConfig` presets, the `[agent]` block is a partial `AgentConfig` override, and the resolved Workspace is a fully computed `AgentConfig`.

## Acceptance Criteria

- [ ] `AgentConfig` struct is the single schema for: skills, mcp_servers, model, max_cost, max_turns, permissions, rules
- [ ] Mode files (`agents/modes/*.toml`) parse into `AgentConfig`
- [ ] Feature `[agent]` block is a partial `AgentConfig` (missing fields inherit from mode)
- [ ] Resolution chain: project defaults → active mode → feature `[agent]` → resolved Workspace
- [ ] `resolve_agent_config(ship_dir, feature_agent_config)` returns fully resolved config
- [ ] Schema documented and validated at CLI/MCP write time
- [ ] No `model = "claude"` string — model must be a valid model ID or omitted

## Delivery Todos

- [ ] Finalize `AgentConfig` struct in `agent_config.rs`
- [ ] Parse mode files and integrate into resolution chain
- [ ] Update feature `[agent]` parsing to use partial `AgentConfig`
- [ ] Remove `model = "claude"` and `max_cost_per_session` from FEATURE.md template (done)
- [ ] Wire resolved config into CLAUDE.md / .mcp.json generation
- [ ] MCP: `get_agent_config` returns resolved config for current branch

## Notes

This feature is the schema half of Scoped Workspaces. Workspace owns the runtime session and SQLite state; this feature owns the config schema and resolution logic. They are sister features and should ship together. See also: Pre-defined Agent Modes (the user-facing preset layer built on top of this schema).
