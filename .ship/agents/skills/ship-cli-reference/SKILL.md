---
name: ship-cli-reference
description: Complete CLI reference for Ship commands. Use when users ask how to use a specific ship command, what flags are available, or need examples of ship CLI usage. Covers init, use, compile, agent, skill, install, add, publish, status, and help.
tags: [reference, cli, documentation]
authors: [ship]
---

# Ship CLI Reference

Agent configuration studio — compose, compile, distribute.

## Setup

### `ship init`
Scaffold `.ship/` in the current project, or configure `~/.ship/` globally.
```
ship init [--global] [--provider <id>] [--force]
```
- `--global` — configure `~/.ship/` identity and defaults instead of current project
- `--provider <id>` — default provider: `claude`, `gemini`, `codex`, `cursor`
- `--force` — overwrite existing `.ship/`

```sh
ship init --global --provider claude
```

### `ship validate`
Check `.ship/` config for errors before compile — TOML, skill refs, MCP fields, permissions.
```
ship validate [--profile <id>] [--json] [--path <dir>]
```
- `--profile <id>` — validate a single profile (omit for all)
- `--json` — emit errors as JSON array
- `--path <dir>` — project root

```sh
ship validate --profile rust-expert
```

## Agents

### `ship use`
Activate an agent profile and compile immediately.
```
ship use <profile-id> [--path <dir>]
```
- `--path <dir>` — bind to this path instead of cwd
- `<profile-id>` — local ID, registry ref (`@org/profile`), or URL

```sh
ship use rust-expert
ship use cli-lane --path ~/projects/my-app
```

### `ship compile`
Compile the active profile to provider-native config (CLAUDE.md, .cursor/, .mcp.json).
```
ship compile [--provider <id>] [--dry-run] [--watch] [--path <dir>]
```
- `--provider <id>` — compile for one provider only
- `--dry-run` — preview without writing files
- `--watch` — recompile on file changes
- `--path <dir>` — project root

```sh
ship compile --provider claude --dry-run
```

### `ship agent list`
List available agent profiles.
```
ship agent list [--local] [--project]
```
- `--local` — only `~/.ship/modes/`
- `--project` — only `.ship/modes/`

### `ship agent create`
Create a new agent profile (project-local by default). IDs: lowercase + hyphens.
```
ship agent create <name> [--global]
```
```sh
ship agent create rust-expert
ship agent create shared-reviewer --global
```

### `ship agent edit`
Open an agent profile in `$EDITOR`.
```
ship agent edit <name> [--editor <cmd>]
```

### `ship agent delete`
Delete an agent profile.
```
ship agent delete <name>
```

### `ship agent clone`
Clone an agent profile under a new ID.
```
ship agent clone <source> <target>
```
```sh
ship agent clone rust-expert rust-reviewer
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

```sh
ship skill add https://github.com/anthropics/skills --skill tdd
ship skill add rivet-dev/skills
```

### `ship skill list`
List installed skills (project and global).
```
ship skill list
```

### `ship skill remove`
Remove an installed skill.
```
ship skill remove <id> [--global]
```

### `ship skill create`
Scaffold a new skill with a SKILL.md template. IDs: lowercase, digits, hyphens.
```
ship skill create <id> [--name <name>] [--description <desc>]
```
```sh
ship skill create code-review --name "Code Review"
```

## MCP Servers

### `ship mcp serve`
Run the Ship MCP server (stdio by default).
```
ship mcp serve [--http] [--port <n>]
```
- `--http` — serve over HTTP instead of stdio
- `--port <n>` — HTTP port (default: 3000, requires `--http`)

```sh
ship mcp serve --http --port 8080
```

### `ship mcp add`
Register an HTTP/SSE MCP server.
```
ship mcp add <id> --url <url> [--name <name>] [--global]
```
```sh
ship mcp add linear --url https://mcp.linear.app/sse --name "Linear"
```

### `ship mcp add-stdio`
Register a stdio MCP server.
```
ship mcp add-stdio <id> <command> [args...] [--name <name>] [--global]
```
```sh
ship mcp add-stdio github npx -y @modelcontextprotocol/server-github
```

### `ship mcp list`
List configured MCP servers.
```
ship mcp list
```

### `ship mcp remove`
Remove an MCP server by ID.
```
ship mcp remove <id>
```

## Registry

### `ship install`
Resolve and install all dependencies from `.ship/ship.toml`, then compile.
```
ship install [--frozen]
```
- `--frozen` — fail if lockfile would change (CI-safe)
- Requires `[module]` section in `.ship/ship.toml`

```sh
ship install --frozen
```

### `ship add`
Add a package dependency to `.ship/ship.toml` and install it. Restores ship.toml on failure.
```
ship add <package>[@version]
```
Version defaults to `main` if omitted.
```sh
ship add github.com/acme/shared-rules
ship add github.com/acme/tools@^1.0.0
```

### `ship import`
Import a profile from getship.dev, GitHub, or a local path.
```
ship import <source>
```
Sources: `https://getship.dev/p/<id>`, `https://github.com/owner/repo`, local directory.
```sh
ship import https://getship.dev/p/rust-expert
ship import https://github.com/acme/agent-config
```

### `ship export`
Export compiled output for a specific provider.
```
ship export <provider> [--zip]
```
- `--zip` — download all formats as a zip archive

```sh
ship export claude
```

## Auth

### `ship login`
Authenticate with getship.dev via browser-based PKCE OAuth flow.
```
ship login
```

### `ship logout`
Remove the stored auth token.
```
ship logout
```

### `ship whoami`
Show current identity and authentication status.
```
ship whoami
```

## Info

### `ship status`
Show active profile and compilation status.
```
ship status [--path <dir>]
```

### `ship events list`
Query the project event log.
```
ship events list [--since <time>] [--actor <name>] [--entity <type>] [--action <name>] [--limit <n>] [--json]
```
- `--since <time>` — ISO 8601 or relative (`1h`, `24h`)
- `--actor <name>` — filter by actor
- `--entity <type>` — filter by entity type (`workspace`, `session`, `note`)
- `--action <name>` — filter by action (`create`, `update`, `delete`)
- `--limit <n>` — max events (default: 50)
- `--json` — output as JSON array

```sh
ship events list --since 24h --entity workspace --json
```

### `ship surface`
Print the CLI command tree and MCP core tools as markdown.
```
ship surface [--emit] [--check]
```
- `--emit` — write to `docs/surface.md`
- `--check` — diff against `docs/surface.md`; exit 1 on drift

### `ship view`
Browse workflow state in a read-only terminal UI.
```
ship view
```
