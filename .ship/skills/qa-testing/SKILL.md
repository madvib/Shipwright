---
name: qa-testing
stable-id: qa-testing
description: Use when testing a web application — navigate, interact, screenshot, find bugs. Outputs to .ship-session/ for live viewing in Ship Studio Session page.
tags: [qa, testing, browser]
authors: [ship]
---

# QA Testing

Systematic QA with results viewable in Ship Studio's Session page. All artifacts go to `.ship-session/` where the Session canvas can render them.

## Setup

All screenshots and reports go to `.ship-session/qa/`. Create the directory structure:

```bash
mkdir -p .ship-session/qa/screenshots
```

## Workflow

### 1. Target

Ask for the URL. Default tier is Standard.

| Tier | Scope |
|------|-------|
| Quick | Critical + high only |
| Standard | + medium |
| Exhaustive | + cosmetic |

### 2. Inventory

Navigate systematically. Write inventory to `.ship-session/qa/inventory.md`:

```markdown
# QA Inventory

| Page | URL | Status | Issues |
|------|-----|--------|--------|
| Landing | / | pending | - |
| Dashboard | /dashboard | pending | - |
```

### 3. Test each page

For every page, take a screenshot with a descriptive name:

```bash
$B goto <url>
$B screenshot .ship-session/qa/screenshots/<page-name>.png
$B console --errors
```

Screenshot naming: `landing.png`, `settings-form-empty.png`, `dashboard-loading.png`. Descriptive, no numbers.

Check per page:
- **Critical**: loads, primary action works, auth correct
- **High**: forms validate, errors render, data loads
- **Medium**: responsive, loading states, empty states
- **Cosmetic**: spacing, contrast, typography

### 4. File bugs

Each bug is a markdown file in `.ship-session/qa/bugs/`:

```markdown
---
severity: critical | high | medium | cosmetic
page: /settings
screenshot: screenshots/settings-broken-form.png
---

# Form submit button unresponsive

The submit button on the settings page does nothing when clicked.

## Steps to reproduce
1. Navigate to /settings
2. Fill in any field
3. Click Save

## Expected
Form submits and shows success message.

## Actual
Nothing happens. No console errors.
```

### 5. Write report

Summarize in `.ship-session/qa/report.md`:

```markdown
# QA Report

**URL**: http://localhost:3000
**Date**: 2026-03-28
**Tier**: Standard
**Pages tested**: 8
**Issues found**: 5

## Summary

| Severity | Count |
|----------|-------|
| Critical | 1 |
| High | 2 |
| Medium | 2 |
| Cosmetic | 0 |

## Top issues

1. **[Critical]** Form submit unresponsive on /settings
2. **[High]** Missing loading state on /dashboard
3. **[High]** Console error on /agents detail

## Screenshots

All screenshots in `qa/screenshots/`.
```

### 6. Update TODO

Append unfixed issues to `.ship-session/todo.md` so the Session page TODO tab shows them:

```markdown
- [ ] [Critical] Fix form submit on /settings
- [ ] [High] Add loading state to /dashboard
- [ ] [High] Fix console error on /agents/$id
```

## Quality

Every issue needs a screenshot. No exceptions. Use descriptive filenames so they're identifiable in the Session artifacts tab.
