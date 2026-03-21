---
name: ship-schema-reference
description: Schema reference for Ship configuration files — ship.toml manifest, agent profile TOML, permissions.toml, and ship.lock. Use when users ask about configuration format, available fields, valid values, or how to structure their .ship/ directory.
tags: [reference, schema, configuration, documentation]
authors: [ship]
---

# Ship Configuration Schema Reference

> **Format transition**: `.ship/` configuration is moving from TOML to JSONC. This
> reference documents the current TOML format since that is what the compiler and
> runtime parse today. Field names and structure will stay the same in JSONC.

---

## 1. ship.toml — Project Manifest

**Purpose**: Declares module identity, dependencies, and exported artifacts for a Ship project.

**Location**: `.ship/ship.toml`

### Top-level fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | Auto-generated | Stable project identifier. The runtime creates this on first run if missing. Do not edit. |

### `[module]` — Package identity

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | **yes** | Namespaced package path, e.g. `github.com/owner/repo`. |
| `version` | string | **yes** | Semver string (`1.0.0`). Optional `v` prefix is stripped before validation. |
| `description` | string | no | Human-readable summary. |
| `license` | string | no | SPDX identifier, e.g. `MIT`. |

### `[dependencies]` — Dependency map

Keys are namespaced paths. Values use shorthand or full form:

```toml
# Shorthand — version/branch only
"github.com/owner/repo" = "main"
"github.com/owner/repo" = "^1.0.0"

# Full — version + tool grants
"github.com/owner/repo" = { version = "~2.1.0", grant = ["Bash", "Read"] }
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `version` | string | **yes** | Semver constraint, branch name, or commit SHA. Must not be empty. |
| `grant` | string[] | no | Tool categories the dependency is permitted to use. Default: none. |

### `[exports]` — Published artifacts

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `skills` | string[] | no | Skill directory paths relative to `.ship/` (each must contain a `SKILL.md`). |
| `agents` | string[] | no | Agent profile TOML paths relative to `.ship/`. |

### Minimal example

```toml
id = "bzwmeMdT"

[module]
name = "github.com/owner/myproject"
version = "0.1.0"
```

### Full example

```toml
id = "bzwmeMdT"

[module]
name = "github.com/owner/myproject"
version = "1.0.0"
description = "My Ship project"
license = "MIT"

[dependencies]
"github.com/acme/skills" = "main"
"github.com/acme/tools" = { version = "^2.0.0", grant = ["Bash"] }

[exports]
skills = ["agents/skills/my-skill"]
agents = ["agents/profiles/my-agent.toml"]
```

---

## 2. Agent Profile TOML

**Purpose**: Defines a specialist agent — identity, skills, MCP servers, permissions, and system prompt.

**Location**: `.ship/agents/profiles/<id>.toml`

### `[profile]` — Agent identity (required)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | **yes** | Unique agent identifier. Matches the filename stem by convention. |
| `name` | string | **yes** | Display name. |
| `version` | string | no | Semver version of the profile. |
| `description` | string | no | What this agent does. |
| `providers` | string[] | no | Target providers: `"claude"`, `"cursor"`, `"codex"`, `"gemini"`. Default: all. |

### `[skills]` — Skill references

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `refs` | string[] | no | Skill IDs to attach. Resolved from local `.ship/agents/skills/` and dependencies. |

### `[mcp]` — MCP server references

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `servers` | string[] | no | MCP server IDs to connect. |

### `[plugins]` — Provider plugins

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `install` | string[] | no | Plugin identifiers to install, e.g. `"rust-analyzer-lsp@claude-plugins-official"`. |
| `scope` | string | no | Install scope: `"project"` or `"workspace"`. |

### `[permissions]` — Tool access control

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `preset` | string | no | Permission preset name from `permissions.toml` (e.g. `"ship-standard"`). |
| `tools_allow` | string[] | no | Glob patterns for tools to always allow. |
| `tools_ask` | string[] | no | Glob patterns for tools requiring confirmation. |
| `tools_deny` | string[] | no | Glob patterns for tools to block. |
| `default_mode` | string | no | Permission mode: `"default"`, `"acceptEdits"`, `"plan"`, `"dontAsk"`, `"bypassPermissions"`. |

### `[rules]` — System prompt

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `inline` | string | no | Inline rules body injected as the agent's system prompt. Use triple-quoted TOML strings for multiline. |

### `[provider_settings]` — Opaque provider config

Free-form key/value table. Passed through to provider-specific output without validation.

### Minimal example

```toml
[profile]
id = "basic-agent"
name = "Basic Agent"
```

### Full example

```toml
[profile]
id = "rust-compiler"
name = "Rust Compiler"
version = "0.1.0"
description = "Ship compiler crate — WASM build, compiler logic"
providers = ["claude"]

