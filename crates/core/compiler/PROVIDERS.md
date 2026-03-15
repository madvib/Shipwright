# Provider Reference

Authoritative file paths, config schemas, and compiler support status for each agent provider.

**Rule:** When changing a `ProviderDescriptor` field, update the corresponding row here first.
Add a `<!-- verified: YYYY-MM-DD source: <url> -->` comment when re-verifying a section.

---

## Support Matrix

<!-- Last updated: 2026-03-14 -->

| Feature | Claude Code | Gemini CLI | Codex CLI | Cursor |
|---|:---:|:---:|:---:|:---:|
| MCP servers | ✅ | ✅ | ✅ TOML `codex_config_patch` | ✅ |
| Rules / context file | ✅ `CLAUDE.md` | ✅ `GEMINI.md` | ✅ `AGENTS.md` | ✅ `rule_files` per-file `.mdc` |
| Skills | ✅ `.claude/skills/` | ✅ `.gemini/skills/` | ✅ `.agents/skills/` | ✅ `.cursor/skills/` |
| Permissions allow/deny | ✅ | ✅ `gemini_policy_patch` → `.gemini/policies/ship.toml` | — no compatible model | ✅ `cursor_cli_permissions` |
| Permissions ask tier | ✅ `permissions.ask` | — | — | — |
| defaultMode | ✅ `permissions.defaultMode` | — | — | — |
| additionalDirectories | ✅ `permissions.additionalDirectories` | — | — | — |
| Hooks | ✅ `claude_settings_patch` | ✅ `gemini_settings_patch` | — | ✅ `cursor_hooks_patch` |
| Agent limits (cost/turns) | ✅ | — | — | — |
| Model override | ✅ `model` in settings patch | — needs `provider_config` | — needs `provider_config` | — needs `provider_config` |
| MCP config path | ✅ `.mcp.json` | ✅ `.gemini/settings.json` | ✅ `.codex/config.toml` | ✅ `.cursor/mcp.json` |
| Provider-specific settings | — (model ✅, rest needs `provider_config`) | — needs `provider_config` | — needs `provider_config` | — needs `provider_config` |
| Multi-agent roles | — | — | ⚠️ `agents/*.toml` + `[agents]` in config | — |

**Key:** ✅ implemented · ⚠️ partial/not compiled · — not implemented / out of scope

---

## Claude Code

<!-- verified: 2026-03-14 source: https://docs.anthropic.com/en/docs/claude-code -->

| Asset | Path | Format |
|---|---|---|
| Rules / context | `CLAUDE.md` | Markdown. Also `~/.claude/CLAUDE.md` (user-global) |
| Skills | `.claude/skills/<id>/SKILL.md` | Markdown + YAML frontmatter |
| MCP config | `.mcp.json` | JSON `{ "mcpServers": { ... } }` |
| Settings | `.claude/settings.json` | JSON |

**`.claude/settings.json` — compiled fields:**
```json
{
  "model": "claude-opus-4-6",
  "permissions": {
    "allow": ["Read", "Glob"],          // omit when allow=[*] — strict allowlist otherwise
    "ask":   ["Bash(git push *)"],      // confirm before executing
    "deny":  ["Bash(rm -rf *)"],
    "defaultMode": "acceptEdits",       // "default" | "acceptEdits" | "plan" | "bypassPermissions"
    "additionalDirectories": ["../docs"]
  },
  "hooks": {
    "PreToolUse": [{ "matcher": "Bash", "command": "...", "type": "command" }]
  },
  "maxCostPerSession": 5.0,
  "maxTurns": 20
}
```
Note: `maxCostPerSession` / `maxTurns` are compiled from `AgentLimits` but are not in the official Claude Code settings docs — they may be agent-SDK-only fields. Verify if these are ignored in current Claude Code versions.

**MCP entry (stdio — no `type` field):**
```json
"server-id": { "command": "npx", "args": [...], "env": {} }
```
HTTP/SSE: `"url"` field. No `type` field — transport inferred from field presence.

---

## Gemini CLI

<!-- verified: 2026-03-14 source: https://geminicli.com/docs/tools/mcp-server/ -->
<!-- verified: 2026-03-14 source: https://geminicli.com/docs/cli/skills -->
<!-- verified: 2026-03-14 source: https://geminicli.com/docs/hooks -->
<!-- verified: 2026-03-14 source: https://geminicli.com/docs/cli/settings -->

