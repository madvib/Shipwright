---
name: workspace-session-lifecycle
description: Run workspace and session lifecycle operations with clean state transitions (start, restart, end, archive), provider/mode correctness, and artifact update tracking. Use this whenever users ask to operate, troubleshoot, or summarize workspace/session flow.
---

# Workspace Session Lifecycle

Use this skill for operational workflow control across workspaces and sessions.

## Use this skill when

- The user asks to start or end a session.
- The user asks to restart after context/config changes.
- The user asks to archive/activate workspace lifecycle state.
- The user asks why workspace/session state appears inconsistent.

## Lifecycle model

- Workspace runtime status: `active` or `archived`
- Session status: `active` or `ended`
- Session captures:
  - `goal` (start intent)
  - `summary` (end narrative)
  - `updated_feature_ids`
  - `updated_spec_ids`

## Standard flow

1. Confirm target workspace branch.
2. Validate provider/mode availability for that workspace.
3. Start session with:
  - `goal`
  - `mode_id` (workspace override or active mode)
  - `provider`
4. During work, keep feature/spec linkage current.
5. End session with:
  - concise `summary`
  - explicit `updated_feature_ids`
  - explicit `updated_spec_ids`

## Restart flow (stale context / compile drift)

Use restart when context generation is stale or provider resolution changed:

1. End current session with restart note.
2. Activate/sync workspace context.
3. Start a new session with the prior goal and a valid provider.

## Archive flow

Archive workspaces when they should remain discoverable but inactive:

1. Transition workspace status to `archived`.
2. Stop active terminal session for that branch if running.
3. Keep links/session history intact.

## Session summary template

Use this shape for high-signal end summaries:

- `Goal completed`: yes/no + brief reason
- `Primary changes`: 1-3 bullets
- `Updated entities`: feature/spec IDs
- `Risks or follow-ups`: 0-2 bullets

## Guardrails

- Do not end a session without a meaningful summary unless user requests minimal output.
- Do not fabricate `updated_feature_ids` or `updated_spec_ids`.
- Keep lifecycle transitions explicit; avoid hidden state changes.
- Prefer clear operational messages over silent failure retries.

## Output contract

When completing lifecycle actions, report:

1. Workspace branch and resulting workspace status
2. Session ID and resulting session status
3. Provider/mode used
4. Updated entity IDs recorded at session end

