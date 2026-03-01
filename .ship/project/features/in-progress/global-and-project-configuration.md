+++
id = "pUccKNoA"
title = "Global and Project Configuration"
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

Shipwright has two configuration scopes: project-level (committed to git, shared with the team) and user-level (local to the machine, personal preferences). Without a clear separation, users either over-commit personal config or lose settings on clone. `ship.toml` is the project config; `~/.ship/config.toml` is the user config. Both have defined schemas and are managed through `ship config` commands.

## Acceptance Criteria

- [ ] `ship.toml` at project root covers: project name, git policy, providers, active mode defaults
- [ ] `~/.ship/config.toml` covers: user preferences, default editor, global provider settings
- [ ] `ship config get <key>` / `ship config set <key> <value>` for both scopes
- [ ] `ship config list` shows all config with scope indicators
- [ ] Providers declared in `ship.toml [[providers]]` — only declared providers get config generated
- [ ] MCP: `get_config`, `set_config` tools (project-scoped)
- [ ] Config schema validated on write; unknown keys are rejected with a helpful error

## Delivery Todos

- [ ] Confirm `config.rs` handles both project and user scope
- [ ] `ship config` CLI subcommand (get, set, list)
- [ ] Validate provider entries in `ship.toml` against the provider registry
- [ ] User config path: `~/.ship/config.toml` (or platform-appropriate config dir)
- [ ] UI settings panel for project and user config

## Notes

The key design decision: provider declarations in `ship.toml` control which agent tool configs get generated. If `[[providers]]` doesn't include `gemini`, no `.gemini/` directory is created on branch checkout. This prevents config clutter for tools the team doesn't use. MCP server registry lives in `agents/mcp.toml`, not `ship.toml`.
