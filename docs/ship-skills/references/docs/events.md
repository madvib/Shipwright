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

## Actor-isolated event architecture

Events are routed through the KernelRouter. Each actor (agent, app, service) has its own event store and mailbox. Actors communicate through kernel-managed message passing, not shared storage.

### How routing works

1. Agent spawns as an actor with namespace `agent.{id}` and a mailbox subscribing to relevant namespaces.
2. Studio is an app actor with namespace `studio`, subscribing to `studio.*` and `agent.*`.
3. User annotates an HTML artifact in Studio → Studio emits `studio.message.visual` via its ActorStore.
4. KernelRouter routes the event to the agent's mailbox based on subscription.
5. Agent receives a `ship/event` SSE notification with the full payload. No follow-up queries needed — payloads are self-contained.

Agents do not have read access to the event store. They receive events through their mailbox only. The `list_events` MCP tool has been removed.

### Namespace boundaries

Every event type is namespaced. `RESERVED_NAMESPACES` blocks agents from emitting platform-controlled prefixes (`actor.*`, `session.*`, `studio.*`, `kernel.*`, etc.). Apps like Studio emit in their own namespace (`studio.*`) via their ActorStore.

### Visual messages

Studio supports visual feedback: users annotate HTML artifacts, add comments, draw on the canvas, then send everything as a single `studio.message.visual` event. Individual annotations are local UI state until the user explicitly sends. The agent receives one event with all annotations as a self-contained payload.

## Studio as the first app actor

Ship Studio is the first application on the runtime. It registers as an actor, owns the `studio.*` event namespace, and defines its own event types and UI interactions. Studio is a peer to agents, not a layer above them — both are actors managed by the kernel.

The event debug panel (toggle: `Ctrl+Shift+E` in dev builds) shows all events flowing through the actor's mailbox in real time.

## Stability

Stable in Ship 0.2.0:

- Per-actor event stores with namespace isolation
- KernelRouter with spawn/route/stop/snapshot/restore
- Studio as app actor with visual message flow
- Artifact-type-to-event mapping (skills declare artifacts, platform infers events)
- `ship/event` SSE notifications via actor mailboxes
- Actor snapshot and restore for migration

Not yet available:

- Service actors (sync, docs, auth)
- Ship SDK for external applications
- Typed artifact schemas with cloud docs API sync
- Custom app event types beyond Studio
