+++
id = "9eba0f24-4035-4c5d-bde3-07b91cc11140"
title = "Feature Catalog — Living Product Documentation"
status = "in-progress"
created = "2026-02-27T15:03:56.696826431Z"
updated = "2026-02-27T15:03:56.696826431Z"
adr_ids = []
tags = []
+++

+++
title = "Feature Catalog — Living Product Documentation"
status = "in-progress"
created = "2026-02-27T00:00:00Z"
updated = "2026-02-27T00:00:00Z"
release_id = "v0.1.0-alpha.md"
+++

## Why

Features in Ship are currently thin project-management artifacts — a title, a branch, a body. They don't capture the full lifecycle of a capability: what it does, what version it shipped in, whether it was refactored, whether it's still the source of truth. 

The opportunity: treat features as the canonical **product record** for every capability the project ships. A feature isn't just a work item — it's a living description of what the software does. That catalog becomes the foundation for documentation generation, changelog construction, marketing copy, and agent context ("what does this product actually do?").

## Acceptance Criteria

- [ ] Feature lifecycle statuses: `planned` → `in-progress` → `implemented` → `deprecated` (replaces active/paused/complete/archived)
- [ ] Feature metadata: `version` field (which release it shipped in), `supersedes` (links to replaced feature), `tags`
- [ ] Features are browsable by status in UI (roadmap view = planned + in-progress; catalog view = implemented)
- [ ] Feature body has a required "Description" section — one paragraph suitable for documentation or marketing
- [ ] `ship feature list --status implemented` works in CLI
- [ ] MCP tool `get_feature_catalog` returns all implemented features with descriptions — usable by agent to generate docs
- [ ] `ship feature changelog` generates a markdown changelog grouped by release version
- [ ] Agent workflow: on `ship git sync`, CLAUDE.md includes summary of implemented features so agent understands what the product already does
- [ ] Feature → branch linkage is enforced on `ship feature start <id>` (creates + checks out branch)
- [ ] Feature template updated to include Description, Acceptance Criteria, Implementation Notes sections

## Delivery Todos

- [ ] Update `FeatureMetadata` to add `version: Option<String>`, `supersedes: Option<String>`, `tags: Vec<String>`, new status enum
- [ ] Migrate existing features: `active` → `in-progress`, `complete` → `implemented`, `archived` → `deprecated`
- [ ] Update feature template (`FEATURE.md`) with richer section structure
- [ ] Add `get_feature_catalog` MCP tool — returns Vec<FeatureSummary> filtered to implemented
- [ ] Add `feature changelog` CLI command — groups implemented features by `version`, formats as markdown
- [ ] Include implemented feature summaries in CLAUDE.md generation (brief, not full body)
- [ ] `ship feature start <id>` — creates branch, links it, checks it out
- [ ] `ship feature done <id>` — marks implemented, sets version from active release
- [ ] Spec: Feature Catalog — document full data model and generation pipeline
- [ ] Tests: status migration, catalog MCP tool, changelog generation

## Notes

**Status model:**
- `planned` — on the roadmap, not started
- `in-progress` — branch exists, work happening
- `implemented` — merged, in a release
- `deprecated` — superseded or removed, kept for history

**The description field is the key investment.** One paragraph per feature, kept current. Agents can be prompted to update it when they complete a feature. Aggregate across all implemented features = instant product overview.

**Documentation pipeline (future):** `get_feature_catalog` → agent prompt → structured docs / marketing landing page. Ship doesn't generate the docs — it maintains the catalog that makes generation trivial.

**Versioning:** `version = "v0.1.0-alpha"` on the feature record. When a feature is refactored significantly, create a new feature that `supersedes` the old one. The old one stays in history; the new one is the live description.
