---
group: Compiler
title: Compilation Pipeline
section: reference
order: 2
---

# Compilation Pipeline

The compiler transforms `.ship/` configuration into provider-native files through three stages. Each stage is a pure function with well-defined input and output types.

## Stage 1: ProjectLibrary (Load)

`ProjectLibrary` is the in-memory representation of everything in `.ship/`. The loader reads these sources:

| Source | Contents |
|--------|----------|
| `ship.jsonc` | Project manifest: provider list, provider defaults |
| `agents/*.jsonc` | Agent profiles: skills, MCP servers, permissions, rules, plugins |
| `mcp.jsonc` | MCP server definitions: stdio/HTTP/SSE servers with env vars |
| `permissions.jsonc` | Permission presets: allow/deny/ask tool lists per mode |
| `skills/*/SKILL.md` | Skill definitions: markdown instructions with optional template vars |
| `skills/*/assets/vars.json` | Smart skill variable schemas and defaults |
| `rules/` | Shared rule files referenced by agents |

After loading, the `ProjectLibrary` struct contains:

- `modes` -- agent definitions from `agents/*.jsonc`
- `active_agent` -- which agent to resolve
- `mcp_servers` -- all server configurations
- `skills` -- all skill content (with templates unresolved)
- `rules` -- all rule content
- `permissions` -- flattened permission settings
- `hooks` -- session hooks (PreToolUse, Stop, etc.)
- `plugins` -- plugin install intent
- `provider_defaults` -- project-level provider settings
- `agent_profiles` -- subagent profile definitions

No filesystem access occurs after this stage. The `ProjectLibrary` is fully self-contained and serializable to JSON (used for the WASM interface).

## Stage 2: ResolvedConfig (Resolve)

Resolution takes the `ProjectLibrary` plus the active agent ID and merges everything into a flat, unambiguous `ResolvedConfig`.

### Merge Order

Precedence is last-wins:

| Priority | Source | What it provides |
|----------|--------|-----------------|
| 1 (base) | Project defaults | Provider list, MCP servers, provider-level settings |
| 2 | Agent profile | Skills, permissions, rules, hooks, plugins, model |
| 3 (top) | Workspace overrides | Model, MCP server/skill restrictions, provider overrides |

The agent profile acts as a filter over project resources. If an agent lists `"servers": ["ship", "github"]`, only those two MCP servers appear in the resolved config, even if the project defines ten servers.

### What Resolution Produces

The `ResolvedConfig` contains:

- `providers` -- final provider list after all overrides
- `mcp_servers` -- resolved server configs (only those the agent references)
- `skills` -- resolved skill content (only those the agent references)
- `rules` -- resolved rule content
- `permissions` -- merged permission settings (preset + per-agent overrides)
- `hooks` -- merged hook configurations
- `plugins` -- plugin manifest
- Provider-specific settings: model, cost limits, turn limits, sandbox mode, approval policy, environment variables, available models, theme, and pass-through settings for each provider

Every field in `ResolvedConfig` is concrete. No IDs to look up, no references to chase.

## Stage 3: CompileOutput (Compile)

Compilation takes the `ResolvedConfig` and a target provider ID. It emits a `CompileOutput` struct with everything needed to write provider-native files.

### Output Fields

| Field | Type | Description |
|-------|------|-------------|
| `context_content` | `Option<String>` | Body of the context file (CLAUDE.md, GEMINI.md, AGENTS.md) |
| `mcp_servers` | `JSON` | MCP server entries keyed by provider convention |
| `mcp_config_path` | `Option<String>` | Where to write MCP config (`.mcp.json`, `.cursor/mcp.json`, etc.) |
| `skill_files` | `HashMap<String, String>` | Path to content map for skill files |
| `rule_files` | `HashMap<String, String>` | Per-file rules (Cursor `.mdc` files) |
| `agent_files` | `HashMap<String, String>` | Subagent definitions in provider-native format |
| `claude_settings_patch` | `Option<JSON>` | Permissions, hooks, model, limits for `.claude/settings.json` |
| `codex_config_patch` | `Option<String>` | TOML content for `.codex/config.toml` |
| `gemini_settings_patch` | `Option<JSON>` | Hooks for `.gemini/settings.json` |
| `gemini_policy_patch` | `Option<String>` | TOML policies for `.gemini/policies/ship.toml` |
| `cursor_hooks_patch` | `Option<JSON>` | Hooks for `.cursor/hooks.json` |
| `cursor_cli_permissions` | `Option<JSON>` | Permissions for `.cursor/cli.json` |
| `opencode_config_patch` | `Option<JSON>` | Full `opencode.json` content |
| `plugins_manifest` | `PluginsManifest` | Plugin install/uninstall instructions |

The compiler emits strings ready to write to disk. It never touches the filesystem itself -- the CLI or runtime handles file I/O.

### Provider-Specific Compilation

Each provider has a dedicated compilation module:

- `claude.rs` -- settings patch with permissions, hooks, model, cost/turn limits
- `gemini.rs` -- settings patch, per-server trust/filter fields, TOML policy files
- `codex.rs` -- TOML config with MCP servers, approval policy, sandbox settings
- `cursor.rs` -- per-file `.mdc` rules, hooks, CLI permissions, MCP with `envFile`
- `opencode.rs` -- combined `opencode.json` with model, MCP, and permission settings

Common modules handle MCP server serialization, context file generation, skill file mapping, and plugin manifests.

## Template Resolution

Skills that use MiniJinja templates are resolved during compilation. The `resolve_template` function takes skill content and a merged variable map, then renders the template.

### How Variables Merge

Variable values come from four layers, merged in order (last wins):

| Layer | Source | Scope |
|-------|--------|-------|
| defaults | `assets/vars.json` `default` field | Built into the skill |
| global | `platform.db` KV `skill_vars:{id}` | Machine-wide, follows the user |
| local | `platform.db` KV `skill_vars.local:{ctx}:{id}` | This project, personal override |
| project | `platform.db` KV `skill_vars.project:{ctx}:{id}` | This project, shared with team |

### Template Syntax

The compiler uses MiniJinja (Jinja2-compatible). Supported constructs:

- `{{ var }}` -- scalar substitution
- `{{ obj.field }}` -- dot-path into objects
- `if`/`endif` blocks for conditionals
- `if`/`elif`/`else`/`endif` for branching on variable values
- `for`/`endfor` for iterating arrays

Undefined variables render as empty string (chainable undefined behavior). Template syntax errors fall back to the original content with a warning to stderr.

### No File Loader

The template environment has no source or file loader. Include and extends directives are disabled. Each skill template is self-contained.
