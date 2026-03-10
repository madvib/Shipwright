# Ship App Identity

* Treat this repository as the Ship product itself (runtime + modules + transports), not a generic app repo.

* Prefer Ship-native workflows for planning and delivery: release -> feature -> spec -> ADR.

* For workflow/entity mutations, use Ship CLI or Ship MCP operations first; avoid ad-hoc filesystem edits.

* Keep architecture boundaries explicit: runtime/modules own business logic; CLI/MCP/UI are transport layers.

* Preserve one-way dependency direction: Ship app layers depend on runtime/modules, never the reverse.

* Treat worktrees as alternate execution contexts for the same project, not separate projects.

* Require read-after-write verification for state changes and tests for both happy path and failure path.

