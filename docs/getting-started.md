# Getting Started

This guide takes you from zero to a fully configured AI agent project in 10 minutes. No external services required.

## Prerequisites

- Rust toolchain (`rustup` + stable)
- Git

## 1. Install Ship

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

## 2. Initialize your project

Navigate to your project root and run:

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
  modes/           # legacy directory (ignored for new projects)
  .gitignore       # ignores compiled artifacts
  README.md        # brief description for collaborators
```

The default provider is `claude`. To use a different default:

```bash
ship init --provider gemini
```

The generated `ship.jsonc` looks like this:

```jsonc
{
  "$schema": "../schemas/ship.schema.json",
  "project": {
    "providers": ["claude"],
  },
}
```

## 3. See the default agent

List available agent profiles:

```bash
ship agents list
```

If you just ran `ship init`, the list will be empty. Create your first agent:

```bash
ship agents create my-agent
```

This writes `.ship/agents/my-agent.jsonc` with the following structure:

```jsonc
{
  "$schema": "../../schemas/agent.schema.json",
  "agent": {
    "id": "my-agent",
    "name": "My Agent",
    "version": "0.1.0",
    "description": "",
    "providers": ["claude"],
  },
  "skills": {
    "refs": [],
  },
  "mcp": {
    "servers": [],
  },
  "permissions": {
    "preset": "ship-standard",
  },
  "rules": {},
}
```

Run `ship agents list` again and you will see it:

```
$ ship agents list
  my-agent    project
```

## 4. Activate the agent

```bash
ship use my-agent
```

This does three things:

1. Reads `.ship/agents/my-agent.jsonc`
2. Resolves skill references, MCP servers, and permissions
3. Compiles provider-native config files into your project root

The compiled output depends on which providers the agent targets. All output files are gitignored -- they are build artifacts, not source.

## 5. Verify the output

Check which files were written. The output depends on the provider:

| Provider | Context file | MCP config | Skills directory | Other |
|----------|-------------|------------|-----------------|-------|
| `claude` | `CLAUDE.md` | `.mcp.json` | `.claude/skills/` | |
| `gemini` | `GEMINI.md` | `.gemini/settings.json` | `.gemini/skills/` | `.gemini/` workspace policy |
| `codex` | `AGENTS.md` | `.codex/config.toml` | `.agents/skills/` | |

Confirm the files exist:

```bash
# For the claude provider (default)
ls CLAUDE.md .mcp.json
```

To see the active agent and compilation status:

```bash
ship status
```

To preview what the compiler would write without touching any files:

```bash
ship compile --dry-run
```

To target a single provider:

```bash
ship compile --provider gemini
```

## 6. Customize

Now that the basic flow works, customize your agent for real work.

### Edit the agent profile

Open it in your editor:

```bash
ship agents edit my-agent
```

Or edit `.ship/agents/my-agent.jsonc` directly. Here is a fully configured example:

```jsonc
{
  "$schema": "../../schemas/agent.schema.json",
  "agent": {
    "id": "my-agent",
    "name": "My Agent",
    "version": "0.1.0",
    "description": "Full-stack development agent for this project",
    // Target multiple providers at once
    "providers": ["claude", "gemini", "codex"],
  },
  "skills": {
    // Reference skills by ID (from .ship/skills/<id>/)
    "refs": ["tdd", "code-review"],
  },
  "mcp": {
    // Reference servers defined in .ship/mcp.jsonc
    "servers": ["ship"],
  },
  "plugins": {
    // Claude Code plugins (claude provider only)
    "install": [
      "rust-analyzer-lsp@claude-plugins-official",
    ],
    "scope": "project",
  },
  "permissions": {
    // Permission preset (see "Permissions" section below)
    "preset": "ship-autonomous",
    // Per-agent deny list layered on top of the preset
    "tools_deny": ["Bash(rm -rf *)", "Bash(git reset --hard*)"],
  },
  "rules": {
    // Inline rules injected into the agent's compiled context
    "inline": "You work on this project. Follow existing conventions.\nFocus on apps/ and packages/ directories.",
  },
}
```

After editing, recompile:

```bash
ship compile
```

### Add skills

Skills are markdown instruction sets that teach an agent how to do something. Each skill is a directory under `.ship/skills/` containing a `SKILL.md` file.

Create a local skill:

```bash
ship skills create code-review
```

This scaffolds `.ship/skills/code-review/SKILL.md`. The file uses YAML frontmatter for metadata:

```markdown
---
name: code-review
description: Structured code review workflow
tags: [review, quality]
authors: [your-name]
---

