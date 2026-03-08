<!-- ship:feature id=A5nhFQVq -->

# Extensible Runtime and SDK

## Why

Ship needs a reusable foundation so multiple product surfaces (CLI, MCP, desktop) and future apps can share one runtime model instead of duplicating orchestration logic.

## Acceptance Criteria

- [x] Core runtime lives in `core/runtime` and is consumed by application crates
- [x] CLI and MCP framework crates exist (`core/cli-framework`, `core/mcp-framework`) and are used by Ship binaries
- [x] Compile-time plugin/namespace extension points exist for first-party modules
- [x] Ship CLI/MCP run through framework metadata + core command hooks
- [ ] Runtime state model is fully app-agnostic (Ship-specific schemas still leak into core state DB)

## Delivery Todos

- [x] Split shared framework crates for CLI and MCP surfaces
- [x] Route Ship CLI through framework command lifecycle and core primitives
- [x] Route Ship MCP through framework wrapper
- [ ] Move Ship-specific planning schemas out of core runtime state layer
- [ ] Define stable public app-extension contract for non-Ship apps

## Current Behavior

The platform now has a working layered architecture: runtime foundation + CLI/MCP frameworks + Ship application modules. Extensibility is compile-time and first-party focused today.

## Follow-ups

- Continue reducing Ship-specific assumptions inside `core/runtime`.
- Publish extension contracts once schema boundaries are cleaned up.