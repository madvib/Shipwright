---
group: CLI
title: Command Reference
description: Complete reference for every Ship CLI command with flags, arguments, and examples.
section: reference
order: 2
---

# Command Reference

## Project Setup

### `ship init`

Scaffold `.ship/` in the current project, or configure `~/.ship/` globally.

| Flag | Description |
|------|-------------|
| `--from <url>` | Fetch a JSON config bundle and scaffold from it |
| `--global` | Configure `~/.ship/` instead of current project |
| `--provider <id>` | Default provider: `claude`, `gemini`, `codex`, `cursor` |
| `--force` | Overwrite existing `.ship/` |

```bash
ship init                        # scaffold with claude default
ship init --provider gemini      # scaffold with gemini default
ship init --global               # configure ~/.ship/ identity
```

### `ship use <agent-id>`

Activate an agent and compile immediately. Resolves skill references, MCP servers, and permissions, then writes provider-native config files. Flag: `--path <dir>` to bind to another path.

### `ship status`

Show active agent and compilation status. Flag: `--path <dir>`.

### `ship compile`

Compile the active agent to provider-native config files.

| Flag | Description |
|------|-------------|
| `--provider <id>` | Compile for a specific provider only |
| `--dry-run` | Preview without writing files |
| `--path <dir>` | Project root |

### `ship validate`

Validate `.ship/` config -- JSONC syntax, skill refs, MCP fields, permissions.

| Flag | Description |
|------|-------------|
| `--agent <id>` | Validate a single agent |
| `--json` | Emit errors as JSON |
| `--path <dir>` | Project root |

### `ship convert <source>`

Convert provider config files (CLAUDE.md, .cursor/) into `.ship/` format.

## Agent Management

| Command | Description | Flags |
|---------|-------------|-------|
| `ship agents list` | List available agent profiles | `--local`, `--project` |
| `ship agents create <name>` | Create a new agent profile (lowercase + hyphens) | `--global` |
| `ship agents edit <name>` | Open agent in `$EDITOR` | |
| `ship agents clone <src> <dst>` | Copy full profile under a new ID | |
| `ship agents delete <name>` | Delete an agent profile | |

## Skills

| Command | Description | Flags |
|---------|-------------|-------|
| `ship skills add <source>` | Install from GitHub URL, `owner/repo`, or registry | `--skill <id>`, `--global` |
| `ship skills list` | List all installed skills | |
| `ship skills remove <id>` | Remove an installed skill | `--global` |
| `ship skills create <id>` | Scaffold a new skill (lowercase, digits, hyphens) | |

`--skill <id>` is required when a repo contains multiple skills.

## Variables

| Command | Description |
|---------|-------------|
| `ship vars get <skill-id> [key]` | Read merged state for a skill. Omit key for all variables. |
| `ship vars set <skill-id> <key> <value>` | Set a variable. Routes to correct scope via `storage-hint`. Validates type/enum. |
| `ship vars append <skill-id> <key> '<json>'` | Append an element to an array variable. |
| `ship vars reset <skill-id>` | Reset all state to defaults from `vars.json`. |

## MCP Servers

| Command | Description |
|---------|-------------|
| `ship mcp serve [--http] [--port <n>]` | Run the Ship MCP server (stdio default, or HTTP) |
| `ship mcp add <id> --url <url>` | Register an HTTP/SSE MCP server |
| `ship mcp add-stdio <id> <cmd> [args...]` | Register a stdio MCP server |
| `ship mcp list` | List configured MCP servers |
| `ship mcp remove <id>` | Remove a configured MCP server |

```bash
ship mcp add-stdio ship ship mcp serve
ship mcp add my-api --url http://localhost:8080/mcp
```

## Registry and Dependencies

### `ship install`

Resolve and install all dependencies from `.ship/ship.toml`, then compile.

| Flag | Description |
|------|-------------|
| `--frozen` | Fail if lockfile would change (CI-safe) |

### `ship add <package>`

Add a dependency to `.ship/ship.toml` and install it. Version defaults to `main` if omitted.

```bash
ship add some-package@0.2.0
```

### `ship publish`

Publish the current package to the Ship registry. Requires `ship login`.

| Flag | Description |
|------|-------------|
| `--dry-run` | Preview without network |
| `--tag <tag>` | Dist-tag for pre-release |

## Authentication

### `ship login` / `ship logout` / `ship whoami`

Authenticate with getship.dev via PKCE OAuth, remove stored token, or show current identity.

```bash
ship login       # PKCE OAuth via browser
ship logout      # remove stored token
ship whoami      # show current identity
```

## Information and Diagnostics

### `ship events list`

Query the project event log.

| Flag | Description |
|------|-------------|
| `--since <time>` | Since timestamp or relative (`1h`, `24h`, `7d`) |
| `--actor <name>` | Filter by actor |
| `--entity <type>` | Filter by entity type |
| `--action <name>` | Filter by action |
| `--limit <n>` | Max events (default 50) |
| `--json` | Output as JSON |

### `ship docs [topic]`

Show detailed help. Topics: `agents`, `skills`, `mcp`, `providers`, `compile`, `workflow`.

```bash
ship docs topics    # list all topics
ship docs agents    # agent management help
```

### `ship view`

Browse and manage project state in the terminal UI.

### `ship surface`

Print the CLI command tree and MCP core tools as markdown.

| Flag | Description |
|------|-------------|
| `--emit` | Write to `docs/surface.md` |
| `--check` | Fail if `docs/surface.md` is out of date |
