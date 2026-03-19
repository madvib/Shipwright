# Ship — Reference Tables

> Companion to ARCHITECTURE.md. Contains schemas, matrices, and command lists.
> Read ARCHITECTURE.md first for principles and layer separation.
> **Updated**: 2026-03-18

---

## Provider Output Matrix

| Provider | Context file | MCP config | Skills dir | Settings |
|---|---|---|---|---|
| `claude` | `CLAUDE.md` | `.mcp.json` | `.claude/skills/<id>/SKILL.md` | `.claude/settings.json` patch |
| `gemini` | `GEMINI.md` | `.gemini/settings.json` (nested) | `.agents/skills/<id>/SKILL.md` | `.gemini/settings.json` + `.gemini/policies/ship.toml` |
| `codex` | `AGENTS.md` | `.codex/config.toml` | `.agents/skills/<id>/SKILL.md` | — |
| `cursor` | — (per-file `.mdc`) | `.cursor/mcp.json` | `.cursor/skills/<id>/SKILL.md` | `.cursor/cli.json` + `.cursor/hooks.json` |
| `windsurf` | `.windsurfrules` | — | `.agents/skills/<id>/SKILL.md` | — |

Cursor uses `.cursor/rules/*.mdc` (one file per rule) instead of a single context file.
Windsurf uses `.windsurfrules` (single markdown file, same content as other context files).

### Provider Feature Flags

| Provider | `supports_mcp` | `supports_hooks` | `supports_tool_permissions` | `supports_memory` |
|---|---|---|---|---|
| `claude` | ✓ | ✓ | ✓ | ✓ (`CLAUDE.md`) |
| `gemini` | ✓ | ✓ | ✓ | ✓ (`GEMINI.md`) |
| `codex` | ✓ | — | — | ✓ (`AGENTS.md`) |
| `cursor` | ✓ | ✓ | ✓ | — (per-file `.mdc` rules) |
| `windsurf` | — | — | — | ✓ (`.windsurfrules`) |

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

---

## platform.db Schema

Location: `~/.ship/state/<project-slug>/platform.db` (SQLite, WAL mode)

| Table | Key columns | Purpose |
|---|---|---|
| `schema_migrations` | `version TEXT PK`, `applied_at TEXT` | Migration tracking |
| `kv_state` | `(namespace, key) PK`, `value_json`, `updated_at` | Generic key-value store |
| `event_log` | `seq INTEGER PK AUTOINCREMENT`, `timestamp`, `actor`, `entity`, `action`, `subject`, `details?` | Append-only event log |
| `workspace` | `id TEXT PK`, `branch TEXT UNIQUE`, `worktree_path?`, `workspace_type`, `status`, `active_profile?`, `providers_json`, `skills_json`, `mcp_servers_json`, `plugins_json`, `compiled_at?`, `compile_error?`, `created_at`, `updated_at` | Workspace records |
| `workspace_session` | `id TEXT PK`, `workspace_id FK`, `branch`, `status`, `profile_id?`, `primary_provider?`, `goal?`, `summary?`, `started_at`, `ended_at?`, `created_at`, `updated_at` | Session records |
| `branch_config` | `branch TEXT PK`, `profile_id`, `workspace_id? FK`, `plugins_json`, `compiled_at`, `updated_at` | Last-compiled profile per branch |
| `job` | `id TEXT PK`, `kind`, `status`, `branch?`, `payload_json`, `created_by?`, `created_at`, `updated_at` | Coordination jobs |
| `job_log` | `id INTEGER PK AUTOINCREMENT`, `job_id? FK`, `branch?`, `message`, `actor?`, `created_at` | Job log entries |
| `note` | `id TEXT PK`, `title`, `content`, `tags_json`, `branch?`, `synced_at?`, `created_at`, `updated_at` | Project notes |
| `adr` | `id TEXT PK`, `title`, `status`, `date`, `context`, `decision`, `tags_json`, `supersedes_id?`, `created_at`, `updated_at` | Architecture decision records |

`workspace.workspace_type` values: `declarative` | `imperative` | `service`
`workspace_session.status` values: `active` | `ended`
`job.status` values: `pending` | `running` | `complete` | `failed`
`adr.status` values: `proposed` | `accepted` | `rejected` | `superseded`

`kv_state` workspace namespace keys: `active_profile`, `compiled_at`, `plugins_installed`

---

## MCP Tools

Server: `ship-mcp` binary (`apps/mcp/`). Core tools are always available; non-core tools require the tool id in the active profile's `active_tools[]`.

| Tool | Purpose | Key params |
|---|---|---|
| `open_project` | Set active project for subsequent calls | `path: string` |
| `create_note` | Create a note in platform.db | `title`, `content?`, `branch?` |
| `create_adr` | Create an ADR record | `title`, `decision` |
| `activate_workspace` | Activate workspace by branch | `branch`, `mode_id?` |
| `create_workspace` | Create workspace + git worktree | `name`, `kind`, `branch?`, `base_branch?`, `profile_id?` |
| `create_workspace_tool` | Create/update workspace runtime record | `branch?`, `workspace_type?`, `mode_id?`, `activate?` |
| `complete_workspace` | Write handoff.md + optionally prune worktree | `workspace_id`, `summary`, `prune_worktree?` |
| `list_stale_worktrees` | List worktrees idle beyond threshold | `idle_hours?` (default 24) |
| `set_mode` | Activate or clear active profile | `id?` |
| `sync_workspace` | Sync workspace to current branch context | `branch?` |
| `repair_workspace` | Detect and repair compile/config drift | `branch?`, `dry_run?` |
| `list_workspaces` | List all workspaces, filter by status | `status?` |
| `start_session` | Start a workspace session | `branch?`, `goal?`, `mode_id?`, `provider_id?` |
| `end_session` | End active session with summary | `branch?`, `summary?` |
| `log_progress` | Record progress note in active session | `note`, `branch?` |
| `list_skills` | List available skills | `query?` |
| `create_job` | Create coordination job | `kind`, `description`, `branch?` |
| `update_job` | Update job status | `id`, `status` |
| `list_jobs` | List jobs, filter by branch/status | `branch?`, `status?` |
| `append_job_log` | Append log entry to a job | `job_id`, `message`, `level?` |

