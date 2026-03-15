# Agent Configuration Reference

> Complete reference for how Ship manages agent configuration — provider registry,
> config resolution, export, import, permissions mapping, skill delivery, hooks,
> teardown, and the managed-state ledger.
>
> Source-read: 2026-03-09. All file paths, field names, and behaviors from source.

---

## Overview

Ship does not call AI APIs. It acts as a **config compiler**: it reads your project's
skills, rules, permissions, MCP servers, and active mode, then writes provider-specific
config files that each agent binary reads on startup.

For Studio page behavior, field-level UX, and tooltip semantics, see:
- `docs/agent-settings-ui.md`

The compilation chain is:

```
ship.toml + agents/ dir
        ↓
  resolve_agent_config()        ← merges project defaults + mode + feature overrides
        ↓
  build_payload()               ← resolves servers, skills, hooks, permissions
        ↓
  export_to("claude"|"gemini"|"codex")
        ↓
  provider config files on disk ← agent binary reads these
```

Trigger: `git checkout` → post-checkout hook → `ship git post-checkout` → `sync_workspace`
Manual: `ship git sync`

## Upstream Validation (2026-03-09)

Validated against:
- Claude settings/hooks docs: <https://code.claude.com/docs/en/settings>, <https://code.claude.com/docs/en/hooks>
- Gemini CLI configuration docs: <https://geminicli.com/docs/reference/configuration/>
- OpenAI Codex config basics and schema: <https://developers.openai.com/codex/config-basic>, <https://raw.githubusercontent.com/openai/codex/main/codex-rs/core/config.schema.json>

Resulting implementation alignment:
- Claude hook export now uses grouped `hooks.<Event>[]` shape with nested command hooks.
- Gemini hook export is now emitted to `.gemini/settings.json` (hooks + MCP coexist).
- Codex remains hook-less by schema; Ship records hooks but intentionally skips Codex native export.
- Model suggestions are dynamically discovered from provider config/env instead of static baked lists.

---

## Provider Registry

Three providers are supported. The registry is static for provider descriptors, but
model suggestions are discovered dynamically from provider config and env.

### Claude Code

| Property | Value |
|---|---|
| ID | `claude` |
| Binary | `claude` |
| Config format | JSON |
| Project config | `.mcp.json` (project root) |
| Global config | `~/.claude.json` |
| MCP key | `mcpServers` |
| HTTP URL field | `url` |
| Emits `"type"` field | Yes (`"stdio"`, `"sse"`, `"http"`) |
| Managed marker | Inline: `"_ship": {"managed": true}` on each entry |
| Context file | `CLAUDE.md` (project root) |
| Skills output | `.claude/skills/<id>/SKILL.md` |

**Models (dynamic):**
Ship no longer ships a static Claude model list. It discovers model IDs from:
- `~/.claude.json` / project config where available (`model`, aliases)
- environment hints (e.g. `ANTHROPIC_MODEL`)
- Ship's own `[ai]` model selection in `ship.toml`

---

### Gemini CLI

| Property | Value |
|---|---|
| ID | `gemini` |
| Binary | `gemini` |
| Config format | JSON |
| Project config | `.gemini/settings.json` |
| Global config | `~/.gemini/settings.json` (same file) |
| MCP key | `mcpServers` |
| HTTP URL field | `httpUrl` (not `url`) |
| Emits `"type"` field | Yes (`"stdio"`, `"sse"`, `"http"`) |
| Managed marker | Inline: `"_ship": {"managed": true}` |
| Context file | `GEMINI.md` (project root) |
| Skills output | `.gemini/skills/<id>/SKILL.md` |

**Models (dynamic):**
Ship no longer ships a static Gemini model list. It discovers model IDs from:
- `.gemini/settings.json` (`model`, `model.name`, `modelConfigs`)
- environment hints (e.g. `GEMINI_MODEL`)
- Ship's own `[ai]` model selection in `ship.toml`

**Gemini quirk:** HTTP/SSE timeout field is converted to milliseconds on export
(all other providers use seconds).

---

### Codex CLI