# Code Review

Review every PR against these criteria:

1. Does the change have tests?
2. Are error messages actionable?
3. Does it follow existing patterns in the codebase?
```

Reference the skill from your agent profile:

```jsonc
{
  "skills": {
    "refs": ["code-review"],
  },
}
```

Run `ship use my-agent` to recompile with the new skill included.

To install a skill from a remote source:

```bash
ship skills add github.com/owner/skill-repo
```

List all installed skills:

```bash
ship skills list
```

### Configure MCP servers

MCP (Model Context Protocol) servers expose tools to agents at runtime. Server definitions live in `.ship/mcp.jsonc`.

Register a stdio server:

```bash
ship mcp add-stdio ship ship mcp serve
```

Register an HTTP server:

```bash
ship mcp add my-api --url http://localhost:8080/mcp
```

This writes the definition to `.ship/mcp.jsonc`:

```jsonc
{
  "$schema": "../schemas/mcp.schema.json",
  "mcp": {
    "servers": {
      "ship": {
        "name": "Ship",
        "command": "ship",
        "args": ["mcp", "serve"],
        "server_type": "stdio",
        "env": {},
      },
      "github": {
        "name": "GitHub MCP",
        "command": "npx",
        "args": ["-y", "@modelcontextprotocol/server-github"],
        "server_type": "stdio",
        "env": {
          "GITHUB_TOKEN": "${GITHUB_TOKEN}",
        },
      },
    },
  },
}
```

Reference servers by ID in your agent profile:

```jsonc
{
  "mcp": {
    "servers": ["ship", "github"],
  },
}
```

List configured servers:

```bash
ship mcp list
```

### Set permissions

Permissions control what tools an agent can use without asking. Ship provides four presets on a strict-to-loose continuum.

Define presets in `.ship/permissions.jsonc`:

```jsonc
{
  "$schema": "../schemas/permissions.schema.json",

  "ship-readonly": {
    // Read-only. Reviewers, auditors, tutors.
    "default_mode": "plan",
    "tools_allow": [
      "Read", "Glob", "Grep",
      "Bash(ship *)",
      "mcp__ship__*",
    ],
    "tools_deny": [
      "Write(*)", "Edit(*)", "Bash(rm*)",
    ],
  },

  "ship-standard": {
    // Interactive sessions. Human-paired work.
    "default_mode": "default",
    "tools_allow": [
      "Read", "Glob", "Grep",
      "Bash(ship *)",
      "mcp__ship__*",
    ],
  },

  "ship-autonomous": {
    // Dispatched specialist agents. Zero prompts.
    "default_mode": "dontAsk",
    "tools_allow": [
      "Read", "Write", "Edit", "Glob", "Grep",
      "Bash(*)",
      "mcp__ship__*",
    ],
    "tools_deny": [
      "Bash(rm -rf *)",
      "Bash(git reset --hard*)",
      "Bash(git push*)",
      "Bash(cargo publish*)",
      "Bash(npm publish*)",
    ],
  },

  "ship-elevated": {
    // CI agents, release automation. Unlocks push/publish.
    "default_mode": "dontAsk",
    "tools_allow": [
      "Read", "Write", "Edit", "Glob", "Grep",
      "Bash(*)",
      "mcp__ship__*",
    ],
    "tools_deny": [
      "Bash(rm -rf *)",
      "Bash(git reset --hard*)",
      "Bash(git push --force*)",
    ],
  },
}
```

Reference a preset from your agent profile. Layer per-agent overrides on top:

```jsonc
{
  "permissions": {
    "preset": "ship-autonomous",
    "tools_deny": ["Bash(rm -rf *)", "Bash(git reset --hard*)"],
    "tools_ask": ["Bash(git push --force*)"],
  },
}
```

| Preset | Mode | Use case |
|--------|------|----------|
| `ship-readonly` | `plan` | Reviewers, gate agents, analysis-only |
| `ship-standard` | `default` | Interactive sessions, paired work |
| `ship-autonomous` | `dontAsk` | Specialist agents in worktrees |
| `ship-elevated` | `dontAsk` | CI agents, release automation |

### Multi-provider agents

Target multiple providers from a single agent profile:

```jsonc
{
  "agent": {
    "id": "my-agent",
    "providers": ["claude", "gemini", "codex"],
  },
}
```

When you run `ship use my-agent`, Ship compiles for every listed provider simultaneously. You can also compile for a single provider:

```bash
ship compile --provider claude
ship compile --provider gemini
ship compile --provider codex
```

Each provider gets its own output format:

**Claude** writes `CLAUDE.md`, `.mcp.json`, and `.claude/skills/`.

**Gemini** writes `GEMINI.md`, `.gemini/settings.json`, `.gemini/skills/`, and a workspace policy file.

**Codex** writes `AGENTS.md`, `.codex/config.toml`, and `.agents/skills/`.

All output files are listed in `.ship/.gitignore` and should never be committed.

### Clone and specialize

Create variants of an agent quickly:

```bash
ship agents clone my-agent rust-expert
ship agents edit rust-expert
```

This copies the full profile. Edit the clone to specialize it -- narrow the skill set, adjust permissions, add domain-specific rules.

### Validate before committing

Check your entire `.ship/` configuration for errors:

```bash
ship validate
```

Validate a single agent:

```bash
ship validate --agent my-agent
```

## Project structure reference

After full setup, your `.ship/` directory looks like this:

```
.ship/
  ship.jsonc                    # project manifest
  permissions.jsonc             # permission presets
  mcp.jsonc                     # MCP server definitions
  .gitignore                    # ignores compiled output
  agents/
    my-agent.jsonc              # agent profile
    rust-expert.jsonc           # another agent profile
  skills/
    code-review/
      SKILL.md                  # skill definition
    tdd/
      SKILL.md
  rules/                        # shared rule files (optional)