**MCP Resources:** `ship://project_info`, `ship://adrs`, `ship://adrs/{id}` — read-only context snapshots.

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
ship sync                          # sync personal skills/profiles to account
ship cache clean                   # evict ~/.ship/cache/
```

`ship use` is the primary command. It installs any missing deps, activates the profile, and emits all provider files.

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
  "permissions": { "tools": { "allow": ["*"], "ask": [], "deny": [] }, "filesystem": { "allow": [], "deny": [] }, "network": { "policy": "none" }, "default_mode": null },
  "hooks": [{ "id": "...", "trigger": "PreToolUse", "matcher": null, "command": "..." }],
  "plugins": { "install": [], "scope": "project" }
}
```

### WASM API (`packages/compiler` / `@ship/compiler`)

```typescript
compileLibrary(library_json: string, provider: string, active_mode?: string): string
compileLibraryAll(library_json: string, active_mode?: string): string
listProviders(): string[]   // ["claude", "gemini", "codex", "cursor", "windsurf"]
```

### `CompileResult` fields

| Field | Type | Notes |
|---|---|---|
| `provider` | string | Provider id |
| `context_content` | string? | CLAUDE.md / GEMINI.md / AGENTS.md content |
| `mcp_servers` | JSON | MCP server entries |
| `mcp_config_path` | string? | Relative path where MCP config is written |
| `skill_files` | map | `path → content` for each skill file |
| `rule_files` | map | `path → content` for per-file rules (Cursor .mdc) |
| `claude_settings_patch` | JSON? | `permissions`, `hooks`, agent limits (claude only) |
| `codex_config_patch` | string? | TOML `[mcp_servers.<id>]` entries (codex only) |
| `gemini_settings_patch` | JSON? | `hooks` for `.gemini/settings.json` (gemini only) |
| `cursor_hooks_patch` | JSON? | Full `.cursor/hooks.json` content (cursor only) |
| `plugins_manifest` | object | `{ install: [{id, provider}], scope }` |

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
refs = ["ship-coordination"]  # optional; skill ids to activate

[mcp]
servers = ["ship"]            # optional; MCP server ids to activate

[plugins]
install = [
  "superpowers@claude-plugins-official",
]
scope = "project"             # "project" (default) or "user"

[permissions]
preset = "ship-guarded"       # ship-standard | ship-guarded | read-only | full-access
tools_deny = []
tools_ask = ["Bash(rm -rf*)"]
default_mode = "default"      # "default" | "acceptEdits" | "plan" | "bypassPermissions"

[rules]
inline = """
Freeform rule text injected directly into the context output.
"""
```

**Permission resolution order** (highest wins):
1. `[permissions] default_mode` in profile TOML
2. `default_mode` in the named preset section in `agents/permissions.toml`
3. Base `Permissions::default()`

---

## .ship/ File Layout

| Path | Format | Written by | Read by | Purpose |
|---|---|---|---|---|
| `ship.toml` | TOML | `ship init` / user | CLI, compiler, MCP | Project manifest: module, dependencies, exports |
| `agents/presets/<id>.toml` | TOML | user / `ship profile create` | CLI, compiler | Project-scoped profile definitions |
| `agents/skills/<id>/SKILL.md` | Markdown + frontmatter | user / `ship skill create` | CLI, compiler | Project-scoped skills |
| `agents/rules/*.md` | Markdown | user | compiler | Always-on rules |
| `agents/mcp.toml` | TOML | user | CLI, compiler | Project MCP server definitions |
| `agents/permissions.toml` | TOML | user | compiler | Base permissions applied to all profiles |
| `agents/hooks.toml` | TOML | user | compiler | Event hook definitions |
| `ship.lock` | TOML | `ship install` | CLI | Registry lockfile (committed to git) |

**Global paths (`~/.ship/`):**

| Path | Format | Purpose |
|---|---|---|
| `config.toml` | TOML | Identity (name, email) + defaults |
| `profiles/<id>.toml` | TOML | Installed/authored profiles |
| `skills/<id>/SKILL.md` | Markdown | Installed/authored skills |
| `mcp/registry.toml` | TOML | Named MCP server definitions |
| `cache/objects/<sha256>/` | blobs | Fetched package content (content-addressed) |
| `state/<slug>/platform.db` | SQLite | Per-project workspace/session/event DB |

---

## Ownership Map

| What | Owner |
|---|---|
| Compiler types + WASM | `crates/core/compiler` |
| CLI commands + config types | `apps/ship-studio-cli` |
| Studio web UI | `apps/web` |
| Shared UI primitives | `packages/primitives` |
| WASM package | `packages/compiler` |
| Auth + API endpoints | `apps/web/src/routes/api/` |
| Platform runtime types + DB | `crates/core/runtime` |
| MCP server | `apps/mcp` |
| Workflow types | shipflow package (not yet built) |
