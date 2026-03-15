# Provider Integration Dogfood (2026-03-09)

Applied `provider-integration-hardening` across the current provider set.
Skill package path: `.ship/agents/skills/provider-integration-hardening/`.

## Claude

- Audit result:
  - Hook export schema was stale (flat array, no grouped `hooks` entries).
- Implemented:
  - Grouped hook export in `.claude/settings.json` (project-local) with nested command hooks.
  - Added support for newer Claude events in Ship trigger model.
  - Added hook metadata export (`timeout`, `description`).
- Verification:
  - Runtime export tests validate grouped schema and metadata fields.

## Gemini

- Audit result:
  - Native hook export was missing.
  - Export omitted MCP `type` metadata, reducing transport clarity.
- Implemented:
  - Hook export to `.gemini/settings.json` under `hooks.<Event>[]` grouped structure.
  - Trigger mapping for Gemini hook lifecycle events.
  - MCP export now emits `type` for Gemini entries.
- Verification:
  - Runtime export tests validate hook persistence and hook payload shape.

## Codex

- Audit result:
  - No native hooks section in Codex config schema.
- Implemented:
  - Explicit non-export stance: Ship keeps hooks in project state and skips Codex native hook write.
  - UI now surfaces this capability gap clearly in Hooks settings.
- Verification:
  - Behavior documented in `docs/agent-configuration.md` and Hooks UI reference panel.

## Follow-up Template For New Providers

1. Add provider descriptor and config transport mapping.
2. Extend hook-trigger mapping only if provider docs support hooks.
3. Implement dynamic model discovery (provider config + env + Ship ai.model).
4. Add provider-specific runtime export/import tests.
5. Update `docs/agent-configuration.md` + this dogfood file.
