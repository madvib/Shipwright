# Ship

**Project memory and execution infrastructure for AI-assisted software teams.**

Every agent session starts blank. Ship fixes that.

Ship is a local-first project OS that persists your team's context — decisions, features, specs, open work — and injects exactly the right context into every AI agent, on every branch, for every provider. Claude, Gemini, Codex: each gets its native config format, automatically, on checkout.

---

## The Problem

AI coding agents are powerful but amnesiac. Every session starts from scratch. Teams compensate by pasting context into every prompt, keeping mental models of what the agent already "knows," and watching agents repeat the same mistakes across sessions.

The underlying issue is structural: there's no persistent, structured project memory that agents can actually read — and no system to keep that memory current as work moves forward.

---

## What Ship Does

Ship sits in your repository as a `.ship/` directory. It stores your project's working memory as structured markdown files with TOML frontmatter, versioned in git alongside your code. A git hook fires on every branch checkout and writes the right context files for your active agents — `CLAUDE.md`, `GEMINI.md`, `AGENTS.md` — each populated with the current feature spec, open issues, applicable skills, and always-on rules.

**The workflow loop:**

```
Vision → Release → Feature → Spec → Issues → Close Feature → Ship Release
```

At each transition, Ship knows where you are and what your agents need to know. Notes and ADRs exist outside the loop — ambient records created whenever a decision or insight surfaces, never blocking progress.

---

## How It Works

### Branch checkout triggers context injection

```
git checkout feature/payments-v2
→ Ship reads the feature document linked to this branch
→ Writes CLAUDE.md with: feature spec + open issues + inlined skills + rules
→ Writes .mcp.json with the servers declared for this feature
→ Writes GEMINI.md / AGENTS.md for other connected providers
→ Agent opens the project and immediately understands what's in scope
```

No prompts. No copy-paste. The agent has context before you type the first message.

### Structured documents, not raw notes

Every entity has typed frontmatter with stable UUIDs for cross-linking:

```toml
# .ship/project/features/payments-v2.md
id = "f3a7c291"
title = "Payments v2 — Stripe Connect"
status = "in-progress"
release_id = "8b2d4e10"
spec_id   = "c9f1a033"
branch    = "feature/payments-v2"

[agent]
skills     = [{id = "payment-compliance"}]
mcp_servers = [{id = "stripe-docs"}]
```

### Multi-provider, native formats

Ship knows how each agent tool works. It writes config in the format each provider actually reads:

| Provider     | Context file | MCP config                  | Skills                        |
| ------------ | ------------ | --------------------------- | ----------------------------- |
| Claude Code  | `CLAUDE.md`  | `.mcp.json` (JSON)          | `.claude/skills/<id>/SKILL.md`  |
| Gemini CLI   | `GEMINI.md`  | `.gemini/settings.json`     | `.gemini/skills/<id>/SKILL.md`  |
| OpenAI Codex | `AGENTS.md`  | `.codex/config.toml` (TOML) | `.agents/skills/<id>/SKILL.md`  |

MCP sync contract (import/export paths, guardrails, precedence): `docs/mcp-import-export.md`
CLI/MCP binary surfaces + PATH install/update workflow: `docs/cli-mcp-offerings.md`

Add a provider in one command. Ship handles the rest:

```bash
ship providers connect gemini
# → gemini added to ship.toml
# → next branch checkout writes GEMINI.md automatically
```

### MCP server — agents as first-class consumers

Ship runs as an MCP server, giving agents structured read/write access to the entire project state: issues, specs, features, releases, ADRs, skills, providers, events. Agents don't need file access — they use typed tools.

```bash
ship mcp # stdio transport, works with any MCP-compatible agent
```

Forty-plus tools including `get_project_info` (full context in one call), `create_issue`, `move_issue`, `connect_provider`, `list_providers_tool`, `git_feature_sync`, and more.

### Skills and rules

Reusable agent instructions, scoped to project or user:

```markdown
# .ship/agents/skills/task-policy/SKILL.md

---
name: task-policy
description: Ship workflow policy and execution guardrails for daily delivery.
metadata:
  display_name: Ship Workflow Policy
  source: builtin
---

Always start from a feature document. File issues for every gap found.
Run tests before closing feature todos.
```

