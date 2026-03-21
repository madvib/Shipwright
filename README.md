# Ship

Compiler and package manager for AI agent configuration.

Ship compiles a single `.ship/` source directory into provider-native config files for Claude Code, Gemini CLI, OpenAI Codex, and Cursor. One command — `ship use <agent>` — writes context files, installs skills, configures MCP servers, and manages plugins.

## Provider output matrix

| Provider | Context file | MCP config | Skills | Plugins |
|---|---|---|---|---|
| Claude Code | `CLAUDE.md` | `.mcp.json` | `.claude/skills/` | `claude plugin install` |
| Gemini CLI | `GEMINI.md` | `.gemini/settings.json` | `.agents/skills/` | — |
| OpenAI Codex | `AGENTS.md` | `.codex/config.toml` | `.agents/skills/` | — |
| Cursor | — | `.cursor/mcp.json` | `.cursor/rules/` | — |

`.ship/` is the source of truth. Provider files are generated artifacts — gitignored, never hand-edited.

## Quick start

```bash
ship init                  # scaffold .ship/ in the current project
ship use default           # activate the default agent → writes provider files
ship status                # show active agent and compile state
```

`ship init` creates `.ship/ship.toml` and a `.gitignore`. `ship use` finds the agent profile in `.ship/agents/`, compiles it, writes all provider config files, and installs any declared plugins.

See [Getting Started](docs/getting-started.md) for the full walkthrough.

## Agent profiles

An agent profile is a TOML file in `.ship/agents/` that declares everything an agent needs:

```toml
[agent]
id = "rust-expert"
name = "Rust Expert"
version = "0.1.0"
providers = ["claude", "gemini"]

[skills]
refs = ["ship-coordination"]

[mcp]
servers = ["ship"]

[plugins]
install = [
  "superpowers@claude-plugins-official",
  "rust-analyzer-lsp@claude-plugins-official",
]
scope = "project"

[permissions]
preset = "ship-autonomous"
tools_deny = ["Bash(git push --force*)"]

[rules]
inline = """
Your domain is Rust backend code.
Run cargo test before marking work done.
"""
```

`ship use rust-expert` resolves skills, configures MCP servers, applies permissions, compiles to every declared provider, and installs plugins.

## How compilation works

```
.ship/                           Compiled output
├── ship.toml          ──┐
├── agents/            ──┤       CLAUDE.md
│   ├── default.toml     │       GEMINI.md
│   ├── rust-expert.toml ├──→    AGENTS.md
│   ├── permissions.toml │       .mcp.json
│   ├── mcp.toml         │       .claude/skills/
│   └── skills/        ──┘       .cursor/rules/
└── ship.lock
```

The compiler reads agent profiles, resolves skill references and MCP servers, merges permission presets, and emits provider-native files. The compiler is built as WASM — the same compilation logic runs in the CLI (native), the MCP server, and Ship Studio (browser).

## Permission presets

Presets control what tools an agent can use. Four tiers, strict to loose:

| Preset | Use case | Default mode |
|---|---|---|
| `ship-readonly` | Reviewers, auditors, tutors | `plan` |
| `ship-standard` | Interactive sessions, paired work | `default` |
| `ship-autonomous` | Specialist agents in worktrees | `dontAsk` |
| `ship-elevated` | Deploy and release agents | `dontAsk` |

Define custom presets in `.ship/agents/permissions.toml`. Override per-agent with `tools_allow` / `tools_deny` in the profile.

## Registry

Ship uses a git-native package model. Dependencies point to GitHub repos:

```toml
# .ship/ship.toml
[module]
name = "github.com/owner/repo"
version = "0.1.0"

[dependencies]
"github.com/garrytan/gstack" = "main"

[exports]
skills = ["agents/skills/configure-agent"]
agents = ["agents/profiles/default.toml"]
```

`ship install` resolves dependencies, writes `ship.lock`, and caches packages locally. `ship add github.com/owner/repo` adds a dependency and installs it.

## Project layout

```
.ship/
├── ship.toml                 # project manifest: module identity, deps, exports
├── ship.lock                 # pinned dependency versions
└── agents/
    ├── *.toml                # agent profiles
    ├── permissions.toml      # permission presets
    ├── mcp.toml              # MCP server declarations
    ├── rules/                # shared rule files
    ├── skills/               # installed skills (SKILL.md per directory)
    └── teams/                # team coordination config
```

## Architecture

```
apps/
  ship-studio-cli/   CLI binary (ship init, ship use, ship compile, ...)
  mcp/               MCP stdio server (workspace, session, job, skill tools)
  web/               Ship Studio (TanStack Start + Cloudflare Workers)
crates/core/
  compiler/          WASM compiler: ProjectLibrary → ResolvedConfig → CompileOutput
  runtime/           State management: workspaces, sessions, events, agents (SQLite)
  cli-framework/     Shared CLI scaffolding
  mcp-framework/     Shared MCP scaffolding
packages/
  compiler/          @ship/compiler — WASM output consumed by Studio
  primitives/        @ship/primitives — shared UI components
```

The compiler is a pure function: project files in, provider config out. The runtime manages persistent state in `~/.ship/platform.db` (SQLite). CLI and MCP are thin transport layers — domain logic lives in the runtime crate.

## Documentation

- [Getting Started](docs/getting-started.md) — install, init, create an agent, compile, verify
- [CLI Reference](docs/cli.md) — every command with flags and examples
- [Schema Reference](docs/schema.md) — ship.toml, agent profiles, permissions, database entities
- [Architecture](docs/architecture.md) — compiler pipeline, runtime model, repo layout

## Status

v0.1.0 — used to build itself. 312 runtime tests, 6 CLI tests, 3 MCP tests passing.

**Working:**
- WASM compiler: `.ship/` → CLAUDE.md, GEMINI.md, AGENTS.md, .mcp.json, skills, plugins
- `ship init`, `ship use`, `ship compile`, `ship status`, `ship validate`
- Agent management: `ship agent list/create/edit/delete/clone`
- Skill management: `ship skill add/list/remove/create`
- MCP server with 24 core tools (workspace, session, job, skill, event coordination)
- Permission presets with 4-tier continuum
- Event log, session tracking, file ownership claims
- TUI dashboard (`ship view`)

**Not yet implemented:**
- Post-checkout hook for automatic agent switching (run `ship use` manually after branch switch)
- `ship compile --watch`
- Ship Studio import UI and auth

## License

MIT
