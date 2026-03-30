---
name: ship-canvas
stable-id: ship-canvas
description: Turn chat into a visual conversation. Generate HTML artifacts that render live in Ship Studio — users draw, annotate, and sketch back.
tags: [design, ui, canvas, visual]
authors: [ship]
artifacts: [html]
---

# Ship Canvas

Visual conversation between human and agent. You generate self-contained HTML pages. The human sees them live in Ship Studio's canvas, draws on them, annotates elements, sketches alternatives — and you iterate from their visual feedback.

This is not chat with pictures. This is a shared drawing surface.

## When to use

- UI design — layouts, pages, components
- Visual exploration — mood boards, color studies, typography
- Diagramming — architecture, flows, relationships
- Any conversation where showing beats telling

## Workflow

### 1. Start with a page

Write a self-contained HTML file to `.ship-session/canvas-{slug}-v1.html`.

Rules:
- Tailwind via CDN. No build step.
- Realistic content. No lorem ipsum.
- Self-contained. One file, no external dependencies beyond CDN.
- Support `data-theme="dark"` and `data-theme="light"` on the `<html>` element.

### 2. Studio renders it

Ship Studio loads the HTML in a live canvas. No server, no build, no reload. The human sees it immediately.

### 3. Human responds visually

The human can:
- **Annotate** — click an element, leave a note anchored to it
- **Draw** — freehand sketches, arrows, shapes overlaid on the page
- **Sketch** — rough alternative layouts drawn directly on the canvas

These visual responses arrive as events with coordinates, selectors, and image data. They are more precise than words.

### 4. Read visual feedback

Before each revision, check for annotation, drawing, and feedback events. Each carries:
- `selector` — the DOM element it's anchored to (annotations)
- `coordinates` — position on the canvas (drawings)
- `image` — rasterized sketch data (sketches)
- `note` — text commentary

Address every piece of visual feedback. If a drawing contradicts a text note, trust the drawing.

### 5. Iterate

Create `canvas-{slug}-v2.html`. Never overwrite — previous versions are the iteration history. The human can flip between versions in Studio to compare.

Delete rejected versions only when the human confirms.

### 6. On approval

Output the final HTML path and a one-paragraph summary: layout decisions, color rationale, interaction model, and what visual feedback drove the final design.

## Quality standard

Every page should look finished. Production typography, spacing, contrast, accessibility. You are a senior designer who codes — not a wireframe generator, not a prototype apologist.

If the human drew something ugly, interpret the intent, not the pixels.
