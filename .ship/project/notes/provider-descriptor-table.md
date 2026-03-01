+++
id = "e1d865ca-a551-46b9-a99c-472d552d1054"
title = "Provider Descriptor Table"
created = "2026-02-27T05:29:08.766230960Z"
updated = "2026-02-27T05:29:08.766230960Z"
tags = []
+++

# Provider Descriptor Table

Reference for implementing new agent config providers in Ship.
Each row is a `ProviderDescriptor` entry in the static registry.

## Known Providers (Alpha: claude, gemini, codex)

| Concern | Claude | Gemini | Codex |
|---|---|---|---|
| `id` | `claude` | `gemini` | `codex` |
| `name` | Claude Code | Gemini CLI | Codex CLI |
| `binary` | `claude` | `gemini` | `codex` |
| Project config path | `.mcp.json` | `.gemini/settings.json` | `.codex/config.toml` |
| Global config path | `~/.claude.json` | `~/.gemini/settings.json` | `~/.codex/config.toml` |
| Config format | JSON | JSON | TOML |
| MCP container key | `mcpServers` | `mcpServers` | `mcp_servers` (**underscore** — hyphen silently fails) |
| Stdio entry fields | `command`, `args`, `type: "stdio"` | `command`, `args` (no type field) | `command`, `args` |
| HTTP entry fields | `type: "http"`, `url` | `httpUrl` (**not** `url`) | `url` |
| Managed marker | `_ship.managed: true` inline in entry | `_ship.managed: true` inline | `mcp_managed_state.toml` only (TOML can't embed markers) |
| System prompt | `CLAUDE.md` file at project root | `GEMINI.md` file at project root | `instructions` key in `.codex/config.toml` |
| Skills / commands | `.claude/commands/<id>.md` | none | none |
| Hooks | `~/.claude/settings.json` | none | none |
| Detection binary | `claude` | `gemini` | `codex` |

## Descriptor Fields

```rust
pub struct ProviderDescriptor {
    pub id: &'static str,
    pub name: &'static str,
    pub binary: &'static str,           // for PATH detection
    pub project_config: &'static str,   // relative to project root
    pub global_config: &'static str,    // relative to home dir
    pub config_format: ConfigFormat,    // Json | Toml
    pub mcp_key: &'static str,          // "mcpServers" or "mcp_servers"
    pub http_url_field: &'static str,   // "url" or "httpUrl"
    pub type_field: bool,               // emit "type": "stdio" in stdio entries
    pub managed_marker: ManagedMarker,  // Inline | StateFileOnly
    pub prompt_output: PromptOutput,    // ClaudeMd | GeminiMd | InstructionsKey | None
    pub skills_output: SkillsOutput,    // ClaudeCommands | None
}
```

## Adding Provider #4+

1. Add a new `ProviderDescriptor` entry to the static `PROVIDERS` slice in `agent_export.rs`
2. Add a `providers` entry to `ship.toml` or let users add it via `ship config provider add <id>`
3. If the provider has quirks not covered by existing fields, add a new variant to the relevant enum

## Candidate Future Providers

- Cursor (`cursor`) — `.cursor/mcp.json`, JSON, `mcpServers`
- Windsurf / Codeium — TBD
- Copilot (GitHub) — TBD
- Zed — TBD
- Continue.dev — `.continue/config.json`
- Cline / RooCode — `.vscode/settings.json` partial
- OpenHands — TBD
- Amp — TBD
