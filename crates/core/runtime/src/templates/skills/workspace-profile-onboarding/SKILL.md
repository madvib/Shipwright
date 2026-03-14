---
name: workspace-profile-onboarding
description: Initialize a project for effective workspace execution by connecting providers, selecting a profile seed (`environment_id`), creating or activating a workspace, and starting the first session. Use this whenever users ask to "set up", "initialize", "get started", "bootstrap", or "start working" in Ship.
---

# Workspace Profile Onboarding

Use this skill to turn a newly opened project into an execution-ready workspace with clear profile intent.

## Use this skill when

- The user wants first-run setup.
- The user asks what `environment_id` means or how profiles should work.
- The user wants a smooth start into the planning → workspace → session loop.

## Core model clarification

- Every workspace has its own runtime configuration.
- `environment_id` is a profile seed, not a shared mutable runtime toggle.
- Profiles are reusable presets that initialize workspace settings; after creation, workspace state is independent.

## Onboarding flow

1. Validate provider readiness:
   - `ship providers list`
   - `ship providers detect` (if none are connected)
   - `ship providers connect <id>` (if user wants explicit provider selection)
2. Confirm profile seed intent:
   - pick an existing preset profile ID, or
   - proceed with no profile seed and explicit workspace config
3. Create or activate workspace:
   - create: `ship workspace create <branch> --type <feature|patch|service> --environment-id <profile_id>`
   - activate: `ship workspace switch <branch>`
4. Start first session:
   - `ship workspace session start --branch <branch> --goal "<goal>" --provider <provider>`
5. Confirm ready state:
   - active workspace
   - active session id
   - provider/mode currently in effect

## UX guardrails

- Do not describe profiles as a global runtime "environment switch."
- Do not imply that changing a profile mutates existing workspaces automatically.
- If profile input is unclear, propose one concrete default profile and move forward.
- Keep initialization copy concise and operational.

## Output contract

When complete, report:

1. Workspace branch + workspace type
2. Profile seed used (`environment_id`) or explicit "none"
3. Connected provider(s) and selected session provider
4. Session status (`active` with session id, or why not started)
