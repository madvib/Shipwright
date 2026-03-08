<!-- ship:feature id=qdqSyJXN -->

# Shipwright MCP Server

## Why

Ship needs a machine-consumable operations surface so external agent clients can execute planning/workspace actions through MCP with deterministic behavior.

## Acceptance Criteria

- [x] MCP server exposes Ship planning and workflow operations through typed tools
- [x] MCP requests route into the same runtime/module logic used by CLI/UI
- [x] Core operations are documented and discoverable for provider integration
- [ ] Tool surface is fully audited for launch with strict permission/error semantics per provider

## Delivery Todos

- [x] Stand up and ship the MCP server binary/surface
- [x] Wire workspace/planning operations into MCP request handlers
- [x] Integrate MCP registration/config paths in agent configuration model
- [ ] Complete launch hardening for error handling and provider reset/resume behavior

## Current Behavior

Ship MCP is functional and integrated with core runtime operations. Remaining work centers on launch-grade guardrails, diagnostics, and provider-specific resilience.

## Follow-ups

- Add deeper telemetry for cross-session coordination and conflict warnings.
- Continue narrowing product-surface tools vs internal ops.