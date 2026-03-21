# CLI Reference

The `ship` CLI compiles agent profiles into provider-native config files and manages platform state.

## `ship init`

Scaffold `.ship/` in the current project, or configure `~/.ship/` globally.

```bash
ship init                        # project-local
ship init --global               # configure ~/.ship/
ship init --provider gemini      # set default provider
ship init --force                # overwrite existing .ship/
```

| Flag | Description |
|---|---|
| `--global` | Configure `~/.ship/` instead of the current project |
| `--provider <id>` | Default provider: `claude`, `gemini`, `codex`, `cursor` |
| `--force` | Overwrite existing `.ship/` |

Creates `.ship/ship.toml`, `.ship/.gitignore`, and `.ship/README.md`.

## `ship login` / `ship logout` / `ship whoami`

```bash
ship login       # authenticate with getship.dev
ship logout      # sign out
ship whoami      # show current identity
```

## `ship use <agent-id>`

Activate an agent profile. Compiles immediately, writes provider config files, and installs declared plugins.

```bash
ship use default
ship use rust-compiler
ship use default --path ~/projects/myapp
```

| Flag | Description |
|---|---|
| `--path <dir>` | Bind to this path instead of the current directory |

Finds the profile in `.ship/agents/`, resolves dependencies, compiles to all declared providers, and manages plugin lifecycle.

## `ship status`

Show the active agent and compilation status.

```bash
ship status
ship status --path ~/projects/myapp
```

## `ship compile`

Recompile the active agent without changing it.

```bash
ship compile
ship compile --provider claude   # one provider only
ship compile --dry-run           # preview without writing
```

| Flag | Description |
|---|---|
| `--provider <id>` | Compile for one provider only |
| `--dry-run` | Preview output without writing files |
| `--watch` | Recompile on changes (not yet implemented) |
| `--path <dir>` | Project root |

## `ship agent`

Manage agent profiles in `.ship/agents/` (project) or `~/.ship/agents/` (global).

### `ship agent list`

```bash
ship agent list                  # all agents
ship agent list --local          # global agents only
ship agent list --project        # project agents only
```

### `ship agent create <name>`

```bash
ship agent create my-expert
ship agent create my-expert --global
```

### `ship agent edit <name>`

Open an agent profile in `$EDITOR`.

### `ship agent delete <name>`

Delete an agent profile.

### `ship agent clone <source> <target>`

```bash
ship agent clone rust-compiler my-rust-compiler
```

## `ship skill`

Manage skills. Skills are markdown instruction sets in `.ship/agents/skills/`.

### `ship skill add <source>`

```bash
ship skill add github.com/owner/skill-repo
ship skill add ./my-local-skill --skill my-skill-id
ship skill add github.com/owner/repo --global
```

| Flag | Description |
|---|---|
| `--skill <id>` | Skill ID within the source |
| `--global` | Install to `~/.ship/skills/` |

### `ship skill list`

List installed skills (project + global).

### `ship skill remove <id>`

```bash
ship skill remove my-skill
ship skill remove my-skill --global
```

### `ship skill create <id>`

Scaffold a new skill directory with `SKILL.md`.

```bash
ship skill create code-review
```

## `ship mcp`

Manage MCP server registrations.

### `ship mcp serve`

Run the Ship MCP server.

```bash
ship mcp serve                   # stdio (for .mcp.json)
ship mcp serve --http            # HTTP daemon
ship mcp serve --http --port 4000
```

### `ship mcp add <id> --url <url>`

Register an HTTP/SSE server.

```bash
ship mcp add my-server --url http://localhost:3001/sse
```

### `ship mcp add-stdio <id> <command> [args...]`

Register a stdio server.

```bash
ship mcp add-stdio my-tool /usr/local/bin/my-tool --flag value
```

### `ship mcp list`

List configured MCP servers.

### `ship mcp remove <id>`

Remove an MCP server.

## `ship publish`

Publish the current package to the Ship registry. Requires authentication via `ship login`.

```bash
ship publish
ship publish --dry-run
ship publish --tag beta
```

| Flag | Description |
|---|---|
| `--dry-run` | Preview what would be published without making any network requests |
| `--tag <tag>` | Dist-tag for pre-release publishing (e.g. `beta`, `next`) |

The command reads `.ship/ship.toml` for package metadata, computes a content hash of `.ship/`, and uploads to the registry. Use `--dry-run` to verify metadata before publishing.

## `ship install`

Install all dependencies declared in `.ship/ship.toml`. Resolves versions, writes `ship.lock`, fetches to cache.

```bash
ship install
ship install --frozen            # fail if lockfile would change
```

## `ship add <package>`

Add a dependency to `.ship/ship.toml` and install it.

```bash
ship add github.com/owner/repo
```

## `ship import <source>`

Import an agent from a URL, local path, or provider config directory.

```bash
ship import https://getship.dev/p/<id>
ship import ./my-agent.toml
ship import .cursor/
```

## `ship export <provider>`

Export compiled output for a specific provider.

```bash
ship export claude
ship export gemini --zip
```

| Flag | Description |
|---|---|
| `--zip` | Download all formats as a zip archive |

## `ship validate`

Validate `.ship/` config before compile. Checks TOML syntax, skill references, MCP fields, permissions.

```bash
ship validate
ship validate --profile rust-compiler
ship validate --json
```

| Flag | Description |
|---|---|
| `--profile <id>` | Validate a single profile |
| `--json` | Machine-readable output |
| `--path <dir>` | Project root |

Exit code 0 = valid. Exit code 1 = errors.

## `ship events list`

Query the project event log.

```bash
ship events list
ship events list --since 24h
ship events list --entity workspace --action create
ship events list --limit 100 --json
```

| Flag | Description |
|---|---|
| `--since <time>` | ISO 8601 or relative: `1h`, `24h`, `7d` |
| `--actor <name>` | Filter by actor |
| `--entity <type>` | `workspace`, `session`, `note`, `adr`, `job`, etc. |
| `--action <name>` | `create`, `update`, `delete`, `start`, `stop`, etc. |
| `--limit <n>` | Max events (default 50) |
| `--json` | Output as JSON array |

## `ship surface`

Print the CLI and MCP surface as markdown. Used by CI to detect surface drift.

```bash
ship surface                     # print to stdout
ship surface --emit              # write to docs/surface.md
ship surface --check             # diff; exit 1 if drift
```

## Configuration files

| Path | Purpose |
|---|---|
| `.ship/ship.toml` | Project manifest: module identity, deps, exports |
| `.ship/agents/*.toml` | Agent profiles |
| `.ship/agents/skills/` | Installed skills |
| `.ship/agents/mcp.toml` | MCP server declarations |
| `.ship/agents/permissions.toml` | Permission presets |
| `~/.ship/config.toml` | Global identity and defaults |
| `~/.ship/agents/` | Global agent profiles |

Compiled artifacts (`CLAUDE.md`, `.mcp.json`, `.cursor/`, etc.) are gitignored — generated by `ship use`, never committed.
