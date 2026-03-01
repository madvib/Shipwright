+++
id = "3RB7tB54"
title = "Ship vNext: UI Plugin Host Rewrite (Aggressive Path)"
created = "2026-02-28T21:15:00Z"
updated = "2026-02-28T21:15:00Z"
tags = []
+++

# Ship vNext: UI Plugin Host Rewrite (Aggressive Path)

## Summary
This is a platform pivot, not a feature add.
Goal: replace task-opinionated UI with a minimal plugin host where Issues/Specs/ADRs become an official plugin (`Task OS`) on top of a core runtime.

Chosen direction:
1. Aggressive rewrite.
2. Minimal host core.
3. Official plugins only in first release.
4. JS/TS public plugin SDK + official Rust host modules.
5. Keep current `.ship` markdown+TOML data model via official plugin.
6. New vNext spec becomes source of truth (alpha spec remains historical).

## Scope (v1 of the new direction)

### In scope
1. Plugin runtime host shell (routes/nav/commands/settings contributions).
2. Plugin lifecycle (discover, validate, enable/disable, boot, fail-safe isolation).
3. Capability-gated host API for plugins.
4. Theme system with user themes + plugin theme packs.
5. Migration of current task tooling into official `Task OS` plugin.
6. Official plugin catalog view (no public marketplace yet).

### Out of scope
1. Open third-party marketplace publishing.
2. Public Rust plugin SDK.
3. Arbitrary DOM/layout injection into any shell zone.
4. Cloud accounts/subscriptions backend.

## Target UI Architecture

1. `Host Shell`:
- Owns app chrome, navigation frame, command palette, settings shell, plugin manager, plugin route host.
- Contains no task-domain rendering logic.

2. `Plugin Runtime`:
- Loads plugin manifests and JS bundles.
- Registers contributions (routes, nav items, commands, settings panels, themes).
- Enforces capability checks per plugin call.

3. `Host API Bridge`:
- Typed TS facade in UI.
- Backed by Tauri commands for privileged operations.
- Capability middleware validates plugin permission before invoking host action.

4. `Official Plugins`:
- `task-os` plugin: issues/specs/adrs/activity/editor UX.
- Additional official plugins can add monetizable features later without changing host core.

## Public API / Interface Changes

| Interface | Change |
|---|---|
| `crates/ui/src/types.ts` | Split into `core` host types and plugin SDK types; remove task-domain types from host global namespace. |
| `crates/ui/src/platform/tauri/commands.ts` | Add plugin-runtime commands (plugin discovery/state/lifecycle/capability calls). Existing task commands move behind `task-os` plugin API module. |
| `crates/ui/src/router.tsx` | Replace hardcoded domain routes with host routes + plugin host route (`/plugins`, `/settings`, `/p/$pluginId/$page`). |
| `crates/ui/src/App.tsx` | Reduce to shell composition only; no task domain logic, no task modals/details mounted directly. |
| `WorkspaceContext` | Replace monolithic workspace controller with host runtime store + per-plugin state stores. |
| `.ship/specs` | Add `plugin-runtime-vnext.md` and mark alpha spec as historical for scope decisions. |

### New plugin manifest (proposed)
`ship.plugin.toml`:
- `id`, `name`, `version`, `api_version`
- `entry` (UI bundle entry)
- `permissions` (capability list)
- `contributes.routes`
- `contributes.nav`
- `contributes.commands`
- `contributes.settings`
- `contributes.themes`
- `signature` (official signing metadata)

### New JS/TS SDK contract (proposed)
`definePlugin({...})` exports:
- `setup(ctx)`
- `routes[]`
- `navItems[]`
- `commands[]`
- `settingsPanels[]`
- `themes[]`

`PluginContext` includes:
- `hostApi` (capability-gated)
- `router`
- `events`
- `storage` (namespaced per plugin)
- `logger`

## Capability Model (v1)

### Core capabilities
1. `project.read`
2. `project.write`
3. `issues.read`
4. `issues.write`
5. `specs.read`
6. `specs.write`
7. `adrs.read`
8. `adrs.write`
9. `log.read`
10. `settings.read`
11. `settings.write`
12. `ui.theme.write`
13. `commands.register`

