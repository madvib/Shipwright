+++
id = "30c33d39-383e-42cc-a13e-c4a905838217"
title = "Config schema v2 — modes, AI providers, MCP server registry"
created = "2026-02-24T04:09:41.261772267Z"
updated = "2026-02-24T04:09:41.262084568Z"
tags = []
links = []
+++

## What
Evolve `.ship/config.toml` to support modes, AI provider configuration, and a centralized MCP server registry. This is the foundational schema change that all other alpha AI features depend on.

## Changes to ProjectConfig

```toml
version = "2"
name = "my-project"

# AI provider — which CLI to use for pass-through generation
[ai]
provider = "claude"        # claude | codex | gemini
cli_path = "claude"        # override if not on PATH
model = ""                 # optional model override (passed as --model flag)

# MCP server registry — all servers available to this project
[[mcp-servers]]
id = "ship"
name = "Ship"
command = "/path/to/ship-mcp"
args = []
env = {}
modes = ["all"]            # available in all modes; or list mode IDs

[[mcp-servers]]
id = "github"
name = "GitHub"
command = "npx"
args = ["-y", "@modelcontextprotocol/server-github"]
env = { GITHUB_TOKEN = "$GITHUB_TOKEN" }
modes = ["execution", "review"]

# Mode definitions
[[modes]]
id = "planning"
name = "Planning"
description = "Spec writing and issue creation"
mcp_tools = ["ship_list_specs", "ship_create_spec", "ship_extract_issues", "ship_list_issues", "ship_create_issue"]
ai_context = ["AGENTS.md", "specs/"]
ui_layout = "spec-editor"

[[modes]]
id = "execution"
name = "Execution"
description = "Working issues — human or agent"
mcp_tools = ["ship_list_issues", "ship_get_issue", "ship_move_issue", "ship_update_issue"]
ai_context = ["AGENTS.md", "issues/in-progress/"]
ui_layout = "kanban"
```

## Scope
- Update `ProjectConfig` struct in `logic/src/config.rs`
- Add `ModeConfig`, `McpServerConfig` structs with `#[derive(specta::Type)]`
- Add `active_mode` to global config (`~/.ship/config.toml`) — persists last used mode
- Write migration: v1 config → v2 (statuses move to `[modules.issues]`, add default modes)
- Update all callers of `get_config` / `save_config`
- Update `bindings.ts` export

## Acceptance
- Existing `.ship/config.toml` files auto-migrate to v2 on first read
- v2 config round-trips correctly through serde
- All logic/mcp/cli callers still compile