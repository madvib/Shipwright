<!-- ship:feature id=SjS4tQUW -->

# Desktop User Interface

## Why

Ship needs a workspace-first desktop command center that makes planning, execution, and agent operations visible in one place.

## Acceptance Criteria

- [x] Desktop app provides project home, workspace command center, planning hubs, and agent config surfaces
- [x] Workspace detail supports session lifecycle controls and embedded terminal sessions
- [x] Users can activate/sync/repair workspaces and launch supported editors from the workspace surface
- [ ] Loading/error states are polished across all heavy views and provider failures are always actionable
- [ ] Workspace UI supports full feature/spec linking and safe delete/archive flows without CLI fallback

## Delivery Todos

- [x] Ship Tauri shell and React UI architecture for core pages
- [x] Workspace-centric layout with command and detail views
- [x] Integrated terminal/session surface for provider and shell sessions
- [ ] Complete workspace action affordances (linking, lifecycle edits, deletion) end-to-end in UI
- [ ] Finish UX hardening for responsive layouts, tooltips, and failure recovery paths

## Current Behavior

Desktop UI is the primary execution surface. Workspace and planning pages are live, with significant coverage of session operations and config controls. Remaining work is mostly UX hardening, error handling polish, and final workflow completeness.

## Follow-ups

- Add docs-dirty indicators and context drift warnings tied to workspace file changes.
- Expand session telemetry panels (changed files, provider diagnostics, restart guidance).