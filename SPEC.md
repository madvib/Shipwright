# Ship — Platform Specification

> Single reference for types, config formats, file locations, ownership, and contracts.
> Read ARCHITECTURE.md first for principles and layer separation.
> **Updated**: 2026-03-15

---

## Artifact Taxonomy

Ship manages three versioned artifact types. All follow the same registry model.

| Type | Format | Atomic? | Description |
|---|---|---|---|
| **Skill** | `.md` (frontmatter + markdown) | ✓ | Single-purpose agent instruction |
| **Preset** | `.toml` | — | Named config: references skills + MCP + permissions |
| **Workflow** | `.toml` (future) | — | Orchestration: references presets + execution logic |

Skills are atoms. Presets compose skills. Workflows compose presets.

### Versioning and Provenance

Every installed artifact (registry or local) is tracked in `~/.ship/ship.lock`:

```toml
[skills."rust-idioms@1.2.0"]
source = "registry"
r2_key = "skills/rust-idioms/1.2.0/SKILL.md"
checksum = "sha256:abc123"
installed_at = "2026-03-15T10:00:00Z"

[skills."my-deploy-flow"]
source = "local"
# no version, no key — authored locally, not published

[presets."ship-studio-default@2.1.0"]
source = "registry"
r2_key = "presets/ship-studio-default/2.1.0/preset.toml"
checksum = "sha256:def456"
skills = ["rust-idioms@1.2.0"]
installed_at = "2026-03-15T10:00:00Z"
```

`source = "local"` means authored on this machine, not fetched from registry.
`source = "registry"` means fetched — re-fetchable, content-addressed by checksum.

---

## Storage Model

```
Registry (getship.dev)
  R2  — artifact content blobs (immutable, CDN-served)
        skills/:id/:version/SKILL.md
        presets/:id/:version/preset.toml
        workflows/:id/:version/workflow.toml
  D1  — artifact metadata (queryable)
        skills table: id, name, description, tags, author, version, r2_key, downloads
        presets table: id, name, description, tags, author, version, r2_key, skill_refs
        workflows table: id, name, description, tags, author, version, r2_key, preset_refs
  DO  — user/org state (Rivet actors, self-hostable)
        UserActor: profile, personal skills (authored), installed manifest, usage
        OrgActor: members, shared presets, billing

Local (~/.ship/)
  ship.lock     — installed artifact manifest (source, version, checksum, r2_key)
  skills/       — installed skill content (registry-fetched + locally authored)
  presets/      — installed preset content
  cache/        — download cache (R2 objects, keyed by r2_key, LRU eviction)
  config.toml   — identity + defaults

Project (.ship/, committed to git)
  ship.toml     — project identity + active preset ref
  agents/       — rules, skills, MCP config (project-scoped, always active)
```

**Rules:**
- R2 stores content. D1 stores metadata + R2 keys. Never blob-store content in D1.
- `~/.ship/cache/` is transparent — populated on `ship use`, evictable at any time.
- `~/.ship/skills/` is the installed layer — analogous to global node_modules.
- Local authored skills (`source = "local"`) are never synced automatically.
  Publishing is an explicit `ship publish` action.
- Compiled provider files (CLAUDE.md, .mcp.json, etc.) are generated artifacts —
  gitignored, never committed. `.ship/` is the source of truth.

---

## Config Files

### Global (`~/.ship/`)

| File | Purpose |
|---|---|
| `config.toml` | Identity (name, email) + defaults (provider, preset) |
| `ship.lock` | Installed artifact manifest |
| `presets/<id>.toml` | Installed/authored presets |
| `skills/<id>/SKILL.md` | Installed/authored skills |
| `cache/` | R2 download cache (keyed by r2_key) |
| `mcp/registry.toml` | Named MCP server definitions |
| `path-context.toml` | Maps project paths → active preset id |

### Project (`.ship/`, committed to git)

| File | Purpose |
|---|---|
| `ship.toml` | Project identity, active preset ref |
| `agents/rules/*.md` | Always-on rules compiled into every output |
| `agents/skills/<id>/SKILL.md` | Project-scoped skills |
| `agents/mcp.toml` | Project MCP server definitions |
| `agents/permissions.toml` | Base permissions (presets override) |
| `agents/hooks.toml` | Event hook definitions |
| `agents/presets/<id>.toml` | Project-scoped presets |

### Generated (gitignored — never commit)

```
CLAUDE.md              ← claude context
AGENTS.md              ← codex/openai context
GEMINI.md              ← gemini context
.mcp.json              ← claude MCP config
.cursor/               ← cursor rules, mcp, hooks
.codex/                ← codex config patch
.gemini/               ← gemini settings + policies
.claude/skills/        ← compiled skills for claude
.agents/skills/        ← compiled skills for codex/gemini
```

These are outputs. `ship use` produces them. They belong in `.gitignore`.

---

## Config Schemas

### `~/.ship/config.toml`

```toml
[identity]
name = "Alice"
email = "alice@example.com"

[defaults]
provider = "claude"
preset = "rust-expert"
```

### `.ship/ship.toml`

```toml
version = "1"
id = "hRvMUz4p"           # nanoid, stable
name = "ship"
description = "..."

[defaults]
preset = "default"        # active preset id
providers = ["claude"]
```

---

## Preset Format

File: `.ship/agents/presets/<id>.toml` or `~/.ship/presets/<id>.toml`

