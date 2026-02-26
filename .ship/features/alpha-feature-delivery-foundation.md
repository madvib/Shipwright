+++
id = "feat-alpha-feature-delivery-foundation"
title = "Alpha Feature Delivery Foundation"
status = "active"
created = "2026-02-25T00:00:00Z"
updated = "2026-02-25T00:00:00Z"
owner = "ship"
release = "v0.1.0-alpha"
spec = "alpha-feature-delivery-workflow.md"
adrs = []
tags = ["alpha", "workflow", "primitives"]
+++

## Why

Shipwright needs one opinionated workflow with clear semantics and low git noise.

## Acceptance Criteria

- [x] Primitive model documented (vision, feature, spec, issue, adr, mode, events)
- [x] Alpha workflow documented as a single path
- [x] Default git policy commits planning/decision artifacts and keeps issues local
- [ ] Feature CRUD surfaced in CLI/MCP/UI
- [ ] Workflow policy surfaced in agent context end-to-end

## Delivery Todos

- [x] Add workflow and primitives spec docs
- [x] Add future-ideas backlog doc (graph + auto mode switch)
- [x] Scaffold `features/` primitive in `.ship/` init + template
- [x] Update e2e checks for default git policy expectations
- [ ] Implement feature commands and MCP tools
- [ ] Add feature board and detail views in UI

## Notes

This feature intentionally prioritizes semantics and defaults before adding broad customization.
