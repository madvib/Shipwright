---
title: "Compilation Pipeline"
description: "Each pipeline stage in detail -- loading, resolution merge order, compilation, and template variable resolution."
sidebar:
  label: "Compilation Pipeline"
  order: 2
---
Three stages, each a pure function with defined input and output types.

## Stage 1: ProjectLibrary (Load)

The loader reads `.ship/` into a `ProjectLibrary` struct. Sources:

| Source | Contents |
|--------|----------|
| `ship.jsonc` | Project manifest: provider list, provider defaults |
| `agents/*.jsonc` | Agent profiles: skills, MCP servers, permissions, rules, plugins |
| `mcp.jsonc` | MCP server definitions (stdio, HTTP, SSE) |
| `permissions.jsonc` | Permission presets: allow/deny tool lists |
| `skills/*/SKILL.md` | Skill definitions with YAML frontmatter |
| `skills/*/assets/vars.json` | Variable schemas and defaults for smart skills |
| `rules/` | Shared rule files |
| `agents/*.toml` | Subagent profile definitions |
| `agents/teams/claude/*.md` | Legacy Claude team agent files |

After loading, `ProjectLibrary` is a self-contained, JSON-serializable struct. No filesystem access occurs after this point. This is what gets passed to the WASM entry point.

## Stage 2: ResolvedConfig (Resolve)

`resolve_library` takes a `ProjectLibrary`, optional `WorkspaceOverrides`, and an optional active agent ID. It merges everything into a flat `ResolvedConfig`.

### Merge Order (last wins)

| Priority | Source | Provides |
|----------|--------|----------|
| 1 (base) | Project defaults | Provider list, MCP servers, provider-level settings |
| 2 | Agent profile | Skills, permissions, rules, hooks, plugins, model |
| 3 (top) | Workspace overrides | Model, MCP server/skill restrictions, provider list |

The agent profile acts as a filter. If an agent lists specific servers or skills, only those appear in the resolved config.

### Provider Settings Merge

Provider-specific settings use a two-level merge. `provider_defaults` from `ship.jsonc` provides the base. The agent's `*_settings_extra` provides overrides. A deep merge function handles nested JSON objects (arrays are replaced, not concatenated).

### WorkspaceOverrides

The `WorkspaceOverrides` struct allows feature branches to restrict or override:
- `model` -- model override
- `max_cost_per_session` -- cost limit
- `mcp_servers` -- restrict to specific server IDs
- `skills` -- restrict to specific skill IDs
- `providers` -- override provider list

### Output

`ResolvedConfig` has ~40 fields covering: providers, model, cost/turn limits, mcp_servers, skills, rules, permissions, hooks, plugins, agent_profiles, claude_team_agents, env, available_models, and per-provider settings for Claude, Gemini, Codex, Cursor, and OpenCode.

Every field is concrete. No IDs to look up, no references to chase.

## Stage 3: CompileOutput (Compile)

`compile(resolved, provider_id)` dispatches to provider-specific builders.

### Compilation Steps

1. Look up the `ProviderDescriptor` for the provider ID. Return `None` if unknown.
2. Check feature flags (supports_mcp, supports_hooks, supports_tool_permissions, supports_memory).
3. Build MCP server entries (provider-specific serialization for Gemini, Cursor; generic for others).
4. Build context file content (CLAUDE.md, GEMINI.md, AGENTS.md, or none for Cursor).
5. Map skills to provider-native paths and format with YAML frontmatter.
6. Compile subagent profiles to provider-native agent files.
7. Build provider-specific patches (settings, policies, hooks, permissions, environment).
8. Build the plugins manifest filtered by provider.

Each step is handled by a dedicated module under `compile/`.

### Skill File Output

Skills are mapped to provider-native directories:

| Provider | Skills Path |
|----------|-------------|
| Claude | `.claude/skills/<id>/SKILL.md` |
| Gemini | `.agents/skills/<id>/SKILL.md` |
| Codex | `.agents/skills/<id>/SKILL.md` |
| Cursor | `.cursor/skills/<id>/SKILL.md` |
| OpenCode | `.opencode/skills/<id>/SKILL.md` |

Each skill file gets YAML frontmatter (name, description, optional license, compatibility, allowed-tools, metadata) followed by the skill content with template variables resolved.

## Template Resolution

Skills can contain template syntax. Resolution happens during the compile stage, inside `build_skill_files` via the `vars::resolve_template` function.

### How It Works

The `resolve_template` function uses the MiniJinja engine (Jinja2-compatible). It takes skill content and a HashMap of variable names to JSON values.

Supported constructs: scalar substitution, dot-path access into nested objects, conditionals with equality checks and else branches, and iteration over arrays. Each of these is described in the MiniJinja documentation.

{% aside type="tip" %}
Undefined variables render as empty string (MiniJinja's "chainable" undefined behavior). Template syntax errors fall back to the original content with a warning to stderr. Skills degrade gracefully when variables have not been configured.
{% /aside %}

### Variable Merge Order

Variable values come from four layers, merged in order (last wins):

| Layer | Source | Scope |
|-------|--------|-------|
| defaults | `assets/vars.json` `default` field | Built into the skill |
| global | `platform.db` KV `skill_vars:{id}` | Machine-wide |
| local | `platform.db` KV `skill_vars.local:{ctx}:{id}` | Per-project, personal |
| project | `platform.db` KV `skill_vars.project:{ctx}:{id}` | Per-project, shared |

The merged variables are attached to the `Skill.vars` HashMap before the compiler sees them.

### No File Loader

The MiniJinja environment has no source or file loader configured. Include and extends directives are disabled. Each skill template is self-contained.

### Fast Path

If skill content contains no template markers, `resolve_template` returns the original string without constructing a MiniJinja environment.
