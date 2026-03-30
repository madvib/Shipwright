---
group: Smart Skills
title: Events
description: events.json schema, Ship built-in events, custom events, direction, and how the runtime routes events to agents.
audience: public
section: reference
order: 4
---

# Events

Events make skills interactive. Declared in `assets/events.json`, events define the communication protocol between agents, humans, and surfaces. The runtime validates, persists, and routes events based on these declarations.

## events.json schema

The JSON Schema is published at `https://getship.dev/schemas/events.schema.json`.

```json
{
  "$schema": "https://getship.dev/schemas/events.schema.json",
  "ship": ["annotation", "feedback"],
  "custom": [
    {
      "id": "page_created",
      "direction": "out",
      "label": "Page Created",
      "description": "Agent created a new brainstorm page.",
      "schema": {
        "type": "object",
        "required": ["filename"],
        "properties": {
          "filename": { "type": "string" },
          "title": { "type": "string" }
        }
      }
    }
  ]
}
```

## Ship built-in events

Ship provides well-known event types for common interaction patterns. Reference them by name in the `"ship"` array instead of redeclaring schemas.

| Name | Direction | Description |
|------|-----------|-------------|
| `annotation` | in | User annotated an element on a rendered surface. Payload: note, x, y, selector, artifact. |
| `feedback` | in | User approved, rejected, or commented on an artifact. Payload: action (approve/reject/comment/request_changes), comment, artifact. |
| `selection` | in | User selected text or content. Payload: text, selector, artifact. |
| `artifact_created` | out | Agent created a new artifact file. Payload: filename, mime_type, title. |
| `artifact_deleted` | both | Artifact removed by human (rejection) or agent (cleanup). Payload: filename, reason. |

Ship built-in events are namespaced as `ship.{name}` at runtime. When a skill declares `"ship": ["annotation"]`, the agent receives events of type `ship.annotation`.

Full schemas for each built-in event are published at `https://getship.dev/schemas/ship-events.json`.

## Custom events

Custom events live in the skill's namespace. The `id` field becomes `{stable-id}.{id}` at runtime.

A skill with `stable-id: ship-brainstorm` that declares a custom event `page_created` produces events of type `ship-brainstorm.page_created`.

### Event fields

| Field | Required | Description |
|-------|----------|-------------|
| `id` | yes | Event identifier. Lowercase, underscores. Pattern: `[a-z][a-z0-9_]*`. |
| `direction` | yes | `in` (human → agent), `out` (agent → human), or `both`. |
| `label` | no | Human-readable name for Studio and docs. |
| `description` | no | What this event means and when it fires. |
| `schema` | no | JSON Schema for the event payload. Validated by the runtime at ingress. |

## Direction

Direction controls who can emit and who can receive.

| Direction | Emitter | Receiver | Example |
|-----------|---------|----------|---------|
| `in` | Human / SDK / external | Agent | User annotates, agent reacts |
| `out` | Agent | Human / SDK / Studio | Agent creates artifact, Studio renders |
| `both` | Either side | Either side | Artifact deletion (human rejects or agent cleans up) |

The runtime enforces direction. An agent cannot emit an `in`-only event. A human cannot emit an `out`-only event.

## How routing works

Skills define events. Agents use skills. The runtime routes events to agents based on their active skills.

1. Agent activates with skills `[ship-brainstorm, code-review]`.
2. Runtime reads `events.json` from each skill.
3. `in` and `both` events from all active skills become the agent's `allowed_events`.
4. When an event arrives on the workspace bus, the EventRelay checks each connected agent's `allowed_events`.
5. System events (`session.*`, `workspace.*`, `actor.*`, etc.) bypass filtering — all agents receive them.
6. Skill-namespaced events only reach agents whose skills declared them.

Events are delivered as `ship/event` custom MCP notifications with the full payload. The agent receives the event directly — no resource polling, no file watching.

## Relationship to variables

Variables (`vars.json`) configure skills at compile time. Events (`events.json`) communicate at runtime. They are independent — a skill can have vars without events, events without vars, or both.

Both use JSON Schema for type safety. Both are scoped to the skill's `stable-id`. Future: schema migrations may cover both vars and events in a unified migration plan.

## Stability

Stable in Ship 0.1.0:

- `assets/events.json` with `ship` and `custom` arrays
- Five Ship built-in event types (annotation, feedback, selection, artifact_created, artifact_deleted)
- Direction enforcement (in/out/both)
- Payload schema validation at runtime ingress
- Event routing to agents via EventRelay
- `ship/event` custom MCP notifications

Not yet available:

- `app/` directory for custom frontends (specced, not yet wired)
- Event schema migrations
- Studio event debug panel
- Ship SDK npm package for standalone mode