```toml
[preset]
id = "rust-expert"
name = "Rust Expert"
version = "0.1.0"
description = "Deep Rust focus with compiler context"
providers = ["claude"]        # overrides project providers if set

[skills]
refs = ["rust-idioms", "cargo-workflow"]   # empty = all installed skills

[mcp]
servers = ["github", "search"]             # empty = all configured servers

[permissions]
preset = "ship-guarded"       # ship-standard | ship-guarded | read-only | full-access
tools_deny = ["mcp__*__delete*"]
tools_ask = []
default_mode = "plan"

[rules]
inline = """
Prefer safe Rust. No unwrap() in library code.
"""
```

**Permission presets:**
- `ship-standard` — base permissions from `agents/permissions.toml`
- `ship-guarded` — base + deny destructive MCP operations
- `read-only` — Read, Glob, LS only
- `full-access` — allow `*`

---

## Skill Format

File: `<skills-dir>/<id>/SKILL.md`

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
```

Skills are filtered by `[skills] refs` in the preset. Empty refs = all installed skills active.

---

## CLI Commands

```
ship init [--global]               # scaffold .ship/ or ~/.ship/
ship login / logout / whoami

ship use [<preset-id>]             # activate preset + emit provider files
                                   # no args = re-emit current preset
ship use --list                    # list available presets (local + registry)
ship status                        # show active preset, providers, last built

ship skill list                    # local + registry
ship skill add <source>            # install from registry or local path
ship skill create <id>             # scaffold new skill
ship skill publish <id>            # publish local skill to registry

ship preset list
ship preset add <id>               # install from registry
ship preset create <id>
ship preset publish <id>

ship import                        # detect existing provider configs, import to .ship/
ship mcp list | add | remove

ship publish                       # publish active library to registry (requires auth)
ship sync                          # sync personal skills/presets to account (requires auth)
ship cache clean                   # evict ~/.ship/cache/
```

`ship use` is the primary command. It installs any missing deps, activates the preset,
and emits all provider files. Called automatically on branch switch (git hook).

---

## Compiler — Input / Output Contract

### Input: `ProjectLibrary` (JSON)

```json
{
  "modes": [{ "id": "...", "name": "...", "skills": [...], "mcp_servers": [...] }],
  "mcp_servers": [{ "id": "...", "name": "...", "type": "stdio|http" }],
  "skills": [{ "id": "...", "name": "...", "content": "...", "source": "inline|file" }],
  "rules": [{ "name": "...", "content": "..." }],
  "permissions": { "tools": { "allow": [], "deny": [], "ask": [] }, "default_mode": "plan" }
}
```

### WASM API (`packages/compiler` / `@ship/compiler`)

```typescript
compileLibrary(library_json: string, provider: string, active_mode?: string): string
compileLibraryAll(library_json: string, active_mode?: string): string
listProviders(): string
```

### Provider Output Matrix

| Provider | Context file | MCP config | Skills dir | Settings |
|---|---|---|---|---|
| `claude` | `CLAUDE.md` | `.mcp.json` | `.claude/skills/` | `.claude/settings.json` |
| `codex` | `AGENTS.md` | `.codex/config.toml` | `.agents/skills/` | — |
| `gemini` | `GEMINI.md` | `.gemini/settings.json` | `.agents/skills/` | `.gemini/settings.json` |
| `cursor` | — | `.cursor/mcp.json` | `.cursor/skills/` | `.cursor/rules/*.mdc` |

---

## GitHub Integration

### Import (unauthenticated, public repos)

`POST /api/github/import { url: "https://github.com/owner/repo" }`

Fetches and extracts from the repo:
- `CLAUDE.md` → rules + skills
- `.mcp.json` → MCP servers
- `.cursor/rules/` → rules
- `AGENTS.md` → rules
- `.gemini/` → rules

Returns a `ProjectLibrary` JSON ready for the Studio compiler.

### PR Flow (requires GitHub App OAuth)

`POST /api/github/pr { repo: "owner/repo", library: ProjectLibrary }`

Creates a PR that adds:
- `.ship/` scaffold (ship.toml + compiled library as agents/)
- `.gitignore` patch (adds all provider output files)

PR description includes: what Ship is, `npm install -g ship` (or brew), `ship use` quickstart.
Provider files are NOT in the PR — they're generated locally after `ship use`.

---

## Ownership Map

| What | Owner |
|---|---|
| Compiler types + WASM | `crates/core/compiler` |
| CLI commands + config types | `apps/ship-studio-cli` |
| Studio web UI | `apps/web` |
| Shared UI primitives | `packages/primitives` |
| WASM package | `packages/compiler` |
| Auth + API endpoints | `apps/web/src/routes/api/` (Cloudflare Workers) |
| D1 schema | `apps/web/src/db/` |
| Platform runtime types | `crates/core/runtime` |
| Workflow types | shipflow package (not yet built) |

**Platform owns:** Workspace, Preset, Session, Skill, MCP, Permission, Hook, Event
**Workflow owns:** Feature, Release, Issue, Spec, Vision (guest types — not in platform code)

---

## Lookup Order

Resolving a preset or skill by id:

1. `.ship/agents/presets/<id>.toml` — project scope
2. `~/.ship/presets/<id>.toml` — global installed
3. `~/.ship/cache/` — cached registry fetch
4. Registry API — network fetch (requires connectivity)

Same order for skills: project → global → cache → network.

---

## `ship init` Scaffolding

```
.ship/
  ship.toml             # project identity, no preset active by default
  .gitignore            # CLAUDE.md, AGENTS.md, .mcp.json, .cursor/, .codex/, .gemini/
  agents/
    rules/              # always-on rules (.md files)
    skills/             # project-specific skills
    presets/            # project-specific presets
    mcp.toml            # MCP server definitions
```

`ship init --global` creates `~/.ship/` with config.toml, empty presets/, skills/, cache/.

Run `ship use <preset-id>` to activate a preset and emit provider files.
