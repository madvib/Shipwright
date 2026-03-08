<!-- ship:feature id=UMuREHqq -->

# Capability Registry (Skills/Servers)

## Why

The control plane needs a discoverable source of installable skills and MCP server definitions so configuration is composable instead of ad hoc.

## Acceptance Criteria

- [x] Catalog listing/search surfaces are available for skills and MCP servers
- [x] Catalog entries can be consumed by runtime install/import flows
- [x] Capability metadata remains available offline in local-first mode
- [x] Registry data is exposed to UI/CLI surfaces for operator workflows
- [ ] Trust/signature metadata exists for third-party capability verification

## Delivery Todos

- [x] Keep embedded catalog source available through runtime catalog APIs
- [x] Wire catalog queries across UI/CLI surfaces
- [x] Preserve source metadata for install provenance and auditing
- [ ] Add signed provenance and trust-policy controls for marketplace scenarios

## Current Behavior

Capability registry supports discovery and installation workflows for skills/MCP servers in local-first mode.

## Follow-ups

- Add richer trust/signature metadata and policy controls for third-party capability installs.