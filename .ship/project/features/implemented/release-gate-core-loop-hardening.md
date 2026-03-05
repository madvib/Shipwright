+++
id = "KPmx9VzP"
title = "Release Gate: Core Loop Hardening"
created = "2026-03-04T03:00:48.730307+00:00"
updated = "2026-03-04T19:07:21+0000"
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
- [x] Hardened active-mode export target handling:
  - `sync_active_mode` now normalizes and dedupes `target_agents`
  - unknown `target_agents` are skipped with a warning instead of aborting mode sync
  - `set_active_mode` now logs sync failures instead of silently swallowing them
  - added runtime coverage for default empty-target behavior and unknown-target skip behavior
- [x] Hardened feature-level provider override handling:
  - provider IDs are normalized/deduped and unknown values are filtered from resolved agent config
  - invalid feature provider overrides now fall back to project providers
  - invalid project-level provider lists now fall back to `claude` for safe default execution
  - added regression coverage to ensure checkout skips unknown feature provider IDs and still exports valid targets
  - added regression coverage to ensure checkout still works when project provider config is malformed
- [x] Full `cargo test -p e2e` green with new suites included.
- [x] Hardened release CLI read/update consistency:
  - `release get <id>` now exits non-zero when the release does not exist.
  - release updates now overwrite the canonical markdown file instead of creating suffixed duplicates.
  - added module + e2e regressions for release create/update/get round-trip and duplicate-file prevention.
- [x] Added workspace failure-path rollback coverage:
  - failed `workspace create --checkout` does not persist stale workspace rows
  - failed `workspace create --worktree` does not persist stale workspace rows
- [x] Hardened workspace worktree metadata consistency:
  - recreating/updating a workspace with `is_worktree=false` now clears stale `worktree_path` state
  - worktree workspace records now require a non-empty `worktree_path`
  - CLI now rejects `workspace create --worktree-path ...` when `--worktree` is not set
  - CLI now rejects conflicting `workspace create --checkout --worktree` flag combinations
  - failed `workspace create --worktree` now rolls back newly-created git branches when worktree add fails
  - added regression coverage to ensure failed worktree creation does not leave dangling git branches
  - added regression coverage to ensure failed worktree creation preserves pre-existing branches
  - added runtime + e2e regression coverage for worktree-to-non-worktree metadata cleanup

## Next

- [x] Add explicit edge-case e2e coverage for disallowed workspace transitions and expected error messaging.
- [x] Add branch-context hydration edge cases for mixed feature/spec links when switching between branches/worktrees.
- [x] Add workspace failure-path e2e coverage for git checkout/worktree creation errors and rollback expectations.
- [x] Add resolver unit coverage for relative git worktree pointer paths mapping back to the main checkout `.ship`.
- [x] Release-gate hardening scope complete.