| Property | Value |
|---|---|
| ID | `codex` |
| Binary | `codex` |
| Config format | TOML |
| Project config | `.codex/config.toml` |
| Global config | `~/.codex/config.toml` (same file) |
| MCP key | `mcp_servers` (underscore, not camelCase) |
| HTTP URL field | `url` |
| Emits `"type"` field | No |
| Managed marker | State-file only (no inline marker in TOML) |
| Context file | `AGENTS.md` (project root — shared with Roo Code, Amp, Goose) |
| Skills output | `.agents/skills/<id>/SKILL.md` |

**Models (dynamic):**
Ship no longer ships a static Codex model list. It discovers model IDs from:
- `.codex/config.toml` (`model`, `model_name`, `model_providers.*`)
- environment hints (e.g. `OPENAI_MODEL`, `CODEX_MODEL`)
- Ship's own `[ai]` model selection in `ship.toml`

**Codex TOML key difference:** Because Codex doesn't support inline managed markers in
TOML, Ship uses the `mcp_managed_state` DB table exclusively to track which server IDs
it wrote. On teardown, it can only remove servers it previously recorded — it cannot
detect Ship-managed entries by inspecting the file.

---

## Config Resolution (`resolve_agent_config`)

The effective agent config is computed in three layers, highest wins:

```
Layer 1: Project defaults
  └─ ship.toml: providers, hooks, mcp_servers (full list)
  └─ agents/permissions.toml: base permissions
  └─ .ship/agents/skills/: project-scoped skills
  └─ ~/.ship/skills/: user-scoped skills

Layer 2: Active mode override (if mode is set)
  └─ mode.mcp_servers: if non-empty, filters server list to only these IDs
  └─ mode.skills: if non-empty, filters skill list to only these IDs
  └─ mode.rules: if non-empty, filters rules
  └─ mode.permissions.allow/deny: overlays tools.allow/deny (replaces, not merges)
  └─ mode.hooks: appended to project hooks
  └─ mode.prompt_id → skill.content: written as system instructions

Layer 3: Feature [agent] overrides (if on a feature branch)
  └─ feature.agent.model: overrides model
  └─ feature.agent.providers: overrides provider list
  └─ feature.agent.mcp_servers: additional server IDs to include
  └─ feature.agent.skills: additional skill IDs to include
  └─ feature.agent.max_cost_per_session: overrides cost cap
```

**Mode resolution order:** `active_mode_override` arg → `config.active_mode` → no mode.

**Export targeting:** provider sync uses `config.providers` (fallback `["claude"]`) and does
not use mode-scoped `target_agents`.

### Workspace Session Provider Precedence

When compiling/exporting provider config for a workspace session, Ship resolves providers in this order:

1. `workspace.providers` override (workspace row)
2. `feature.agent.providers` override (if workspace links a feature)
3. active mode `target_agents`
4. project `config.providers`
5. fallback default `["claude"]`

Use `ship workspace providers --branch <branch> [--mode <id>]` to inspect the effective source,
allowed providers, and resolution errors.

---

## Export Process (`export_to`)

Called once per enabled provider. For each provider:

### 1. Build payload

`build_payload_with_mode_override` constructs a `SyncPayload`:
```
SyncPayload {
  servers: Vec<McpServerConfig>   ← full project server list
  instruction_skill_id: Option    ← None
  instructions: Option<String>    ← None
  hooks: Vec<HookConfig>          ← project hooks only
  permissions: Permissions        ← canonical agents/permissions.toml
  active_mode_id: Option<String>  ← None
}
```

### 2. Load managed state

Reads `managed_mcp_state` table from project SQLite. This gives the list of server IDs
Ship wrote in the previous sync, so they can be removed and replaced cleanly.

### 3. Write MCP config

**For JSON providers (Claude, Gemini):**
1. Reads existing config file (preserves user-defined entries)
2. Removes Ship-managed entries (identified by `"_ship": {"managed": true}` or state table)
3. Always injects the `ship` server entry:
   ```json
   "ship": {
     "command": "ship",
     "args": ["mcp"],
     "type": "stdio",
     "_ship": { "managed": true }
   }
   ```
4. Adds each non-disabled server from payload with `"_ship": {"managed": true}`
5. Writes updated config atomically

**For TOML providers (Codex):**
1. Reads existing `.codex/config.toml`
2. Removes only previously-managed server IDs (from state table — no inline marker)
3. Injects `ship` server entry:
   ```toml
   [mcp_servers.ship]
   command = "ship"
   args = ["mcp", "serve"]
   ```
