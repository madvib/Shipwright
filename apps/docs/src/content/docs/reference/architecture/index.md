---
title: "Architecture Overview"
description: "Three-layer model, design constraints, and repository layout."
sidebar:
  label: "Architecture Overview"
  order: 1
---
Three layers with strict boundaries: transport (CLI, MCP, web), runtime (state management), and compiler (pure transformation).

## Three-Layer Model

```
+----------------------------------------------------+
|                    Transport                        |
|  CLI (ship-studio-cli)  MCP (ship-mcp)  Web (Studio)  |
|  clap commands          rmcp tools      TanStack Start |
+----------+--------------+-------------+------------+
           |              |             |
+----------v--------------v-------------+
|                    Runtime                          |
|  Workspaces, sessions, events, jobs, file claims   |
|  Targets, capabilities, skill vars, skill paths    |
|  SQLite (platform.db) via sqlx, migrations         |
+----------+------------------------------------------+
           |
+----------v------------------------------------------+
|                    Compiler                          |
|  ProjectLibrary -> ResolvedConfig -> CompileOutput  |
|  Pure function: no filesystem, no network, no DB   |
|  Native + WASM targets (same logic everywhere)     |
+-----------------------------------------------------+
```

### Transport

Dispatchers only. The CLI parses commands via clap and delegates to runtime or compiler functions. The MCP server exposes runtime operations as tools via the rmcp library (stdio and HTTP transports). Ship Studio is a TanStack Start web app that imports the compiler as WASM for in-browser compilation.

### Runtime

Owns all persistent state. The single database lives at `~/.ship/platform.db` (never inside a project directory). Modules: workspaces, sessions, events, jobs, file claims, targets, capabilities, skill vars, skill paths, agents, catalog, config, hooks, plugins, security, registry. SQLite access is via sqlx with compile-time checked queries. Transport layers call runtime functions; they never execute SQL.

### Compiler

A pure transformation pipeline. Takes a `ProjectLibrary` struct (pre-loaded from `.ship/`), resolves it against an active agent profile, and emits provider-native strings. No filesystem access, no network, no database. This purity allows the compiler to compile to both native (CLI, MCP server) and WASM (browser).

The WASM entry points are `compileLibrary` (single provider) and `compileLibraryAll` (all providers), exposed to JavaScript via wasm-bindgen. The `@ship/compiler` npm package wraps this output.

## Design Constraints

**Transport thin, domain in runtime.** If logic needs to coordinate state, it belongs in the runtime crate. CLI and MCP are dispatchers.

**Compiler is pure.** `ProjectLibrary` in, `CompileOutput` out. No side effects. This makes WASM compilation safe and testing straightforward.

**300-line file cap.** Modules are split before they exceed 300 lines.

**Idempotent by default.** `ship use` can be run repeatedly. The compiler overwrites artifacts. The runtime uses upsert patterns.

**Events are append-only.** Every state change emits an event. Events are never updated or deleted.

**Single database, global location.** `~/.ship/platform.db` is shared across all projects on the machine. Tests get automatic isolation via per-thread temp directories.

## Repository Layout

```
apps/
  ship-studio-cli/       CLI binary (clap, ~40 commands)
  mcp/                   MCP server (rmcp, stdio + HTTP)
  web/                   Ship Studio (TanStack Start + Cloudflare Workers)
  docs/                  Documentation site (Astro + Markdoc)
crates/core/
  compiler/              Pure compiler -- types, resolve, compile, vars, WASM bindings
  runtime/               State management -- DB, workspaces, sessions, events, jobs
  cli-framework/         Shared CLI metadata and app lifecycle
  mcp-framework/         Shared MCP app lifecycle
packages/
  compiler/              @ship/compiler -- WASM npm package
  primitives/            @ship/primitives -- shared UI components (shadcn)
  ui/                    @ship/ui -- generated types via Specta
  assets/                Shared static assets
```

## Key Crate Boundaries

The compiler crate (`crates/core/compiler/`) re-exports its public API from `lib.rs`: `ProjectLibrary`, `ResolvedConfig`, `CompileOutput`, `compile`, `resolve`, `resolve_library`, provider descriptors, decompile functions, and type definitions.

The runtime crate (`crates/core/runtime/`) re-exports from `lib.rs`: workspace operations, session lifecycle, event logging, file claims, job coordination, config management, skill/rule CRUD, agent export/import, permissions, catalog, and plugin registry.

Neither crate depends on the other. Transport layers import both.