```

Compiled output (gitignored, at project root):

```
CLAUDE.md                       # claude provider
GEMINI.md                       # gemini provider
AGENTS.md                       # codex provider
.mcp.json                       # claude MCP config
.gemini/                        # gemini config + skills
.codex/                         # codex config
.claude/skills/                 # claude native skills
.agents/skills/                 # codex native skills
```

## Quick command reference

```bash
# Setup
ship init                         # scaffold .ship/ in current project
ship init --global                # configure ~/.ship/ identity and defaults
ship init --provider gemini       # set default provider

# Daily use
ship use <agent-id>               # activate and compile an agent
ship status                       # show active agent
ship compile                      # recompile after config changes
ship compile --dry-run             # preview without writing files
ship compile --provider claude     # compile for one provider
ship validate                     # check config for errors

# Agent management
ship agents list                  # list available agents
ship agents create <name>         # create a new agent profile
ship agents edit <name>           # open agent in $EDITOR
ship agents clone <src> <dst>     # duplicate an agent
ship agents delete <name>         # remove an agent

# Skills
ship skills list                  # list installed skills
ship skills create <id>           # scaffold a new skill
ship skills add <source>          # install from registry or path
ship skills remove <id>           # remove a skill

# MCP servers
ship mcp list                     # list configured servers
ship mcp add-stdio <id> <cmd>     # register a stdio server
ship mcp add <id> --url <url>     # register an HTTP server
ship mcp remove <id>              # remove a server
ship mcp serve                    # run Ship's own MCP server

# Help
ship docs topics                  # list help topics
ship docs <topic>                 # detailed help (agents, skills, mcp, compile, providers, workflow)
```

## Next steps

- `ship docs agents` -- creating and managing agent definitions
- `ship docs skills` -- adding and authoring skills
- `ship docs mcp` -- MCP server configuration
- `ship docs providers` -- supported providers and output formats
- `ship docs compile` -- how the compilation pipeline works
- `ship docs workflow` -- typical day-to-day workflow
