---
name: visual-brainstorm
description: Use to generate and iterate on HTML mockups for UI design goals. Serves a local preview server, iterates on feedback, and outputs final HTML + intent for handoff to visual-spec.
---

# Visual Brainstorm

Ship-native interactive mockup generation. Production-level aesthetics — real typography, spacing, color. Not wireframes.

## When to use

- New UI from scratch
- Redesigns and layout options
- Component variants side-by-side

## Workflow

### 1. Understand the goal

Ask at most 1–2 focused questions. Do not ask more. Make reasonable assumptions about everything else and note them.

Good questions:
- "What's the primary action this UI needs to drive?"
- "Any brand colors or existing design system to align with?"

### 2. Generate the mockup

Create a single self-contained HTML file at `.session/mockup.html`:
- Tailwind CSS via CDN — no build step
- Realistic placeholder data (names, numbers, copy — not "Lorem ipsum")
- Full visual fidelity: real font sizes, weights, line-heights, border-radius, shadows
- Think: senior designer who codes, not a wireframe

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <script src="https://cdn.tailwindcss.com"></script>
  <title>Mockup</title>
</head>
<body>
  <!-- ... -->
</body>
</html>
```

### 3. Serve locally

```bash
python3 -m http.server 8765 --directory .session &
```

Print the URL: `http://localhost:8765/mockup.html`

### 4. Iterate

On each feedback round:
- Overwrite `.session/mockup.html` with the updated version
- Tell the user to refresh their browser
- Do not rename the file — keep the same URL

### 5. On approval

Output:
1. The final HTML (inline or as a file path)
2. A one-paragraph intent description: layout decisions, color rationale, interaction model

### 6. Optional handoff

If the user wants a structured implementation spec, proceed with the `visual-spec` skill.

## Quality standard

Match the quality bar of the `frontend-design` skill. Every mockup should look shippable. If spacing feels off, fix it. If colors lack contrast, fix it. Default to clean, modern, accessible UI.
