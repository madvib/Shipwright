+++
id = "himaKLJq"
title = "Provider Configuration — Agent Tool Registry"
created = "2026-02-27T05:36:05.755357964Z"
updated = "2026-02-27T05:36:05.755357964Z"
tags = []
+++

# Provider Configuration — Agent Tool Registry

## Overview

Ship generates per-provider agent configuration on branch checkout. Each provider (Claude Code, Gemini CLI, Codex CLI, etc.) has a distinct config format, file location, and feature set. The provider registry encodes these differences as static descriptors, enabling generic export/teardown without per-provider branching logic.

## Current Implementation

`crates/runtime/src/agent_export.rs` — `PROVIDERS: &[ProviderDescriptor]` static registry.

### ProviderDescriptor Fields

| Field | Type | Purpose |
|---|---|---|
| `id` | `&str` | Stable identifier used in `ship.toml` and feature frontmatter |
| `name` | `&str` | Human-readable display name |
| `binary` | `&str` | Binary name for PATH detection (future: `ship doctor`) |
| `project_config` | `&str` | Config file path relative to project root |
| `global_config` | `&str` | Config file path relative to `$HOME` |
| `config_format` | `ConfigFormat` | `Json` or `Toml` |
| `mcp_key` | `&str` | Key used for MCP server entries in config |
| `http_url_field` | `&str` | Field name for HTTP transport URL (`"url"` vs `"httpUrl"`) |
| `emit_type_field` | `bool` | Whether to emit `"type": "http"` in server entries |
| `managed_marker` | `ManagedMarker` | How Ship tracks ownership: `Inline` (JSON `_ship`) or `StateFileOnly` |
| `prompt_output` | `PromptOutput` | Where the feature prompt goes: `ClaudeMd`, `GeminiMd`, `InstructionsKey`, `None` |
| `skills_output` | `SkillsOutput` | Where skills go: `ClaudeCommands` (`.claude/commands/`), `None` |

### Alpha Provider Registry

| ID | Name | Config | Format | MCP Key | Prompt |
|---|---|---|---|---|---|
| `claude` | Claude Code | `.mcp.json` | JSON | `mcpServers` | `CLAUDE.md` |
| `gemini` | Gemini CLI | `.gemini/settings.json` | JSON | `mcpServers` | `GEMINI.md` |
| `codex` | Codex CLI | `codex.toml` | TOML | `mcp_servers` | `instructions` key in TOML |

### Config Shape

**`ship.toml`** (project level):
```toml
providers = ["claude", "gemini"]   # which providers to generate config for

[[mcp_servers]]
id = "ship"
url = "http://localhost:7825/sse"
```

**Feature frontmatter** (per-feature override):
```toml
[agent]
providers = ["claude"]             # empty = inherit from project
mcp_servers = [{id = "ship"}]      # empty = all project servers
skills = [{id = "task-policy"}]    # empty = project default skills
```

## ManagedState Tracking

Ship maintains a state file (`.ship/mcp_managed_state.toml`) that tracks which MCP server IDs it wrote per provider. This allows teardown to remove only Ship-owned entries without clobbering user-managed servers.

State structure (keyed by provider id):
```toml
[tools.claude]
managed_ids = ["ship", "github"]

[tools.gemini]
managed_ids = ["ship"]
```

## UI Requirements

### Provider List View
- List all registered providers with: id, name, binary, detection status (installed / not found)
- Show which providers are enabled in `ship.toml`
- Toggle to enable/disable a provider for the project

### Provider Detail / Edit
- View all descriptor fields (read-only for alpha)
- Enable / disable for this project
- View generated config file path and current contents
- "Regenerate" button → calls `ship git sync` for this provider

### Feature-Level Override Panel
- In Feature editor: expand `[agent]` section
- Multi-select providers (defaults to project providers)
- Multi-select MCP servers (defaults to all project servers)
- Multi-select skills (defaults to project default skills)

### MCP Server Management
- List servers from `ship.toml [[mcp_servers]]`
- Add / remove / edit entries
- "Test connection" button (HTTP ping to URL)
- Per-provider view: which servers are currently written to each provider's config

## Future Providers (candidate list)

| ID | Tool | Config Location |
|---|---|---|
| `cursor` | Cursor | `.cursor/mcp.json` |
| `windsurf` | Windsurf / Cascade | `.windsurf/mcp.json` |
| `copilot` | GitHub Copilot (VS Code) | `.vscode/mcp.json` |
| `zed` | Zed | `~/.config/zed/settings.json` |
| `continue` | Continue.dev | `.continue/config.json` |
| `cline` | Cline (VS Code) | `.vscode/cline_mcp_settings.json` |
| `openhands` | OpenHands | `.openhands/config.toml` |
| `amp` | Amp | `~/.amp/settings.json` |

## Open Questions

- Should providers be extensible by users (plugins) or kept as a curated static list for alpha?
- Global vs project config: some tools (Cursor, Zed) use global config — should Ship write global config or only project-scoped?
- Skills output for non-Claude providers: Gemini/Codex have no native slash-command equivalent — embed in prompt file or skip?
- Provider detection: should Ship warn on `ship git sync` if a declared provider's binary is not in PATH?
