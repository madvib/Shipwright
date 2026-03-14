---
name: release-orchestration
description: Plan and execute release workflows end-to-end by creating release records, linking specs/features, managing readiness signals, and driving closeout updates. Use this whenever users ask to ship, stage, or coordinate a release.
---

# Release Orchestration

Use this skill for multi-entity release coordination instead of ad-hoc document edits.

## Use this skill when

- The user asks to create or run a release plan.
- The user asks to attach features/specs to a release.
- The user asks for release readiness tracking or ship checklist flow.
- The user asks to clean up release metadata and status transitions.

## Release orchestration phases

### 1. Initialize release

Set:

- `version` (required)
- `status` (`planned`, `active`, `shipped`, `archived`)
- `supported` (boolean)
- `target_date` (optional)
- `tags` (optional)

Draft body sections:

- Goal
- Scope
- Included Features
- Breaking Changes
- Notes

### 2. Attach scope

- Link existing features/specs to the release.
- Create missing features/specs when required by scope.
- Keep links canonical by ID.

### 3. Drive execution

- Use workspace/session lifecycle to track implementation activity.
- Ensure session closeout records updated feature/spec IDs.
- Keep release notes synchronized with major scope changes.

### 4. Readiness review

Evaluate:

- Feature completion state
- Blockers or missing acceptance criteria
- Breaking-change clarity
- Remaining risks

### 5. Closeout

- Update release status for final state.
- Ensure final summary is accurate and concise.
- Record follow-up work for deferred scope.

## Guardrails

- Do not treat release notes as the only source of truth for status/support/date.
- Keep release metadata structured and queryable where supported.
- Never mark ready-to-ship without calling out open blockers.
- Separate shipped scope from deferred scope clearly.

## Output contract

For each release operation, report:

1. Release identifier/version
2. Metadata set or changed
3. Features/specs linked or created
4. Readiness status and blockers
5. Explicit next action

