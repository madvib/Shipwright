# Ship — Platform Specification

> Single reference for types, config formats, file locations, ownership, and input/output contracts.
> Read ARCHITECTURE.md first for principles and layer separation. This doc is the "what and where."
> **Updated**: 2026-03-15

---

## Config Files

### Global (`~/.ship/`)

| File | Type | Purpose |
|---|---|---|
| `config.toml` | `ShipConfig` | Identity (name, email) + defaults (provider, mode) |
| `path-context.toml` | `PathContext` | Maps absolute paths → active preset id |
| `modes/<id>.toml` | `Mode` | User's personal preset library |
| `skills/<id>.md` | Skill | User's personal skill library |
| `mcp/registry.toml` | MCP registry | Named MCP server definitions |
| `cache/` | — | Catalog cache, registry cache |

### Project (`.ship/`, committed to git)

| File | Type | Purpose |
|---|---|---|
| `ship.toml` | `ShipProject` | Project identity, providers, active preset |
| `modes/<id>.toml` | `Mode` | Project-scoped presets |
| `agents/rules/*.md` | Rule | Always-on rules injected into compiled output |
| `agents/skills/` | Skill | Project-scoped skills |
| `agents/mcp.toml` | MCP config | Project MCP server definitions |
| `agents/permissions.toml` | Permissions | Base permissions (presets override on top) |
| `agents/hooks.toml` | Hooks | Event hook definitions |

### Compiled artifacts (`.ship/.gitignore`d — never commit)

```
CLAUDE.md          ← claude context file
AGENTS.md          ← codex / openai context file
GEMINI.md          ← gemini context file
.mcp.json          ← claude MCP server config
.cursor/           ← cursor rules, mcp, hooks
.codex/            ← codex config patch
.gemini/           ← gemini settings + policies
.agents/skills/    ← compiled skill files
```

---

## Config Schemas

### `~/.ship/config.toml` — ShipConfig

```toml
[identity]
name = "Alice"
email = "alice@example.com"   # optional

[defaults]
provider = "claude"           # optional
mode = "rust-expert"          # optional
```

### `~/.ship/path-context.toml` — PathContext

```toml
[paths."/home/alice/projects/ship"]
mode = "ship-dev"
provider = "claude"           # optional override
```

### `.ship/ship.toml` — ShipProject

```toml
[project]
name = "ship"                 # optional display name
providers = ["claude"]        # default: ["claude"]
active_mode = "default"       # optional; path-context takes precedence
```

---

## Preset (Mode) Format

> **Naming note**: the binary, CLI, and file format currently use `mode`. This will rename to `preset` per ARCHITECTURE.md. Until then, `mode` and `preset` are synonymous in this codebase.

File: `.ship/modes/<id>.toml` or `~/.ship/modes/<id>.toml`

```toml
[mode]
name = "Rust Expert"
id = "rust-expert"
version = "0.1.0"
description = "Deep Rust focus with compiler context"
providers = ["claude"]        # overrides project ship.toml providers

[skills]
refs = ["rust-idioms", "cargo-workflow"]   # empty = all installed skills

[mcp]
servers = ["github", "search"]             # empty = all configured servers

[permissions]
preset = "ship-guarded"       # ship-standard | ship-guarded | read-only | full-access
tools_deny = ["mcp__*__delete*"]
tools_ask = []
default_mode = "plan"         # default | acceptEdits | plan | bypassPermissions

[rules]
inline = """
Prefer safe Rust. No unwrap() in library code. Run clippy before committing.
"""
```

**Permission presets:**
- `ship-standard` — base permissions as defined in `agents/permissions.toml`
- `ship-guarded` — base + deny `mcp__*__delete*` and `mcp__*__drop*`
- `read-only` — allow only `Read`, `Glob`, `LS`
- `full-access` — allow `*`

---

## Skill Format

File: `.ship/agents/skills/<id>.md` or `~/.ship/skills/<id>.md`

```markdown
---
name: Rust Idioms
id: rust-idioms
version: 0.1.0
description: Idiomatic Rust patterns and error handling
triggers: ["rust", "cargo", ".rs"]
---

# Rust Idioms

Use `?` for error propagation. Prefer `thiserror` over `anyhow` for library crates.
...
```

Skills are filtered via `[skills] refs = [...]` in the preset. Empty refs = all installed skills.

---

## Compiler — Input / Output Contract

### Input: `ProjectLibrary` (JSON)

```json
{
  "modes": [{ "id": "...", "name": "...", "skills": [...], "mcp_servers": [...], ... }],
  "mcp_servers": [{ "id": "...", "name": "...", "type": "stdio|http", ... }],
  "skills": [{ "id": "...", "name": "...", "content": "...", "source": "inline|file" }],
  "rules": [{ "name": "...", "content": "..." }],
  "permissions": { "tools": { "allow": [], "deny": [], "ask": [] }, "default_mode": "plan" }
}
```

The Studio web UI and CLI's `compile` command both produce this as the intermediate. It is the canonical compiler input.

### WASM API (packages/compiler / @ship/compiler)

```typescript
compileLibrary(library_json: string, provider: string, active_mode?: string): string
compileLibraryAll(library_json: string, active_mode?: string): string
listProviders(): string
```

### Output: `CompileResult` (per provider)

```json
{
  "provider": "claude",
  "context_content": "# CLAUDE.md content...",
  "mcp_servers": { "github": { "command": "...", "args": [...] } },
  "mcp_config_path": ".mcp.json",
  "skill_files": { ".agents/skills/rust-idioms.md": "# Rust Idioms..." },
  "rule_files": {},
  "claude_settings_patch": { "permissions": {...}, "hooks": [...] },
  "codex_config_patch": null,
  "gemini_settings_patch": null,
  "gemini_policy_patch": null,
  "cursor_hooks_patch": null,
  "cursor_cli_permissions": null
}
```

