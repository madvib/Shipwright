# Agent Settings UI Reference

> User-facing reference for the Agent Settings pages in Studio (`Providers`, `MCP`, `Skills`, `Rules`, `Permissions`).
> Complements backend/export details in `docs/agent-configuration.md`.

---

## Scope Model

All agent settings are edited in one of two scopes:

- `Global`: defaults for all projects on this machine.
- `Project`: overrides for the active workspace only.

Scope behavior:

- if no project is active, UI stays in `Global` and shows an inline warning
- when a project becomes available, scope auto-switches to `Project` once
- project toggle tooltip explains why project scope is unavailable when no project is selected

---

## Providers Page

### AI Clients

Per-provider row controls:

- `Sync On` / `Sync Off` (include provider in export set for this scope)
- lightning selector for which provider powers in-app Ship AI features
- import provider-native config into Ship (project scope only)
- export Ship config to provider-native files
- install status badge (`installed` or `not found`)
- one provider sync-health status badge: `Ready`, `Needs attention`, or `Drift detected`
- per-provider advanced accordion (Claude first) for diagnostics + model/path controls

Tooltips explain:

- what `Sync On/Off` changes in exports
- that import writes into project Ship config (and is disabled outside project scope)
- what Export does
- why a provider is unavailable (`binary not found on PATH`)
- what sync health means:
  - `Ready`: current config can be synced
  - `Needs attention`: blocking issues were detected
  - `Drift detected`: provider config shape diverges from Ship-managed expectations

Provider rendering behavior:

- supported providers always render immediately (`Claude`, `Gemini`, `Codex`)
- install/version status updates asynchronously from provider detection
- provider health checks run from live MCP/provider preflight data (`Run checks`)
- on detection failure, UI shows a retry action and keeps supported rows visible

Advanced accordion fields:

- `Config Paths`: expected project path + expected global path + detected config file paths from diagnostics
- `In-App AI Model`: optional model ID for selected Ship AI provider (autocomplete from detected models)
- `CLI Path Override`: optional absolute binary path (blank = resolve from `PATH`)
- `Hook Surface`: native hook events available for that provider
- provider-scoped diagnostics list (`error`/`warning`/`info`)

### Modes

Mode controls include:

- start from built-in templates (`Frontend React`, `Rust Expert`, `Documentation Expert`)
- create custom template from name (mode ID inferred automatically)
- set active template
- link skill as system prompt
- select MCP servers available in template
- delete template

Templates include concrete tool-policy defaults and auto-link matching installed skills/MCP servers based on template hints.

---

## MCP Page

### Top Actions

- `Search MCP library templates`
- `Search official MCP Registry` (live remote discovery)
- `Use Template`
- `Validate MCP`
- `Probe Tools`
- `Add MCP Server`

Tooltips explain what each action does and when to use it.

Registry discovery behavior:

- queries official MCP Registry API (`/v0.1/servers`, latest versions)
- shows installable matches inline with one-click install
- pre-fills transport/command/url/env placeholders into server config
- flags entries that require headers so users can finish setup manually

### Preflight Validation (`Validate MCP`)

Preflight checks:

- server config validity (missing command/url, malformed JSON-like args, env key format)
- provider config parse/readability checks

Output includes:

- readiness status (`ready` / `needs attention`)
- issue level (`error`, `warning`, `info`)
- message, optional hint, and source path when available

### Runtime Capability Probe (`Probe Tools`)

Probe behavior:

- starts each configured, non-disabled MCP server
- for `stdio` servers: performs MCP `initialize` + `tools/list` and captures discovered tool names
- for `http`/`sse` servers: performs endpoint reachability check (tool enumeration is currently stdio-only)

Output includes:

- per-server status (`ready`, `partial`, `needs-attention`, `disabled`)
- reachability/tool discovery counts
- probe duration
- first warning/error excerpt when available

Persistence:

