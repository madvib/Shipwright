+++
id = "2njbaSvV"
title = "Shipwright Alpha — AI Config and Modes (Canonical Spec)"
created = "2026-02-28T21:15:00Z"
updated = "2026-02-28T21:15:00Z"
tags = []
+++

# Shipwright Alpha — AI Config and Modes (Canonical Spec)

**Status:** Active  
**Version:** 0.1  
**Last Updated:** 2026-02-25

---

## Purpose

Define the single source of truth for alpha behavior around:

- AI pass-through generation
- Unified agent config (global + project)
- MCP registry and export
- Modes as capability boundaries
- UI and backend contracts
- Required test coverage

---

## Scope

### In scope

- Providers: `claude`, `gemini`, `codex`
- Project config: `.ship/config.toml`
- Global config: `~/.ship/config.toml`
- Agent layer fields: `skills`, `prompts`, `context`, `rules`
- Mode definitions + active mode
- MCP server registry + export to target tool configs
- Tauri command surface and settings UI for the above

### Out of scope

- Cursor/Windsurf export targets
- Third-party plugin SDK
- Marketplace/discovery network features
- MCP sampling as primary generation path

---

## Canonical Decisions

1. Naming and paths
- Product is `Shipwright`; CLI is `ship`
- Project storage is `.ship/`; global storage is `~/.ship/`

2. File conventions
- Domain docs are `.md` with TOML frontmatter
- Configuration is TOML only

3. AI architecture
- Generation is pass-through subprocess invocation of installed provider CLIs
- Shipwright does not proxy model APIs in alpha

4. Config layering
- Effective config = global + project with project overrides
- Project-local values can extend global lists (skills/prompts/context/rules)

5. Modes
- Modes are first-class and explicit
- Active mode controls visible/active capability surface

6. MCP export correctness
- Preserve user-owned config data where possible
- Shipwright-managed entries are tracked and safely rewritten
- Codex key is `mcp_servers` (underscore), never `mcp-servers`

7. Workflow policy context
- Mode remains an agent runtime concern, not a PM object
- Active workflow policy/phase must be available in agent context
- Automatic mode switching is deferred; alpha uses explicit mode changes

---

## Architecture and Ownership (Alpha)

### Authoritative implementation path

- `crates/logic`: config model, merge semantics, export logic, core primitives
- `crates/cli`: operational entry points for core workflows
- `crates/ui/src-tauri`: typed command boundary to core logic
- `crates/ui`: settings and mode/agent UX

### Parallel skeleton handling

`crates/runtime`, `crates/modules`, and `crates/sdk` remain as architectural scaffolding for plugin/runtime evolution, but they are not a competing alpha source of truth.

Rule: no duplicate production behavior across both paths during alpha.

---

## Data Model Contract (Alpha)

`ProjectConfig` must support:

- `ai` provider settings
- `agent` layer (`skills`, `prompts`, `context`, `rules`)
- `modes`
- `mcp_servers`
- `active_mode`

Any additions should preserve stable TOML compatibility and default-safe loading.

---

## Export Target Contract (Alpha)

- Claude: project `.mcp.json` and/or global compatibility as implemented
- Gemini: `.gemini/settings.json` with non-MCP field preservation
- Codex: `.codex/config.toml` with `[mcp_servers.*]`

All exporters must handle malformed existing files safely and return actionable errors.

---

## UI Contract (Alpha)

Settings/Agents must provide:

- Global vs project scope selection
- Provider selection and CLI path override
- Agent layer editing (`skills/prompts/context/rules`)
- Mode CRUD + active mode selection
- MCP server CRUD
- Export actions per provider

The mode surface should be prominent and understandable as capability control.

---

## Required Tests

Minimum confidence suite for alpha:

1. Config merge and precedence
- Global/project merge behavior
- List merge and dedupe for agent layer fields

2. Mode and capability invariants
- Active mode resolution
- Mode updates maintain valid active state

3. Export correctness
- Round-trip write/read for each provider
- Preserve user-managed data where expected
- Codex `mcp_servers` strictness

4. Pass-through generation
- Provider command invocation path
- Error propagation for missing binaries and command failures

---

## Deferred (Post-Alpha)

- Public third-party SDK
- Plugin marketplace and distribution
- Advanced module entitlement/commercial packaging
- Broader tool target matrix beyond `claude/gemini/codex`

## Deferred Spec Workflow Enhancements

Capture these for follow-on implementation once alpha core is stable:

- Requirement-to-issue trace links in spec metadata
- Acceptance criteria blocks with test hooks
- Spec conflict/lint checks for duplicate or stale directives
- Spec status lifecycle (`draft -> active -> superseded`) with ownership
- Lightweight CI checks that map spec requirements to test coverage

---

## References

Archived deep-dive docs retained for historical context:

- `.ship/specs/archive/2026-02-consolidation/mcp-cli-config-guide-alpha.md`
- `.ship/specs/archive/2026-02-consolidation/mcp-config-ui-guide-alpha.md`
- `.ship/specs/archive/2026-02-consolidation/ship-alpha-ui-action-plan.md`