[skills]
refs = ["lint-fix", "test-runner"]

[mcp]
servers = ["ship"]

[plugins]
install = ["rust-analyzer-lsp@claude-plugins-official"]
scope = "project"

[permissions]
preset = "ship-autonomous"
tools_deny = ["Bash(git push --force*)", "Bash(*cargo publish*)"]

[rules]
inline = """
Your domain is the Ship compiler.
After changes: `cargo test -p compiler` must pass.
"""

[provider_settings]
claude_model = "opus"
temperature = 0.2
```

---

## 3. permissions.toml — Permission Presets

**Purpose**: Defines named permission presets that agent profiles reference via `[permissions] preset`.

**Location**: `.ship/agents/permissions.toml`

### Structure

Each top-level key is a preset name. Values are permission fields:

| Field | Type | Description |
|-------|------|-------------|
| `default_mode` | string | Permission mode: `"default"`, `"plan"`, `"dontAsk"`, `"bypassPermissions"`. |
| `tools_deny` | string[] | Tool glob patterns to block. |
| `tools_allow_override` | string[] | Patterns that override base deny rules (for elevated presets). |

### Base rules (injected by compiler into all presets)

```
always_allow = ["mcp__ship__*", "Bash(ship *)"]
always_deny  = ["Bash(sqlite3 ~/.ship/*)", "Bash(git push*)", "Bash(*publish*)",
                 "Read(.env*)", "Write(.env*)", "Read(.dev.vars*)", "Write(.dev.vars*)",
                 "Read(credentials*)", "Write(credentials*)", "Read(secrets/*)", "Write(secrets/*)"]
always_ask   = ["Write(.ship/*)", "Edit(.ship/*)"]
```

### Built-in presets

| Preset | Mode | Use case |
|--------|------|----------|
| `ship-readonly` | `plan` | Reviewers, auditors. Denies all writes. |
| `ship-standard` | `default` | Interactive sessions, human-paired work. Default for new profiles. |
| `ship-autonomous` | `dontAsk` | Dispatched specialists in worktrees. Zero prompts. |
| `ship-elevated` | `dontAsk` | Deploy/release agents. Unlocks `git push` and `publish`. |

### Example

```toml
[ship-readonly]
default_mode = "plan"
tools_deny = ["Write(*)", "Edit(*)", "Bash(rm*)"]

[ship-autonomous]
default_mode = "dontAsk"

[ship-elevated]
default_mode = "dontAsk"
tools_allow_override = ["Bash(git push*)", "Bash(*publish*)"]
```

---

## 4. ship.lock — Dependency Lockfile

**Purpose**: Pins resolved dependency versions with commit SHAs and content hashes for reproducible installs.

**Location**: `.ship/ship.lock`

### Top-level fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `version` | integer | **yes** | Lockfile schema version. Current: `1`. |

### `[[package]]` — Locked dependency (array of tables)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `path` | string | **yes** | Namespaced package path, e.g. `github.com/owner/repo`. |
| `version` | string | **yes** | Version constraint or branch from `ship.toml`. |
| `commit` | string | **yes** | Full git commit SHA at resolve time. |
| `hash` | string | **yes** | Content integrity hash, prefixed with algorithm: `sha256:<hex>`. |

### Example

```toml
version = 1

[[package]]
path = "github.com/acme/skills"
version = "main"
commit = "6a1636950a1d7fc53602639ce7505a4a5d39c797"
hash = "sha256:83fb025b015f9472ea8504cbdf8c8e042eff86f87cb0f69757bb00fbacd5acb9"
```

---

## .ship/ Directory Layout

```
.ship/
  ship.toml              # Project manifest
  ship.lock              # Dependency lockfile (generated by `ship install`)
  agents/
    permissions.toml     # Permission presets
    profiles/            # Agent profile TOML files
      default.toml
      commander.toml
      ...
    skills/              # Skill directories (each contains SKILL.md)
      my-skill/
        SKILL.md
```

`ship.lock` and compiled output files are generated artifacts. Do not hand-edit the lockfile.
