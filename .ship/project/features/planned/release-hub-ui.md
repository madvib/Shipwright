+++
id = "7MJiREK6"
title = "Release hub UI"
status = "planned"
created = "2026-03-02T17:11:46.944829439Z"
updated = "2026-03-02T17:11:46.944829439Z"
release_id = "v0.1.0-alpha"
adr_ids = []
tags = []

[agent]
mcp_servers = []
skills = []
+++

## Why

The release page should answer one question at a glance: how close are we to shipping? Right now it's a flat markdown file. It needs to be a live dashboard — progress across all linked features, what's blocking, what's done, what's left.

## Acceptance Criteria

- [ ] Release page shows overall progress bar (todos completed across all linked features)
- [ ] Each linked feature shows its own completion %, status chip, and blocking indicator
- [ ] Breaking changes section is structured and editable
- [ ] "Blocking launch" view surfaces features with incomplete acceptance criteria
- [ ] Release status transitions: planned → active → shipped → archived

## Delivery Todos

- [ ] Release detail layout: header (version, status, target date), progress bar, feature list
- [ ] Progress bar computed from FeatureTodo completion across all linked features
- [ ] Per-feature row: title, status chip, todo %, acceptance criteria met indicator
- [ ] Blocking view: filter to features with unmet acceptance criteria
- [ ] Breaking changes: structured list (not just markdown prose)
- [ ] Release notes section: editable markdown, generated summary option
- [ ] Status transition controls (ship it button)

## Notes

This is the view that answers "are we ready to demo." Build it early so we can dogfood it against our own v0.1.0-alpha features immediately.
