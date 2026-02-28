+++
id = "5jQYMG42"
title = "Alpha Feature Delivery Workflow"
created = "2026-02-28T21:15:00Z"
updated = "2026-02-28T21:15:00Z"
tags = []
+++

# Alpha Feature Delivery Workflow

**Status:** Active  
**Last Updated:** 2026-02-25

---

## Why This Exists

Shipwright will ship one opinionated workflow in alpha to avoid ambiguous semantics and half-baked customization.

The target is reliable end-to-end delivery with strong agent context and low git noise.

---

## Opinionated Workflow

`Vision -> Release -> Feature -> Spec -> Issues -> ADRs -> Close Feature -> Ship Release`

### Step Rules

1. **Vision**
- One project-level document defining goals, boundaries, and constraints.
- Updated only when project direction changes.

2. **Release**
- Canonical version-scoped container (for example: `v0.1.0-alpha`).
- Holds the set of features intended for that release.
- Treated as durable project memory and committed by default.

3. **Feature**
- Primary delivery unit.
- Stored as markdown with checklist-style todos.
- Represents progress and acceptance criteria at a level above individual issues.

4. **Spec**
- Contract for feature implementation details.
- Updated during planning and major scope/decision shifts.

5. **Issues**
- Execution scratch units for humans/agents.
- Default local-only to reduce noisy git history.
- Promotable to committed artifacts when needed.

6. **ADRs**
- Durable architecture decisions tied to feature/spec context.
- Always git-committed by default.
- May include MCP integration notes (command/config references) as context.

7. **Close Feature**
- Confirm acceptance criteria.
- Summarize shipped deltas.
- Link ADRs/spec snapshots as historical record.

8. **Ship Release**
- Close release once committed features meet acceptance criteria.
- Record release notes and final context links.

---

## Primitive Semantics

| Primitive | Scope | Primary Role | Git Default |
| --- | --- | --- | --- |
| Vision | Project | Long-lived strategic context | Committed |
| Release | Project | Canonical version scope | Committed |
| Feature | Project | Delivery container + markdown todos | Committed |
| Spec | Feature | Implementation contract | Committed |
| Issue | Feature execution | Short-lived execution tasks | Local-only |
| ADR | Project/Feature | Durable technical decisions | Committed |
| Mode | Agent runtime | Tooling/policy profile for agents | Config-managed |
| Event | Global + project | Traceability and automation substrate | Mixed |

---

## Git Policy (Alpha Default)

Committed by default:

- `releases`
- `features`
- `specs`
- `adrs`
- `config.toml`
- `templates`

Local-only by default:

- `issues`
- `log.md`
- `plugins`

Rationale: keep canonical planning/decision artifacts in git; keep execution chatter local.

---

## Mode + Workflow Policy

Mode remains an **agent concern**, not a PM primitive.

Workflow policy integrates with mode via context:

- Current workflow phase
- Expected transition rules
- Allowed/disallowed actions per phase
- Suggested mode for that phase (manual switch in alpha)

Deferred:

- Automatic mode switching based on checked-out feature.

---

## Logic Crate Concern Model

The `logic` crate should be organized by concerns, not by surface:

1. `io` — filesystem/document IO, parsing, atomic writes.
2. `domain` — core entities (vision/release/feature/spec/issue/adr/mode).
3. `workflow` — transition and policy engine.
4. `events` — append-only event model and projections.
5. `app` — use-case orchestration for CLI/UI/MCP.
6. `adapters` — external/export integration points.

This enables shared correctness across CLI, MCP, and UI.

---

## Event Model Direction

Events are not project-only in the long term.

Alpha should preserve compatibility with a future **global event service**:

- project-scoped event streams today
- global aggregation/projection service later
- no breaking contract changes across the transition

---

## Customization Trajectory (Post-Alpha)

Alpha ships this single workflow by default. Future customization can be layered by:

1. Opting primitives in/out (for example, teams that skip releases).
2. Swapping workflow-policy templates.
3. Adjusting agent instruction packs and mode mappings.

Constraint: customization must preserve canonical semantics for any primitive that remains enabled.

---

## Done Criteria For This Workflow

1. Feature markdown + todo flow is usable end-to-end.
2. Release markdown + feature linkage is usable and stable.
3. Spec and ADR linkage is consistent and test-backed.
4. Issues remain local by default, with opt-in commit support.
5. Mode/workflow policy is visible in agent context.
6. E2E tests validate the full workflow path.
