# Agent Permissions Schema

Ship uses a single canonical permissions schema at:

- `.ship/agents/permissions.toml`

This file is parsed by `core/runtime/src/agents/permissions.rs` and consumed by agent config resolution + provider export.

## Canonical TOML Shape

```toml
[tools]
allow = ["*"]
deny = []

[filesystem]
allow = []
deny = []

[commands]
allow = []
deny = []

[network]
policy = "none" # none | localhost | allow-list | unrestricted
allow_hosts = []

[agent]
max_cost_per_session = 10.0
max_turns = 40
require_confirmation = ["git push", "rm -rf"]
```

## Mode Override Semantics

Mode-level `permissions` (from `ModeConfig.permissions`) overlays only `tools` permissions:

- `mode.permissions.allow` replaces `permissions.tools.allow` when non-empty.
- `mode.permissions.deny` replaces `permissions.tools.deny` when non-empty.
- `filesystem`, `commands`, `network`, and `agent` remain from canonical `permissions.toml`.

This keeps one base policy surface while still allowing mode-specific tool gating.

## Provider Mapping

Current export mappings are explicit and provider-specific:

- `claude`:
  - Output: `$WORKSPACE_ROOT/.claude/settings.json`
  - Mapping: `permissions.tools.allow/deny` -> `settings.permissions.allow/deny`
- `gemini`:
  - Output: `$WORKSPACE_ROOT/.gemini/policies/ship-permissions.toml`
  - Mapping:
    - `tools.allow/deny` -> `[[rule]] toolName=... decision=allow|deny`
    - `commands.allow/deny` -> `[[rule]] toolName="run_shell_command" commandPrefix|commandRegex ...`
    - `agent.require_confirmation` -> `[[rule]] ... decision="ask_user"`
- `codex`:
  - Output:
    - `$WORKSPACE_ROOT/.codex/config.toml`
    - `$WORKSPACE_ROOT/.codex/rules/ship.rules`
  - Mapping:
    - `network.policy` -> `sandbox_workspace_write.network_access` (lossy)
    - `commands.allow|deny` + `agent.require_confirmation` -> `prefix_rule(...)` entries in `.codex/rules/ship.rules`
      - decisions: `allow` / `forbidden` / `prompt`
    - global safety -> `sandbox_mode = "workspace-write"` + `approval_policy = "on-request"` (or `"on-failure"` for permissive defaults)

When adding new provider mappings, keep conversion in `agent_export.rs` explicit (no implicit key mirroring), and add provider-specific tests.

## Import Back To Canonical

`runtime::agent_export::import_permissions_from_provider(provider, ship_dir)` supports:

- `claude`: reads workspace `.claude/settings.json` permissions allow/deny arrays (falls back to `~/.claude/settings.json` only when project-local file is missing).
- `gemini`: reads workspace `ship-permissions.toml` policy rules.
- `codex`: reads `.codex/rules/*.rules` (`prefix_rule(...)`) plus legacy fallback fields in `.codex/config.toml`.

Imports are lossy where provider models are less expressive than the canonical schema.
