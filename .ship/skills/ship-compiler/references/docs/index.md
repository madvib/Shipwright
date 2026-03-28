---
group: Compiler
order: 1
title: Compiler Overview
description: The pure transformation pipeline -- ProjectLibrary to ResolvedConfig to CompileOutput -- and its WASM target.
---

# Compiler Overview

The compiler crate (`crates/core/compiler/`) transforms agent configuration into provider-native files. It is a pure function: structured data in, strings out. No filesystem, no network, no database.

## Pipeline

```
.ship/ directory (on disk)
       |
       v  [loader reads files -- outside the compiler]
ProjectLibrary  (in-memory representation of .ship/)
       |
       v  [resolve::resolve_library]
ResolvedConfig  (merged, self-contained agent config)
       |
       v  [compile::compile]
CompileOutput   (provider-native strings ready to write)
```

The loader reads `.ship/` into a `ProjectLibrary`. This step happens outside the compiler (in the CLI or MCP server). The compiler receives the `ProjectLibrary` as input.

`resolve_library` takes the library, optional workspace overrides, and the active agent ID. It merges project defaults with the agent profile, producing a `ResolvedConfig` with no unresolved references.

`compile` takes the `ResolvedConfig` and a provider ID (e.g., "claude", "gemini"). It returns a `CompileOutput` with every file the provider needs. Returns `None` for unknown providers.

## Crate Structure

```
compiler/src/
  lib.rs            Re-exports, WASM bindings, nanoid helper
  resolve.rs        ProjectLibrary, ResolvedConfig, WorkspaceOverrides, resolve logic
  compile/
    mod.rs          CompileOutput struct, main compile() entry point
    provider.rs     ProviderDescriptor registry (5 providers)
    claude.rs       Claude settings patch builder
    gemini.rs       Gemini settings/policy/MCP builders
    codex.rs        Codex TOML config builder
    cursor.rs       Cursor rules/hooks/permissions/environment builders
    opencode.rs     OpenCode JSON config builder
    context.rs      Context file content builder (CLAUDE.md, GEMINI.md, AGENTS.md)
    skills.rs       Skill file mapper (frontmatter + template resolution)
    mcp.rs          MCP server entry serializer
    plugins.rs      Plugin manifest builder
    agents.rs       Subagent profile compiler
  vars.rs           MiniJinja template resolution
  types/            All type definitions (Skill, Rule, Permissions, AgentProfile, etc.)
  decompile.rs      Reverse: provider files -> ProjectLibrary
  lockfile.rs       Lock file operations
  manifest.rs       Manifest parsing
  schemas.rs        JSON schema definitions for provider configs
  jsonc.rs          JSONC parser
  agent_parser.rs   Agent file parser
```

## WASM Target

The compiler compiles to both native and `wasm32` targets. The WASM module exposes three functions via `wasm_bindgen`:

- `compileLibrary(library_json, provider, active_agent)` -- compile for one provider
- `compileLibraryAll(library_json, active_agent)` -- compile for all providers in the resolved config
- `listProviders()` -- return supported provider IDs

Both compile functions accept a JSON-serialized `ProjectLibrary` string and return a JSON result string. The `@ship/compiler` npm package wraps these bindings.

This means the CLI, MCP server, and Ship Studio browser app all execute identical compilation logic.

## Key Types

**`ProjectLibrary`** -- everything loaded from `.ship/`. Fields: modes, active_agent, mcp_servers, skills, rules, permissions, hooks, plugins, provider_defaults, agent_profiles, claude_team_agents, env, available_models, and per-provider settings (codex_sandbox, gemini_*, codex_*, opencode_*, cursor_*, claude_*).

**`ResolvedConfig`** -- fully merged config. Same domain fields as ProjectLibrary but with all references resolved and all overrides applied. Every field is concrete.

**`CompileOutput`** -- provider-ready strings. Fields: mcp_servers (JSON), mcp_config_path, context_content, skill_files, rule_files, agent_files, plugins_manifest, and provider-specific patches (claude_settings_patch, codex_config_patch, gemini_settings_patch, gemini_policy_patch, cursor_hooks_patch, cursor_cli_permissions, cursor_environment_json, opencode_config_patch).

## CLI Commands

| Command | Effect |
|---------|--------|
| `ship use <agent-id>` | Load, resolve, compile for all providers, write to disk |
| `ship compile` | Recompile the active agent |
| `ship compile --dry-run` | Preview output without writing files |
| `ship compile --provider <id>` | Compile for one provider only |
| `ship validate` | Check `.ship/` config for errors |
| `ship status` | Show active agent and compilation timestamp |
