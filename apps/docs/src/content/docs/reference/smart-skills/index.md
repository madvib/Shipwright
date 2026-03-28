---
title: "Smart Skills"
description: "Typed variables that personalize skill output at compile time."
sidebar:
  label: "Smart Skills"
  order: 1
---
A smart skill is a skill with typed configuration variables. The compiler resolves those variables into SKILL.md at compile time, so the same skill produces different output for each user, project, and machine.

## What makes a skill "smart"

The presence of `assets/vars.json` in the skill directory. That file declares variables with types, defaults, and storage scopes. Without it, SKILL.md is plain markdown. With it, SKILL.md becomes a template.

## How resolution works

The compiler performs five steps during `ship use` or `ship compile`:

1. Reads `assets/vars.json` to discover declared variables.
2. Queries `platform.db` KV for user state across three scopes (global, local, project).
3. Merges state in fixed order: defaults, then global, then local, then project. Last wins.
4. Passes merged values into the MiniJinja engine and renders SKILL.md.
5. Writes the resolved output to provider config (CLAUDE.md, .cursor/rules, etc.).

Undefined variables render as empty string. Template syntax errors fall back to the original content with a warning to stderr. The template engine uses MiniJinja with chainable undefined behavior -- no file loader, no custom functions.

## Variable storage

All state lives in `platform.db` as key-value pairs. No config files on disk.

| Scope | KV namespace | Semantics |
|-------|-------------|-----------|
| global | `skill_vars:{id}` | Machine-wide. Follows the user across all projects. |
| local | `skill_vars.local:{ctx}:{id}` | This project only. Personal, not shared. |
| project | `skill_vars.project:{ctx}:{id}` | This project. Intended for team sharing. |

`{id}` is the skill's `stable-id` (or directory name if no stable-id is set). `{ctx}` is a 16-character hex token derived from the project path.

## Progressive disclosure

Smart skills use two tiers of information:

- **SKILL.md** is always loaded into the agent's context window. Keep it concise. This is the working instructions.
- **references/docs/** contains detailed reference material retrieved on demand. Same source serves humans (docs site) and agents (filesystem reads).

The agent gets instructions automatically. When it needs depth, it reads specific doc pages without consuming permanent context.

## Interfaces

| Surface | What it does |
|---------|-------------|
| `ship vars` CLI | Read, write, append, reset variable state from the terminal |
| MCP tools | Agents read/write vars programmatically (`get_skill_vars`, `set_skill_var`, `list_skill_vars`) |
| Studio Skills IDE | Visual editor for SKILL.md, vars UI with type-appropriate controls |
| `ship use` / `ship compile` | Resolves templates and writes provider output |

## Stability

Stable in Ship 0.1.0:

- SKILL.md with frontmatter and MiniJinja template resolution
- `assets/vars.json` with all five types (string, bool, enum, array, object)
- Three storage scopes with KV merge order
- `ship vars` CLI commands (get, set, append, reset)
- MCP tools for var access
- `references/docs/` with frontmatter
- `scripts/` directory
- Content hashing with vars state excluded
- Studio Skills IDE

Not yet available:

- `ship skill eval` (evals.json structure is defined but tooling is not implemented)
- Declarative var migrations
- Computed/dynamic vars (env injection, git context)
