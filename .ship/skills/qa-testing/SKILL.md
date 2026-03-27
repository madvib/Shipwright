---
name: qa-testing
stable-id: qa-testing
description: Use when testing a web application — navigate, interact, screenshot, find bugs. Systematic QA checklist with severity tiers and structured bug reports.
tags: [qa, testing, browser]
authors: [ship]
---

# QA Workflow

Systematic QA testing with agent-dispatched fixes. Test → find → fix → verify loop.

## Protocol

### 1. Target

Ask for the URL and scope:
- "What URL should I test?"
- "Any specific flows to focus on?" (skip if they just say "test everything")

### 2. Discover

Navigate the app systematically. For each page:

```bash
# Use gstack browse or any headless browser
/browse goto <url>
/browse screenshot
```

Build a page inventory:
```markdown
<!-- .ship-session/qa-inventory.md -->
# QA Inventory

| Page | URL | Status | Issues |
|------|-----|--------|--------|
| Landing | / | tested | 2 |
| Dashboard | /dashboard | tested | 0 |
| Settings | /settings | pending | - |
```

### 3. Test each page

For every page, check:

**Critical (blocks ship):**
- Page loads without error
- Primary action works (submit, save, navigate)
- Auth-gated pages redirect correctly

**High:**
- Forms validate and submit
- Error states render correctly
- Data loads and displays

**Medium:**
- Responsive layout (mobile + desktop)
- Loading states present
- Empty states handled

**Cosmetic:**
- Spacing/alignment consistency
- Color contrast
- Typography hierarchy

### 4. File bugs

For each issue, write a structured bug:

```markdown
<!-- .ship-session/bugs/<id>.md -->
---
severity: critical | high | medium | cosmetic
page: /settings
status: open
---

# Bug: <title>

## Repro
1. Navigate to /settings
2. Click "Save" without filling required fields

## Expected
Validation error shown

## Actual
Silent failure, no feedback

## Screenshot
.ship-session/screenshots/<id>.png
```

### 5. Dispatch fixes

Group bugs by file scope. For each group:

```bash
bash .ship/skills/mission-control/dispatch.sh \
  --slug fix-settings-validation \
  --agent web-lane \
  --spec .ship-session/bugs/group-settings.md
```

The fix agent gets the bug reports as its job spec.

### 6. Verify

After fix agent completes:
- Re-test the specific pages
- Take after-screenshots
- Compare before/after
- Gate: pass if the bug is fixed, fail if regression

### 7. Report

```markdown
<!-- .ship-session/qa-report.md -->
# QA Report — <date>

## Health Score: 7/10

## Summary
- Pages tested: 12
- Bugs found: 5 (1 critical, 2 high, 2 cosmetic)
- Bugs fixed: 4
- Remaining: 1 cosmetic (deferred)

## Before/After
| Bug | Before | After | Status |
|-----|--------|-------|--------|
| Settings validation | screenshot | screenshot | Fixed |
```

## Tiers

Run at the level of rigor that matches the moment:

| Tier | Checks | When |
|------|--------|------|
| **Quick** | Critical + high only | Pre-merge, CI gate |
| **Standard** | + medium | Pre-release |
| **Exhaustive** | + cosmetic, responsive, perf | Launch, major release |

Default tier: **{{ default_tier }}** (change with `ship vars set qa-testing default_tier <quick|standard|exhaustive>`).
