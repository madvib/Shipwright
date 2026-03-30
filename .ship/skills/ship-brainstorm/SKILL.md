---
name: ship-brainstorm
stable-id: ship-brainstorm
description: Use to generate and iterate on HTML mockups for UI design. Writes to .ship-session/ for live preview in Ship Studio's Session page with visual annotation support.
tags: [design, ui, mockups, prototyping]
authors: [ship]
---

# Ship Brainstorm

Generate production-quality HTML mockups. Real typography, spacing, color — not wireframes. Iterate with visual feedback via Ship Studio's Session page.

## When to use

- New UI from scratch
- Redesigns and layout exploration
- Component variants side-by-side

## Workflow

### 1. Understand the goal

Ask at most 1–2 focused questions. Make reasonable assumptions and note them.

### 2. Generate the mockup

Write a self-contained HTML file to `.ship-session/canvas.html`. Use Tailwind via CDN, Ship's design tokens, and realistic data. No lorem ipsum.

Include Ship's brand styles:
- Font: DM Sans (body), Syne (headings) via Google Fonts
- Colors: `--primary: #c67b2e`, `--bg: #18140f`, `--fg: #f8f4ef` (dark mode)
- Light mode: `--primary: #b06a1f`, `--bg: #faf7f3`, `--fg: #18140f`
- Support `data-theme="dark"` and `data-theme="light"` on the html element

### 3. Preview

The mockup renders live in Ship Studio's Session page at `/studio/session`. No server needed.

Fallback if Studio is not available:
```bash
python3 -m http.server {{ server_port }} --directory .ship-session &
```

### 4. Read annotations

Before each revision, check `.ship-session/annotations.json` for structured user feedback. Each annotation has a `type` (click or box), a `selector` pointing to the DOM element, and a `note` with the user's feedback. Address each one.

### 5. Iterate

Overwrite `.ship-session/canvas.html` with the revision. Copy the previous version to `.ship-session/canvas-v{N}.html` to preserve history.

### 6. On approval

Output the final HTML path and a one-paragraph intent description covering layout decisions, color rationale, and interaction model.

Proceed to `visual-spec` for implementation handoff if requested.

## Quality standard

Every mockup should look shippable. Production aesthetics — proper spacing, contrast, accessibility. Senior designer who codes, not a wireframe.
