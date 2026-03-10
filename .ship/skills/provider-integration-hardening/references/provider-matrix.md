# Provider Matrix (Current Baseline)

## Pass Tracking

- Claude: provider-level pass 1 complete (`references/claude-provider-pass1.md`)
- Gemini: provider-level pass 1 complete (`references/gemini-provider-pass1.md`)
- Codex: provider-level pass 1 complete (`references/codex-provider-pass1.md`)
- MCP settings pass 1 complete (`references/mcp-pass1.md`)

## Claude

- Config format: JSON
- Project config: `.mcp.json`
- Global config: `~/.claude.json`
- MCP key: `mcpServers`
- Hook export: supported (`~/.claude/settings.json`)
- Permission import/export: `~/.claude/settings.json`
- Import precedence: project first, then global fallback

## Gemini

- Config format: JSON
- Project config: `.gemini/settings.json`
- Global config: `~/.gemini/settings.json`
- MCP key: `mcpServers`
- Hook export: supported (`.gemini/settings.json`)
- Permission import/export: `.gemini/policies/ship-permissions.toml`
- Import precedence: project first, then global fallback (same path family)
- Notes: HTTP transport uses `httpUrl` field on export/import

## Codex

- Config format: TOML
- Project config: `.codex/config.toml`
- Global config: `~/.codex/config.toml`
- MCP key: `mcp_servers`
- Hook export: not supported natively (Ship stores hooks but skips Codex-native write)
- Permission import/export: inline in `.codex/config.toml`
- Import precedence: project first, then global fallback
- Notes: supports MCP tool filtering/toggles via permission export; Ship UI now exposes MCP discovery-driven tool toggles in canonical permissions.

## Provider UI (Pass 1)

- Supported provider rows render immediately (`Claude`, `Gemini`, `Codex`).
- Provider import/export is now first-class for all three providers.
- Advanced accordion now shows expected project + global config paths and provider diagnostics.

## How To Extend For New Providers

1. Add provider descriptor (paths, config format, MCP shape, transport fields).
2. Add dynamic model discovery probes (provider config + env vars).
3. Add hook mapping only if provider docs define native hook lifecycle schema.
4. Add import/export + permission mapping tests.
5. Update provider pass docs and this matrix.
