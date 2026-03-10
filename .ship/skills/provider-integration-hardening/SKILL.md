---
name: provider-integration-hardening
description: Internal workflow for provider-level integration hardening in Ship. Execute one element at a time (detection, configuration, import/export), one provider at a time (Claude -> Gemini -> Codex), and capture precise implementation knowledge in references.
compatibility: Requires repo write access and ability to run runtime tests.
allowed-tools:
  - Read
  - Edit
  - Bash
metadata:
  display_name: Provider Integration Hardening
  source: custom
  owner: ship-core
  scope: internal
  skill_version: 3
  last_updated: 2026-03-10
---

# Provider Integration Hardening

This skill is for internal Ship contributors. Use it when hardening provider integrations.
The model is strict:

1. Pick one **element** only (for now: provider-level `detection`, `configuration`, `import/export`).
2. Pick one **provider** only (start with Claude).
3. Complete deeply, document ground truth, then move to next provider.

## Provider-Level Runbook (Element Pass 1)

### Step A: Docs + Runtime Truth

1. Read official provider docs for:
- config file locations (global/user/project/local)
- MCP schema and path precedence
- hooks/events and settings knobs
- permission model and default behavior

2. Read Ship runtime mappings:
- `core/runtime/src/agents/export/sections/provider_registry.rs`
- `core/runtime/src/agents/export/sections/sync_and_mcp.rs`
- `core/runtime/src/agents/export/sections/permissions.rs`
- `crates/ui/src/features/agents/AgentsPanel.tsx`

3. Record exact behavior with file references and dates in `references/<provider>-provider-pass1.md`.

Required output format:
- `Capability`
- `Provider docs say`
- `Ship currently does`
- `Gap`
- `Action`
- `Verification`

### Step B: Detection

Document:
- binary detection method
- version detection method
- model discovery method
- when autodetect runs (init only vs every open)
- failure modes and user-facing diagnostics

### Step C: Configuration

Document:
- all files Ship reads and writes for that provider
- project/global precedence rules
- settings that are intentionally Ship-managed vs untouched
- advanced/escape-hatch settings (and whether to expose)

### Step D: Import / Export

Document:
- import source order and merge behavior
- export target files and merge behavior
- teardown behavior
- data loss risks, id collisions, managed marker behavior

### Step E: Hooks as Internal Detail

Treat raw hooks as implementation detail by default.
Prefer provider-level policy toggles that compile into hook/settings output.
Only expose raw hook editing in advanced mode.

## MCP Element Runbook (Element Pass 2)

After provider pass 1 is complete for all supported providers, run MCP as a dedicated element:

1. **Validation:** ensure config parsing and provider shape checks are explicit and actionable.
2. **Discovery:** probe configured MCP servers and capture reachable status + discovered tools.
3. **Controls:** expose per-server and per-tool policy toggles that compile to canonical permissions.
4. **Export mapping:** verify deny/allow patterns map correctly into Claude/Gemini/Codex policy outputs.
5. **UX density:** keep MCP status + controls inline in server rows; avoid detached empty panels.

Required verification:

- `cargo check -p ui`
- `cargo run -p ui --bin ship -- gen-bindings`
- docs updated (`docs/agent-settings-ui.md`) for any new user-facing behavior

## Sequence

Run this skill in this order:

1. `Claude` (complete first; this is the quality bar)
2. `Gemini`
3. `Codex`

Each provider pass must update:
- provider-specific reference doc
- matrix summary
- tests or validation checklist if behavior changed

## Current References

- `references/claude-provider-pass1.md`
- `references/gemini-provider-pass1.md`
- `references/codex-provider-pass1.md`
- `references/provider-matrix.md`
- `references/hook-mapping.md`
- `references/mcp-pass1.md`

## Validation Checklist (Provider-Level Pass)

- Runtime tests pass for changed provider export paths.
- Import/export order and path behavior are explicit and tested.
- Provider docs links are included with validation date.
- UI labels/tooltips match runtime truth.
- Unsupported provider capabilities are explicit, not implied.
