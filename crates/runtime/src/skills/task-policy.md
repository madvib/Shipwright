+++
id = "task-policy"
name = "Shipwright Workflow Policy"
source = "builtin"
+++

# Shipwright Workflow Policy

This policy defines how humans and agents execute work in a Ship project.

## Canonical Flow

`Vision -> Release -> Feature -> Spec -> Issues -> Close Feature -> Ship Release`

ADRs and notes are ambient records — create them when a decision or insight surfaces, not as a workflow step.

## Execution Rules

0. **Use Ship As System of Record**
   - For project state changes, use Ship tools/commands (`ship ...` or Ship MCP tools), not ad-hoc file edits.
   - Treat SQLite + Ship ops as authoritative for issues/features/specs/releases/notes.
   - Keep transport thin: parse input, call Ship, report output.

1. **Release Then Feature**
   - Associate work to a canonical release document.
   - Start feature work within that release context.

2. **Feature First**
   - Start work from a feature markdown document with delivery todos.
   - Keep acceptance criteria at the feature level.

3. **Spec As Contract**
   - Every non-trivial feature should have a spec.
   - Update the spec when scope or implementation constraints change.

4. **Issues Are Execution Scratch**
   - Issues track day-to-day execution tasks.
   - Issues are local-only by default.
   - Promote issue artifacts to git only when needed for durable records.

5. **ADRs Capture Lasting Decisions**
   - Architecture-impacting decisions must be recorded in ADRs.
   - ADRs are committed by default.

6. **Verify Before Closing**
   - Run relevant tests before marking feature todos done.
   - Update the feature description to reflect what was built.

## Using Ship MCP Tools

- `get_project_info` — call at session start for full project context
- `list_issues` — find in-progress work before starting anything new
- `move_issue` backlog → in-progress before starting; in-progress → done when complete
- `create_adr` — record any architecture decision that affects the project long-term
- `get_feature_catalog` — understand what the product already does before adding more