| Asset | Path | Format |
|---|---|---|
| Rules / context | `GEMINI.md` | Markdown |
| Skills | `.gemini/skills/<id>/SKILL.md` | Markdown + YAML frontmatter. `.agents/skills/` alias takes precedence within same tier |
| MCP config | `.gemini/settings.json` → `mcpServers` key | JSON nested in project settings file |
| Settings / hooks | `.gemini/settings.json` | JSON. Global: `~/.gemini/settings.json` |
| Permissions | `.gemini/policies/ship.toml` | TOML `[[tool_policies]]` array. ✅ compiled to `gemini_policy_patch` |

**MCP entry — transport inferred from field presence (no `type` field):**
```json
"server-id": { "command": "npx", "args": [...] }        // stdio
"server-id": { "url": "https://..." }                    // SSE      ← "url"
"server-id": { "httpUrl": "https://..." }                // HTTP     ← "httpUrl" (different from SSE!)
```

**Hooks schema (trigger names differ from Claude):**
```json
{
  "hooks": {
    "BeforeTool": [{ "matcher": "regex", "hooks": [{ "type": "command", "command": "..." }] }]
  }
}
```
Triggers: `SessionStart`, `SessionEnd`, `BeforeAgent`, `AfterAgent`, `BeforeModel`, `AfterModel`,
`BeforeToolSelection`, `BeforeTool`, `AfterTool`, `PreCompress`, `Notification`

---

## OpenAI Codex

<!-- verified: 2026-03-14 source: https://developers.openai.com/codex/config-basic -->
<!-- verified: 2026-03-14 source: https://developers.openai.com/codex/mcp -->
<!-- verified: 2026-03-14 source: https://developers.openai.com/codex/skills -->
<!-- verified: 2026-03-14 source: https://developers.openai.com/codex/rules -->
<!-- verified: 2026-03-14 source: https://developers.openai.com/codex/multi-agent -->
<!-- verified: 2026-03-14 source: https://developers.openai.com/codex/guides/agents-md -->

| Asset | Path | Format |
|---|---|---|
| Rules / context | `AGENTS.md` | Markdown. Searched from git root → CWD, each level. Also `~/.codex/AGENTS.md` (global) |
| Approval policy | `~/.codex/config.toml` → `approval_policy` | TOML — `"suggest"`, `"auto-edit"`, or `"full-auto"`. Provider-specific setting. |
| Skills | `.agents/skills/<id>/SKILL.md` | Markdown + YAML frontmatter. Also `~/.agents/skills/` |
| MCP config | `.codex/config.toml` | **TOML** `[mcp_servers.<name>]` tables |
| Settings | `~/.codex/config.toml` | TOML. Also `.codex/config.toml` (project) |
| Multi-agent roles | `.codex/config.toml` → `[agents.<name>]` | TOML. Role configs at e.g. `agents/explorer.toml` |

**MCP entry (TOML — NOT JSON):**
```toml
[mcp_servers.server-id]
command = "npx"
args = ["-y", "@org/pkg"]
startup_timeout_sec = 10   # note: _sec not _secs
tool_timeout_sec = 60
enabled = true

[mcp_servers.remote]
url = "https://api.example.com/mcp"
```

**TOML output:** `CompileOutput.codex_config_patch` contains the serialised TOML for `.codex/config.toml`.
The JSON `mcp_servers` field is still populated for internal use but Codex consumers should use `codex_config_patch`.

**Codex has no permission model compatible with Ship's allow/deny patterns.**
`approval_policy` (`suggest` / `auto-edit` / `full-auto`) is a provider-specific setting
that belongs in `provider_config`, not in the permissions model.

---

## Cursor

<!-- verified: 2026-03-14 source: https://cursor.com/docs/context/rules -->
<!-- verified: 2026-03-14 source: https://cursor.com/docs/context/skills -->

| Asset | Path | Format |
|---|---|---|
| Rules | `.cursor/rules/<name>.mdc` | Markdown + YAML frontmatter (per-rule file) ✅ compiled |
| Skills | `.cursor/skills/<id>/SKILL.md` | Markdown + YAML frontmatter |
| MCP config | `.cursor/mcp.json` | JSON `{ "mcpServers": { ... } }` |
| Permissions | `.cursor/cli.json` | JSON `{ "version": 1, "permissions": { ... } }` ✅ compiled |

