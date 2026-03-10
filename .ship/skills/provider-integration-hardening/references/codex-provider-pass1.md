# Codex Provider Pass 1 (Detection / Configuration / Import-Export)

Validated on: 2026-03-09
Sources:
- https://developers.openai.com/codex/config-basic
- https://developers.openai.com/codex/mcp
- https://raw.githubusercontent.com/openai/codex/main/codex-rs/core/config.schema.json
- Runtime mapping: `core/runtime/src/agents/export/sections/provider_registry.rs`, `core/runtime/src/agents/export/sections/sync_and_mcp.rs`, `core/runtime/src/agents/export/sections/permissions.rs`
- Tests: `core/runtime/src/agents/export/sections/tests.rs`

## Capability: Detection

### Provider docs say
- Codex config layering uses `~/.codex/config.toml` (user) and `.codex/config.toml` (project), plus CLI overrides (`-c` / `--config`).
- MCP is configured under `mcp_servers`.

### Ship currently does
- Provider registry models Codex as TOML with project path `.codex/config.toml`, global path `.codex/config.toml` under home, and MCP key `mcp_servers`.
- Binary detection uses `which` + PATH fallback; version detection uses `codex --version`.
- Model discovery reads Codex config/env hints and Ship AI model override.

### Gap
- No capability-level diagnostics yet beyond parse/shape checks.

### Action
- Advanced provider UI now shows both expected project/global config paths and surfaces parse diagnostics.

### Verification
- `codex_writes_to_codex_config_toml`
- preflight provider TOML checks in `validate_provider_configs`

## Capability: Configuration

### Provider docs say
- MCP transport/config uses `mcp_servers.<id>` entries (stdio and HTTP forms).
- Codex schema exposes per-server tool filters (`enabled_tools`, `disabled_tools`) and per-tool toggles via `tools.*` blocks.
- Project-level settings override user-level defaults.

### Ship currently does
- Ship exports MCP into `.codex/config.toml` under `mcp_servers` and preserves unmanaged user entries.
- Ship always injects `mcp_servers.ship` managed entry.
- Ship writes permissions inline into Codex config fields (`sandbox_mode`, `approval_policy`, `allow`, `rules.prefix_rules`, `sandbox_workspace_write.network_access`).
- Ship skill export target for Codex is `.agents/skills/<id>/SKILL.md`.

### Gap
- Pass 1 UI does not yet expose Codex-native `mcp_servers.<id>.enabled_tools/disabled_tools` and all `tools.*` toggles.
- Provider-native controls exist in Codex schema but are not yet compiled from Ship provider UI.

### Action
- Keep canonical controls in Ship permissions + MCP model for now; expand to Codex-native advanced toggles during MCP-focused pass.

### Verification
- `codex_uses_mcp_servers_underscore_not_hyphen`
- `codex_preserves_user_servers`
- `codex_exports_permissions_to_native_fields`

## Capability: Import / Export

### Provider docs say
- User and project config files are both valid sources; project should take precedence in-session.

### Ship currently does
- Export is non-destructive merge into project `.codex/config.toml`.
- Import resolution is project-first with global fallback.
- Import filters out reserved/invalid entries and dedupes existing server ids.
- Permissions import reads Codex-native fields (`allow`, `sandbox_workspace_write.network_access`, `rules.prefix_rules`) back into Ship canonical permissions.

### Gap
- Import/export existed in runtime but import action was not first-class in provider UI.

### Action
- Added provider import command in backend and Import button in providers page (project scope only) with result summary.

### Verification
- `import_from_codex_reads_project_config`
- `import_from_codex_uses_global_fallback_when_project_config_missing`
- `codex_permissions_round_trip_imports_back_to_canonical`

## Capability: Hooks + Tool Control (Security Surface)

### Provider docs say
- Current Codex config schema and docs expose tool/sandbox controls and MCP tool filtering.
- No native lifecycle hook section equivalent to Claude/Gemini hook events was identified in Codex config docs/schema.

### Ship currently does
- Ship stores hooks in canonical config but intentionally skips Codex native hook export.
- Security control for Codex today is primarily via sandbox/approval/prefix rules plus MCP-level controls.

### Gap
- No Codex-native hook interception layer to mirror Claude/Gemini hook lifecycle behavior.

### Action
- Keep hook orchestration provider-agnostic in Ship core and only export where provider supports it.
- Continue investing in MCP + permission enforcement for Codex path.

### Verification
- `claude_trigger_name`/`gemini_trigger_name` mappings exist; no Codex hook exporter path exists by design.
- Codex permission export/import tests validate the current security control channel.
