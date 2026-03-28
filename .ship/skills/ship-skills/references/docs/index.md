---
group: Smart Skills
title: Smart Skills
description: Overview of Ship's smart skills system — typed variables, template resolution, and personalized agent output.
audience: public
section: concepts
order: 1
---

# Smart Skills

Smart skills are Ship skills with typed, scoped configuration variables that resolve into content at compile time. A single skill produces different output for each user, project, and configuration. Same skill, personalized behavior.

## The key innovation

Traditional agent skills are static markdown. Every user gets the same instructions. Smart skills add a variable layer: authors declare typed variables, users set values, and the compiler resolves templates before writing provider output. The skill adapts without forking.

```
SKILL.md (MiniJinja template)
assets/vars.json (schema + defaults)      ->  ship compile  ->  resolved provider output
platform.db KV (user state)
```

## What makes a skill "smart"

A skill becomes smart when it has `assets/vars.json`. That file declares typed variables with defaults, storage scopes, and metadata. The compiler detects the presence of vars.json and activates template resolution for SKILL.md.

Without vars.json, a skill is plain markdown. With it, SKILL.md becomes a MiniJinja template where variable references, conditionals, and loops resolve against the user's variable state at compile time.

## How resolution works

1. The compiler reads `assets/vars.json` to discover declared variables.
2. It queries `platform.db` KV for user state across three scopes (global, local, project).
3. It merges state in a fixed order: defaults, then global, then local, then project. Last wins.
4. It passes the merged values into the MiniJinja engine and renders SKILL.md.
5. The resolved output is written to provider config (CLAUDE.md, .cursor/rules, etc.).

Undefined variables render as empty string. Template errors fall back to the original unresolved content with a warning to stderr. The template engine is pure WASM with no file loader and no custom functions.

## Variable storage

All variable state lives in `platform.db` as key-value pairs. No scattered config files.

| Scope | KV namespace | Semantics |
|-------|-------------|-----------|
| global | `skill_vars:{id}` | Machine-wide. Follows the user across all projects. |
| local | `skill_vars.local:{ctx}:{id}` | This project only. Personal override, not shared. |
| project | `skill_vars.project:{ctx}:{id}` | This project. Intended for team sharing. |

`{id}` is the skill's `stable-id`. `{ctx}` is a stable 16-character hex token derived from the project path, scoping local/project state without embedding the path in storage keys.

## Progressive disclosure

Smart skills use a two-tier information architecture:

- **SKILL.md** is always loaded into the agent's context window. Keep it concise -- under 100 lines. This is the working instructions.
- **references/docs/** contains detailed documentation retrieved on demand. Same source serves humans (docs site) and agents (filesystem reads). This is the reference library.

The agent gets the instructions automatically. When it needs depth -- variable details, template syntax, troubleshooting -- it retrieves specific doc pages without consuming permanent context.

## Interfaces

Users and agents interact with smart skills through multiple surfaces:

| Surface | What it does |
|---------|-------------|
| `ship vars` CLI | Read, write, append, reset variable state from the terminal |
| MCP tools | Agents read/write vars programmatically (`get_skill_vars`, `set_skill_var`, `list_skill_vars`) |
| Studio Skills IDE | Visual editor for SKILL.md, vars UI with type-appropriate controls, file explorer |
| `ship use` / `ship compile` | Resolves templates and writes provider output |

## Stability

The smart skills system shipped in Ship 0.1.0. The following are stable:

- SKILL.md with frontmatter and MiniJinja template resolution
- `assets/vars.json` with all five types (string, bool, enum, array, object)
- Three storage scopes with KV merge order
- `ship vars` CLI commands (get, set, append, reset)
- MCP tools for var access
- `references/docs/` with frontmatter
- `scripts/` directory
- Content hashing with vars state excluded
- Studio Skills IDE

**Planned -- not yet available in stable releases:**

- `ship skill eval` tooling (evals.json structure is defined but cannot be run automatically)
- Declarative var migrations
- WASM audit sandbox
- Computed/dynamic vars (env injection, git context)
- `min-runtime-version` frontmatter field
- Structured `allowed-tools` (currently a flat string list)

For implementation details, see the reference pages in this documentation set.
