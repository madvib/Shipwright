+++
id = "a24JC7g9"
title = "Feature Catalog — Data Model and Generation Pipeline"
created = "2026-02-27T15:04:26.185199422Z"
updated = "2026-02-27T15:04:26.185199422Z"
tags = []
+++

# Feature Catalog — Data Model and Generation Pipeline

## Overview

Features are the canonical product record for every capability Ship tracks. This spec defines the enriched data model, lifecycle, CLI/MCP surface, and documentation generation pipeline.

## Status Lifecycle

```
planned → in-progress → implemented → deprecated
                ↓
           (abandoned)
```

| Status | Meaning | Branch |
|---|---|---|
| `planned` | Roadmap item, not started | none |
| `in-progress` | Active branch, work underway | required |
| `implemented` | Merged to main, in a release | archived |
| `deprecated` | Superseded or removed | — |
| `abandoned` | Cancelled, kept for history | — |

**Migration from current:** `active` → `in-progress`, `complete` → `implemented`, `archived` → `deprecated`.

## Enriched FeatureMetadata

```toml
+++
id = "uuid"
title = "Feature Title"
status = "planned"          # planned | in-progress | implemented | deprecated | abandoned
created = "2026-01-01T00:00:00Z"
updated = "2026-01-01T00:00:00Z"

# Lifecycle
version = "v0.1.0-alpha"   # release this shipped in (set on → implemented)
supersedes = "old-feature-id"  # replaces a prior feature record

# Git integration
branch = "feature/my-branch"   # linked branch

# Organization
tags = ["auth", "api"]
release = "v0.1.0-alpha.md"   # linked release doc
spec = "my-spec.md"            # linked spec

# Agent config (unchanged)
[agent]
skills = [{id = "task-policy"}]
providers = []
+++
```

## Feature Body — Required Sections

The FEATURE.md template enforces this structure:

```markdown
## Description

One paragraph. What this feature does, for whom, and why it matters.
Written to be used directly in documentation or marketing copy.
Keep it current — agents should update this on completion.

## Acceptance Criteria

- [ ] ...

## Implementation Notes

Key technical decisions, constraints, or context an agent needs.
Not a full spec — just the decisions that affect ongoing work.

## History

- v0.1.0-alpha — initial implementation
- v0.2.0 — refactored to use provider registry
```

## MCP Tools

### `get_feature_catalog`

Returns all `implemented` features as a structured summary list. Used by agents to understand what the product does.

```
Input: { status?: string }   # default: "implemented"; pass "all" for full catalog
Output: Vec<FeatureSummary { id, title, status, version, tags, description }>
```

The `description` field is extracted from the "## Description" section of the feature body.

### `get_project_overview`

Combines `get_project_info` + `get_feature_catalog` into a single call. Gives an agent a complete picture: open issues, active features, and what the product already does.

## CLI

```
ship feature list [--status planned|in-progress|implemented|all]
ship feature start <file>      # create + checkout branch, set status=in-progress
ship feature done <file>       # set status=implemented, stamp version from active release
ship feature deprecate <file>  # set status=deprecated, optionally --superseded-by <id>

ship feature changelog [--release v0.1.0-alpha]   # markdown changelog grouped by version
ship feature catalog           # print all implemented features with descriptions
```

## CLAUDE.md Integration

`generate_claude_md` currently includes open issues and skills. It should also include a **"What This Product Does"** section: the description paragraph from each `implemented` feature, grouped by release version.

```markdown
## What This Product Does

### v0.1.0-alpha

- **MCP Server** — Persistent project memory across agent sessions via ...
- **Git Hook** — Auto-generates CLAUDE.md + .mcp.json on branch checkout ...
- **Feature Catalog** — Living product record for every capability ...
```

This gives agents starting cold sessions immediate product context without reading all feature files.

## Documentation Pipeline

The catalog enables external generation without Ship doing any rendering:

1. Agent calls `get_feature_catalog`
2. Agent is prompted to generate docs / changelog / landing page copy
3. Output written to project (e.g. `docs/features.md`, `CHANGELOG.md`)

Ship maintains the data. Generation is an agent prompt, not a Ship feature. The pipeline works with any MCP-connected agent.

## File Layout

Features remain in `.ship/workflow/features/`. Status is the frontmatter field, not the folder — unlike issues, features are not kanban items and don't need folder-based status routing.

## Open Questions

- Should `ship feature start` also create the git branch, or just link an existing one?
- Should `description` be a separate frontmatter field (easy to extract) or always inferred from the `## Description` section? Frontmatter is cleaner for the catalog API but adds duplication.
- Status folder vs frontmatter: keep in frontmatter (current) to avoid file movement churn on status changes. Features move less frequently than issues.
