---
name: ship-workflow
description: Run delivery through Ship entities with verified state transitions, linkage integrity, and completion gates.
metadata:
  display_name: Ship Workflow
  source: builtin
---

# Ship Workflow

Use this skill for work that changes project state, not just source code.

## When To Apply

Apply this skill when a task touches any of:

- release scope, release readiness, or release status
- feature lifecycle, acceptance criteria, or delivery TODOs
- spec creation/update/activation/archive
- issue creation, movement, closure, or dependency changes
- ADR creation for architecture-impacting decisions
- workspace, branch, or worktree linkage to active work

## System Of Record Contract

- Ship entities are the canonical record for lifecycle status.
- Prefer Ship CLI or Ship MCP tools for state mutations.
- Do not manually move files between workflow folders unless explicitly doing recovery work.
- After each mutation, re-read state and verify expected delta.

## Required Execution Loop

For each state-changing action:

1. Read current state and collect the relevant entity IDs.
2. Perform one mutation (single intent).
3. Re-read state and confirm the mutation landed.
4. Record linkage integrity (release <-> feature <-> spec <-> issue).

Do not batch unrelated mutations into one blind operation.

## Canonical Delivery Sequence

1. Align release intent:
Ensure a target release exists for scoped delivery.
2. Align feature contract:
Create or attach the feature that owns the work.
3. Align specification:
Use a spec for non-trivial behavior, architecture, data, or API changes.
4. Execute with issues:
Track implementation tasks as issues linked to feature/spec.
5. Close with evidence:
Only close entities after tests and acceptance checks pass.

## Link Integrity Rules

- Keep release, feature, spec, and issue links coherent at all times.
- Do not close a feature while linked issues remain open.
- Do not close a release with open in-scope features unless explicitly deferred and documented.
- Move architecture-level decisions into ADRs, not issue comments.

## Workspace And Branch Discipline

- Keep active branch/workspace mapped to active feature/spec.
- Treat worktrees as execution contexts for one project, not separate projects.
- If branch linkage is stale or incorrect, correct linkage before coding.

## Verification Gates

Before marking done:

- run tests for touched behavior
- cover happy path and at least one meaningful failure path
- verify sync/import/migration actions are idempotent where applicable
- confirm docs/spec/ADR updates for changed behavior or decisions

## Completion Output

On completion, report:

- entities changed (IDs and final statuses)
- verification evidence (tests/checks run)
- open risks, follow-ups, or deferred decisions

## Anti-Patterns

- closing entities based on chat memory without state verification
- bypassing entity links and tracking work in ad-hoc notes
- marking done in code while workflow state remains stale
