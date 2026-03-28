---
title: "Agent Configuration"
sidebar:
  label: "Agent Configuration"
  order: 2
---
Complete reference for every field in `.ship/agents/<id>.jsonc`, derived from the `agent.schema.json` schema.

## agent (required)

Agent identity and metadata. Both `id` and `name` are required.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | yes | Machine identifier. Must match the filename stem. Pattern: `^[a-z0-9-]+$` |
| `name` | string | yes | Human-readable name. Shown in `ship agents list` and Studio UI. |
| `version` | string | no | Semver version. Default: `"0.1.0"` |
| `description` | string | no | What this agent does. Shown in listings and editor UI. |
| `tags` | string[] | no | Searchable tags for registry discovery (e.g., `["frontend", "review"]`) |
| `providers` | string[] | no | Provider targets. Values: `claude`, `cursor`, `codex`, `gemini`, `opencode`. Overrides the project-level providers list when set. |

## model

Top-level string field. Overrides the default model for this agent.

```jsonc
{
  "model": "claude-opus-4-20250514"
}
```

Compiled to each provider's model field: `model` in Claude settings, `model.name` in Gemini settings, `model` in Codex config.

## skills

Activates skill content for this agent. Skills are markdown instruction sets resolved by ID.

| Field | Type | Description |
|-------|------|-------------|
| `refs` | string[] | Skill IDs to activate |

Skill IDs can be local (matching a directory in `.ship/skills/`) or namespaced (from an installed package, e.g., `github.com/owner/repo/skill-name`). The compiler resolves each reference and includes the skill content in compiled output.

```jsonc
{
  "skills": {
    "refs": ["tdd", "code-review", "github.com/better-auth/skills/better-auth"]
  }
}
```

## mcp

Activates MCP servers for this agent. Server definitions live in `.ship/mcp.jsonc`.

| Field | Type | Description |
|-------|------|-------------|
| `servers` | string[] | Server IDs from `.ship/mcp.jsonc` to activate |

```jsonc
{
  "mcp": {
    "servers": ["ship", "github"]
  }
}
```

The compiler resolves each server ID to its full configuration and writes it into the provider's MCP config (`.mcp.json` for Claude, `.gemini/settings.json` for Gemini, etc.).

## plugins

Provider-specific extension installs. The compiler emits a manifest; the CLI executes the installs.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `install` | string[] | -- | Plugin IDs (e.g., `"rust-analyzer-lsp@claude-plugins-official"`) |
| `scope` | `"project"` or `"user"` | `"project"` | `project` installs locally, `user` installs globally |

```jsonc
{
  "plugins": {
    "install": ["rust-analyzer-lsp@claude-plugins-official"],
    "scope": "project"
  }
}
```

## permissions

Permission configuration. Starts from a named preset and layers per-agent overrides on top.

| Field | Type | Description |
|-------|------|-------------|
| `preset` | string | Named preset from `.ship/permissions.jsonc` (e.g., `"ship-standard"`) |
| `tools_allow` | ToolPattern[] | Additional tool patterns to allow |
| `tools_deny` | ToolPattern[] | Additional tool patterns to deny |
| `tools_ask` | ToolPattern[] | Tool patterns requiring user confirmation |

Overrides are merged onto the preset, not replacements. The deny list always wins over allow.

```jsonc
{
  "permissions": {
    "preset": "ship-autonomous",
    "tools_allow": ["Bash(docker *)"],
    "tools_deny": ["Bash(rm -rf *)"],
    "tools_ask": ["Bash(git push --force*)"]
  }
}
```

See [Permissions](./permissions.md) for the preset definitions and tool pattern syntax.

## rules

Inline rules appended to the compiled context file after shared `.ship/rules/*.md` files.

| Field | Type | Description |
|-------|------|-------------|
| `inline` | string | Inline rules text. Use `\n` for line breaks. |

```jsonc
{
  "rules": {
    "inline": "You are a web specialist working in apps/web/.\nNEVER touch: wrangler.toml, package.json"
  }
}
```

Shared rules from `.ship/rules/*.md` are automatically included for all agents. The `inline` field adds agent-specific rules on top.

## provider_settings

Per-provider configuration pass-through. Deep-merged on top of project-level `provider_defaults` from `ship.jsonc`. Agent values win on conflict.

Keys are provider names: `claude`, `codex`, `gemini`, `cursor`, `opencode`.

```jsonc
{
  "provider_settings": {
    "claude": {
      "contextFileFormat": "markdown"
    },
    "gemini": {
      "codeExecution": true
    }
  }
}
```

{% aside type="caution" %}
Ship-managed fields cannot be set in `provider_settings`. Use Ship's own fields for: `permissions`, `hooks`, `model`, `env`, MCP servers. The schema enforces this -- setting a managed field here is a validation error.
{% /aside %}

Provider-specific schemas are validated against upstream definitions:
- **Claude** -- `schemastore.org/claude-code-settings.json` (excludes permissions, hooks, model, env, availableModels, maxCostPerSession, maxTurns, autoMemoryEnabled)
- **Codex** -- upstream `config.schema.json` (excludes model, mcp_servers)
- **Gemini** -- upstream `settings.schema.json` (excludes model, hooks, mcpServers)
- **Cursor** -- no upstream schema; accepts any object
- **OpenCode** -- upstream `config.json` (excludes model, mcp, permission)

## Tool pattern syntax

Tool patterns are used in `tools_allow`, `tools_deny`, and `tools_ask` arrays.

| Pattern | Matches |
|---------|---------|
| `Read` | The Read tool with any arguments |
| `Bash(git *)` | Bash where the command starts with `git ` |
| `Bash(*)` | Any Bash command |
| `mcp__ship__*` | All tools from the Ship MCP server |
| `mcp__ship__create_job` | A specific MCP tool |
| `Write(*)` | Write tool with any file path |
| `Write(**/migrations/**/*.sql)` | Write to files matching a glob |

Patterns use glob-style matching. `*` matches any sequence of characters within the argument.