### Provider output matrix

| Provider | Context file | MCP config | Skills | Rules | Settings patch |
|---|---|---|---|---|---|
| `claude` | `CLAUDE.md` | `.mcp.json` | `.claude/skills/` | inline in CLAUDE.md | `.claude/settings.json` (permissions + hooks) |
| `codex` | `AGENTS.md` | `.codex/config.toml` patch | `.agents/skills/` | inline | — |
| `gemini` | `GEMINI.md` | `.gemini/settings.json` | `.agents/skills/` ¹ | `.gemini/policies/ship.toml` | `.gemini/settings.json` (hooks) |
| `cursor` | — | `.cursor/mcp.json` | `.cursor/skills/` ² | `.cursor/rules/<name>.mdc` | `.cursor/hooks.json` + `.cursor/cli.json` |

**Notes:**
- ¹ Gemini CLI reads `.agents/skills/` per the agentskills.io multi-provider spec. Many providers cascade through `.agents/` and `.claude/` as fallbacks — skills written to those paths work across the ecosystem without Ship installed.
- ² Cursor primary is `.cursor/skills/`; Cursor also cascades `.agents/` and `.claude/` as fallbacks.

---

## CLI Commands (ship-studio-cli → binary: `ship-studio`, rename target: `ship`)

```
ship-studio init [--global] [--provider <id>]
ship-studio whoami
ship-studio login / logout

ship-studio use <mode-id> [--path <dir>] [--compile]
ship-studio status [--path <dir>]
ship-studio modes [--local | --project | --cloud]

ship-studio mode create <name> [--global]
ship-studio mode edit <name>
ship-studio mode delete <name>
ship-studio mode clone <src> <dst>
ship-studio mode publish <name>           # stub — requires account

ship-studio compile [--provider <id>] [--dry-run] [--watch] [--path <dir>]
ship-studio export <provider>

ship-studio skill list
ship-studio skill create <id> [--name] [--description]
ship-studio skill remove <id> [--global]
ship-studio skill add <source>            # stub — registry
ship-studio skill publish                 # stub — requires account

ship-studio mcp list
ship-studio mcp add <id> --url <url>
ship-studio mcp add-stdio <id> --command <cmd> [--args ...]
ship-studio mcp remove <id>

ship-studio sync                          # stub — requires account
ship-studio server                        # stub — local HTTP server
```

**Upcoming (Day 4 roadmap):**
```
ship workspace create|activate|list
ship session start [--goal "..."] | log [--note "..."] | end [--summary "..."]
ship use <preset-id>           # install from registry
```

---

## Ownership Map

| What | Owner | Scope |
|---|---|---|
| Compiler types (ProjectLibrary, ResolvedConfig, CompileResult) | `crates/core/compiler` | shared |
| WASM bindings | `crates/core/compiler` (feature = wasm) | web only |
| CLI commands | `apps/ship-studio-cli` | CLI only |
| CLI config types (ShipConfig, PathContext, Mode) | `apps/ship-studio-cli/src/` | CLI only |
| Studio web UI | `apps/web` | web only |
| Studio primitives (shared UI) | `packages/primitives` | shared |
| WASM package | `packages/compiler` | JS/TS consumers |
| Platform runtime types | `crates/core/runtime` | platform (future) |
| Workflow types (Feature, Release, Issue, etc.) | shipflow package (not yet built) | workflow layer |

**Platform layer owns:** Workspace, Preset, Session, Document, Event, Skill, MCP, Permission, Hook
**Workflow layer owns:** Feature, Release, Issue, Spec, Vision, Target, ADR (as document types)

---

## What Ship Publishes vs Consumes

### Publishes (outputs from Ship)
- Compiled provider configs: `CLAUDE.md`, `AGENTS.md`, `.mcp.json`, `.cursor/rules/`, etc.
- Preset packages to registry: `~/<id>.toml` → `@org/preset-name`
- Skill packages to registry: `.md` → `@org/skill-name`
- Compiled WASM: `packages/compiler/@ship/compiler`

### Consumes (inputs to Ship)
- `ProjectLibrary` JSON — the compiler's only input (from Studio UI or CLI loader)
- `.ship/modes/<id>.toml` — preset definitions (CLI assembles these into ProjectLibrary)
- `.ship/agents/rules/*.md` — rules (inlined into ProjectLibrary)
- `.ship/agents/skills/*.md` — skill content
- Registry packages: `ship use @org/preset-name` installs into `~/.ship/modes/`

---

## Lookup Order

When resolving a preset or skill by ID, the CLI searches in this order:

1. `.ship/modes/<id>.toml` — project scope
2. `~/.ship/modes/<id>.toml` — global scope
3. Registry cache `~/.ship/cache/` — cloud scope (requires account)

Same pattern for skills: `.ship/agents/skills/` → `~/.ship/skills/` → registry cache.

---

## Default Behavior at `ship init`

`ship init` creates:
```
.ship/
  ship.toml             # [project] providers = ["claude"]
  .gitignore            # CLAUDE.md, .mcp.json, .cursor/, .codex/, .gemini/
  README.md             # onboarding note
  modes/                # empty — user creates presets here
  agents/
    rules/              # empty — add .md rule files
    skills/             # empty — add .md skill files
```

`ship init --global` creates:
```
~/.ship/
  config.toml           # [identity] name = ""
  README.md
  modes/
  skills/
  mcp/
  cache/
```

No preset is activated by default. Run `ship use <id>` to activate one. Run `ship compile` to emit provider files.
