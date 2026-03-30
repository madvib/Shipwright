---
name: ship-compiler
stable-id: ship-compiler
description: Use when working with Ship's compilation pipeline — resolving agent config, generating provider outputs, understanding the three-stage transformation.
tags: [ship, compiler, architecture]
authors: [ship]
---

# Ship Compiler

Ship's compiler is a pure function: it reads `.ship/` configuration and emits provider-native config files. No filesystem access, no network, no database. The same logic runs in the CLI, MCP server, and browser (via WASM).

## Three-Stage Pipeline

| Stage | Type | What it does |
|-------|------|-------------|
| Load | `ProjectLibrary` | Reads `.ship/` into memory: `ship.jsonc`, agent profiles, skills, MCP servers, permissions, rules. No filesystem access after this step. |
| Resolve | `ResolvedConfig` | Merges project defaults, active agent profile, and workspace overrides into a single self-contained config. All references resolved, no ambiguity. |
| Compile | `CompileOutput` | Takes `ResolvedConfig` + target provider ID. Emits strings ready to write: context files, MCP config, skill files, settings patches. |

## Key Commands

```
ship use <agent-id>          # load + resolve + compile + write for all agent providers
ship compile                 # recompile the active agent
ship compile --dry-run       # preview output without writing files
ship compile --provider X    # compile for a single provider
```

## Resolution Order

Merge precedence (last wins):
1. Project defaults (`ship.jsonc`, `mcp.jsonc`, `permissions.jsonc`)
2. Agent profile (`agents/<id>.jsonc` -- skills, MCP servers, permissions, rules)
3. Workspace overrides (model, providers, server/skill restrictions)

## CompileOutput Fields

The compiler emits these artifacts per provider:

- `context_content` -- body of the context file (`CLAUDE.md`, `GEMINI.md`, `AGENTS.md`)
- `mcp_servers` -- MCP server entries as JSON
- `skill_files` -- skill content mapped to provider-native paths
- `rule_files` -- per-file rules (Cursor `.mdc` files)
- `agent_files` -- subagent definitions in provider-native format
- Provider-specific patches: `claude_settings_patch`, `codex_config_patch`, `gemini_settings_patch`, `gemini_policy_patch`, `cursor_hooks_patch`, `cursor_cli_permissions`, `opencode_config_patch`
- `plugins_manifest` -- plugin install/uninstall instructions

## Template Resolution

Skills can use MiniJinja templates (`{{ var }}`, `{% if %}`, `{% for %}`). Variables are declared in `assets/vars.json` and resolved at compile time from merged state (defaults, global, local, project). Undefined variables render as empty string.

## Design Constraints

- The compiler never touches the filesystem. It takes data in and returns strings out.
- The compiler compiles to both native and WASM. The `@ship/compiler` npm package wraps the WASM output for browser use.
- All output files are gitignored build artifacts. Run `ship use` or `ship compile` to regenerate.
