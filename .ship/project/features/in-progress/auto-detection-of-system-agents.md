+++
id = "TyEDHXCL"
title = "Auto-Detection of System Agents"
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

Shipwright generates agent configs (CLAUDE.md, .mcp.json, .gemini/, .codex/) for the agents the developer actually has installed. Without detection, it generates configs for tools the user doesn't have, clutters their environment, or silently produces useless files. Detection also lets the UI show which providers are available and guide configuration accordingly.

## Acceptance Criteria

- [ ] Detect presence of: `claude`, `gemini`, `codex` binaries via PATH
- [ ] Read installed version for each detected agent
- [ ] `ProviderInfo` includes `installed: bool` and `version: Option<String>`
- [ ] Config generation skips providers that are not installed (unless explicitly configured)
- [ ] `ship config list-providers` shows detection results
- [ ] UI settings panel shows detected providers with install status and version

## Delivery Todos

- [ ] `detect_binary(binary)` and `detect_version(binary)` already implemented — verify correctness
- [ ] Wire detection into `export_agent_config` to skip uninstalled providers
- [ ] `ship config list-providers` CLI command
- [ ] MCP: `list_providers` tool (already partially in `agent_export.rs`)
- [ ] UI provider status display in settings

## Notes

Detection is best-effort — we check PATH, not system package managers. If a binary exists but fails version check, `installed = true`, `version = None`. The user can always override detection by explicitly listing providers in `ship.toml`. Multi-provider dispatch is already implemented in the `providers` field — this feature wires detection to make the default smarter.
