+++
id = "TSHi25on"
title = ".ship directory structure — module namespaces"
date = "2026-02-28"
spec_id = ""
supersedes_id = ""
tags = []
+++

## Context

As Shipwright's document types grew, the `.ship/` directory needed a clear ownership model. Without namespacing, different document types would compete for top-level directories and the git/ignore policy would become complex. The decision also needed to settle where releases and features live — planning artifacts vs execution artifacts — and where agent configuration lives relative to the document it configures.

## Decision

The `.ship/` directory is organised into module namespaces. Each module owns its namespace exclusively.

**Current structure:**
```
.ship/
  ship.toml             — project config (git policy, providers, active mode defaults)
  project/              — project module
    VISION.md           — singleton, no frontmatter
    notes/
    adrs/
      proposed/
      accepted/
      rejected/
      superseded/
      deprecated/
    releases/
    features/           — features live here (moved from workflow/ in alpha-dogfood)
      planned/
      in-progress/
      implemented/
      deprecated/
  workflow/             — workflow module
    specs/
      draft/
      active/
      archived/
    issues/
      backlog/
      in-progress/
      done/
  agents/               — agents module
    mcp.toml            — MCP server registry
    permissions.toml    — agent permissions
    skills/             — directory-based: <id>/index.md + skill.toml
    rules/              — always-active rules: *.md
    modes/              — named agent config presets
  generated/            — gitignored, reserved for V1
  ship.db               — gitignored SQLite (project runtime state)
  .gitignore            — managed by Shipwright
```

**Key rules:**
- Status is the folder, not a frontmatter field — applies to features, specs, ADRs, issues
- Feature frontmatter contains `[agent]` block — skills, mcp_servers, optional model override
- Templates are colocated: each document directory may contain `TEMPLATE.md`
- `generated/` is gitignored entirely. For alpha, native tool config locations are used (see ADR: native-ai-tool-config-locations-for-alpha)
- Features live under `project/features/` (planning artifacts tied to a release), not `workflow/`
- Releases live under `project/releases/`

**Workflow hierarchy:** Vision → Release → Feature → Spec → Issue
- Vision: direction (`project/VISION.md`)
- Release: milestone (`project/releases/`)
- Feature: chunk of work in a release, owns branch agent config (`project/features/`)
- Spec: detailed plan for a feature (`workflow/specs/`)
- Issue: atomic work item (`workflow/issues/`)

## Consequences

### Positive
- Clear ownership: any document type has exactly one home
- Git policy is simple: `project/` and `workflow/specs/` committed; `workflow/issues/`, `agents/`, `ship.db` gitignored
- Module boundaries map directly to `crates/modules/` sub-crates
- Colocated templates mean new projects get the right template without configuration

### Negative
- Features moved from `workflow/features/` to `project/features/` mid-alpha — existing cross-references needed updating
- Two levels of nesting (module → status directory) can feel deep for simple projects
- The `generated/` directory is reserved but unused in alpha — may confuse users who find it empty
