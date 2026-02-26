+++
id = "630bfe1c-453d-4c5d-98cb-69575c231906"
title = ".ship directory structure — module namespaces"
status = "accepted"
date = "2026-02-26"
tags = []
+++

## Decision

The .ship directory is organised into module namespaces. Each module owns its namespace exclusively.

**Structure:**
```
.ship/
  project/              — project module
    project.toml        — project identity, git config, team policy, agent policy
    vision.md
    overview.md
    notes/
    adrs/
  workflow/             — workflow module  
    releases/
    features/           — agent config lives in feature frontmatter
    specs/
    issues/
      backlog/
      in-progress/
      review/
      blocked/
      done/
      archived/
  agents/               — agents module
    modes/
    skills/
    prompts/
  generated/            — gitignored, V1 target for AI tool configs
  ship.db               — gitignored SQLite
  .gitignore            — managed by Shipwright
```

**Key rules:**
- Status is the folder for issues. No status field in issue frontmatter.
- Feature frontmatter contains [agent] block — MCP servers, skills, model, cost. This is the source of truth for branch agent config.
- Templates are colocated: each document directory may contain TEMPLATE.md
- generated/ is gitignored entirely (one entry). For alpha, native tool config locations are used instead (see ADR: native-ai-tool-config-locations-for-alpha).
- Releases are under project/ (planning artifacts), not workflow/ (execution artifacts). Features reference release IDs.

**Workflow hierarchy:** Vision → Release → Feature → Spec → Issue
- Vision: direction (project/)
- Release: milestone (project/releases/)  
- Feature: chunk of work in a release, owns branch agent config (workflow/features/)
- Spec: detailed plan for a feature (workflow/specs/)
- Issue: atomic work item (workflow/issues/)
