# Agent Rules Contract

Ship rules are always-on markdown guardrails stored in:

- `.ship/agents/rules/*.md`

Rules are file-backed (not SQLite-backed). Modes in SQLite can filter which rules are active by rule ID.

## Runtime Rule File Contract

- Rule files must be markdown files (`.md`).
- `README.md` in the rules directory is reserved and is not treated as a rule.
- Rule CRUD only accepts a single file name (no directory traversal or nested paths).
- If extension is omitted (e.g. `core-principles`), runtime normalizes to `core-principles.md`.

## Mode Rule References

`ModeConfig.rules` stores rule IDs, not full paths.

Matching behavior:

- `core-principles` matches `core-principles.md`
- legacy numeric prefixes are normalized during matching:
  - mode value `runtime-hardening` matches file `010-runtime-hardening.md`
  - mode value `010-runtime-hardening` also matches `010-runtime-hardening.md`

This preserves compatibility while allowing simpler IDs in modes.

## Resolution Order

Rules are resolved in `resolve_agent_config`:

1. List all rule files from `.ship/agents/rules`.
2. If active mode has `rules` filters, retain only matching rule IDs.
3. Return filtered rules in `AgentConfig.rules`.

## Test Coverage

- Rule CRUD + filename validation:
  - `core/runtime/src/agents/rule.rs` tests
- Mode-based rule filtering:
  - `core/runtime/src/agents/config.rs` tests (`resolve_agent_config_mode_filters_skills_and_rules`)
