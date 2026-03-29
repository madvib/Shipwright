---
group: Agents
title: Agents
order: 1
---

# Agents

An agent is a JSONC profile that controls how an AI assistant behaves -- which skills it loads, which MCP servers it connects to, what permissions it gets, and which rules it follows.

Agents are not running processes. They are configuration files in `.ship/agents/` that the compiler transforms into provider-specific output (CLAUDE.md, .cursor/rules, .gemini/settings.json, etc.).

## File structure

Each agent is a single `.jsonc` file. The filename stem must match the `id` field inside:

```
.ship/agents/
  default.jsonc
  web-lane.jsonc
  reviewer.jsonc
```

## Anatomy of an agent

An agent profile has a required `agent` section and optional sections that compose its behavior:

```jsonc
{
  "$schema": "../../schemas/agent.schema.json",
  "agent": {
    "id": "default",
    "name": "Ship Dev",
    "version": "0.1.0",
    "description": "Default development preset for the Ship project",
    "providers": ["claude", "codex", "cursor", "gemini", "opencode"]
  },
  "skills": {
    "refs": ["ship-permissions", "ship-tutorial"]
  },
  "mcp": {
    "servers": []
  },
  "plugins": {
    "install": ["rust-analyzer-lsp@claude-plugins-official"],
    "scope": "project"
  },
  "permissions": {
    "preset": "ship-standard",
    "tools_deny": ["Bash(rm -rf *)", "Bash(git reset --hard*)"]
  },
  "rules": {}
}
```

This is the actual `default.jsonc` from the Ship project. It targets all five providers, activates two skills, installs a Claude plugin, and uses the `ship-standard` permission preset with additional deny rules.

## What each section does

| Section | Purpose |
|---------|---------|
| `agent` | Identity: id, name, version, description, target providers, tags |
| `skills` | Skill IDs to activate (local or from installed packages) |
| `mcp` | MCP server IDs to connect (defined in `.ship/mcp.jsonc`) |
| `plugins` | Provider-specific plugin installs (currently Claude Code) |
| `permissions` | Permission preset + per-agent tool allow/deny/ask overrides |
| `rules` | Inline rules appended after shared `.ship/rules/*.md` files |
| `model` | Model override compiled to each provider's model field |
| `provider_settings` | Per-provider config pass-through (deep-merged over project defaults) |

## Agent sources

Agents come from two locations:

- **Project agents** -- `.ship/agents/` in your repository. Version-controlled, shared with the team.
- **Library agents** -- `~/.ship/agents/` in your home directory. Personal, not committed.

`ship agents list` shows both sources.

## Multi-provider compilation

A single agent can target multiple providers. The `providers` array controls which outputs `ship use` generates:

```jsonc
{
  "agent": {
    "providers": ["claude", "gemini", "codex"]
  }
}
```

Running `ship use web-lane` compiles for every listed provider simultaneously. Each provider gets its native format. All output files are gitignored build artifacts.

## CLI commands

| Command | Description |
|---------|-------------|
| `ship agents list` | List all agents (project and library) |
| `ship agents create <name>` | Scaffold a new agent profile |
| `ship agents edit <name>` | Open agent in your editor |
| `ship agents clone <src> <dst>` | Duplicate an agent |
| `ship agents delete <name>` | Remove an agent profile |
| `ship use <agent-id>` | Compile and activate an agent |

## A specialized agent

Here is the `web-lane.jsonc` agent, which configures an autonomous frontend specialist:

```jsonc
{
  "agent": {
    "id": "web-lane",
    "name": "Web Lane",
    "version": "0.1.0",
    "description": "Active context for the Web lane agent",
    "providers": ["claude", "codex", "cursor", "gemini", "opencode"]
  },
  "skills": {
    "refs": [
      "github.com/better-auth/skills/better-auth",
      "ship-coordination",
      "studio-mcp-bridge",
      "visual-brainstorm",
      "visual-spec"
    ]
  },
  "mcp": { "servers": ["ship"] },
  "plugins": {
    "install": ["frontend-design@claude-plugins-official"],
    "scope": "project"
  },
  "permissions": {
    "preset": "ship-autonomous",
    "tools_deny": ["Bash(rm -rf *)", "Bash(git reset --hard*)"]
  },
  "rules": {
    "inline": "You are a web specialist working in apps/web/."
  }
}
```

This agent uses `ship-autonomous` permissions (zero confirmation prompts), references both local and remote skills, connects the Ship MCP server, and injects scope-restricting inline rules.

See [Configuration](./configuration.md) for every field and [Permissions](./permissions.md) for the preset system.
