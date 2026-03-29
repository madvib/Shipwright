---
name: diff-review
stable-id: diff-review
description: Use when reviewing a git diff or PR. Renders an interactive diff view in Ship Studio's Session page with annotatable file changes.
tags: [review, git, diff, workflow]
authors: [ship]
---

# Diff Review

Render git diffs as an interactive review surface in Ship Studio's Session page.

## Usage

Invoked as `/diff` or triggered when reviewing changes.

## Workflow

### 1. Get the diff

Determine what to diff:
- No args: diff unstaged changes
- Branch name: diff against that branch
- PR number: fetch PR diff via gh CLI

### 2. Render to Session canvas

Write `.ship-session/canvas.html` with an interactive diff view:

- File list at the top, clickable, scrolls to that file's diff
- Per-file sections with line numbers, green additions, red deletions
- Hunk headers as section dividers
- Stats: files changed, total insertions, total deletions
- Ship design tokens: JetBrains Mono for code, warm amber accents

Include review action buttons:
```html
<button data-ship-action="approve">Approve</button>
<button data-ship-action="request-changes">Request Changes</button>
```

### 3. Read annotations

After the reviewer annotates in Studio, read `.ship-session/annotations.json`:
- Click annotations on diff lines = inline review comments
- Action annotations = the review decision
- Box annotations = section-level feedback

### 4. Write review summary

Output to `.ship-session/review.md` with the decision, per-file comments, and referenced line numbers.

## Styling

Dark theme with light mode support. Use CSS variables:
- `--primary: #c67b2e` (amber accent)
- `--add: #2d5a27` / `--add-text: #7ec876` (green for additions)
- `--del: #5a2727` / `--del-text: #e87676` (red for deletions)
- Font: JetBrains Mono for code, DM Sans for UI
