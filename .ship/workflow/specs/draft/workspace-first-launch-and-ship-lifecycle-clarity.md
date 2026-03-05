+++
id = "Lr9kW2pQ"
title = "Workspace-First Launch and Ship Lifecycle Clarity"
status = "draft"
created = "2026-03-05T20:20:00Z"
updated = "2026-03-05T21:09:05Z"
author = ""
tags = ["launch", "workspace", "ux", "workflow", "messaging"]
+++

## Overview

Ship has hardened many mechanical layers (runtime persistence, agent config import/export, MCP/CLI/UI transports, file + SQLite operations), but the launch risk is now mostly product coherence. We need one clear message, one clear default workflow, and one clear "first 10 minutes" experience.

This product competes near heavyweight developer tools, so unclear positioning and workflow friction are launch-critical risks.

This spec sets the launch direction:

- Shape runtime: vertical-agnostic intent + context + provisioning engine.
- Ship app: full-lifecycle software development system built on the runtime.
- Workspace: primary unit connecting project context to agent operating context.

## Goals

- Make workspace the default center of gravity across UI, CLI, and MCP.
- Define a clear lifecycle story users can follow without guessing.
- Resolve or explicitly stage open questions in entity hierarchy (feature/spec/issue/release/tasks).
- Make `ship init` and onboarding flow obvious and low-friction.
- Keep provider/agent integration seamless in normal daily flow.
- Publish a launch narrative that clearly distinguishes:
  - Shape (vertical-agnostic runtime/AgentOS)
  - Ship (software lifecycle application built on Shape)

## Non-Goals

- Full redesign of all entity schemas in one release.
- Cloud workspace execution scope (tracked in separate feature).
- Marketplace rollout for skills/MCP in this spec.

## Product Message (Launch)

### Shape Runtime

Shape is a vertical-agnostic runtime that:

- captures intention,
- resolves the right context/tools,
- provisions them in the active environment.

### Ship

Ship is the software lifecycle application built on Shape. It should let teams:

- model software work end-to-end,
- operate through workspace-first flows,
- collaborate with agents without context drift.

## Where We Are Still Weak

### Messaging + first-use friction

- Runtime/app distinction is not consistently explained in UX or docs.
- Init does not always make "what to do next" obvious enough.
- New users can still miss workspace-first flow and fall into fragmented commands.

### Workflow model ambiguity

- Features carry too much planning and execution responsibility.
- Specs are used inconsistently and often treated as disposable scratch docs.
- Issues are underused in day-to-day flow.
- "Release" may be the wrong primary mental model for many teams (milestone/target may fit better).

### Cross-surface smoothness

- UI/CLI/MCP are mechanically capable but not always aligned around the same default journey.
- Context recompilation/refresh expectations are not always explicit when mode/workspace changes.

## Workspace-First Direction

Workspaces are the bridge between:

- project lifecycle context (features/specs/issues/releases),
- agent context compilation (skills/rules/providers/mcp/mode),
- execution environment (branch/worktree and provider exports).

### Launch Expectations

- UI default landing view is Workspaces.
- CLI/MCP default flows orient around workspace creation, switching, activation, and sync.
- On workspace activation, agent context recompiles and provider artifacts are refreshed.
- Workspace state is never "side metadata"; it is a first-class operational primitive.
- Help text across surfaces points to the same workspace-first happy path.

## Current Gaps and Risks

### 1. Workflow story is not yet obvious

- Users can perform commands, but "where to start" is still ambiguous.
- Init + first workflow path needs stronger guidance and defaults.

### 2. Entity boundaries are fuzzy

- Features currently carry too much responsibility.
- Specs are often treated as disposable.
- Issues are underused in active execution.
- Release semantics may be better represented as milestone/target in some flows.

### 3. Cross-surface consistency still needs polish

- UI/CLI/MCP expose similar capability but not always in the same conceptual order.
- Workspace-first behavior must be equally explicit in all three surfaces.

## Decisions for Launch Track

- Workspace is the primary operational view and top-level journey.
- Preserve current entities for alpha, but document a single recommended hierarchy and flow.
- Avoid silent compatibility shims when changing UX and command flows.

## Proposed Alpha Workflow Story

1. `ship init`
2. Choose target (`release` vs `milestone`) and create/select feature
3. Create workspace from feature (branch/worktree + mode)
4. Context compile + provider export on activation
5. Execute through spec/task units with agent support
6. Close workspace -> roll up into feature and target

## Entity Decision Track (Must Resolve)

1. Keep `release` as top-level planning construct, or shift to `milestone/target`.
2. Define role of specs:
   - top-level namespaced entity, or
   - feature-scoped execution unit.
3. Define where tasks/todos live:
   - under feature,
   - under spec,
   - or dual-link model.
4. Explicitly decide if issues remain first-class in alpha default flow.

## Open Questions to Resolve Next

- Should release remain primary, or rename/reframe as milestone/target?
- Should spec be top-level lifecycle entity or nested/optional under feature?
- Should task/todo live as first-class entity under feature/spec (or both)?
- What is the minimum mandatory hierarchy for a smooth default flow?

## Execution Plan (Immediate)

1. Define and document canonical workspace-first happy path for UI/CLI/MCP.
2. Promote workspace view/commands/tools as default entry points.
3. Draft and ratify entity hierarchy decision ADR (feature/spec/issue/release/task).
4. Refine init output/help text to point users into the canonical first flow.
5. Add end-to-end tests for "new user -> init -> workspace -> agent-assisted execution".
6. Ensure MCP tool guidance mirrors CLI onboarding language for same flow.

## Acceptance Criteria

- A new user can follow one documented workspace-first path without ambiguity.
- UI defaults to workspace-centered operations.
- CLI and MCP expose equivalent workspace-first primitives in naming and flow.
- Entity hierarchy decision is captured and reflected in docs + command/tool guidance.
- At least one end-to-end test validates the full launch workflow path.
- Init output explicitly routes users into workspace-first execution steps.
