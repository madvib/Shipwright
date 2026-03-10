# MCP Pass 1 (Ship Studio Settings)

Validated: 2026-03-09  
Scope: Studio MCP settings (`Providers` + `MCP` + `Permissions`) and export-aligned policy behavior.

## Capability Matrix

### Validation
- Provider docs say: MCP config shape differences and parser failures are common integration breakpoints.
- Ship currently does: `validate_mcp_servers_cmd` checks server config validity plus provider config parse/shape issues.
- Gap: none for preflight coverage baseline.
- Action: keep validation issues visible in both Providers and MCP pages.
- Verification: `crates/ui/src-tauri/src/lib.rs` (`validate_mcp_servers_cmd`), MCP page preflight panel.

### Runtime Discovery
- Provider docs say: runtime tool surfaces can differ from static template metadata.
- Ship currently does: `probe_mcp_servers_cmd` performs runtime probing:
  - `stdio`: MCP `initialize` + `tools/list`
  - `http`/`sse`: endpoint reachability check (tool listing currently stdio-only)
- Gap: network transport tool enumeration is not implemented yet.
- Action: expose partial status for network transports and keep tool discovery best-effort.
- Verification: `crates/ui/src-tauri/src/lib.rs` (`probe_mcp_servers_cmd`, `probe_mcp_stdio_server`, `probe_mcp_network_server`).

### Discovery Persistence
- Provider docs say: runtime discoverability should remain usable across sessions.
- Ship currently does: persists discovery cache to JSON (project/global scope) so autocomplete and policy controls are durable.
- Gap: cache invalidation policy is basic (manual refresh + probe updates).
- Action: keep disk cache as source for suggestions; expand invalidation heuristics later.
- Verification: `get_agent_discovery_cache_cmd`, `refresh_agent_discovery_cache_cmd` in `crates/ui/src-tauri/src/lib.rs`; cache file paths in `docs/agent-settings-ui.md`.

### Tool-Level Controls
- Provider docs say: policy should be explicit and auditable per tool/server.
- Ship currently does: MCP server rows render discovered tools with one-click block/allow and server-wide block-all.
- Gap: controls compile through canonical permissions; hook-level runtime filtering expansion is future work.
- Action: store control state in `permissions.tools.deny` patterns (`mcp__<server>__*`, `mcp__<server>__<tool>`).
- Verification: `crates/ui/src/features/agents/AgentsPanel.tsx` (MCP row controls + permissions mutations).

### Permissions UX (Autocomplete)
- Provider docs say: command/tool/path policy should be fast to author and explicit.
- Ship currently does:
  - `Tools` suggestions from MCP cache + catalog + skill `allowed-tools`.
  - `Commands` tab with allow/deny/require-confirmation and CLI autocomplete.
  - `Filesystem` suggestions from discovered workspace paths.
- Gap: no UI-side fuzzy ranking yet (currently de-duplicated union).
- Action: keep deterministic suggestions for alpha; layer ranking later.
- Verification: `AgentsPanel.tsx` permissions tab and discovery queries.

### Autocomplete / Inference
- Provider docs say: configuration UX should minimize brittle manual entry.
- Ship currently does: permission tool suggestions include discovered runtime tool patterns after probe.
- Gap: none for pass-1 inference target.
- Action: keep probe-derived suggestions merged with catalog/server baselines.
- Verification: `permissionToolSuggestions` memo in `AgentsPanel.tsx`.

## User-Facing Notes

- Probe status values: `ready`, `partial`, `needs-attention`, `disabled`.
- MCP page now shows both static preflight and runtime probe summaries.
- Per-tool controls are immediate and auditable; they feed provider-native policy export on sync.
