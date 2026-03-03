+++
id = "ENZ3iX6C"
title = "Feature hub UI"
created = "2026-03-02T17:11:33.592910150Z"
updated = "2026-03-03T04:33:27.176534+00:00"
release_id = "v0.1.0-alpha"
adr_ids = []
tags = []
+++

## Why

A feature is not just markdown — it is a planning and execution container. The hub should surface readiness, links, and workspace context while keeping density high and interactions direct.

Specs are first-class units of work inside workspaces (closer to commit-sized execution slices). Issues are optional and should remain de-emphasized until proven high-value.

## Acceptance Criteria

- [x] Feature detail is a full-page, read-first view (not modal)
- [x] Fullscreen edit mode is available from the read view
- [x] Feature hub rows show readiness and status signals
- [x] Feature detail shows planning links (release/spec) and lightweight ADR link chips
- [ ] Workspace context is embedded as first-class feature context (status + activate + active spec)
- [ ] Specs are the primary execution section in feature detail/workspace flow
- [ ] Issues are not prominent in feature IA (kept secondary/minimal)
- [ ] Interactive checklist updates persist directly to SQLite without heavy save flow

## Delivery Todos

- [x] Feature detail full-page layout with compact centered header
- [x] Readiness panel and planning link cards in detail view
- [x] ADRs represented as inline links/chips (no dedicated feature ADR page required)
- [ ] Add spec-first execution panel (ordered specs, status, quick open/activate)
- [ ] Add workspace quick actions in hub/detail (activate, status, stale-context warning)
- [ ] Keep issues secondary; avoid dedicated prominent feature issues surface for now
- [ ] Tighten SQLite-native interactions for checklist state persistence

## Notes

Do not over-invest in issue-centric UI right now. Prioritize workspace + spec execution flow, with ADR context as lightweight references inside the feature experience.