User-scoped shared skills live in `~/.ship/skills/<id>/SKILL.md`.

Rules in `agents/rules/*.md` are always-on — inlined into every provider's context file on every checkout.

Rules contract (naming, mode matching, validation): `docs/agent-rules-contract.md`

---

## Quick Start

```bash
# Install (requires Rust)
cargo install --path crates/cli

# Initialize in your repo
ship init
# → detects installed providers (Claude, Gemini, Codex) automatically
# → installs git hooks
# → creates .ship/ structure

# Create a feature
ship feature create "User authentication"

# Create and move issues
ship issue create "Implement JWT refresh" "Access tokens expire after 15min..."
ship issue move jwt-refresh.md backlog in-progress

# See what's connected
ship providers list
# ID           NAME                 INSTALLED  CONNECTED  VERSION
# claude       Claude Code          yes        yes        2.1.63
# gemini       Gemini CLI           yes        no         0.23.0
# codex        Codex CLI            yes        no         -

ship providers connect gemini

# Manually sync agent context for current branch
ship git sync
```

---

## Project Structure

```
.ship/
├── ship.toml                 # project config, providers, git policy
├── project/
│   ├── features/             # feature documents (committed)
│   ├── specs/                # spec documents (committed)
│   ├── releases/             # release documents (committed)
│   ├── adrs/                 # architecture decisions (committed)
│   │   └── accepted/
│   └── vision.md
└── agents/
    ├── skills/               # reusable agent instructions
    ├── rules/                # always-on rules, inlined into every context
    └── modes/                # named agent configurations
```

**Git policy** — Ship defaults to a config-first posture: `ship.toml`, MCP config, permissions, and rules are tracked; project docs, skills, and templates are local unless explicitly included.

---

## Architecture

Ship is a Rust monorepo:

| Crate                | Role                                                         |
| -------------------- | ------------------------------------------------------------ |
| `core/runtime`       | Core data model, CRUD, event stream, agent config resolution |
| `crates/cli`         | `ship` binary — workflow CLI                                 |
| `crates/mcp`         | `ship-mcp` binary — MCP stdio server                         |
| `crates/modules/git` | Git hook handler, context file generation                    |
| `crates/ui`          | Tauri + React desktop app (macOS/Windows)                    |
| `crates/plugins/*`   | Time tracker, ghost issue scanner                            |

- **Storage:** Structured markdown with TOML frontmatter (git-native) + SQLite for workspace state and managed MCP ledger
- **Events:** Append-only event stream — the replication unit for future cloud sync
- **Agent config resolution:** Project defaults → active mode → feature-level overrides, consistent across CLI, MCP, and Tauri
- **Transport:** MCP over stdio today; HTTP/SSE scoped for post-alpha

---

## Status

Ship is in **alpha**. It is used to build itself — this repo runs Ship on every branch. The core loop is functional end-to-end.

**Working now:**

- Full CRUD for features, specs, issues, releases, ADRs, notes, skills, rules
- Git hook → context injection for Claude, Gemini, Codex
- MCP server with 40+ tools
- Provider detection, connect/disconnect, model registry
- SQLite workspace state with branch-scoped context
- 180+ passing tests including end-to-end branch lifecycle tests

**Coming next:**

- Desktop UI (macOS/Windows) — Tauri + React, designs in progress
- `ship providers` HTTP/SSE transport for editor integrations (Cursor, Windsurf)
- Cloud sync via event log replication
- CLI porcelain surface (`ship status`, `ship log`, `ship new`)

---

## Why Now

The MCP protocol standardized how agents consume external context. Every major AI coding tool — Claude Code, Gemini CLI, Codex, Cursor, Windsurf, Zed — now supports it. The tooling layer has arrived. What's missing is the project memory layer that sits above it: structured, versioned, agent-readable, and wired into the developer's actual workflow.

Ship is that layer.

---

## Contributing / Interest

This project is in active development. If you're building with AI agents at scale, working on developer tooling, or interested in the future of software engineering workflows, we'd like to talk.

Open issues, file bugs, and follow development here. The `.ship/` directory in this repo is live — the same workflow described above is what we use every day.

---

_Built with Ship · Rust · MCP · Local-first_
