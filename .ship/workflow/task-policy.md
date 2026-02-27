# Shipwright Workflow Policy (Alpha)

This policy defines how humans and agents execute work in alpha.

## Canonical Flow

`Vision -> Release -> Feature -> Spec -> Issues -> ADRs -> Close Feature -> Ship Release`

## Execution Rules

1. **Release Then Feature**
- Associate work to a canonical release document (e.g. `v0.1.0-alpha.md`).
- Start feature work within that release context.

2. **Feature First**
- Start work from a feature markdown document with delivery todos.
- Keep acceptance criteria at feature level.

3. **Spec As Contract**
- Every non-trivial feature should have a spec.
- Update spec when scope or implementation constraints change.

4. **Issues Are Execution Scratch**
- Issues track day-to-day execution.
- Issues are local-only by default to avoid polluting git history.
- Promote issue artifacts to git only when needed for durable records.

5. **ADRs Capture Lasting Decisions**
- Architecture-impacting decisions must be recorded in ADRs.
- ADRs are committed by default.
- MCP integration decisions should be recorded in ADR notes (without storing secrets).

6. **Mode Is Agent Runtime, Not PM State**
- Mode changes are explicit in alpha.
- Workflow policy and current phase should be included in agent context.

7. **Verification**
- Run relevant tests before closing feature todos.

## Logging and Events

- Project actions append to `.ship/events.ndjson`.
- Human-readable logs are derived from the event stream.
- Event model should stay compatible with future global aggregation.
