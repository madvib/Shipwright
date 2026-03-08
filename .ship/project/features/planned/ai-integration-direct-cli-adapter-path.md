<!-- ship:feature id=brbWqudi -->

# AI Integration: Direct CLI Adapter Path

## Why

Ship needs deterministic AI execution paths that work with local provider CLIs without forcing API-key-first setup.

## Acceptance Criteria

- [x] In-app AI actions execute through provider CLI adapters for supported providers
- [x] Invocation attempts are normalized per provider command shape
- [x] Failures return actionable diagnostics instead of silent no-ops
- [x] Provider selection comes from resolved agent configuration
- [ ] End-to-end integration coverage exists for all supported providers
- [ ] Session refresh/restart semantics are explicit after context recompilation

## Delivery Todos

- [x] Implement provider CLI invocation routing in backend command layer
- [x] Add fallback attempt behavior for provider CLI invocation formats
- [x] Surface CLI execution errors back to UI responses
- [ ] Add deterministic integration tests with provider stubs
- [ ] Define operator guidance for re-sync/restart after context updates

## Current Behavior

Direct CLI adapter flow is active in the desktop AI pathways. Core reliability is good, but formal integration coverage and lifecycle restart semantics are still being hardened.

## Follow-ups

- Add provider-specific health checks in preflight UX.
- Add explicit recovery actions when provider/session context drifts.