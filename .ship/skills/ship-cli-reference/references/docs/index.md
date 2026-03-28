---
group: CLI
title: CLI Overview
description: Ship CLI command reference — manage agents, skills, MCP servers, variables, and the registry.
section: guide
order: 1
---

# CLI Overview

The `ship` command manages your agents, skills, and projects from the terminal. Most tasks can also be done through Studio or by asking your agent directly.

## Installation

```bash
curl -fsSL https://getship.dev/install | sh
```
- Git

## Installation

Clone the repository and install from source:

```bash
git clone https://github.com/madvib/ship.git
cd ship
cargo install --path apps/ship-studio-cli
```

Verify the installation:

```bash
ship --version
```

## Basic Workflow

The three-step workflow is: **init**, **use**, **compile**.

### 1. Initialize the project

```bash
cd /path/to/your-project
ship init
```

This scaffolds the `.ship/` directory:

```
.ship/
  ship.jsonc       # project manifest
  agents/          # agent profiles
  skills/          # skill definitions
  rules/           # shared rule files
  .gitignore       # ignores compiled artifacts
```

To use a different default provider:

```bash
ship init --provider gemini
```

### 2. Create and activate an agent

```bash
ship agents create my-agent
ship use my-agent
```

`ship use` reads the agent profile, resolves skill references, MCP servers, and permissions, then compiles provider-native config files into the project root.

### 3. Recompile after changes

Edit your agent profile or skills, then recompile:

```bash
ship compile
```

Preview without writing files:

```bash
ship compile --dry-run
```

Target a single provider:

```bash
ship compile --provider gemini
```

## Project Structure

After setup, the `.ship/` directory contains:

```
.ship/
  ship.jsonc               # project manifest
  permissions.jsonc         # permission presets
  mcp.jsonc                # MCP server definitions
  agents/
    my-agent.jsonc         # agent profile
  skills/
    code-review/
      SKILL.md             # skill definition
  rules/                   # shared rule files (optional)
```

Compiled output (gitignored, at project root):

```
CLAUDE.md                  # claude provider
GEMINI.md                  # gemini provider
AGENTS.md                  # codex provider
.mcp.json                  # claude MCP config
.gemini/                   # gemini config + skills
.codex/                    # codex config
.claude/skills/            # claude native skills
.cursor/rules/             # cursor rule files
.cursor/mcp.json           # cursor MCP config
.agents/skills/            # codex native skills
```

## Validation

Check the entire `.ship/` configuration for errors before committing:

```bash
ship validate
```

Validate a single agent:

```bash
ship validate --agent my-agent
```

Emit errors as JSON for tooling:

```bash
ship validate --json
```

## Getting Help

```bash
ship docs topics                 # list available help topics
ship docs agents                 # agent management
ship docs skills                 # skill authoring
ship docs mcp                    # MCP server configuration
ship docs providers              # supported providers
ship docs compile                # compilation pipeline
ship docs workflow               # day-to-day workflow
```
