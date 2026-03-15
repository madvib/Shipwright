# Development Workflow

* ARCHITECTURE.md is the context firewall. Read it before touching code.

* Platform primitives (workspace, session, event) are stable contracts. Workflow types (feature, release, spec, issue) are not platform — do not add them to platform code.

* For state changes: use `ship workspace` and `ship session` CLI commands. Do not use `ship feature`, `ship spec`, `ship release`, or `ship adr` — these reference the old workflow layer which is being rebuilt.

* Record architecture decisions by updating ARCHITECTURE.md directly, not by creating ADR files through the CLI.

* Events are append-only. Never update or delete event records.

* File length cap: 300 lines per file. New modules require tests.
