# Agent Settings UI Reference

> User-facing reference for the Agent Settings pages in Studio (`Providers`, `MCP`, `Skills`, `Rules`, `Hooks`, `Permissions`).
> Complements backend/export details in `docs/agent-configuration.md`.

---

## Scope Model

All agent settings are edited in one of two scopes:

- `Global`: defaults for all projects on this machine.
- `Project`: overrides for the active workspace only.

The scope toggle now includes tooltips explaining exactly where changes are persisted.

---

## Providers Page

### AI Clients

Per-provider row controls:

- enable/disable provider in Ship config
- export Ship config to provider-native files
- install status badge (`installed` or `not found`)

Tooltips explain:

- what enable/disable changes in exports
- what Export does
- why a provider is unavailable (`binary not found on PATH`)

### Ship Generation

Fields:

- `Provider`: which installed client Ship uses for generation features
- `Model`: optional model ID (blank = provider default)
- `CLI Path Override`: optional absolute binary path (blank = resolve from `PATH`)

### Modes

Mode controls include:

- create mode from a name (mode ID inferred automatically)
- set active mode
- link skill as system prompt
- select MCP servers available in mode
- delete mode

Tooltips are attached to mode actions and key mode fields.

---

## MCP Page

### Top Actions

- `Search MCP library templates`
- `Use Template`
- `Validate MCP`
- `Add MCP Server`

Tooltips explain what each action does and when to use it.

### Preflight Validation (`Validate MCP`)

Preflight checks:

- server config validity (missing command/url, malformed JSON-like args, env key format)
- provider config parse/readability checks

Output includes:

- readiness status (`ready` / `needs attention`)
- issue level (`error`, `warning`, `info`)
- message, optional hint, and source path when available

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

---

## Skills and Rules Pages

### Skills

Capabilities:

- create/edit/delete skills
- install from curated library
- install from URL/repo path with inferred skill ID
- studio folder-audit view via file-tree

Tooltips now cover:

- creating new skills
- catalog install action
- URL/repo install action
- studio/list toggle
- delete action

### Rules

Capabilities:

- create/edit/delete global rule documents

Rules are global instructions applied across sessions; they share the same editor ergonomics as skills.

---

## Hooks Page

Hooks are provider-agnostic in Ship config and exported where supported.

Provider status:

- `Claude`: native hooks supported
- `Gemini`: native hooks supported
- `Codex`: no native hook surface yet (stored in Ship config, not exported natively)

Each hook row now has field-level tooltips for:

- `Hook ID`
- `Event`
- `Command`
- `Description`
- `Timeout`
- `Matcher`
- delete hook action

Defaults when adding a hook:

- `trigger`: first provider-supported event (fallback `PreToolUse`)
- `command`: `$SHIP_HOOKS_BIN`
- `matcher`: empty
- `timeout_ms`: empty

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
- `Filesystem`: allow/deny path globs
- `Limits`: max cost and max turns

Autocomplete suggestions are inferred from:

- configured MCP server IDs
- MCP catalog IDs
- seeded safe pattern/path templates

---

## Defaults Shipped Out of the Box

From current config normalization/runtime behavior:

- providers default: `['claude']`
- AI defaults: `provider='claude'`, `model=null`, `cli_path=null`
- modes default: none
- MCP servers default: none
- hooks default: none
- agent layer defaults: empty `skills/prompts/context/rules`

Permissions are managed in `agents/permissions.toml` and can be seeded via presets in UI.

---

## Save and Export Semantics

- `Save Global Agent Config`: persists global defaults.
- `Save Project Agent Config`: persists project overrides.
- provider `Export`: writes provider-native files from Ship's unified config.

Save buttons now include tooltips clarifying persistence scope.
