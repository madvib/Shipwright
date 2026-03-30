---
name: visual-spec
stable-id: visual-spec
description: Use to convert a chosen design (HTML file or live URL) into a structured spec that a frontend agent can implement from. Captures screenshots, extracts markup, and writes design-spec/spec.md.
tags: [design, ui, specification, handoff]
authors: [ship]
---

# Visual Spec

Converts a chosen design into a structured implementation brief. Combines screenshot + markup + requirements in one artifact.

**Opening context:** Always read `.ship-session/design-spec/spec.md` if it exists before starting — it is your implementation brief.

## When to use

- After `ship-brainstorm`: HTML mockup → spec → implement
- Existing UI iteration: screenshot current state → describe delta → spec target state → implement delta

## Input

Either:
- Path to a local HTML file (e.g. `mockup.html`)
- URL of a live or existing UI

## Workflow

### 1. Serve if needed

If input is an HTML file, serve it first:

```bash
python3 -m http.server 8766 --directory .ship-session &
```

Use the resulting URL (e.g. `http://localhost:8766/mockup.html`) for all browse commands.

### 2. Screenshot with browse

```
/browse goto <url>
/browse screenshot
```

For responsive coverage:

```
/browse responsive <url>
```

This captures mobile + desktop viewports. Save outputs to `design-spec/screenshots/`.

### 3. Extract markup

- For HTML input: pull the relevant component markup directly from the file
- For a live URL: `/browse snapshot` to capture the DOM

### 4. Write outputs

Create two things:

**`.ship-session/design-spec/screenshots/`** — screenshot files (desktop.png, mobile.png)

**`.ship-session/design-spec/spec.md`** — the full spec document (see structure below)

### 5. Hand off

Tell the agent: "Open `.ship-session/design-spec/spec.md`. That is your implementation brief."

## spec.md structure

```markdown
# Design Spec: <name>

## Screenshots
![Desktop](.ship-session/design-spec/screenshots/desktop.png)
![Mobile](.ship-session/design-spec/screenshots/mobile.png)

## Component Breakdown
- <component name>: <what it does, what it contains>
(one line per component)

## Design Tokens
- Colors: (extracted hex values)
- Typography: (font, sizes, weights)
- Spacing: (key spacing values)

## Interaction Notes
- Hover states, transitions, responsive behavior

## Implementation Checklist
- [ ] <concrete thing the frontend agent must build>
(one checkbox per discrete requirement)
```

Keep the checklist concrete and actionable — one checkbox per discrete UI element or behavior, not per file or function.
