---
name: ship-cli-reference
stable-id: ship-cli-reference
description: Use when working with Ship CLI commands — init, use, compile, agents, skills, mcp, vars, validate. Complete command reference with flags and examples.
tags: [ship, cli, reference]
authors: [ship]
---

# Ship CLI Reference

Ship compiles declarative agent configuration (`.ship/`) into provider-native files. The workflow is: `init` a project, `use` an agent, `compile` to emit outputs.

For full details see `references/docs/`.

## Core Workflow

```
ship init                         # scaffold .ship/ in current project
ship use <agent-id>               # activate agent and compile
ship compile                      # recompile after config changes
ship status                       # show active agent and compilation state
ship validate                     # check config for errors
```

## Agent Management

```
ship agents list [--local] [--project]
ship agents create <name> [--global]
ship agents edit <name>
ship agents clone <source> <target>
ship agents delete <name>
```

## Skills

```
ship skills list
ship skills create <id>
ship skills add <source> [--skill <id>] [--global]
ship skills remove <id> [--global]
```

## MCP Servers

```
ship mcp serve [--http] [--port <n>]
ship mcp add <id> --url <url>
ship mcp add-stdio <id> <command> [args...]
ship mcp list
ship mcp remove <id>
```

## Variables

```
ship vars get <skill-id> [key]
ship vars set <skill-id> <key> <value>
ship vars append <skill-id> <key> '<json-value>'
ship vars reset <skill-id>
```

## Registry, Auth, and Other Commands

```
ship install [--frozen]                  # install all deps from ship.toml
ship add <package>[@version]             # add and install a dependency
ship publish [--dry-run] [--tag]         # publish to registry
ship login / logout / whoami             # authentication
ship convert <source>                    # convert provider configs to .ship/
ship events list [--since] [--json]      # query event log
ship docs <topic>                        # extended help
ship view                                # terminal UI
```

## Provider Outputs

All outputs are gitignored build artifacts. See `references/docs/providers.md`.

| Provider | Context | MCP config | Skills |
|----------|---------|------------|--------|
| `claude` | `CLAUDE.md` | `.mcp.json` | `.claude/skills/` |
| `gemini` | `GEMINI.md` | `.gemini/settings.json` | `.gemini/skills/` |
| `codex` | `AGENTS.md` | `.codex/config.toml` | `.agents/skills/` |
| `cursor` | `.cursor/rules/*.mdc` | `.cursor/mcp.json` | `.cursor/skills/` |