**Cross-agent skill paths Cursor also scans:**
`.agents/skills/`, `.claude/skills/`, `.codex/skills/`, `~/.cursor/skills/`

**MCP entry (same shape as Claude — no `type` field):**
```json
"server-id": { "command": "npx", "args": [...] }
"server-id": { "url": "https://..." }   // HTTP or SSE — Cursor does not distinguish at config level
```

**`.cursor/rules/<name>.mdc` frontmatter schema** (all three fields optional):
```yaml
---
description: "When and why this rule applies"  # drives "Apply Intelligently"
globs:                                          # drives "Apply to Specific Files"
  - src/**/*.ts
alwaysApply: true                               # true = inject always (default)
---
```

Rule application modes:

| Mode | `alwaysApply` | `globs` | `description` |
|---|:---:|:---:|:---:|
| Always Apply (default) | `true` | — | — |
| Apply Intelligently | `false` | — | ✅ required |
| Apply to Specific Files | `false` | ✅ | optional |
| Apply Manually (@-mention) | `false` | — | — |

Keep rules under 500 lines. Cursor ignores rules in Inline Edit (Cmd+K) — Agent Chat only.

**Context file behavior:** Cursor uses `rule_files` (individual `.mdc`) instead of a single
`AGENTS.md`. `context_content` is always `None` for Cursor. Other providers concatenate all
rules into one file (`CLAUDE.md`, `GEMINI.md`, `AGENTS.md`) — keep individual rules concise
to avoid bloating those files.

---

## SKILL.md Frontmatter (all providers)

```yaml
---
name: my-skill           # required. Lowercase, hyphens. Must match directory name.
description: >           # required. Used for implicit skill matching/activation.
  Describe exactly when this skill should and should not be used.
# Optional (Cursor adds):
license: MIT
compatibility: "node >= 18"
disable-model-invocation: false   # true = requires explicit /skill-name invocation
---
```

Optional subdirectories inside a skill folder: `scripts/`, `references/`, `assets/`
Codex also supports `agents/openai.yaml` for provider-specific metadata.

---

---

## Permissions Models

Each provider uses a fundamentally different model. Claude's is the primary compilation target; others are best-effort translations.

### Claude Code — allow/deny lists

Already in `.claude/settings.json`. See Claude Code section above.
Translation is lossless — Claude Code uses the same pattern format as Ship's internal model.

### Gemini CLI — policy engine (TOML files)

<!-- verified: 2026-03-14 source: https://geminicli.com/docs/reference/policy-engine/ -->

Gemini's policy engine uses TOML files, **not** `settings.json`. Two locations:
- Global: `~/.gemini/policies/*.toml`
- Project: `.gemini/policies/*.toml` (project files take precedence)

**Policy file format:**
```toml
[[tool_policies]]
tool = "shell"       # or "mcp", "file_read", "file_write", "web_fetch"
pattern = "rm -rf"   # regex; omit for any-match
decision = "deny"    # "allow", "deny", "ask_user"

[[tool_policies]]
tool = "mcp"
pattern = ".*delete.*"
decision = "ask_user"
```

**Decisions:** `allow` · `deny` · `ask_user`
**Tool names:** `shell`, `mcp`, `file_read`, `file_write`, `web_fetch`, `code_execution`

Compiled to `gemini_policy_patch` → write to `.gemini/policies/ship.toml`.
Translation: Claude `Bash(cmd)` → `tool="shell" pattern="cmd"` (glob converted to regex), `mcp__s__t` → `tool="mcp" pattern="s/t"`.

### Codex CLI — no Ship-compatible permission model

<!-- verified: 2026-03-14 source: https://developers.openai.com/codex/config-advanced -->

Codex's `approval_policy` (`suggest` / `auto-edit` / `full-auto`) controls how the agent handles tool use, but it is a global approval mode — not a per-tool allow/deny list. It belongs in `provider_config`, not in Ship's permissions model.

Ship does not compile any permission rules for Codex.

### Cursor — typed permissions (IDE agents + CLI)

<!-- verified: 2026-03-14 source: https://cursor.com/docs/cli/reference/permissions -->

