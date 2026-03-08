<!-- ship:feature id=fahbueRy -->

# Ship Command Line Interface

## Why

Ship needs a deterministic, scriptable interface for every core workflow so users can bootstrap, operate, and recover projects without relying on the desktop UI.

## Acceptance Criteria

- [x] CLI supports init, planning entities, workspace lifecycle, sessions, modes, providers, and diagnostics
- [x] Workspace operations (create/sync/activate/repair/start/end/open) are available and exercised in real workflows
- [x] CLI can manage and inspect agent configuration, MCP registration, and project/global state
- [ ] Product-facing command set is fully separated from internal migration/dev-only operations

## Delivery Todos

- [x] Consolidate primary workflow commands under workspace-first model
- [x] Add `ship ui` and doctor/version/product-surface commands for operational clarity
- [x] Harden feature/release/spec/workspace commands and context sync wiring
- [ ] Final pass on command ergonomics and help text for launch polish

## Current Behavior

The CLI is production-capable for daily use and underpins UI/MCP workflows. It remains the most complete operational interface, with some command-surface cleanup still pending.

## Follow-ups

- Split framework vs Ship-app specific surfaces where appropriate.
- Keep docs aligned with final launch command taxonomy.