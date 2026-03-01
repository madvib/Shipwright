+++
id = "WetzhDnY"
title = "Crate restructure — logic to runtime, plugins to modules"
date = "2026-02-28"
spec_id = ""
supersedes_id = ""
tags = []
+++

## Context

The original crate layout (`crates/logic`, `crates/plugins/`) didn't communicate the intended layering — runtime primitives mixed with module-specific logic, and "plugins" implied third-party extension before the API was stable. As the codebase grew, it became unclear where new code should live. The crate names needed to reflect the actual architecture: a stable foundation layer and optional capability modules built on top.

## Decision

Rename and restructure the workspace crates to reflect the intended architecture.

**Current:** crates/logic, crates/plugins/{time-tracker,ghost-issues}

**Target:**
```
crates/runtime/           — core types, file I/O, project primitives (was logic)
crates/modules/
  project/                — vision, notes, ADRs, releases
  workflow/               — features → specs → issues  
  agents/                 — modes, skills, prompts, agent config export
  git/                    — hooks, worktrees, context generation
```

**Migration strategy:** Incremental, not big-bang.
1. Rename logic → runtime (path change + re-export, CI stays green)
2. Create crates/modules/ shell crates
3. New features land in new structure immediately
4. Old code migrates as touched, not all at once

**Rationale:** The current flat structure doesn't communicate the layering. runtime is the foundation everything else builds on. modules are optional capabilities that register document types, MCP tools, and CLI commands. This maps directly to how the codebase will grow — third-party modules eventually slot in alongside first-party ones.

**Three first-class interfaces:** CLI, MCP, UI all depend on runtime + modules. No interface owns business logic. All state lives in runtime.

## Consequences

### Positive
- Crate names communicate the architecture to new contributors
- Clear separation: runtime is stable, modules are the extension point
- Incremental migration preserves CI green throughout
- Third-party modules have a natural home alongside first-party ones when the time comes

### Negative
- More crates = more `Cargo.toml` maintenance
- The module trait is not yet dynamically dispatched — "module" is currently a convention, not a runtime concept
- Incremental migration means old code co-exists with new structure during the transition period