Cursor permissions live in `.cursor/cli.json` (project) or `~/.cursor/cli-config.json` (global).
**These apply to both Cursor IDE agents and the Cursor CLI** — not CLI-only.

**Format:**
```json
{
  "permissions": {
    "allow": ["Shell(git *)", "Read(**/*)", "Mcp(ship:*)"],
    "deny":  ["Shell(rm -rf *)", "Mcp(*:delete*)"]
  }
}
```

**Typed pattern syntax:**
- `Shell(cmd)` — shell command (maps from Claude `Bash(cmd)`)
- `Read(glob)` — file read; `Read(**/**)` = any file
- `Write(glob)` — file write
- `WebFetch(domain)` — HTTP fetch
- `Mcp(server:tool)` — MCP tool call

**Translation from Ship's model:**

| Ship (Claude) pattern | Cursor CLI pattern |
|---|---|
| `Bash(cmd)` | `Shell(cmd)` |
| `Bash` (bare) | `Shell(*)` |
| `Read` / `Glob` / `LS` | `Read(*)` |
| `Write` / `Edit` / `MultiEdit` | `Write(*)` |
| `WebFetch` (bare) | `WebFetch(*)` |
| `WebFetch(domain)` | `WebFetch(domain)` |
| `mcp__server__tool` | `Mcp(server:tool)` |
| `mcp__*__delete*` | `Mcp(*:delete*)` |
| `*` (wildcard) | omitted (CLI default) |
| Other tools (e.g. `NotebookEdit`) | omitted (no CLI equivalent) |

**Permissive preset** (`CURSOR_PERMISSIVE_ALLOW` constant in the compiler):
```json
{ "permissions": { "allow": ["Shell(*)", "Read(*)", "Write(*)", "WebFetch(*)", "Mcp(*:*)"] } }
```
This must only be emitted when the user has **explicitly selected** a permissive mode (with UI warnings). The bare `"*"` wildcard is intentionally NOT auto-expanded — Cursor's default without a config file is interactive/prompt, which is the safe default.

**Global permissive** (`~/.cursor/cli-config.json`) must never be written unless the user explicitly selects it at the global level (with additional warnings — it grants full access to all Cursor agents on the machine).

Compiled to `cursor_cli_permissions` field in `CompileOutput`.

---

## Provider-Specific Settings

These settings affect **how the agent runs** (model, sandbox, approval policy, telemetry) rather than what it can do — orthogonal to workspace modes. They require a `provider_config: HashMap<String, Value>` concept in `ProjectLibrary` (not yet implemented).

<!-- verified: 2026-03-14 source: https://geminicli.com/docs/cli/settings -->
<!-- verified: 2026-03-14 source: https://developers.openai.com/codex/config-advanced -->

### Gemini CLI (`~/.gemini/settings.json` or `.gemini/settings.json`)

Key settings relevant to Ship:
```json
{
  "model": "gemini-2.5-pro",
  "general": {
    "defaultApprovalMode": "auto-edit",  // "suggest" | "auto-edit" | "yolo"
    "autoAcceptedEdits": true
  },
  "security": {
    "hideSensitiveEnvVars": true,
    "sandbox": "none"                    // "none" | "docker" | "...
  },
  "telemetry": {
    "enabled": false
  }
}
```
`defaultApprovalMode` maps closest to our workspace mode concept.

### OpenAI Codex (`.codex/config.toml`)

Key settings relevant to Ship:
```toml
model = "o4-mini"
approval_policy = "auto-edit"      # "suggest" | "auto-edit" | "full-auto"
sandbox_mode = "network-disabled"  # "danger-full-internet" | "network-disabled" | "disabled"
shell_environment_policy = "inherit"
notify = true

[otel]
otlp_endpoint = "http://localhost:4317"
```
`approval_policy` maps to our workspace mode concept.

---

## Adding a New Provider

1. Fetch the provider's official docs for: rules, skills, MCP, settings, hooks
2. Add a verified section above with `<!-- verified: YYYY-MM-DD source: <url> -->` on each source
3. Add a row to the Support Matrix
4. Add a `ProviderDescriptor` entry in `crates/core/compiler/src/compile/mod.rs`
5. Add `"<id>"` to `normalize_providers()` in `crates/core/compiler/src/resolve.rs`
6. Add tests: provider_exists, mcp_format, skill_dir, context_file, no_settings_patch (if applicable)
