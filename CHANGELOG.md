# Changelog

## v0.1.0 — 2026-03-19

First release of the Ship CLI and MCP server.

### What's included

**Compiler**
- `ship use <preset>` — activate a preset: compiles and writes provider config files for Claude, Gemini, Codex, and Cursor simultaneously
- `ship compile` — compile without activating
- `ship validate` — validate `.ship/` config without compiling
- Multi-provider output: `CLAUDE.md`, `GEMINI.md`, `AGENTS.md`, `.mcp.json`, `.cursor/rules/`, `.codex/config.toml`
- Skill files compiled to `.claude/skills/`, `.agents/skills/`, `.gemini/agents/`
- Agent profiles compiled to provider-native agent definition files

**Package manager**
- `ship install` — install preset dependencies declared in `ship.toml`
- Git-native registry: dependencies are git repositories, no central blob store
- Lockfile (`ship.lock`), versioning, transitive dependency resolution
- `ship use @org/preset` — install and activate from a registry

**MCP server (`ship-mcp`)**
- Project intelligence over MCP: workspaces, sessions, jobs, capabilities, targets, skills, notes, ADRs
- Attach to Claude Code or any MCP-compatible client
- Session lifecycle: `start_session`, `log_progress`, `end_session`
- Job queue: `create_job`, `list_jobs`, `update_job`, `append_job_log`

**CLI**
- `ship init` — initialize a project
- `ship status` — show active workspace and session
- `ship workspace` — workspace management
- `ship agent` — agent profile management
- `ship skill` — skill management
- `ship mcp` — MCP server config management
- `ship job` — job queue (create, list, update)
- `ship auth` — authentication

### Platforms
- macOS (arm64, x86_64)
- Linux (x86_64)
- Windows (x86_64)

### Install

```bash
# macOS / Linux — see release assets for your platform
curl -fsSL <release-url>/ship-<platform>.tar.gz | tar -xz
sudo mv ship ship-mcp /usr/local/bin/
```

Both `ship` and `ship-mcp` are required. `ship-mcp` is the MCP server your agent tools connect to.
