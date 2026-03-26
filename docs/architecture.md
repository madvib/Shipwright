# Architecture

Ship is structured in three layers: the **compiler** (pure transformation), the **runtime** (state management), and **transport** (CLI, MCP, web). Domain logic lives in the compiler and runtime crates. Transport layers are thin.

## Three-layer model

```
┌──────────────────────────────────────────────────┐
│                    Transport                      │
│  CLI (ship-studio-cli)  MCP (ship-mcp)  Web (Studio) │
│  Parses args, routes    Exposes tools    Browser UI   │
│  to runtime/compiler    via rmcp         + WASM       │
└────────────────┬────────────┬────────────┬────────┘
                 │            │            │
┌────────────────▼────────────▼────────────┘
│                    Runtime                        │
│  Workspace lifecycle, sessions, events, agents    │
│  SQLite state (platform.db), migrations           │
│  File ownership, job coordination, plugin mgmt    │
└────────────────┬──────────────────────────────────┘
                 │
┌────────────────▼──────────────────────────────────┐
│                    Compiler                        │
│  ProjectLibrary → ResolvedConfig → CompileOutput  │
│  Pure function: files in, provider config out     │
│  WASM target: same logic in CLI and browser       │
└───────────────────────────────────────────────────┘
```

## Compilation pipeline

The compiler is a three-stage pipeline:

### 1. ProjectLibrary — load

Reads `.ship/` into memory: `ship.toml`, agent profiles, skills, MCP server declarations, permission presets, rules. No filesystem access after this step.

### 2. ResolvedConfig — resolve

Takes the loaded library plus the active agent ID. Merges:
- Project defaults (providers, MCP servers)
- Agent profile (skills, permissions, rules, plugins)
- Feature branch overrides (workspace-specific config)

Output is a fully resolved, self-contained configuration with no unresolved references.

### 3. CompileOutput — compile

Takes `ResolvedConfig` and a target provider. Emits:

| Output | Description |
|---|---|
| `context_content` | Context file body (`CLAUDE.md`, `GEMINI.md`, `AGENTS.md`) |
| `mcp_servers` | MCP server entries as JSON |
| `mcp_config_path` | Where to write MCP config (`.mcp.json`, `.cursor/mcp.json`, etc.) |
| `skill_files` | Skill content mapped to provider-native paths |
| `claude_settings_patch` | Claude-specific settings (permissions, hooks) |
| `codex_config_patch` | Codex-specific TOML config |
| `plugins_manifest` | Plugin install/uninstall instructions |

The compiler emits strings ready to write to disk. It never touches the filesystem.

## WASM compilation target

The compiler crate compiles to both native (for the CLI and MCP server) and WASM (for Ship Studio in the browser). The WASM entry point is `compile_library()`, which takes a JSON-serialized `ProjectLibrary` and returns compiled output.

This means:
- The CLI, MCP server, and web UI all use the same compilation logic
- No server round-trip needed for compilation in the browser
- The `@ship/compiler` npm package wraps the WASM output

## Runtime — state management

The runtime crate owns all persistent state in `~/.ship/platform.db` (SQLite via sqlx):

- **Workspaces** — units of parallel work, mapped to git branches/worktrees
- **Sessions** — one agent visit to a workspace (start, work, end with summary)
- **Jobs** — cross-agent coordination (pending → running → complete/failed)
- **Events** — append-only audit log of all state changes
- **Targets / Capabilities** — milestone and feature tracking
- **File claims** — ownership tracking to prevent concurrent modification

The runtime is the only layer that talks to SQLite. Transport layers call runtime functions.

## Transport — thin layers

### CLI (`apps/ship-studio-cli/`)

Parses commands via clap, delegates to runtime and compiler. The CLI never contains domain logic — it maps flags to function calls and formats output.

Key commands: `init`, `use`, `compile`, `status`, `agents`, `skills`, `mcp`, `validate`, `convert`, `events`, `view` (TUI).

### MCP server (`apps/mcp/`)

Exposes runtime operations as MCP tools via the rmcp library. Supports stdio (for `.mcp.json`) and HTTP transports. Core tools: workspace management, session lifecycle, job coordination, skill listing, event queries.

### Web — Ship Studio (`apps/web/`)

TanStack Start application on Cloudflare Workers. Imports `@ship/compiler` (WASM) for in-browser compilation. Allows importing agent config from GitHub URLs.

## Repository layout

```
apps/
  ship-studio-cli/       CLI binary — ship init, ship use, ship compile, ...
  mcp/                   MCP stdio/HTTP server
  web/                   Ship Studio (TanStack Start + Cloudflare Workers)
crates/core/
  compiler/              WASM compiler — types, resolution, output generation
  runtime/               State management — workspaces, sessions, events, DB
  cli-framework/         Shared CLI metadata and core command scaffolding
  mcp-framework/         Shared MCP app lifecycle scaffolding
packages/
  compiler/              @ship/compiler — WASM npm package
  primitives/            @ship/primitives — shared UI components
  ui/                    @ship/ui — web component library
  assets/                Shared static assets
```

## Design constraints

**Transport thin, domain in runtime.** CLI and MCP are dispatchers, not decision-makers. If logic needs to coordinate state, it belongs in the runtime crate.

**Compiler is pure.** No filesystem, no network, no database. `ProjectLibrary` → `CompileOutput`. This makes it safe to run in WASM and easy to test.

**300-line file cap.** Modules are split before they exceed 300 lines.

**Idempotent by default.** `ship use` can be run repeatedly. The compiler overwrites artifacts. The runtime uses upsert patterns.

**Events are append-only.** Every state change emits an event. Events are never updated or deleted.
