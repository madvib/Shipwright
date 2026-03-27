---
name: ship-cli-reference
stable-id: ship-cli-reference
description: Use when the user asks how to use a specific ship command, what flags are available, or needs examples of ship CLI usage. Covers init, use, compile, agent, skill, install, add, publish, status, docs, and view.
tags: [reference, cli, documentation]
authors: [ship]
---

# Ship CLI Reference

## Setup

### `ship init`
Scaffold `.ship/` in the current project, or configure `~/.ship/` globally.
```
ship init [--global] [--provider <id>] [--force]
```
- `--global` — configure `~/.ship/` identity and defaults
- `--provider <id>` — default provider: `claude`, `gemini`, `codex`, `cursor`
- `--force` — overwrite existing `.ship/`

### `ship validate`
Check `.ship/` config for errors before compile — JSONC syntax, skill refs, MCP fields, permissions.
```
ship validate [--agent <id>] [--json] [--path <dir>]
```

## Agents

### `ship use`
Activate an agent and compile immediately.
```
ship use <agent-id> [--path <dir>]
```
`<agent-id>` — local ID, registry ref (`@org/agent`), or URL.

### `ship compile`
Compile the active agent to provider-native config (CLAUDE.md, .cursor/, .mcp.json).
```
ship compile [--provider <id>] [--dry-run] [--path <dir>]
```

### `ship agent list`
List available agent profiles.
```
ship agent list [--local] [--project]
```

### `ship agent create`
Create a new agent profile (project-local by default). IDs: lowercase + hyphens.
```
ship agent create <name> [--global]
```

### `ship agent edit`
Open an agent profile in `$EDITOR`.
```
ship agent edit <name> [--editor <cmd>]
```

### `ship agent delete` / `ship agent clone`
```
ship agent delete <name>
ship agent clone <source> <target>
```

## Skills

### `ship skill add`
Install a skill from GitHub, registry, or local path.
```
ship skill add <source> [--skill <id>] [--global]
```
- `--skill <id>` — required when repo has multiple skills
- `--global` — install to `~/.ship/skills/`
- Source formats: GitHub URL, `owner/repo`, or `skill-id@registry`

### `ship skill list` / `ship skill remove`
```
ship skill list
ship skill remove <id> [--global]
```

### `ship skill create`
Scaffold a new skill with a SKILL.md template. IDs: lowercase, digits, hyphens.
```
ship skill create <id> [--name <name>] [--description <desc>]
```

## MCP Servers

### `ship mcp serve`
Run the Ship MCP server (stdio by default).
```
ship mcp serve [--http] [--port <n>]
```

### `ship mcp add`
Register an HTTP/SSE MCP server.
```
ship mcp add <id> --url <url> [--name <name>] [--global]
```

### `ship mcp add-stdio`
Register a stdio MCP server.
```
ship mcp add-stdio <id> <command> [args...] [--name <name>] [--global]
```

### `ship mcp list` / `ship mcp remove`
```
ship mcp list
ship mcp remove <id>
```

## Registry

### `ship install`
Resolve and install all dependencies from `.ship/ship.jsonc`, then compile.
```
ship install [--frozen]
```
`--frozen` — fail if lockfile would change (CI-safe). Requires `"module"` in `ship.jsonc`.

### `ship add`
Add a package dependency to `.ship/ship.jsonc` and install it. Restores ship.jsonc on failure.
```
ship add <package>[@version]
```
Version defaults to `main` if omitted.

### `ship publish`
Publish the current package to the Ship registry. Requires `ship login`.
```
ship publish [--dry-run] [--tag <tag>]
```
`--dry-run` — preview without network. `--tag` — dist-tag for pre-release.

## Auth

### `ship login` / `ship logout` / `ship whoami`
```
ship login      # PKCE OAuth via browser
ship logout     # remove stored token
ship whoami     # show current identity
```

## Info

### `ship status`
Show active agent and compilation status.
```
ship status [--path <dir>]
```

### `ship events list`
Query the project event log.
```
ship events list [--since <time>] [--actor <name>] [--entity <type>] [--action <name>] [--limit <n>] [--json]
```

### `ship surface`
Print the CLI command tree and MCP core tools as markdown.
```
ship surface [--emit] [--check]
```

### `ship docs`
Extended help topics.
```
ship docs [topic]
```
Topics: agents, compile, config, mcp, providers, skills, workflow.

### `ship view`
Browse workflow state in the terminal UI.
```
ship view
```