- probe-discovered MCP tools are cached to disk (not memory-only)
- project scope cache: `.ship/agents/discovery-cache.json`
- global scope cache: `~/.ship/agents/discovery-cache.json`
- permissions autocomplete reads cached discoveries even after app restart

### Tool Audit + Policy Controls

On each MCP server row:

- runtime status badge from the latest probe
- discovered tool chips (when available)
- one-click `Block All` / `All Blocked` toggle for `mcp__<server>__*`
- one-click per-tool toggle (blocked/allowed) for `mcp__<server>__<tool>`

These controls write directly into canonical `permissions.tools.deny` and are exported to provider-native policy surfaces at sync time.

### MCP Server Form

Fields include tooltips for:

- `Name`
- `Server ID`
- `Transport` (`stdio`, `sse`, `http`)
- `Command`/`Arguments` for `stdio`
- `URL` for `sse`/`http`
- `Environment Variables`

Inference/default behavior:

- server ID inferred from ID/name/command if omitted
- catalog templates default to `scope = "project"`
- common command/env suggestions are auto-populated from catalog + existing servers
- permission/autocomplete suggestions include discovered MCP tool patterns after probe

---

## Skills and Rules Pages

### Skills

Capabilities:

- create/edit/delete skills
- install from curated library
- install from `skills.sh` (skill ID or copied install command)
- studio folder-audit view via file-tree

Tooltips now cover:

- creating new skills
- catalog install action
- skills.sh install action
- studio/list toggle
- delete action

### Rules

Capabilities:

- create/edit/delete global rule documents

Rules are global instructions applied across sessions; they share the same editor ergonomics as skills.

---

## Hooks (Managed Runtime)

Hooks remain provider-agnostic in Ship config and are exported where supported, but there is currently no standalone Hooks settings page in Studio.
Hook policy is Ship-managed from provider export/runtime.

Provider status:

- `Claude`: native hooks supported
- `Gemini`: native hooks supported
- `Codex`: no native hook surface yet (stored in Ship config, not exported natively)

Managed runtime artifacts are written to:
- `.ship/generated/runtime/hook-context.md`
- `.ship/generated/runtime/envelope.json`
- `~/.ship/state/telemetry/hooks/events.ndjson` (internal telemetry)

Default managed hook command:
- `ship hooks run --provider <id>` (or `$SHIP_HOOKS_BIN` when set)

---

## Permissions Page

### Rule Sets

Presets:

- `Read-only`
- `Standard`
- `Full Access`

Applying a preset overwrites current permissions; tooltip warns about this.

### Capabilities

Tabs:

- `Tools`: allow/deny glob patterns
- `Commands`: allow/deny/require-confirmation command patterns with CLI autocomplete
- `Filesystem`: allow/deny path globs

Autocomplete suggestions are inferred from:

- configured MCP server IDs
- probe-discovered MCP tool IDs (persisted cache)
- MCP catalog IDs
- skill `allowed-tools` frontmatter hints
- discovered shell/CLI binaries (`ship`, `gh`, `git`, etc.)
- discovered workspace filesystem paths (persisted cache)
- seeded safe pattern/path templates

---

## Defaults Shipped Out of the Box

From current config normalization/runtime behavior:

- providers default: `['claude']`
- AI defaults: `provider='claude'`, `model=null`, `cli_path=null`
- modes default: none pre-created (preset templates are available in UI)
- MCP servers default: none
- hooks default: none
- agent layer defaults: empty `skills/prompts/context/rules`

Permissions are managed in `agents/permissions.toml` and can be seeded via presets in UI.

---

## Save and Export Semantics

- `Save Global Agent Config`: persists global defaults.
- `Save Project Agent Config`: persists project overrides.
- provider `Export`: writes provider-native files from Ship's unified config.
- provider `Import`: reads provider-native files and merges into project Ship config (`.ship/ship.toml` + permissions file when available).

Immediate-save provider actions:

- `Sync On/Off` persists immediately for active scope
- Ship AI provider selection (lightning) persists immediately for active scope
