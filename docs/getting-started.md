# Getting Started

This guide walks you from zero to a working agent configuration in under 10 minutes. No external services required.

## Prerequisites

- Rust toolchain (`rustup` + stable)
- Git

## Install Ship

```bash
cargo install --path apps/ship-studio-cli
```

Verify:

```bash
ship --version
```

## Initialize a project

In your project root:

```bash
ship init
```

This creates:

```
.ship/
├── ship.toml       # project manifest
├── .gitignore      # ignores compiled artifacts (CLAUDE.md, .mcp.json, etc.)
└── README.md       # brief description for collaborators
```

The default provider is `claude`. To use a different default:

```bash
ship init --provider gemini
```

## Create an agent profile

An agent profile defines what an agent sees and can do. Create one:

```bash
ship agent create my-agent
```

This scaffolds `.ship/agents/my-agent.toml`. Open it and fill in the sections:

```toml
[agent]
id = "my-agent"
name = "My Agent"
version = "0.1.0"
description = "General-purpose assistant for this project"
providers = ["claude"]

[skills]
refs = []

[mcp]
servers = []

[plugins]
install = []
scope = "project"

[permissions]
preset = "ship-standard"

[rules]
inline = """
You work on this project. Follow existing code conventions.
"""
```

**Key sections:**

| Section | Purpose |
|---|---|
| `[agent]` | Identity: id, name, providers to compile for |
| `[skills]` | Skill references — markdown instruction sets the agent receives |
| `[mcp]` | MCP servers exposed to the agent at runtime |
| `[plugins]` | Claude Code plugins to install (Claude provider only) |
| `[permissions]` | Permission preset + per-agent allow/deny overrides |
| `[rules]` | Inline rules injected into the agent's context file |

## Activate the agent

```bash
ship use my-agent
```

This does several things:

1. Finds `.ship/agents/my-agent.toml`
2. Resolves skill references and MCP server declarations
3. Compiles to provider-native config files
4. Installs any declared plugins

For the Claude provider, `ship use` writes:

```
CLAUDE.md              # context file with rules, skill content, permissions
.mcp.json              # MCP server configuration
.claude/skills/        # skill files in Claude's native format
```

These files are gitignored — they're generated artifacts, not source.

## Verify the output

Check what was compiled:

```bash
ship status
```

This shows the active agent and when it was last compiled.

To preview what the compiler would write without writing files:

```bash
ship compile --dry-run
```

To recompile after editing the agent profile:

```bash
ship compile
```

## Add a skill

Skills are markdown instruction sets that extend what an agent knows how to do. Each skill lives in its own directory with a `SKILL.md` file.

Create a local skill:

```bash
ship skill create code-review
```

This scaffolds `.ship/agents/skills/code-review/SKILL.md`. Edit it with your instructions, then reference it from your agent profile:

```toml
[skills]
refs = ["code-review"]
```

Run `ship use my-agent` again to recompile with the new skill.

To install a skill from a remote repo:

```bash
ship skill add github.com/owner/skill-repo
```

## Add an MCP server

MCP servers expose tools to agents at runtime. Ship's own MCP server provides workspace, session, and job coordination tools.

Register it:

```bash
ship mcp add-stdio ship ship mcp serve
```

Then reference it in your agent profile:

```toml
[mcp]
servers = ["ship"]
```

Run `ship use my-agent` to recompile — the MCP server entry will appear in `.mcp.json`.

## Next steps

- [CLI Reference](cli.md) — every command with flags and examples
- [Schema Reference](schema.md) — full field reference for all config files
- [Architecture](architecture.md) — how the compiler pipeline works
