+++
id = "6bd1af81-9808-4b24-960e-196cd00255bb"
title = "CLI surface — workflow-first, plumbing hidden"
status = "accepted"
date = "2026-03-01"
tags = []
+++

## Decision

## Context

The current CLI exposes every document type with full CRUD: `ship issue create`, `ship feature list`, `ship spec start`, etc. This is a database admin surface, not a workflow tool. Issues are primarily managed by AI agents via MCP. The human CLI user wants to understand project state, capture context, and manage planning artifacts — not issue CRUD. The CLI is also the setup and config surface.

## Decision

Split the CLI into two tiers: **porcelain** (primary, in `--help`) and **plumbing** (hidden, always functional).

### Porcelain — 9 commands, all human-workflow

```
ship                         Status: current branch, linked feature, in-progress count
ship status                  Same but verbose — providers, skills loaded, rules active
ship log [--n N]             Event stream, last 20 by default
ship note ["text"]           Quick capture. Opens $EDITOR if no text arg.
ship sync                    Regenerate agent configs for current branch
ship providers               Detected providers, config health, installed binaries
ship open                    Launch desktop app

ship new feature "title"     Create feature (planning artifact)
ship new spec "title"        Create spec
ship new release "v1.0.0"   Create release (enforces semver)
ship new adr "title"         Capture an architecture decision
```

### Plumbing — hidden (`#[hide = true]` in Clap), still functional

All existing CRUD subcommands remain: `ship issue create/list/move`, `ship feature list/start/done`, `ship spec list`, `ship adr list`, etc. Used by scripts, CI, and power users who know what they want. Not advertised.

### Rationale

- Issues are an agent tool. Agents use MCP, not CLI. `ship issue create` is plumbing.
- Planning artifacts (features, specs, releases, ADRs) are human-authored. `ship new` is porcelain.
- `ship` alone (no subcommand) shows status — the most common "what's going on" question.
- `ship sync` is the primary human-agent handoff: "set up my branch context."
- `ship note` is the lowest-friction capture surface — faster than opening the UI.
- `ship providers` is discovery: "does Ship know about my tools?"

### Not in CLI (by design)

- Issue CRUD — agent/MCP surface
- Feature/spec/ADR update — UI or MCP (editing markdown files directly is also fine)
- Mode management — UI surface
- MCP server management — edit `agents/mcp.toml` directly, or use UI

## Consequences

### Positive
- `--help` output fits in a terminal without scrolling
- New users understand what Ship does from the CLI alone
- Power users keep full CRUD via hidden commands
- `ship` as a status command is discoverable and memorable

### Negative
- `ship new` is a new command requiring implementation
- Existing `--help` will show fewer commands than before — may confuse users who found CRUD via help
- `ship note` requires `$EDITOR` integration or inline input handling
