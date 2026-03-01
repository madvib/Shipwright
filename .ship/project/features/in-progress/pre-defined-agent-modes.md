+++
id = "KVVaSUTx"
title = "Pre-defined Agent Modes"
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

Developers working across different kinds of tasks — planning a spec, implementing an issue, reviewing an ADR — want the agent configured differently for each. Modes are named, pre-configured agent environments that set sensible defaults for skills, MCP servers, model, and cost limits. Switching modes reconfigures the agent without touching individual feature files. They are the "preset" layer in the workspace config resolution chain.

## Acceptance Criteria

- [ ] At least three built-in modes: Planning, Execution, Review
- [ ] Mode config lives in `agents/modes/<name>.toml` with same schema as feature `[agent]` block
- [ ] Active mode stored in Workspace (SQLite) per branch session
- [ ] `ship mode set <name>` switches active mode and regenerates CLAUDE.md/.mcp.json
- [ ] UI mode switcher reflects current mode and shows what it configures
- [ ] Feature `[agent]` overrides apply on top of active mode (not replace)
- [ ] Custom modes can be created by users (copy a mode file, edit, `ship mode add`)

## Delivery Todos

- [ ] Define `AgentConfig` as the canonical schema for both modes and `[agent]` blocks
- [ ] Implement mode file parsing in `runtime` (`agents/modes/*.toml`)
- [ ] `ship mode list` / `ship mode set` CLI commands
- [ ] MCP tools: `list_modes`, `set_mode`
- [ ] Wire active mode into workspace resolution chain
- [ ] Built-in mode templates: Planning, Execution, Review
- [ ] UI mode switcher component

## Notes

**Modes are workspace config presets, not PM state.** The UI mode (Planning vs Execution) shapes tool availability and agent behavior — it is not a project management status.

**Schema unification:** Mode files and the feature `[agent]` block share the same `AgentConfig` schema. This means a Mode is just a named, reusable `[agent]` block. The resolution chain is: project defaults → active mode → feature `[agent]` overrides → resolved Workspace.

Built-in modes (embedded in binary, copy-to-customize):
- **Planning** — task-policy skill, ship MCP, conservative cost limit. Good for spec/feature drafting.
- **Execution** — task-policy + git-commit + rust-conventions skills, ship MCP, higher cost limit. Day-to-day implementation.
- **Review** — pr-review skill, minimal MCP surface, read-heavy prompts.
