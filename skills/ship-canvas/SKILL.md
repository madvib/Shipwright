---
id: ship-canvas
stable-id: ship-canvas
name: Ship Canvas
description: Receive and act on visual messages from Studio — annotations, comments, and drawings on HTML artifacts.
version: 0.1.0
---

# Ship Canvas

You receive visual messages when a user annotates an HTML artifact in Studio and hits Send. Each message is a `studio.message.visual` event delivered as a `ship/event` notification.

## Receiving a visual message

When you receive a `ship/event` notification with `event_type: "studio.message.visual"`, read the payload directly. No follow-up queries are needed — the payload is self-contained.

Payload shape:

```json
{
  "annotations": [
    {
      "selector": "#hero-heading",
      "note": "Make this larger",
      "artifact": "index.html",
      "x": 120,
      "y": 45
    }
  ],
  "summary": "Optional free-text message from the user"
}
```

Fields:

| Field | Type | Description |
|-------|------|-------------|
| `annotations` | array | One entry per user annotation |
| `annotations[].selector` | string | CSS selector of the annotated element |
| `annotations[].note` | string | The user's comment on that element |
| `annotations[].artifact` | string | File path of the HTML artifact |
| `annotations[].x` | number | Viewport x coordinate of the annotation marker |
| `annotations[].y` | number | Viewport y coordinate of the annotation marker |
| `summary` | string? | Free-text summary the user typed alongside the annotations |

## Workflow

1. Receive the `studio.message.visual` notification.
2. Read `payload.summary` for the user's overall intent.
3. Iterate `payload.annotations`. For each annotation, open the referenced `artifact` file and apply the change described in `note` to the element matching `selector`.
4. Write the updated artifact back using `write_session_file`.
5. Emit a `studio.artifact.updated` event (or equivalent) so Studio re-renders the artifact.

## Notes

- `selector` is a CSS selector. Prefer targeting by id or specific class over positional selectors.
- `x`/`y` are viewport coordinates for reference only — use `selector` for precise targeting.
- Multiple annotations may reference the same artifact. Apply all changes to that file before writing it.
- If a `selector` does not match any element in the artifact, log the issue and continue with remaining annotations rather than stopping.
