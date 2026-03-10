# Gemini Provider Pass 1 (Detection / Configuration / Import-Export)

Validated on: 2026-03-09
Sources:
- https://geminicli.com/docs/reference/configuration/
- Runtime mapping: `core/runtime/src/agents/export/sections/provider_registry.rs`, `core/runtime/src/agents/export/sections/sync_and_mcp.rs`, `core/runtime/src/agents/export/sections/permissions.rs`
- Tests: `core/runtime/src/agents/export/sections/tests.rs`

## Capability: Detection

### Provider docs say
- Gemini has layered settings precedence; user settings are `~/.gemini/settings.json`, project settings are `.gemini/settings.json`.
- `mcpServers` is the settings key for MCP registry.

### Ship currently does
- Provider registry describes Gemini as JSON config with project path `.gemini/settings.json`, global path `.gemini/settings.json` (resolved under home), and MCP key `mcpServers`.
- Binary detection uses `which` + PATH fallback; version detection uses `gemini --version`.
- Model discovery pulls from env and config hints (`model`, `model.name`, `modelConfigs`) via provider model discovery.

### Gap
- No explicit capability snapshot tied to provider docs version.
- Detection status exists, but capability-level diagnostics are still limited to preflight parse checks.

### Action
- Keep provider row always visible; enrich advanced diagnostics with project/global config paths and preflight issues.
- Keep hooks/tool capability matrix in internal docs, not hardcoded in user-facing copy.

### Verification
- `gemini_writes_to_gemini_settings_json`
- `gemini_preserves_non_mcp_fields`
- `validate_provider_configs` surfaces JSON/MCP-key issues in UI preflight.

## Capability: Configuration

### Provider docs say
- Project and user settings can coexist with precedence.
- Hooks are configured under `hooks.*`; canonical toggle is `hooksConfig.enabled`.
- MCP server entries support include/exclude tool lists (e.g. `excludeTools`) and transport-specific fields.

### Ship currently does
- Ship exports MCP into `.gemini/settings.json` under `mcpServers` and preserves non-MCP fields.
- HTTP/SSE entries use Gemini field conventions (`httpUrl` mapping).
- Ship exports hook groups to `.gemini/settings.json` using mapped event names (`BeforeTool`, `AfterTool`, `SessionStart`, `SessionEnd`, `PreCompress`, etc.).
- Tool/command policy exports to `.gemini/policies/ship-permissions.toml`.

### Gap
- Ship does not yet compile provider-specific toggles like `hooksConfig.enabled` / `hooksConfig.disabled`; current UX centers on Ship canonical hooks + permissions.
- MCP include/exclude tool controls are not surfaced yet in provider advanced UI.

### Action
- Keep hooks as internal implementation detail, expose only high-signal policy controls first.
- Add provider-specific advanced controls later for hook global enable/disable and server tool include/exclude.

### Verification
- `gemini_http_uses_httpurl_not_url`
- `gemini_exports_hooks_to_settings_json`
- `gemini_exports_workspace_policy_from_permissions`

## Capability: Import / Export

### Provider docs say
- Settings are layered across user/project files; project overrides user.
- MCP definitions are stored in `mcpServers`.

### Ship currently does
- Export writes/merges project `.gemini/settings.json` (MCP + hooks) non-destructively.
- Import resolves paths project-first then global fallback; imports only valid non-managed servers.
- Permission import currently targets Ship-exported policy file `.gemini/policies/ship-permissions.toml`.

### Gap
- Import path behavior is runtime-correct but previously under-exposed in UI.
- Permission import from arbitrary third-party Gemini policy docs is intentionally limited.

### Action
- Added provider import command to UI/backend and surfaced import status in provider rows.
- Provider advanced accordion now shows expected project/global paths for clarity.

### Verification
- `import_from_gemini_reads_project_config`
- `import_from_provider` project-first behavior via shared import path resolver
- `gemini_permissions_round_trip_imports_back_to_canonical`

## Capability: Hooks + Tool Control (Security Surface)

### Provider docs say
- Gemini supports lifecycle hooks and hook-level enable/disable controls.
- MCP server entries support tool filtering (`includeTools`/`excludeTools`).

### Ship currently does
- Hook mapping supports a broad Gemini event surface; unsupported cross-provider events are skipped intentionally.
- Command/tool restrictions compile into Gemini policy rules (allow/deny/ask_user) rather than exposing raw provider policy internals in default UX.

### Gap
- UI has not yet exposed provider-native hook runtime toggles or MCP per-server tool include/exclude controls.

### Action
- Keep pass-1 scope focused: detection/config/import/export parity and safe defaults.
- Defer full provider-native fine-grained controls to next pass (MCP-focused element).

### Verification
- Hook mapping in `gemini_trigger_name` + export tests above
- Policy round-trip test for allow/deny/ask_user command semantics
