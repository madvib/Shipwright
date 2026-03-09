# Provider Matrix (Current Baseline)

## Claude

- Config: JSON
- MCP key: `mcpServers`
- Hook export: supported
- Typical hook events: `SessionStart`, `UserPromptSubmit`, `PreToolUse`, `PostToolUse`, `Stop`, `Notification`
- Notes: prefers grouped hook entries under `hooks.<Event>[]`.

## Gemini

- Config: JSON
- MCP key: `mcpServers`
- Hook export: supported
- Typical hook events: `BeforeTool`, `AfterTool`, `SessionStart`, `SessionEnd`, `BeforeModel`, `AfterModel`
- Notes: supports grouped hook entries; MCP transport type should be explicit.

## Codex

- Config: TOML
- MCP key: `mcp_servers`
- Hook export: not supported natively (current schema)
- Notes: preserve internal hook config but skip native write.

## How To Extend For New Providers

1. Add provider descriptor (paths, MCP shape, transport fields).
2. Add dynamic model discovery probes (provider config + env vars).
3. Add hook mapping only if provider docs define hook lifecycle schema.
4. Add export/import tests.
5. Update docs and this matrix.
