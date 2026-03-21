# Ship

**Compiler and package manager for AI agent configuration.**

`ship use <preset>` — one command activates your agent stack: writes context files, installs skills, configures MCP servers, and manages Claude Code plugins. Works with Claude, Gemini, Codex, and Cursor. Switching git branches switches your agent config automatically.

---

## What it does

Your agent tools each read config from different places in different formats. Ship is the single source of truth that compiles to all of them:

| Provider | Context | MCP | Skills | Plugins |
|---|---|---|---|---|
| Claude Code | `CLAUDE.md` | `.mcp.json` | `.claude/skills/` | `claude plugin install` |
| Gemini CLI | `GEMINI.md` | `.gemini/settings.json` | `.agents/skills/` | — |
| OpenAI Codex | `AGENTS.md` | `.codex/config.toml` | `.agents/skills/` | — |
| Cursor | — | `.cursor/mcp.json` | `.cursor/rules/` | — |

Your `.ship/` directory is the source. Provider files are generated artifacts — gitignored, never committed. `ship use` produces them on demand.

---

## Quick start

```bash
# Fresh machine — handles Rust, ship binary, Node, pnpm, plugins
bash scripts/setup.sh

# In your project
ship init
ship use default
# → CLAUDE.md, .mcp.json, .claude/skills/ written
# → Claude Code plugins declared in preset installed
```

---

## Preset format

A preset is what you activate. It declares everything your agent stack needs:

```toml
[preset]
id = "rust-expert"
name = "Rust Expert"
version = "0.1.0"
providers = ["claude", "gemini"]

[skills]
refs = ["rust-idioms", "cargo-workflow"]

[mcp]
servers = ["github", "rust-docs"]

[plugins]
install = [
  "superpowers@claude-plugins-official",
  "rust-analyzer-lsp@claude-plugins-official",
]
scope = "project"

[permissions]
preset = "ship-guarded"
default_mode = "plan"
```

`ship use rust-expert` installs skills, configures MCP, installs plugins, emits all provider files.

---

## Branch-aware config

Ship tracks which preset is active per branch in a local SQLite DB (`~/.ship/platform.db`).

```bash
git checkout feature/payments    # post-checkout hook fires (planned)
# → ship looks up stored preset for this branch
# → runs ship use <preset> silently
# → your agent stack switches without any manual steps
```

The post-checkout hook is planned — `ship init` will install it automatically. Until then, run `ship use <preset>` manually after switching branches.

---

## Registry

Ship uses a git-native package model. Skills and presets are published from git repos, not a central blob store:

```toml
# .ship/ship.toml
[module]
name = "github.com/owner/repo"
version = "0.1.0"

[dependencies]
# "github.com/org/skill-pack" = "v1.0.0"

[exports]
skills = ["agents/skills/my-skill"]
agents = ["agents/profiles/default.toml"]
```

`ship install` resolves dependencies, writes `ship.lock`, and fetches to `~/.ship/cache/`.

---

## Distribution

Ship participates in the [agentskills.io](https://agentskills.io) open standard. Skills emitted to `.agents/skills/` are automatically readable by all compliant providers — no per-marketplace submissions.

The viral loop: anyone can paste a GitHub URL into Ship Studio, extract the repo's agent config, compile it for their stack. For project owners, `ship use` + a PR that adds `.ship/` means every collaborator gets your agent config on checkout.

---

## Architecture

```
apps/
  web/         — Ship Studio (TanStack Start + Cloudflare Workers) — active
  mcp/         — MCP server library (served via `ship mcp serve`) — active
crates/
  core/
    compiler/  — WASM compiler: ProjectLibrary → provider files
    runtime/   — workspace, session, event, preset, skill data model
packages/
  compiler/    — @ship/compiler WASM output (consumed by Studio)
  primitives/  — @ship/primitives shared UI components
```

The compiler is WASM — runs in the browser (Studio) and on the server (CLI via native). Same compilation logic everywhere.

---

## Repo layout for contributors

```
ARCHITECTURE.md  — platform principles, layer separation, naming conventions
REFERENCE.md     — provider matrix, CLI commands, MCP tools, schemas
scripts/setup.sh — fresh machine setup (run this first)
```

---

## Status

Early. Used to build itself. The compiler and Studio UI work end-to-end.

**Working:**
- WASM compiler: ProjectLibrary → CLAUDE.md / GEMINI.md / AGENTS.md / .mcp.json
- Ship Studio: paste any GitHub URL → extract agent config → compile → download
- `ship mcp serve`: workspace, session, skill, note, job coordination tools

**In progress:**
- `ship init` + `ship use` (CLI)
- Better Auth + GitHub OAuth + `/api/github/import`
- Studio import UI + auth
- Branch-preset tracking with post-checkout hook
- Plugin lifecycle management via `ship use`
- Registry: `ship install` dep resolution from git sources
