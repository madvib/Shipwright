---
name: ship-brainstorm
stable-id: ship-brainstorm
description: Use to generate and iterate on HTML mockups for UI design. Creates versioned page artifacts in .ship-session/ for live preview in Ship Studio.
tags: [design, ui, mockups, prototyping]
authors: [ship]
artifacts: [html]
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

### 2. Generate the page

Write a self-contained HTML file to `.ship-session/brainstorm-{slug}-v1.html`. Each page is a separate artifact — never overwrite, always create a new version.

Use Tailwind via CDN and realistic data. No lorem ipsum.

Include Ship's brand styles:
- Font: DM Sans (body), Syne (headings) via Google Fonts
- Colors: `--primary: #c67b2e`, `--bg: #18140f`, `--fg: #f8f4ef` (dark mode)
- Light mode: `--primary: #b06a1f`, `--bg: #faf7f3`, `--fg: #18140f`
- Support `data-theme="dark"` and `data-theme="light"` on the html element

### 3. Preview

Pages render live in Ship Studio's Session page. No server needed.

Fallback if Studio is not available:
```bash
python3 -m http.server {{ server_port }} --directory .ship-session &
```

### 4. Read feedback

Before each revision, check for annotation and feedback events. Each annotation has a selector pointing to the DOM element and a note with the user's feedback. Address each one.

### 5. Iterate

Create a new version: `brainstorm-{slug}-v2.html`. Keep previous versions — they are the iteration history. Delete rejected versions when the user confirms.

### 6. On approval

Output the final HTML path and a one-paragraph intent description covering layout decisions, color rationale, and interaction model.

## Quality standard

Every page should look shippable. Production aesthetics — proper spacing, contrast, accessibility. Senior designer who codes, not a wireframe.
