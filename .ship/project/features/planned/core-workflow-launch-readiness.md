+++
id = "7cwccEzi"
title = "Core Workflow Launch Readiness"
created = "2026-03-06T01:03:52.623771+00:00"
updated = "2026-03-06T02:15:54.648757+00:00"
branch = "feature/core-workflow-launch"
tags = []
+++

## Why

Make the workspace-first loop production-grade: session start is deterministic, provider/config compilation is predictable, and workspace execution paths are smooth across CLI + runtime.

## Delivery Passes

- [x] Pass 1 (`xxLm93id`): session provider pinning + compile propagation + compile freshness metadata.
- [x] Pass 2 (`kLFwcMmu`): workspace IDE open command + editor detection.
- [x] Pass 3 (`m5S6EHiN`): remove issue extraction from generated branch context (issues are no longer part of the active workflow).

## Notes

- `workspace session start` now supports `--provider` and validates against allowed providers.
- Session records now include `primary_provider`, `compiled_at`, and `compile_error` fields.
- Workspace context generation now compiles from feature/spec + rules + skills only; no open-issues section.
- Follow-up hardening can remove remaining issue CRUD/surfaces as a separate large cut.