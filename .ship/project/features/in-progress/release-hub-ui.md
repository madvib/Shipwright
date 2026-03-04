+++
id = "7MJiREK6"
title = "Release hub UI"
created = "2026-03-02T17:11:46.944829439Z"
updated = "2026-03-03T04:33:27.167644+00:00"
release_id = "v0.1.0-alpha"
adr_ids = []
tags = []
+++

## Why

The release page should answer one question at a glance: how close are we to shipping? It should be a live dashboard with rollout readiness, blockers, and feature-level execution signals.

## Acceptance Criteria

- [x] Release page shows overall progress/readiness across linked features
- [x] Linked features show status and completion/readiness signal with blocking visibility
- [ ] Breaking changes section is structured and editable (not markdown-only)
- [x] "Blocking launch" view surfaces releases/features with incomplete readiness
- [ ] Release status transitions: planned -> active -> shipped -> archived with explicit controls
- [x] Release detail is full-page read-first, with fullscreen edit mode available

## Delivery Todos

- [x] Release detail layout: compact header, document view, linked feature section
- [x] Readiness/progress computation from linked feature checklist signals
- [x] Per-feature tile: title, status chip, readiness/progress color
- [x] Blocking/ready filtering in hub list
- [ ] Breaking changes structured editor UI
- [x] Release notes/document editable in fullscreen mode
- [ ] Status transition controls (explicit ship/archive actions)

## Notes

This page now favors dense operational scanning over marketing-card spacing. Keep extending feature-linked execution context; do not invest in issues as a primary planning surface here.