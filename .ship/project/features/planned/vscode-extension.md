<!-- ship:feature id=jrstM5CS -->

# VSCode Extension

## Why

Ship needs an editor-native surface in VS Code so users can consume workspace state and orchestration signals without context switching.

## Acceptance Criteria

- [ ] Extension can discover active Ship project/workspace context
- [ ] Core workspace actions (activate/sync/open/session status) are exposed in VS Code
- [ ] Planning entity navigation is available from editor commands/panels
- [ ] Extension behavior is aligned with Ship mode/config compilation semantics

## Delivery Todos

- [ ] Define extension command model and minimal UI panels
- [ ] Implement runtime bridge for workspace/project state reads
- [ ] Implement action commands for workspace lifecycle hooks
- [ ] Add launch-quality install and troubleshooting docs

## Current Behavior

This feature is planned; no shipped extension surface yet.

## Follow-ups

- Reuse mode and provider semantics from core runtime to avoid editor-specific drift.