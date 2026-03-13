---
name: start-session
description: Define or refine concrete feature intent, ensure workspace links are correct, and launch a tracked workspace session with the right goal/provider. Use this when users ask to "start a session" but the workspace context is incomplete.
---

# Start Session

Use this skill when the user wants to begin execution and needs orchestration across feature intent, workspace, and session setup.

## Use this skill when

- The user asks to start working but the feature intent is missing, vague, or stale.
- The user asks to "bootstrap" a session from feature intent.
- The user asks for a guided start flow that ends in an active session.

## Orchestration flow

1. Resolve target workspace branch (or create/select one if missing).
2. Gather feature intent:
  - problem statement
  - scope boundaries
  - acceptance criteria
  - risks/constraints
3. Create or update the feature with concise, testable content.
4. Ensure workspace links are correct (`feature_id`, optional `release_id`).
5. Resolve provider for the workspace.
6. Start session with explicit goal and selected provider.
7. Return session ID + workspace linkage summary.

## Goal template

Use this shape when setting `goal`:

- `Implement <capability> for <user outcome>; validate with <acceptance criteria summary>.`

## Guardrails

- Do not start a session without a concrete goal unless user explicitly requests minimal setup.
- Do not fabricate link IDs; use discovered IDs or create them first.
- Keep acceptance criteria measurable.
- If provider resolution is empty, stop and fix workspace/provider config before starting.

## Output contract

When complete, report:

1. Workspace branch + provider used
2. Feature ID created/updated
3. Linked feature/release IDs
4. Session ID and active status
