---
group: Smart Skills
title: Platform Events
description: Ship built-in events, how they map to artifact types, and how the runtime routes events to agents.
audience: public
section: reference
order: 4
---

# Platform Events

Ship events are platform primitives. They are not declared by skills — they are inferred from artifact types. When a skill produces HTML artifacts, Studio automatically enables annotation and feedback events for that content.

## Ship built-in events

| Name | Direction | Triggered by |
|------|-----------|--------------|
| `ship.annotation` | in | User annotated an element on a rendered surface. Payload: note, x, y, selector, artifact. |
| `ship.feedback` | in | User approved, rejected, or commented on an artifact. Payload: action (approve/reject/comment/request_changes), comment, artifact. |
| `ship.selection` | in | User selected text or content. Payload: text, selector, artifact. |
| `ship.artifact_created` | out | Agent created a new artifact file. Payload: filename, mime_type, title. |
| `ship.artifact_deleted` | both | Artifact removed by human (rejection) or agent (cleanup). Payload: filename, reason. |

Direction: `in` = human to agent, `out` = agent to human, `both` = either direction.

Full schemas for each built-in event are published at `https://getship.dev/schemas/ship-events.json`.

## Artifact type to event mapping

The platform maps artifact types to applicable events automatically:

| Artifact type | Applicable events |
|---------------|-------------------|
| `html` | annotation, feedback, selection, artifact_created, artifact_deleted |
| `pdf` | selection, feedback, artifact_created, artifact_deleted |
| `markdown` | feedback, selection, artifact_created, artifact_deleted |
| `image` | annotation, feedback, artifact_created, artifact_deleted |
| `adr` | feedback, artifact_created, artifact_deleted |
| `note` | feedback, artifact_created, artifact_deleted |
| `url` | feedback |
| `json` | feedback, artifact_created, artifact_deleted |

Skills do not opt into events. They declare what they produce via the `artifacts` frontmatter field. The platform handles the rest.

## How routing works

1. Agent activates with skills that declare `artifacts: [html, pdf]`.
2. Runtime knows which events apply to those artifact types.
3. When a user annotates an HTML artifact in Studio, `ship.annotation` fires.
4. EventRelay delivers the event to agents whose active skills produce compatible artifact types.
5. System events (`session.*`, `workspace.*`, `actor.*`, etc.) bypass filtering — all agents receive them.

Events are delivered as `ship/event` custom MCP notifications with the full EventEnvelope payload. No resource polling, no file watching.

## Studio as the first event surface

Ship Studio is the first UI that sends and receives Ship events. It renders artifacts by type, provides annotation/feedback tools, and delivers events to agents via the MCP notification channel.

Future: the Ship SDK (`@ship/overlay`) will allow any web application to send and receive Ship events. A skill's `app/` directory could include a custom frontend that integrates with the event bus. But skills themselves do not define event protocols — that belongs at the app tier.

## Event debug panel

In development builds, Studio includes an event debug panel (toggle: `Ctrl+Shift+E`). It shows all events flowing in real time with type, entity, actor, and expandable payload.

## Stability

Stable in Ship 0.2.0:

- Five Ship built-in event types
- Artifact-type-to-event mapping
- Event routing to agents via EventRelay
- `ship/event` custom MCP notifications
- Event debug panel (dev builds)

Not yet available:

- Ship SDK for external applications
- `app/` directory with event bus integration
- Typed artifact schemas (adr, note) with cloud docs API sync
- Custom event types (app tier, future)
