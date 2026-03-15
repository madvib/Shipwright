# Development Workflow

* ARCHITECTURE.md is the context firewall. Read it before touching code.

* Platform primitives (preset, session, skill, MCP, permission, event) are stable contracts. Workflow types (feature, release, spec, issue) are not platform — do not add them to platform code.

* Record architecture decisions by updating ARCHITECTURE.md directly.

* Events are append-only. Never update or delete event records.

* File length cap: 300 lines per file. New modules require tests.

* `ship use [<preset-id>]` is the only activation command — installs deps, activates preset, emits all provider files. No separate compile step.

* Provider output files (CLAUDE.md, .mcp.json, .cursor/, etc.) are generated artifacts — gitignored, never committed. `.ship/` is the source of truth.
