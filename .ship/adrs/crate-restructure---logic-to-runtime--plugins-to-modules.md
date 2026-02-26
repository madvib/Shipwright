+++
id = "a01cadf9-c9b1-4a60-b9da-3d9872844a9e"
title = "Crate restructure — logic to runtime, plugins to modules"
status = "accepted"
date = "2026-02-26"
tags = []
+++

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
