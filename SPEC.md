# Ship — Platform Specification

> Single reference for types, config formats, file locations, ownership, and contracts.
> Read ARCHITECTURE.md first for principles and layer separation.
> **Updated**: 2026-03-16

---

## Artifact Taxonomy

Ship manages three versioned artifact types. All follow the same registry model.

| Type | Format | Atomic? | Description |
|---|---|---|---|
| **Skill** | `.md` (frontmatter + markdown) | ✓ | Single-purpose agent instruction |
| **Profile** | `.toml` | — | Named config: references skills + MCP + permissions |
| **Workflow** | `.toml` (planned) | — | Orchestration: references profiles + execution logic |

Skills are atoms. Profiles compose skills. Workflows compose profiles.

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

[profiles."ship-studio-default@2.1.0"]
source = "registry"
r2_key = "profiles/ship-studio-default/2.1.0/profile.toml"
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
        profiles/:id/:version/profile.toml
        workflows/:id/:version/workflow.toml
  D1  — artifact metadata (queryable)
        skills table: id, name, description, tags, author, version, r2_key, downloads
        profiles table: id, name, description, tags, author, version, r2_key, skill_refs
        workflows table: id, name, description, tags, author, version, r2_key, profile_refs
  DO  — user/org state (Rivet actors, self-hostable)
        UserActor: profile, personal skills (authored), installed manifest, usage
        OrgActor: members, shared profiles, billing

Local (~/.ship/)
  ship.lock     — installed artifact manifest (source, version, checksum, r2_key)
  skills/       — installed skill content (registry-fetched + locally authored)
  profiles/     — installed profile content (legacy: ~/.ship/modes/)
  cache/        — download cache (R2 objects, keyed by r2_key, LRU eviction)
  config.toml   — identity + defaults
  state/<slug>/platform.db  — per-project SQLite DB (see platform.db section)
  mcp/registry.toml         — named MCP server definitions

