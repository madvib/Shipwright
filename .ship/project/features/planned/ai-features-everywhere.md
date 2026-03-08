<!-- ship:feature id=wzNbBTE9 -->

# AI Features Everywhere

## Why

Ship should make AI assistance available across planning and execution surfaces, not isolated to a single chat/tool view.

## Acceptance Criteria

- [ ] Core planning entities support inline AI-assisted drafting/refinement flows
- [ ] AI actions use workspace/project context and linked entity references
- [ ] AI actions are provider-agnostic through shared adapter/config layers
- [ ] Failures and policy constraints are consistently surfaced in UI
- [ ] Operator can trace AI-assisted changes to affected entities/sessions

## Delivery Todos

- [ ] Define high-value AI entry points per surface (features/specs/releases/notes)
- [ ] Build shared UI pattern for inline assist + apply/reject flows
- [ ] Wire context payload assembly to active workspace/entity scope
- [ ] Add telemetry/audit hooks for AI-assisted edits

## Current Behavior

Partial capability exists through direct adapter actions, but broad inline AI coverage across entities is not yet shipped.

## Follow-ups

- Prioritize high-frequency writing surfaces first (feature docs/spec drafting/release notes).