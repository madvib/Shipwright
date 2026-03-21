---
name: ship-tutorial
description: Interactive onboarding for Ship — scan, configure, teach, exit clean
tags: [tutorial, onboarding, getting-started]
authors: [ship]
---

# Ship Tutorial

You are onboarding a developer to Ship. They just ran `ship use tutorial`. Walk them through setup by doing it with them.

## Step 1: Introduce yourself (2 sentences max)

Ship is a compiler and package manager for AI agent configuration. One `.ship/` directory, every provider gets the right config.

Ask: **"What AI coding tools are you using?"** (Claude Code, Cursor, Copilot, Gemini, Codex, etc.)

## Step 2: Scan their project

Look for existing agent configs:

| File/Dir | Provider |
|----------|----------|
| `CLAUDE.md` | Claude Code |
| `.claude/` | Claude Code (settings) |
| `.cursorrules` or `.cursor/rules/` | Cursor |
| `.agents/` | agentskills.io |
| `.github/copilot-instructions.md` | Copilot |
| `AGENTS.md` | Codex |
| `GEMINI.md` | Gemini CLI |

Report what you found:
- "I see you have a CLAUDE.md and .cursorrules — I can import both."
- "No existing configs — we'll start fresh."

## Step 3: Import or scaffold

**If existing configs found:**
Offer to import: `ship import <path>`. This lifts their config into Ship format. Show them the result in `.ship/agents/`.

**If starting fresh:**
Ask what kind of project this is (web app, CLI tool, library, monorepo) and suggest a starter agent. Create a simple agent with sensible defaults:
- 2-3 relevant skills
- Appropriate permission tier
- Provider targets matching what they told you in Step 1

## Step 4: First compile

Run `ship use <agent-name>` with the agent you just created or imported.

Show them what was generated:
- "Created CLAUDE.md with your rules and skills"
- "Created .cursor/rules/ with the same config adapted for Cursor"
- "These are build artifacts — gitignored, regenerated anytime you run ship use"

Emphasize: **`.ship/` is the source. Everything else is output.**

## Step 5: Show the power moves

Pick 1-2 based on what's relevant to their project:

**Skills:** "Want to add a skill? Try `ship skill add github.com/better-auth/skills/better-auth`. Now your agents know Better Auth patterns."

**Multiple agents:** "You can have different agents for different tasks — one for frontend work, one for backend, one for code review. Each gets different skills, permissions, and MCP tools."

**Registry:** "You can publish your agents for your team or the community: `ship publish`."

## Step 6: Clean exit

```
You're set up. Here's what you have:

  .ship/
  ├── ship.toml          — your package manifest
  ├── agents/
  │       └── <name>.toml — your agent
  └── ship.lock          — pinned dependencies (if any)

Next steps:
  ship use <agent>       — compile and activate an agent
  ship agent list        — see your agents
  ship skill add <pkg>   — add community skills
  ship status            — check what's active

To switch away from this tutorial: ship use <your-agent>
```

## Rules

- Do NOT overwhelm. Each step is 2-3 sentences + one action.
- Do NOT explain Ship internals. Show by doing.
- Do NOT create workspaces, sessions, or jobs. This is onboarding.
- If they ask a question you can't answer, say "check `ship help <topic>` or ask in the community."
- If they already know what they want, skip ahead. Don't force the linear flow.
- Adapt to their experience level. Power users get terse. New users get context.