Project (.ship/, committed to git)
  ship.toml               — project identity + active profile ref
  agents/
    presets/*.toml         — project-scoped profiles (directory named presets/ for now)
    skills/<id>/SKILL.md  — project-scoped skills
    rules/*.md             — always-on rules compiled into every output
    mcp.toml               — project MCP server definitions
    permissions.toml       — base permissions (profiles layer on top)
    hooks.toml             — event hook definitions
  modes/                   — legacy (renamed to presets; still read by CLI)
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

## .ship/ File Layout

Every path under `.ship/`, its owner, format, and who reads/writes it.

| Path | Format | Written by | Read by | Purpose |
|---|---|---|---|---|
| `ship.toml` | TOML | `ship init` / user | CLI, compiler, MCP | Project identity, default profile, providers |
| `agents/presets/<id>.toml` | TOML | user / `ship profile create` | CLI, compiler | Project-scoped profile definitions |
| `agents/skills/<id>/SKILL.md` | Markdown + frontmatter | user / `ship skill create` | CLI, compiler | Project-scoped skills |
| `agents/rules/*.md` | Markdown | user | compiler | Always-on rules, included in every output |
| `agents/mcp.toml` | TOML | user | CLI, compiler | Project MCP server definitions |
| `agents/permissions.toml` | TOML | user | compiler | Base permissions applied to all profiles |
| `agents/hooks.toml` | TOML | user | compiler | Event hook definitions |
| `modes/<id>.toml` | TOML | legacy | CLI (legacy path) | Legacy profile location; still resolved |
| `worktrees/<branch>/` | dir | MCP `create_workspace` | MCP, git | Git worktrees for imperative/declarative workspaces |
| `worktrees/<branch>/workspace.toml` | TOML | MCP `create_workspace` | MCP `complete_workspace` | Workspace name, kind, profile_id |
| `sessions/<workspace_id>/handoff.md` | Markdown | MCP `complete_workspace` | agents | Session handoff document |

**Global paths (`~/.ship/`):**

| Path | Format | Written by | Read by | Purpose |
|---|---|---|---|---|
| `config.toml` | TOML | `ship init --global` | CLI | Identity (name, email) + defaults |
| `ship.lock` | TOML | `ship use` | CLI | Installed artifact manifest |
| `profiles/<id>.toml` | TOML | `ship use` / registry | CLI, compiler | Installed/authored profiles |
| `skills/<id>/SKILL.md` | Markdown | `ship use` / user | CLI, compiler | Installed/authored skills |
| `modes/<id>.toml` | TOML | legacy | CLI (legacy) | Legacy global profile location |
| `mcp/registry.toml` | TOML | user | CLI, compiler | Named MCP server definitions |
| `cache/` | blobs | `ship use` | CLI | R2 download cache |
| `state/<slug>/platform.db` | SQLite | runtime | runtime, MCP | Per-project workspace/session/event DB |

**platform.db location:** `~/.ship/state/<project-slug>/platform.db`
where `<project-slug>` is derived from the project's `.ship/` directory path.
The DB is stored globally (outside the repo) and never committed to git.

---

## Config Schemas

### `~/.ship/config.toml`

```toml
[identity]
name = "Alice"
email = "alice@example.com"

[defaults]
provider = "claude"
profile = "rust-expert"
```

### `.ship/ship.toml`

```toml
version = "1"
id = "hRvMUz4p"           # nanoid, stable — cross-machine project identity
name = "ship"
description = "..."

[defaults]
profile = "default"       # fallback profile when no branch-specific profile is set
providers = ["claude"]
```

Full `ProjectConfig` fields (from `crates/core/compiler/src/types/config.rs`):

| Field | Type | Default | Notes |
|---|---|---|---|
| `version` | string | `"1"` | Schema version |
| `id` | string | `""` | nanoid, set by `ship init` |
| `name` | string? | — | Human name |
| `description` | string? | — | — |
| `providers` | string[] | `[]` | e.g. `["claude", "gemini"]` |
| `ai.provider` | string? | `"claude"` | Default AI provider |
| `ai.model` | string? | — | Model override |
| `ai.cli_path` | string? | — | CLI binary path override |
| `modes` | ModeConfig[] | `[]` | Inline mode definitions |
| `active_mode` | string? | — | Currently active mode id |
| `mcp_servers` | McpServerConfig[] | `[]` | Inline MCP server definitions |
| `hooks` | HookConfig[] | `[]` | Inline hook definitions |
| `git.ignore` | string[] | `[]` | Extra gitignore patterns |
| `git.commit` | string[] | `["agents","ship.toml",...]` | Paths committed by default |
| `statuses` | StatusConfig[] | backlog/in-progress/blocked/done | Workflow status definitions |

---

## Profile TOML Schema

File: `.ship/agents/presets/<id>.toml` or `~/.ship/profiles/<id>.toml`

```toml
[profile]
id = "rust-runtime"           # required; kebab-case identifier
name = "Rust Runtime"         # required; human display name
version = "0.1.0"             # optional; semver string
description = "..."           # optional
providers = ["claude"]        # optional; overrides project providers if set

[skills]
refs = ["ship-coordination"]  # optional; skill ids to activate. empty = all installed

[mcp]
servers = ["ship"]            # optional; MCP server ids to activate. empty = all configured

[plugins]
# Claude Code plugins managed by this preset.
# ship use installs on activation, uninstalls on deactivation.
# Format: "<id>@<marketplace>" — same as `claude plugin install`
install = [
  "superpowers@claude-plugins-official",
]
scope = "project"             # "project" (default) or "user"

[permissions]
preset = "ship-guarded"       # ship-standard | ship-guarded | read-only | full-access
tools_deny = []               # additional deny patterns (glob)
tools_ask = ["Bash(rm -rf*)"] # patterns that require confirmation
default_mode = "default"      # "default" | "acceptEdits" | "plan" | "bypassPermissions"

[rules]
inline = """
Freeform rule text injected directly into the context output.
"""
# files = ["path/to/rule.md"]   # (planned) — file refs not yet implemented in parser
```

### Profile section fields

| Section | Field | Type | Default | Notes |
|---|---|---|---|---|
| `[profile]` | `id` | string | — | required; kebab-case; unique in scope |
| `[profile]` | `name` | string | — | required; human display name |
| `[profile]` | `version` | string | — | semver string |
| `[profile]` | `description` | string | — | — |
| `[profile]` | `providers` | string[] | — | overrides project `providers` when set |
| `[skills]` | `refs` | string[] | `[]` | skill ids; empty = all installed skills |
| `[mcp]` | `servers` | string[] | `[]` | server ids; empty = all configured |
| `[plugins]` | `install` | string[] | `[]` | plugin ids in `<id>@<marketplace>` format |
| `[plugins]` | `scope` | string | `"project"` | `"project"` or `"user"` |
| `[permissions]` | `preset` | string | — | named preset from `agents/permissions.toml`; built-in: `ship-standard` \| `ship-guarded` \| `read-only` \| `full-access` |
| `[permissions]` | `tools_deny` | string[] | `[]` | additional deny glob patterns (layered on top of preset) |
| `[permissions]` | `tools_ask` | string[] | `[]` | confirmation-required patterns (layered on top of preset) |
| `[permissions]` | `default_mode` | string | from preset | overrides preset `default_mode`; values: `default` \| `acceptEdits` \| `plan` \| `bypassPermissions` |
| `[rules]` | `inline` | string | — | freeform text injected into context output |
| `[provider_settings.claude]` | any | object | — | merged verbatim into `.claude/settings.json` |

**Permission resolution order** (highest wins):
1. `[permissions] default_mode` in profile TOML
2. `default_mode` in the named preset section in `agents/permissions.toml`
3. Base `Permissions::default()`

**Named preset sections** in `agents/permissions.toml` (resolved when profile sets `preset = "<name>"`):

```toml
[ship-standard]
default_mode = "acceptEdits"
tools_ask = ["Bash(rm -rf*)", "Bash(*--force*)", ...]
tools_deny = ["Bash(git push --force*)", ...]

[ship-guarded]
default_mode = "default"
tools_ask = ["Bash(rm -rf*)", "Bash(*deploy*)", ...]
tools_deny = ["Bash(git push --force*)"]

[ship-open]
default_mode = "bypassPermissions"

[ship-plan]
default_mode = "plan"
```

**Built-in fallback tiers** (used when `agents/permissions.toml` section is absent):
- `ship-standard` = base tools from `Permissions::default()`
- `ship-guarded` = base + deny `mcp__*__delete*` and `mcp__*__drop*`
- `read-only` = allow `Read`, `Glob`, `LS` only
- `full-access` = allow `*`

**Global Claude approval:** when ship is in a profile's MCP servers list and the `claude` provider is compiled, `ship use` writes `mcp__ship__*` to `~/.claude/settings.json` permissions allow. This is a one-time global allow — avoids per-session approval prompts.

Skill resolution: `.ship/agents/skills/` → `~/.ship/skills/` → cache → registry.
Server resolution: `agents/mcp.toml` (project) → `~/.ship/mcp/registry.toml` (global).

### MCP server config fields (`agents/mcp.toml` or inline in `ship.toml`)

| Field | Type | Default | Notes |
|---|---|---|---|
| `id` / `name` | string | — | identifier + human name |
| `command` | string | — | binary to execute (stdio) |
| `args` / `env` | string[] / map | `[]` / `{}` | arguments + environment |
| `scope` | string | `"global"` | `"global"` or `"project"` |
| `server_type` | enum | `stdio` | `stdio` \| `sse` \| `http` |
| `url` | string? | — | URL for SSE/HTTP transport |
| `disabled` | bool | `false` | exclude from compile output |
| `timeout_secs` | u32? | — | connection timeout |

Hook `trigger` values: `PreToolUse` \| `PostToolUse` \| `Notification` \| `Stop` \| `SubagentStop` \| `PreCompact`.

### Plugin lifecycle (`ship use` manages automatically)

1. Read `[plugins] install` from incoming profile
2. Read previously active profile's `[plugins]` from `ship.lock`
3. Install plugins in incoming but not current: `claude plugin install <id> --scope <scope>`
4. Uninstall plugins in current but not incoming: `claude plugin uninstall <id>`
5. Record installed plugin manifest in `ship.lock` under `[plugins]`

---

## Skill Format (`SKILL.md`)

File: `<skills-dir>/<id>/SKILL.md`

```markdown
---
name: Rust Idioms
id: rust-idioms
version: 0.1.0
description: Idiomatic Rust patterns and error handling
author: ship
---

# Rust Idioms

Use `?` for error propagation. Prefer `thiserror` over `anyhow` for library crates.
```

### Frontmatter fields (YAML)

| Field | Type | Required | Notes |
|---|---|---|---|
| `name` | string | yes | Human display name |
| `id` | string | no | Kebab-case; inferred from directory name if omitted |
| `version` | string | no | semver string |
| `description` | string | no | Short summary |
| `author` | string | no | Author identifier |

**Body:** Freeform markdown. The compiler writes the full file content into the provider's skills directory. No special body conventions — write instructions as plain markdown.

**Skill id constraints** (from `is_valid_skill_name`): lowercase ASCII, digits, and `-` only; 1–64 chars; no leading/trailing `-`; no `--`.

**Skill paths (resolution order):**
1. `.ship/agents/skills/<id>/SKILL.md` — project scope
2. `~/.ship/skills/<id>/SKILL.md` — global installed
3. `~/.ship/cache/` — cached registry fetch
4. Registry API — network fetch

---

## Compiler — Input / Output Contract

### Input: `ProjectLibrary` (JSON)

```json
{
  "modes": [{ "id": "...", "name": "...", "active_tools": [], "skills": [], "mcp_servers": [], "rules": [], "hooks": [], "permissions": {} }],
  "active_mode": null,
  "mcp_servers": [{ "id": "...", "name": "...", "command": "...", "args": [], "env": {}, "scope": "global", "server_type": "stdio" }],
  "skills": [{ "id": "...", "name": "...", "description": null, "version": null, "content": "...", "source": "custom" }],
  "rules": [{ "name": "...", "content": "..." }],
  "permissions": { "tools": { "allow": ["*"], "ask": [], "deny": [] }, "filesystem": { "allow": [], "deny": [] }, "commands": { "allow": [], "deny": [] }, "network": { "policy": "none", "allow_hosts": [] }, "agent": { "require_confirmation": [] }, "default_mode": null },
  "hooks": [{ "id": "...", "trigger": "PreToolUse", "matcher": null, "command": "..." }],
  "plugins": { "install": [], "scope": "project" }
}
```

### WASM API (`packages/compiler` / `@ship/compiler`)

```typescript
// Compile for a single provider. Returns JSON string of CompileResult.
compileLibrary(library_json: string, provider: string, active_mode?: string): string

// Compile for all providers in the resolved config. Returns JSON object keyed by provider id.
compileLibraryAll(library_json: string, active_mode?: string): string

// List supported provider ids. Returns string[].
listProviders(): string[]   // ["claude", "gemini", "codex", "cursor", "windsurf"]
```

### `CompileResult` shape (JSON returned by WASM)

| Field | Type | Notes |
|---|---|---|
| `provider` | string | Provider id |
| `context_content` | string? | CLAUDE.md / GEMINI.md / AGENTS.md content |
| `mcp_servers` | JSON | MCP server entries object |
| `mcp_config_path` | string? | Relative path where MCP config is written |
| `skill_files` | map | `path → content` for each skill file |
| `rule_files` | map | `path → content` for per-file rules (Cursor .mdc) |
| `claude_settings_patch` | JSON? | `permissions`, `hooks`, agent limits (claude only) |
| `codex_config_patch` | string? | TOML `[mcp_servers.<id>]` entries (codex only) |
| `gemini_settings_patch` | JSON? | `hooks` section for `.gemini/settings.json` (gemini only) |
| `gemini_policy_patch` | string? | TOML policy file for `.gemini/policies/ship.toml` (gemini only) |
| `cursor_hooks_patch` | JSON? | Full `.cursor/hooks.json` content (cursor only) |
| `cursor_cli_permissions` | JSON? | `.cursor/cli.json` permissions (cursor only) |
| `plugins_manifest` | object | `{ install: [{id, provider}], scope }` |

### Provider Output Matrix

| Provider | Context file | MCP config | Skills dir | Settings |
|---|---|---|---|---|
| `claude` | `CLAUDE.md` | `.mcp.json` | `.claude/skills/<id>/SKILL.md` | `.claude/settings.json` patch |
| `gemini` | `GEMINI.md` | `.gemini/settings.json` (nested) | `.agents/skills/<id>/SKILL.md` | `.gemini/settings.json` + `.gemini/policies/ship.toml` |
| `codex` | `AGENTS.md` | `.codex/config.toml` | `.agents/skills/<id>/SKILL.md` | — |
| `cursor` | — (per-file `.mdc`) | `.cursor/mcp.json` | `.cursor/skills/<id>/SKILL.md` | `.cursor/cli.json` + `.cursor/hooks.json` |
| `windsurf` | `.windsurfrules` | — | `.agents/skills/<id>/SKILL.md` | — |

Cursor uses `.cursor/rules/*.mdc` (one file per rule) instead of a single context file.
Windsurf uses `.windsurfrules` (single markdown file, same content as other context files).

### Provider Feature Matrix

Defines what the compiler emits per provider. Governed by `ProviderFeatureFlags` in the Rust compiler.

| Provider | `supports_mcp` | `supports_hooks` | `supports_tool_permissions` | `supports_memory` |
|---|---|---|---|---|
| `claude` | ✓ | ✓ | ✓ | ✓ (`CLAUDE.md`) |
| `gemini` | ✓ | ✓ | ✓ | ✓ (`GEMINI.md`) |
| `codex` | ✓ | — | — | ✓ (`AGENTS.md`) |
| `cursor` | ✓ | ✓ | ✓ | — (per-file `.mdc` rules) |
| `windsurf` | — | — | — | ✓ (`.windsurfrules`) |

**Flag semantics:**
- `supports_mcp` — provider reads an MCP server config file; compiler emits MCP entries and `mcp_config_path`
- `supports_hooks` — provider supports session hooks (Stop, PreToolUse, etc.)
- `supports_tool_permissions` — provider supports allow/deny tool lists
- `supports_memory` — provider has a persistent rules/context file that survives sessions

---

## Generated Files (gitignored — never commit)

```
CLAUDE.md              ← claude context
AGENTS.md              ← codex/openai/gemini fallback context
GEMINI.md              ← gemini context
.windsurfrules         ← windsurf rules file
.mcp.json              ← claude MCP config
.cursor/               ← cursor rules, mcp, hooks, permissions
.codex/config.toml     ← codex MCP + config patch
.gemini/               ← gemini settings + policies
.claude/skills/        ← compiled skills for claude
.agents/skills/        ← compiled skills for codex/gemini/windsurf
.cursor/skills/        ← compiled skills for cursor
```

These are outputs. `ship use` produces them. They belong in `.gitignore`.

---

## platform.db Schema

Location: `~/.ship/state/<project-slug>/platform.db` (SQLite, WAL mode)

| Table | Key columns | Purpose |
|---|---|---|
| `schema_migrations` | `version TEXT PK`, `applied_at TEXT` | Migration tracking |
| `kv_state` | `(namespace, key) PK`, `value_json`, `updated_at` | Generic key-value store |
| `event_log` | `seq INTEGER PK AUTOINCREMENT`, `timestamp`, `actor`, `entity`, `action`, `subject`, `details?` | Append-only event log; indexed on timestamp and (timestamp, actor, entity, action, subject) |
| `workspace` | `id TEXT PK`, `branch TEXT UNIQUE`, `worktree_path?`, `workspace_type`, `status`, `active_profile?`, `providers_json`, `skills_json`, `mcp_servers_json`, `plugins_json`, `compiled_at?`, `compile_error?`, `created_at`, `updated_at` | Workspace records; indexed on status |
| `workspace_session` | `id TEXT PK`, `workspace_id FK`, `branch`, `status`, `profile_id?`, `primary_provider?`, `goal?`, `summary?`, `started_at`, `ended_at?`, `created_at`, `updated_at` | Session records; indexed on (workspace_id, started_at DESC) and (status, started_at DESC) |
| `branch_config` | `branch TEXT PK`, `profile_id`, `workspace_id? FK`, `plugins_json`, `compiled_at`, `updated_at` | Last-compiled profile per branch |
| `job` | `id TEXT PK`, `kind`, `status`, `branch?`, `payload_json`, `created_by?`, `created_at`, `updated_at` | Coordination jobs; indexed on (status, created_at DESC) and (branch, status) |
| `job_log` | `id INTEGER PK AUTOINCREMENT`, `job_id? FK`, `branch?`, `message`, `actor?`, `created_at` | Job log entries; indexed on (branch, created_at DESC) |
| `note` | `id TEXT PK`, `title`, `content`, `tags_json`, `branch?`, `synced_at?`, `created_at`, `updated_at` | Project notes; indexed on (branch, updated_at DESC) |
| `adr` | `id TEXT PK`, `title`, `status`, `date`, `context`, `decision`, `tags_json`, `supersedes_id?`, `created_at`, `updated_at` | Architecture decision records; indexed on status |

`workspace.workspace_type` values: `declarative` \| `imperative` \| `service` (default: `declarative`)
`workspace_session.status` values: `active` \| `ended`
`job.status` values: `pending` \| `running` \| `complete` \| `failed`
`adr.status` values: `proposed` \| `accepted` \| `rejected` \| `superseded`

---

## Job Payload Schema

Jobs use `payload_json` (free-form JSON object) with these standard fields:

| Field | Type | Notes |
|---|---|---|
| `description` | string | Human-readable job description (set by `create_job`) |
| `requesting_workspace` | string? | Branch/id of the workspace that created the job |
| `title` | string? | Short title (optional, used by some job kinds) |
| `milestone` | string? | Target milestone or branch (optional) |

Additional fields are job-kind-specific. The runtime does not validate `payload_json` beyond JSON well-formedness.

---

## MCP Tools

Server: `ship-mcp` binary (`apps/mcp/`). All tools are available unless gated by active mode.

**Core tools** (always available regardless of active mode):

| Tool | Purpose | Key params | Returns |
|---|---|---|---|
| `open_project` | Set active project for subsequent calls | `path: string` | Confirmation string |
| `create_note` | Create a note in platform.db | `title`, `content?`, `branch?` | Note id |
| `list_notes_tool` | List project notes | — | Notes list |
| `create_adr` | Create an ADR record | `title`, `decision` | ADR id |
| `list_adrs_tool` | List ADRs | — | ADR list |
| `activate_workspace` | Activate workspace by branch, optionally set mode | `branch`, `mode_id?` | Workspace JSON |
| `create_workspace` | Create workspace + git worktree | `name`, `kind`, `branch?`, `base_branch?`, `profile_id?`, `file_scope?` | Workspace id + worktree path |
| `create_workspace_tool` | Create/update workspace runtime record | `branch?`, `workspace_type?`, `mode_id?`, `activate?`, `is_worktree?`, `worktree_path?` | Workspace JSON |
| `complete_workspace` | Write handoff.md + optionally prune worktree | `workspace_id`, `summary`, `prune_worktree?` | Confirmation + handoff path |
| `list_stale_worktrees` | List worktrees idle beyond threshold | `idle_hours?` (default 24) | Worktree list with idle duration |
| `set_mode` | Activate or clear active mode | `id?` | Confirmation string |
| `sync_workspace` | Sync workspace to current branch context | `branch?` | Workspace JSON |
| `repair_workspace` | Detect and repair compile/config drift | `branch?`, `dry_run?` (default true) | Repair report JSON |
| `list_workspaces` | List all workspaces, optionally filter by status | `status?` | Workspace list |
| `start_session` | Start a workspace session | `branch?`, `goal?`, `mode_id?`, `provider_id?` | Session JSON |
| `end_session` | End active session with summary | `branch?`, `summary?`, `updated_feature_ids?` | Session JSON |
| `log_progress` | Record progress note in active session | `note`, `branch?` | Confirmation string |
| `list_skills` | List available skills | `query?` | Skill list |
| `create_job` | Create coordination job | `kind`, `description`, `branch?`, `requesting_workspace?` | Job id |
| `update_job` | Update job status | `id`, `status` | Confirmation string |
| `list_jobs` | List jobs, filter by branch/status | `branch?`, `status?` | Job list |
| `append_job_log` | Append log entry to a job | `job_id`, `message`, `level?` | Confirmation string |

**Tool gating:** Non-core tools require an active mode with `active_tools` listing the tool, or a service workspace. Core tools bypass the gate.

**MCP Resources** (`ship://project_info`, `ship://adrs`, `ship://adrs/{id}`) — read-only context snapshots.

---

## CLI Commands

```
ship init [--global]               # scaffold .ship/ or ~/.ship/
ship login / logout / whoami

ship use [<profile-id>]            # activate profile + emit provider files
                                   # no args = re-emit current profile
ship use --list                    # list available profiles (local + registry)
ship status                        # show active profile, providers, last built

ship skill list                    # local + registry
ship skill add <source>            # install from registry or local path
ship skill create <id>             # scaffold new skill
ship skill publish <id>            # publish local skill to registry

ship profile list
ship profile add <id>              # install from registry
ship profile create <id>
ship profile publish <id>

ship import                        # detect existing provider configs, import to .ship/
ship mcp list | add | remove

ship publish                       # publish active library to registry (requires auth)
ship sync                          # sync personal skills/profiles to account (requires auth)
ship cache clean                   # evict ~/.ship/cache/
```

`ship use` is the primary command. It installs any missing deps, activates the profile,
and emits all provider files. Called automatically on branch switch (git hook).

---

## Workspace Tracking

Ship tracks workspace state in platform.db (`~/.ship/state/<slug>/platform.db`). No project state lives in git-tracked files.

### Project identity

`ship.toml` carries a stable `id` (nanoid). This is the cross-machine project key. When the same repo is cloned on multiple machines, they share the same `id` because `ship.toml` is committed. The DB slug is derived from the `.ship/` directory path.

### Branch-profile tracking

The `branch_config` table records the last profile compiled per branch. The `workspace` table records workspace runtime state keyed by `id` (nanoid), with `branch` as a unique alternate key for git-bound workspaces.

### How it works

1. `ship init` — creates project, writes `ship.toml` with nanoid
2. `ship use <profile>` — compiles profile, upserts `branch_config` for current branch
3. Post-checkout git hook (installed by `ship init`) — on branch switch:
   - Look up `branch_config` for the new branch
   - If found: `ship use <stored_profile_id>` (silent, fast)
   - If not found: inherit from base branch or `[defaults] profile` in `ship.toml`

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
| Platform runtime types + DB | `crates/core/runtime` |
| MCP server | `apps/mcp` |
| CLI path helpers | `apps/ship-studio-cli/src/paths.rs` |
| Workflow types | shipflow package (not yet built) |

**Platform owns:** Workspace, Profile, Session, Skill, MCP, Permission, Hook, Event
**Workflow owns:** Feature, Release, Issue, Spec, Vision (guest types — not in platform code)

---

## Lookup Order

Resolving a profile or skill by id:

1. `.ship/agents/presets/<id>.toml` — project scope
2. `~/.ship/profiles/<id>.toml` — global installed
3. `~/.ship/cache/` — cached registry fetch
4. Registry API — network fetch (requires connectivity)

Same order for skills: `.ship/agents/skills/` → `~/.ship/skills/` → cache → network.

Legacy mode path also checked: `.ship/modes/<id>.toml` (project) → `~/.ship/modes/<id>.toml` (global).

---

## `ship init` Scaffolding

```
.ship/
  ship.toml             # project identity, no profile active by default
  .gitignore            # CLAUDE.md, AGENTS.md, .mcp.json, .cursor/, .codex/, .gemini/
  agents/
    rules/              # always-on rules (.md files)
    skills/             # project-specific skills
    presets/            # project-specific profiles (directory named presets/ for now)
    mcp.toml            # MCP server definitions
    permissions.toml    # base permissions
```

`ship init --global` creates `~/.ship/` with config.toml, empty profiles/, skills/, modes/, mcp/, cache/.

Run `ship use <profile-id>` to activate a profile and emit provider files.
