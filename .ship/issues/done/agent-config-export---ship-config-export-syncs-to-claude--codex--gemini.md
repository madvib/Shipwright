+++
id = "f61bf740-9832-4a59-b925-6c5129a1c5b0"
title = "Agent config export — ship config export syncs to Claude, Codex, Gemini"
created = "2026-02-24T04:10:35.800663186Z"
updated = "2026-02-24T04:10:35.800663986Z"
tags = []
links = []
+++

## What
Implement `ship config export --target <provider>` — Ship reads its MCP registry and mode definitions and writes provider-native config files. Ship becomes the single source of truth for AI agent configuration.

## What Gets Exported Per Provider

### Claude Code (`--target claude`)
**Writes two files:**

1. `CLAUDE.md` (project root) — mode context and instructions
```markdown
# Project: my-project

## Active Mode: Planning
Spec writing and issue creation with AI assistance.

## MCP Tools Available in This Mode
- ship_list_specs, ship_create_spec, ship_extract_issues
- ship_list_issues, ship_create_issue

## Context Files
Read specs/ for current specifications before making changes.
```

2. `~/.claude.json` — MCP server registry (merges Ship servers, preserves existing entries)
```json
{
  "mcpServers": {
    "ship": {
      "type": "stdio",
      "command": "/home/user/.cargo/bin/ship-mcp"
    },
    "github": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "env": { "GITHUB_TOKEN": "${GITHUB_TOKEN}" }
    }
  }
}
```

### Codex (`--target codex`)
**Writes:** `.codex/config.toml` (project-scoped)
```toml
[mcp-servers.ship]
command = "/home/user/.cargo/bin/ship-mcp"
args = []

[mcp-servers.github]
command = "npx"
args = ["-y", "@modelcontextprotocol/server-github"]
```

### Gemini (`--target gemini`)
**Writes:** `.gemini/settings.json` (project-scoped)
```json
{
  "mcpServers": {
    "ship": { "command": "/home/user/.cargo/bin/ship-mcp" },
    "github": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"]
    }
  }
}
```

### All providers (`--target all`)
Runs all three exports.

## CLI Commands
```
ship config export --target claude
ship config export --target codex
ship config export --target gemini
ship config export --target all
```

## Tauri Command
```rust
fn export_agent_config_cmd(target: String, state: State<AppState>) -> Result<ExportResult, String>
// ExportResult: { files_written: Vec<String>, warnings: Vec<String> }
```

## Important Behavior
- Only exports MCP servers active for the current mode
- `~/.claude.json` merges (doesn't overwrite unrelated entries)
- `.codex/config.toml` and `.gemini/settings.json` are project-scoped (safe to overwrite)
- `$VAR` in env values are passed through literally — expansion happens in the provider at runtime
- Warns if ship-mcp binary path is not found

## Acceptance
- After `ship config export --target all`, Claude Code / Codex / Gemini can connect to ship-mcp without manual config
- CLAUDE.md reflects current active mode's tool list and context files
- Re-running export is idempotent