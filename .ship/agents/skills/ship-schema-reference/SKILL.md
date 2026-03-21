---
name: ship-schema-reference
description: Use when the user asks about Ship configuration format, available fields, valid values, or how to structure their .ship/ directory. Covers ship.toml manifest, agent profile TOML, permissions.toml, and ship.lock schemas.
tags: [reference, schema, configuration, documentation]
authors: [ship]
---

# Ship Configuration Schema Reference

## 1. ship.toml — Project Manifest

**Location**: `.ship/ship.toml`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | auto | Stable project identifier. Created on first run. Do not edit. |

### `[module]` — Package identity

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | **yes** | Namespaced package path, e.g. `github.com/owner/repo`. |
| `version` | string | **yes** | Semver string (`1.0.0`). Optional `v` prefix is stripped. |
| `description` | string | no | Human-readable summary. |
| `license` | string | no | SPDX identifier, e.g. `MIT`. |

### `[dependencies]` — Dependency map

Keys are namespaced paths. Values: version string or `{ version = "...", grant = ["Bash"] }`.

### `[exports]` — Published artifacts

| Field | Type | Description |
|-------|------|-------------|
| `skills` | string[] | Skill directory paths relative to `.ship/` (each must contain `SKILL.md`). |
| `agents` | string[] | Agent profile TOML paths relative to `.ship/`. |

---

## 2. Agent Profile TOML

**Location**: `.ship/agents/profiles/<id>.toml`

### `[profile]` — Agent identity (required)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | **yes** | Unique agent identifier. Matches filename stem. |
| `name` | string | **yes** | Display name. |
| `version` | string | no | Semver version. |
| `description` | string | no | What this agent does. |
| `providers` | string[] | no | Target providers: `"claude"`, `"cursor"`, `"codex"`, `"gemini"`. Default: all. |

### `[skills]` — Skill references

`refs` (string[]): Skill IDs to attach. Resolved from local `.ship/agents/skills/` and dependencies.

### `[mcp]` — MCP server references

`servers` (string[]): MCP server IDs to connect.

### `[plugins]` — Provider plugins

`install` (string[]): Plugin identifiers, e.g. `"rust-analyzer-lsp@claude-plugins-official"`.
`scope` (string): `"project"` or `"workspace"`.

### `[permissions]` — Tool access control

| Field | Type | Description |
|-------|------|-------------|
| `preset` | string | Preset name from `permissions.toml` (e.g. `"ship-standard"`). |
| `tools_allow` | string[] | Glob patterns for tools to always allow. |
| `tools_ask` | string[] | Glob patterns for tools requiring confirmation. |
| `tools_deny` | string[] | Glob patterns for tools to block. |
| `default_mode` | string | `"default"`, `"acceptEdits"`, `"plan"`, `"dontAsk"`, `"bypassPermissions"`. |

### `[rules]` — System prompt

`inline` (string): Rules body injected as the agent's system prompt. Use triple-quoted TOML for multiline.

### `[provider_settings]` — Opaque provider config

Free-form key/value table. Passed through without validation.

### Example

```toml
[profile]
id = "rust-compiler"
name = "Rust Compiler"
providers = ["claude"]

[skills]
refs = ["lint-fix", "test-runner"]

[permissions]
preset = "ship-autonomous"
tools_deny = ["Bash(git push --force*)"]

[rules]
inline = """
Your domain is the Ship compiler.
After changes: `cargo test -p compiler` must pass.
"""
```

---

## 3. permissions.toml — Permission Presets

**Location**: `.ship/agents/permissions.toml`

Each top-level key is a preset name with fields: `default_mode` (string), `tools_deny` (string[]), `tools_allow_override` (string[]).

### Base rules (injected by compiler into all presets)

```
always_allow = ["mcp__ship__*", "Bash(ship *)"]
always_deny  = ["Bash(sqlite3 ~/.ship/*)", "Bash(git push*)", "Bash(*publish*)",
                 "Read(.env*)", "Write(.env*)", "Read(credentials*)", "Write(credentials*)"]
always_ask   = ["Write(.ship/*)", "Edit(.ship/*)"]
```

### Built-in presets

| Preset | Mode | Use case |
|--------|------|----------|
| `ship-readonly` | `plan` | Reviewers, auditors. Denies all writes. |
| `ship-standard` | `default` | Interactive sessions, human-paired work. |
| `ship-autonomous` | `dontAsk` | Dispatched specialists in worktrees. |
| `ship-elevated` | `dontAsk` | Deploy/release. Unlocks `git push` and `publish`. |

---

## 4. ship.lock — Dependency Lockfile

**Location**: `.ship/ship.lock`

Top-level `version` (integer, current: `1`). Array of `[[package]]` entries:

| Field | Type | Description |
|-------|------|-------------|
| `path` | string | Package path, e.g. `github.com/owner/repo`. |
| `version` | string | Resolved tag or branch name. |
| `commit` | string | Full 40-char git commit SHA. |
| `hash` | string | Content integrity hash: `sha256:<hex>`. |

---

## .ship/ Directory Layout

```
.ship/
  ship.toml              # Project manifest
  ship.lock              # Dependency lockfile (generated)
  agents/
    permissions.toml     # Permission presets
    profiles/            # Agent profile TOML files
    skills/              # Skill directories (each contains SKILL.md)
```

`ship.lock` and compiled output files are generated artifacts. Do not hand-edit the lockfile.
