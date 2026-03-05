# AgentOS Framework Split (Proposed)

## Current State

- `core/runtime`: domain and state engine (entities, persistence, context resolution, export).
- `crates/cli`, `crates/mcp`, `crates/ui/src-tauri`: Ship app transports that call runtime directly.
- `crates/modules/*`: Ship-specific workflow modules.

## Target Shape

Use transport frameworks that Ship composes, so future apps can reuse the same substrate.

### Layer 1: Engine

- `core/runtime` (existing): canonical state, policies, context compiler, provider exporters.

### Layer 2: Transport Frameworks

- `core/cli-framework`
  - command registry, app bootstrap, project resolution, shared flags/output primitives.
  - extension API: app registers command modules.
- `core/mcp-framework`
  - server bootstrap, tool registration, auth/policy gates, event streaming helpers.
  - extension API: app registers tool sets and capability descriptors.
- `core/ui-framework`
  - shared Tauri command surface contracts, event bus wiring, sync lifecycle helpers.
  - extension API: app contributes command groups + feature routes.

### Layer 3: App Bindings

- `apps/ship-cli`, `apps/ship-mcp`, `apps/ship-ui` (or keep under `crates/*` initially)
  - depend on framework + Ship modules.
  - contain only app composition and branding-specific behavior.

## Incremental Extraction Plan

1. Introduce framework crates with minimal wrappers around existing transport logic.
2. Move shared transport concerns first:
   - provider bootstrap and sync orchestration
   - standardized command/tool error mapping
   - lifecycle hooks + telemetry envelopes
3. Keep Ship-specific commands/tools in Ship modules and register them via framework APIs.
4. Once stable, split app crates into `apps/*` with thin composition roots.

## Hard Boundaries

- Runtime must not depend on app or transport crates.
- Framework crates may depend on runtime, but not on Ship modules.
- App crates compose framework + modules and own user-facing command/tool catalogs.

## Why This Works

- Preserves a single AgentOS engine with strong invariants.
- Makes CLI/MCP/UI reusable for future products without forking infrastructure.
- Keeps Ship implementation velocity while enabling controlled multi-app expansion.
