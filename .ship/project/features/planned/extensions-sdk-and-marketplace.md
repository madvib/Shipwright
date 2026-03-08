<!-- ship:feature id=4fRXWgVQ -->

# Extensions SDK and Marketplace

## Why

Ship needs a safe and scalable distribution model for reusable capabilities (skills, MCP definitions, templates, future app packs) without forcing manual copy/paste configuration.

## Acceptance Criteria

- [ ] Extension package format is defined with identity, provenance, and compatibility metadata
- [ ] Install flow supports trusted registry + explicit untrusted-source warnings
- [ ] Capabilities can be installed to user or project scope with deterministic registration
- [ ] Permission model gates script/tool execution in third-party extensions
- [ ] Marketplace listing surface supports search, provenance, and version awareness

## Delivery Todos

- [ ] Define extension manifest and package layout
- [ ] Implement package install/update/remove lifecycle APIs
- [ ] Add trust-policy enforcement for scripts/commands/assets
- [ ] Expose marketplace browsing/install surfaces in CLI/UI

## Current Behavior

Partially prepped: skill install/import flows exist, but no full extension package standard or marketplace runtime.

## Follow-ups

- Align extension packaging with security model and cloud sync roadmap.