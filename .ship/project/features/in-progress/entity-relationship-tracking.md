+++
id = "E5Tf2KNN"
title = "Entity Relationship Tracking"
created = "2026-02-28T15:56:07Z"
updated = "2026-02-28T15:56:07Z"
branch = ""
release_id = "v0.1.0-alpha"
spec_id = ""
adr_ids = []
tags = []

[agent]
model = "claude"
max_cost_per_session = 10.0
mcp_servers = []
skills = []
+++

## Why

Shipwright documents don't exist in isolation — a spec belongs to a feature, a feature belongs to a release, an ADR is motivated by a spec. These relationships need to be stable, machine-readable, and resolve correctly even when files are renamed or moved. Cross-references via short IDs (not filenames) provide this guarantee and enable Shipwright to surface the right context automatically.

## Acceptance Criteria

- [ ] All cross-references use short ID (8-char nanoid), never filename or title
- [ ] Feature: `release_id`, `spec_id`, `adr_ids`
- [ ] Spec: `feature_id`, `release_id`
- [ ] ADR: `spec_id`, `supersedes_id`
- [ ] Issue: `spec_id`, `feature_id`
- [ ] Release: `feature_ids`, `adr_ids`
- [ ] `get_project_info` resolves all relationships and returns a linked object graph
- [ ] Broken references (ID with no matching file) are reported, not silently ignored

## Delivery Todos

- [ ] Audit all entity structs to confirm ID-based refs (no filename strings)
- [ ] Implement reference resolution in `project.rs` or a dedicated `relations.rs`
- [ ] `get_project_info` returns fully resolved graph (already partially implemented — verify)
- [ ] CLI `ship feature show` resolves and displays linked spec/release
- [ ] Broken reference detection and reporting

## Notes

Direction of reference: Feature points to Release and Spec. Release points to Features. This is deliberate — the higher-level document knows what it contains; the lower-level document knows what it belongs to. Bidirectional is not required; resolution builds it on demand.
