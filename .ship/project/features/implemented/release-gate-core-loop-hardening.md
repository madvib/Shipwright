+++
id = "KPmx9VzP"
title = "Release Gate: Core Loop Hardening"
created = "2026-03-04T03:00:48.730307+00:00"
updated = "2026-03-04T15:05:56+0000"
tags = []
+++

## Progress

- [x] Worktree project identity resolves to main checkout `.ship` even when worktree has a local `.ship` copy.
- [x] Registry load auto-normalizes and dedupes existing project entries (`projects.json`) and preserves custom naming.
- [x] Added command-level e2e coverage for `ship projects` identity/name behavior.
- [x] Added command-level e2e coverage for worktree identity parity across `ship projects` ops:
  - `projects rename <worktree-path>` updates the canonical main-project registry row
  - `projects untrack <worktree-path>` removes the canonical main-project registry row
- [x] Added command-level e2e coverage for workspace lifecycle happy paths:
  - create/list/archive
  - checkout activation and active-workspace demotion
  - worktree creation metadata
- [x] Hardened branch-switch teardown for feature-level provider overrides:
  - switching from a feature with `providers = [\"codex\"]` now tears down stale Codex outputs even when project defaults differ
  - unmanaged/manual `AGENTS.md` content is preserved on non-feature branch sync
- [x] Hardened generated artifact protections for Codex:
  - root `.gitignore` now includes `AGENTS.md` and `.agents/`
  - pre-commit hook blocks staging `AGENTS.md` and `.agents/` outputs
  - added e2e parity check that Codex-generated files stay ignored
  - added pre-commit regression coverage for both `AGENTS.md` and `.agents/` staging attempts
- [x] Full `cargo test -p e2e` green with new suites included.
- [x] Added workspace failure-path rollback coverage:
  - failed `workspace create --checkout` does not persist stale workspace rows
  - failed `workspace create --worktree` does not persist stale workspace rows

## Next

- [x] Add explicit edge-case e2e coverage for disallowed workspace transitions and expected error messaging.
- [x] Add branch-context hydration edge cases for mixed feature/spec links when switching between branches/worktrees.
- [x] Add workspace failure-path e2e coverage for git checkout/worktree creation errors and rollback expectations.
- [x] Add resolver unit coverage for relative git worktree pointer paths mapping back to the main checkout `.ship`.
- [x] Release-gate hardening scope complete.