4. Adds payload servers
5. Applies Codex permission fields (see permissions section)
6. Writes updated config atomically

### 4. Write context file

Writes the agent's system prompt file:
- Claude → `CLAUDE.md`
- Gemini → `GEMINI.md`
- Codex/others → `AGENTS.md`

Content is built by the git module (`ship_module_git`) from feature metadata, open issues,
skills, and rules. Written separately from the MCP config export.

**Mode instruction override:** If `payload.instructions` is set (from `mode.prompt_id`),
Ship writes `GEMINI.md` / `AGENTS.md` with a managed header:
```
<!-- managed by ship — instructions skill: <skill-id> -->

<skill content>
```
For Claude, `CLAUDE.md` is written by the git module directly — the mode instruction is
included inline in the CLAUDE.md content, not via this path.

### 5. Write skills

Skills are written using the [agentskills.io](https://agentskills.io) layout:
`<skills_dir>/<skill-id>/SKILL.md`

Each file gets a managed header:
```
<!-- managed by ship — skill: <id> -->

<skill content>
```

**Stale skill pruning:** Before writing, Ship scans the skills directory and removes any
`<id>/SKILL.md` directories whose `SKILL.md` starts with `<!-- managed by ship` but whose
ID is no longer in the active skill set. User-created skill dirs (no managed header) are
never touched.

| Provider | Skills directory |
|---|---|
| Claude | `.claude/skills/<id>/SKILL.md` |
| Gemini | `.gemini/skills/<id>/SKILL.md` |
| Codex | `.agents/skills/<id>/SKILL.md` |

### 6. Write hooks and native permissions

**Claude — `.claude/settings.json` (project-local):**
Written when managed Ship hooks are enabled (default) or tool permissions differ from
default (`["*"]` allow, empty deny). Merges into existing settings, preserving user entries.

```json
{
  "permissions": {
    "allow": ["Bash", "mcp__ship__*"],
    "deny": ["mcp__dangerous__*"]
  },
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "ship hooks run",
            "timeout": 2000,
            "description": "policy gate"
          }
        ]
      }
    ],
    "PostToolUse": [...]
  }
}
```

Hook trigger keys supported by export include:
`SessionStart`, `UserPromptSubmit`, `PreToolUse`, `PermissionRequest`,
`PostToolUse`, `PostToolUseFailure`, `Notification`, `SubagentStart`,
`SubagentStop`, `Stop`, `PreCompact`

**Gemini — `.gemini/settings.json` hooks:**
When managed hooks are enabled or custom hooks are configured, Ship writes Gemini-compatible grouped hook entries under
`hooks.<Event>[]` in `.gemini/settings.json` (in addition to MCP server export).
Example event keys include: `BeforeTool`, `AfterTool`, `BeforeAgent`, `AfterAgent`,
`SessionStart`, `SessionEnd`, `PreCompress`, `BeforeModel`, `AfterModel`,
`BeforeToolSelection`, `Notification`.

**Gemini — `.gemini/policies/ship-permissions.toml`:**
Written when tool permissions, command permissions, or `require_confirmation` patterns
are non-default. Deleted if permissions are default (no file = no policy).

Mapping from Ship permissions to Gemini policy rules:

| Ship field | Gemini decision | Priority |
|---|---|---|
| `tools.deny` patterns | `deny` | 900 |
| `commands.deny` patterns | `deny` on `run_shell_command` | 900 |
| `agent.require_confirmation` | `ask_user` on `run_shell_command` | 800 |
| `tools.allow` patterns | `allow` | 700 |
| `commands.allow` patterns | `allow` on `run_shell_command` | 700 |

Command patterns with a single trailing `*` become `commandPrefix` rules.
Other patterns are converted from glob to regex and become `commandRegex` rules.

Example output:
```toml
# managed by ship
# source: .ship/agents/permissions.toml

[[rule]]
toolName = "dangerous_tool"
decision = "deny"
priority = 900

[[rule]]
toolName = "run_shell_command"
commandPrefix = "rm -rf"
decision = "deny"
priority = 900
```

**Codex — `.codex/config.toml` + `.codex/rules/ship.rules` permissions fields:**
Codex network/sandbox settings are written to `.codex/config.toml`; command policy is written
as execpolicy rules to `.codex/rules/ship.rules`.

| Ship field | Codex field |
|---|---|
| `network.policy` (allow-list or unrestricted) | `sandbox_workspace_write.network_access = true` |
| `commands.allow` patterns | `.codex/rules/ship.rules` `prefix_rule(... decision="allow")` |
| `commands.deny` patterns | `.codex/rules/ship.rules` `prefix_rule(... decision="forbidden")` |
| `agent.require_confirmation` | `.codex/rules/ship.rules` `prefix_rule(... decision="prompt")` |

Codex `sandbox_mode` is always set to `"workspace-write"`.
`approval_policy` is `"on-failure"` if no restrictions, `"on-request"` if any deny/confirmation rules exist.

Example Codex config output:
```toml
sandbox_mode = "workspace-write"
approval_policy = "on-request"

[sandbox_workspace_write]
network_access = false

[mcp_servers.ship]
command = "ship"
args = ["mcp", "serve"]

[mcp_servers.my-server]
command = "my-server"
args = ["--port", "3000"]
```

### 7. Save managed state

Updates `managed_mcp_state` table in project SQLite with the list of server IDs just
written and the active mode ID. Used by next sync to identify which entries to replace.

---

## Import Process (`import_from_provider`)

Non-destructive. Reads the provider's existing config and adds any previously-unknown
servers to `ship.toml`. Does not overwrite existing Ship entries.

**Import path resolution:**
1. Prefers the project-local config file if it exists
2. Falls back to the global config file
3. Derives scope: project-path match → `"project"`, otherwise → `"global"`

**Filtering on import:**
- Servers with empty IDs are skipped
- The `ship` server itself is always skipped (it's runtime-managed)
- Servers already in `mcp_managed_state` (previously written by Ship) are skipped
- Servers already present in `ship.toml` by ID are skipped

**After import:** `ship.toml` is saved with new entries appended. Does not trigger sync
automatically — run `ship git sync` to regenerate provider configs.

---

## Permission Import (`import_permissions_from_provider`)

Reads provider-native permission files and writes them to `.ship/agents/permissions.toml`.
Destructive on the permissions file — previous content is replaced.

### From Claude (`.claude/settings.json`, project-local)

Reads `permissions.allow` and `permissions.deny` arrays directly.
Maps 1:1 to `tools.allow` / `tools.deny`.

### From Gemini (`.gemini/policies/ship-permissions.toml`)

Reads Ship's own exported policy file and reverses the mapping:

| Gemini rule | Ship field |
|---|---|
| `commandPrefix` + `deny` | `commands.deny` (as `prefix*`) |
| `commandPrefix` + `ask_user` | `agent.require_confirmation` (as `prefix*`) |
| `commandRegex` + any | `commands.*` (as `regex:<pattern>`) |
| `toolName` (not shell) + `allow` | `tools.allow` |
| `toolName` (not shell) + `deny` | `tools.deny` |
| `mcpName` + `toolName` | `tools.allow/deny` as `mcpName__toolName` |
| `mcpName` only | `tools.allow/deny` as `mcpName__*` |

### From Codex (`.codex/config.toml` + `.codex/rules/*.rules`)

| Codex field | Ship field |
|---|---|
| `sandbox_workspace_write.network_access = true` | `network.policy = unrestricted` |
| `sandbox_workspace_write.network_access = false` | `network.policy = none` |
| `.codex/rules/*.rules` `prefix_rule(... decision = "allow")` | `commands.allow` (as `prefix*`) |
| `.codex/rules/*.rules` `prefix_rule(... decision = "forbidden")` | `commands.deny` (as `prefix*`) |
| `.codex/rules/*.rules` `prefix_rule(... decision = "prompt")` | `agent.require_confirmation` (as `prefix*`) |
| legacy `allow = [...]` (if present) | `commands.allow` |
| legacy `rules.prefix_rules` (if present) | deny/prompt mapping above |

---

## Teardown (`teardown`)

Removes all Ship-generated config for a provider. Called when a provider is disabled
or a workspace is archived.

**Process:**
1. Load managed state for the provider
2. Remove managed MCP server entries from config file
   - JSON: removes entries marked `"_ship": {"managed": true}` or in state table
   - TOML: removes only entries in state table
   - If all servers were Ship-managed (file is now empty), deletes the file entirely
3. Delete context file (`CLAUDE.md` / `GEMINI.md` / `AGENTS.md`) if it exists
4. Delete Ship-managed skill directories (identified by `<!-- managed by ship` header)
5. Clear managed state for this provider in DB

User-defined MCP servers (no managed marker, not in state table) are always preserved.

---

## Managed State Ledger

Ship maintains a `managed_mcp_state` table in project SQLite to track what it wrote.

```sql
managed_mcp_state (
  provider        TEXT PRIMARY KEY,  -- e.g. "claude"
  server_ids_json TEXT,              -- JSON array of server IDs Ship wrote
  last_mode       TEXT,              -- mode ID active at last sync
  updated_at      TEXT
)
```

This is the mechanism that allows safe partial updates:
- On each sync, Ship loads the previous `server_ids_json`
- Removes those IDs from the config file
- Writes the new set of IDs
- Saves the new `server_ids_json`

This prevents Ship from ever removing servers it didn't write — even on providers like
Codex where inline markers aren't possible.

---

## Skills Delivery

### Storage

Skills live in the filesystem, not SQLite.

| Scope | Location |
|---|---|
| Project | `.ship/agents/skills/<id>/SKILL.md` |
| User (global) | `~/.ship/skills/<id>/SKILL.md` |

File format: `SKILL.md` with YAML frontmatter + markdown body.

```markdown
---
name: my-skill
description: What this skill does and when to use it.
metadata:
  display_name: My Skill
  source: custom
---

# My Skill

Instructions here...
```

### Effective skill list

`list_effective_skills` merges project + user skills. Project skills take precedence
(same ID in both = project wins).

### Export filtering

Skill export is not mode-filtered. Export writes all effective skills (project + user).

### Skill directories written at sync

All providers use the agentskills.io layout: `<dir>/<skill-id>/SKILL.md`

| Provider | Directory |
|---|---|
| Claude | `.claude/skills/<id>/SKILL.md` |
| Gemini | `.gemini/skills/<id>/SKILL.md` |
| Codex | `.agents/skills/<id>/SKILL.md` |

Each file gets a managed header: `<!-- managed by ship — skill: <id> -->`.
Stale directories (managed header, ID no longer in active set) are removed on next sync.

---

## Hooks

Defined in `ship.toml` under `[[hooks]]` or per-mode under `[[modes.hooks]]`.
Mode hooks are appended to project hooks (not replaced).

Ship also synthesizes a managed baseline for Claude/Gemini at export time (unless
`SHIP_MANAGED_HOOKS=0`), using `ship hooks run` as the default hook command
(`$SHIP_HOOKS_BIN` overrides this command). Managed baseline entries append
`--provider <id>` for provider-specific runtime output contracts.

```toml
[[hooks]]
id = "log-tool-use"
trigger = "PreToolUse"
matcher = "Bash"           # optional: glob/regex for tool name
timeout_ms = 2000          # optional: milliseconds
description = "tool audit" # optional: metadata for export/UI
command = "echo 'running bash'"
```

**Trigger values:** `SessionStart`, `UserPromptSubmit`, `PreToolUse`,
`PermissionRequest`, `PostToolUse`, `PostToolUseFailure`, `Notification`,
`SubagentStart`, `SubagentStop`, `Stop`, `PreCompact`, `BeforeTool`, `AfterTool`,
`BeforeAgent`, `AfterAgent`, `SessionEnd`, `BeforeModel`, `AfterModel`,
`BeforeToolSelection`

**Claude and Gemini** have hooks exported to native config files:
- Claude: `.claude/settings.json`
- Gemini: `.gemini/settings.json`

**Codex assessment:** Codex currently has no native hooks section in config schema,
so Ship keeps hooks provider-agnostic and skips Codex hook export.

**Runtime hook artifacts (Ship-managed):**
At export time for Claude/Gemini, Ship writes runtime hook artifacts to:
- `.ship/generated/runtime/hook-context.md`
- `.ship/generated/runtime/envelope.json`
- `~/.ship/state/telemetry/hooks/events.ndjson` (internal telemetry appended by `ship hooks run`; not user-facing project config)

`ship hooks run` now evaluates pre-tool/permission events against `envelope.json`
and emits provider-native allow/ask/deny decisions for Claude/Gemini.

---

## Provider Detection

`detect_binary(binary)` — uses `which` on Unix, falls back to manual PATH scan.

`detect_version(binary)` — runs `<binary> --version`, returns first line of stdout
(or stderr if stdout empty).

`autodetect_providers(project_dir)` — checks PATH for all three binaries, calls
`enable_provider` for each found. Returns list of newly-enabled IDs. Idempotent.

---

## Files Written Per Provider (Complete Reference)

### Claude

| File | When written | Content |
|---|---|---|
| `.mcp.json` | Every sync | MCP server registry (JSON) |
| `CLAUDE.md` | Every sync | Session context + rules (skills are exported to provider-native skills directories) |
| `.claude/skills/<id>/SKILL.md` | Every sync | One file per active skill |
| `.claude/settings.json` | Every sync when managed hooks are enabled (default), otherwise when hooks/non-default permissions exist | Hooks + tool permissions |

### Gemini

| File | When written | Content |
|---|---|---|
| `.gemini/settings.json` | Every sync (hooks included by managed baseline/custom config) | MCP server registry + hook lifecycle config |
| `GEMINI.md` | When mode has `prompt_id` skill | Mode instruction skill content |
| `.gemini/skills/<id>/SKILL.md` | Every sync | One file per active skill |
| `.gemini/policies/ship-permissions.toml` | When non-default permissions | Policy rules (TOML) |

### Codex

| File | When written | Content |
|---|---|---|
| `.codex/config.toml` | Every sync | MCP servers + permissions inline (TOML) |
| `AGENTS.md` | When mode has `prompt_id` skill | Mode instruction skill content |
| `.agents/skills/<id>/SKILL.md` | Every sync | One file per active skill |

### All providers

| File | When written | Content |
|---|---|---|
| `SHIPWRIGHT.md` | Every sync | Agent layer summary (skills list, mode, MCP servers) |
| `.gitignore` | On init and hook install | Gitignore entries for all generated files |

### Gitignored files (never commit these)

The pre-commit hook blocks staging any of these. The `.gitignore` is updated on init:

```
CLAUDE.md
GEMINI.md
AGENTS.md
.mcp.json
.claude/
.gemini/
.codex/
.agents/
```

---

## Sync Target Selection

When `sync_active_mode` runs, the provider target list is determined:

1. If active mode has `target_agents = [...]` and it's non-empty → use that list
2. Else if `config.providers` is non-empty → use that list
3. Else → default to `["claude"]`

Unknown provider IDs emit a warning and are skipped. Duplicates are deduplicated.

---

## `ship.toml` — Agent Configuration Sections

```toml
# Which providers are enabled for this project
providers = ["claude", "gemini"]

# Active mode (optional — filters tools/skills/servers)
active_mode = "focus"

[ai]
provider = "claude"
model = "claude-sonnet-4-6"
cli_path = "/usr/local/bin/claude"  # optional binary override

[[hooks]]
id = "pre-bash"
trigger = "PreToolUse"
matcher = "Bash"
command = "echo 'running: $TOOL_INPUT'"

# MCP server entries (also in agents/mcp.toml)
[[mcp_servers]]
id = "abc12345"
name = "My Server"
command = "my-server"
args = ["--port", "3000"]
env = { API_KEY = "..." }
scope = "project"          # "global" | "project" | "mode"
# server_type = "stdio"    # default; or "sse", "http"
# url = "http://..."       # for sse/http
# disabled = false
# timeout_secs = 30

[[modes]]
id = "focus"
name = "Focus Mode"
description = "Minimal tool surface for focused coding"
active_tools = ["Bash", "Read", "Edit"]     # empty = all
mcp_servers = ["abc12345"]                  # empty = all
skills = ["task-policy"]                    # empty = all
rules = ["no-comments"]                     # empty = all
prompt_id = "focus-instructions"            # skill ID for system instructions
target_agents = ["claude"]                  # empty = all enabled providers
  [modes.permissions]
  allow = ["Bash", "Read", "Edit", "mcp__ship__*"]
  deny = ["WebSearch", "WebFetch"]
[[modes.hooks]]
id = "mode-hook"
trigger = "Stop"
command = "ship log 'session complete'"

[agents]
skills = ["task-policy", "git-commit"]
prompts = []        # legacy alias for skills
context = []        # context files to preload
```