### Rules
1. Plugins can only call APIs they declare in manifest and are granted by host.
2. Unauthorized capability attempts are blocked and logged.
3. Plugin failure must not crash host shell; failed plugin is disabled for session with actionable error.

## Implementation Plan (Decision Complete)

## Phase 1: Define vNext spec and host boundaries
1. Create `.ship/specs/plugin-runtime-vnext.md` with the decisions above.
2. Freeze new task UX work in host; all new domain UX goes into `task-os` plugin target.
3. Define stable plugin SDK `api_version = 1`.

## Phase 2: Build host runtime skeleton
1. Add `core/plugin-runtime` modules:
- registry
- loader
- capability guard
- contribution registry
- plugin error boundary
2. Add host routes:
- `/projects`
- `/plugins`
- `/settings`
- `/p/$pluginId/$page`
3. Move shell-only UI into host:
- sidebar frame
- header/breadcrumb shell
- command palette shell
- plugin settings shell

## Phase 3: Introduce plugin bridge APIs (UI-first, Rust-backed)
1. Add Tauri commands for plugin metadata and enabled state.
2. Add plugin runtime event channel (`ship://plugins-changed`, `ship://plugin-crashed`).
3. Add capability-gated host API wrappers in TS.

## Phase 4: Extract current task domain into official `task-os` plugin
1. Move issues/specs/adrs/activity/editor components and related hooks under plugin package boundary.
2. Route all existing task commands through plugin-scoped host API client.
3. Remove task-domain assumptions from host context.
4. Keep `.ship` formats unchanged; parser/writer logic remains compatible.

## Phase 5: Theme and customization platform
1. Formalize CSS token contract for host and plugins.
2. Add theme registry with host validation for allowed CSS variables.
3. Add plugin/theme selector in settings.
4. Add per-user theme persistence in global config.

## Phase 6: Official plugin management UX
1. Add `Plugins` page with official catalog cards.
2. Add install/enable/disable/update flows for official plugins.
3. Show permissions requested per plugin.
4. Add diagnostics panel for plugin runtime errors.

## Phase 7: Hardening and alpha-vnext release gate
1. Crash isolation tests for plugin boot/render/runtime failures.
2. Permission enforcement tests.
3. Startup performance budget and lazy plugin boot.
4. Telemetry/logging for plugin errors and blocked capability calls.

## Testing and Validation

### Unit tests
1. Manifest schema parse/validation.
2. Capability guard allow/deny matrix.
3. Contribution registry dedupe/conflict behavior.
4. Theme token whitelist validation.

### Integration tests
1. Plugin boot success/failure isolation.
2. Enabling/disabling plugin updates nav/routes without restart.
3. Plugin command invocation path to Tauri bridge.
4. Plugin route rendering and teardown.

### E2E scenarios
1. Launch app with only host + task-os plugin.
2. Disable task-os plugin and verify host stays functional.
3. Simulate plugin crash and verify host fallback UI.
4. Theme pack apply/revert across restart.
5. Permissions denied flow with clear user-facing messaging.

## Acceptance Criteria

1. Host app runs without any task-specific UI code in core shell.
2. Task management works only through `task-os` official plugin.
3. Plugin-added routes/nav/commands/settings appear dynamically.
4. Unauthorized plugin API calls are blocked and logged.
5. Plugin runtime crash does not blank-screen app.
6. `.ship` issue/spec/adr formats remain fully compatible.
7. Official plugin management UX is usable end-to-end.

## Risks and Mitigations

1. Risk: rewrite stall due to breadth.
Mitigation: strict phase gates; no new host-domain feature work until extraction complete.

2. Risk: plugin API churn.
Mitigation: lock `api_version=1` contract before extracting `task-os`.

3. Risk: host rerender/perf regressions from dynamic contributions.
Mitigation: contribution registry memoization + lazy route loading + plugin mount boundaries.

4. Risk: security overreach with plugin APIs.
Mitigation: explicit capability map, deny-by-default, audit log of denied calls.

## Assumptions and Defaults

1. Package manager remains `pnpm`.
2. Tauri remains desktop runtime.
3. Plugin distribution in v1 is official only.
4. Public plugin SDK in v1 is JS/TS only.
5. Official Rust modules are internal implementation detail in host/runtime.
6. Existing alpha spec remains as historical scope; vNext spec becomes active direction.
