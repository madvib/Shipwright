+++
id = "KPmx9VzP"
title = "Release Gate: Core Loop Hardening"
created = "2026-03-04T03:00:48.730307+00:00"
updated = "2026-03-04T07:24:44+0000"
tags = []
+++

## Progress

- [x] Worktree project identity resolves to main checkout `.ship` even when worktree has a local `.ship` copy.
- [x] Registry load auto-normalizes and dedupes existing project entries (`projects.json`) and preserves custom naming.
- [x] Added command-level e2e coverage for `ship projects` identity/name behavior.
- [x] Added command-level e2e coverage for workspace lifecycle happy paths:
  - create/list/archive
  - checkout activation and active-workspace demotion
  - worktree creation metadata
- [x] Full `cargo test -p e2e` green with new suites included.
- [x] Added workspace failure-path rollback coverage:
  - failed `workspace create --checkout` does not persist stale workspace rows
  - failed `workspace create --worktree` does not persist stale workspace rows

## Next

- [x] Add explicit edge-case e2e coverage for disallowed workspace transitions and expected error messaging.
- [x] Add branch-context hydration edge cases for mixed feature/spec links when switching between branches/worktrees.
- [x] Add workspace failure-path e2e coverage for git checkout/worktree creation errors and rollback expectations.
