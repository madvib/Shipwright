---
name: create-document
description: Create and update Ship planning documents (feature, spec, ADR, release) with correct metadata, links, and lifecycle-safe structure. Use this whenever users ask to draft, initialize, revise, or normalize project documents, even if they do not explicitly mention "create document."
---

# Create Document

Use this skill to turn planning intent into concrete Ship entities and high-quality markdown.

## Use this skill when

- The user asks to create or edit a `feature`, `spec`, `ADR`, or `release`.
- The user asks for richer initialization metadata (status, tags, links, target date, etc.).
- The user asks for a document refactor or normalization pass.

## Entity playbook

### Feature

- Required: `title`
- Optional links: `release_id`, `spec_id`, `branch`
- Lifecycle intent: `planned`, `in-progress`, `implemented`, `deprecated`
- Docs flow: use documentation updates for post-session narratives

### Spec

- Required: `title`
- Common metadata: `status`, `tags`
- Keep acceptance criteria explicit and testable

### ADR

- Required: decision title
- Always structure body as:
  - `## Status`
  - `## Context`
  - `## Decision`
  - `## Consequences`

### Release

- Required: `version`
- Structured metadata:
  - `status` (`planned`, `active`, `shipped`, `archived`)
  - `supported` (`true`/`false`)
  - `target_date` (optional)
  - `tags` (optional list)
- Keep scope explicit: linked features/specs and breaking changes

## Workflow

1. Identify entity type and operation (`create` or `update`).
2. Extract minimum required fields and linked IDs.
3. If required fields are missing, ask for only those fields.
4. Draft or update body with concise, actionable structure.
5. Persist with the correct operation for that entity.
6. Return a short change summary with IDs/links updated.

## Guardrails

- Prefer structured metadata inputs over prose-only metadata.
- Keep IDs stable on updates; do not silently re-key entities.
- Preserve existing body content unless the user asks for rewrite/refactor.
- Avoid inventing links; use provided or discoverable IDs only.

## Output contract

When you finish, report:

1. Entity type and identifier
2. Metadata fields set/changed
3. Linked entities added/removed
4. Any unresolved inputs still needed

