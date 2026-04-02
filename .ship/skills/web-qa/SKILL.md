---
name: web-qa
stable-id: web-qa
description: Use when testing a web application — navigate pages, interact, screenshot, file bugs, write a report. Each run is timestamped. Outputs to .ship-session/{{ stable_id }}/ for live viewing in Ship Studio.
tags: [qa, testing, browser, playwright]
authors: [ship]
license: MIT
compatibility: Requires playwright-cli (@playwright/cli). Install: npm install -g @playwright/cli@latest
artifacts: [image, markdown]
allowed-tools: Bash(playwright-cli:*) Bash(npx:*)
---

# Web QA

Systematic QA using playwright-cli. Each run is isolated in a timestamped folder under `.ship-session/{{ stable_id }}/`. Results are viewable live in Ship Studio's Session page.

## Setup

```bash
QA_ROOT="$(git rev-parse --show-toplevel)/.ship-session/{{ stable_id }}"
QA_RUN="$(date -u +%Y-%m-%dT%H-%M)"
QA_DIR="$QA_ROOT/$QA_RUN"
mkdir -p "$QA_DIR/screenshots" "$QA_DIR/bugs"
npx --no-install playwright-cli --version || npm install -g @playwright/cli@latest
```

Use `$QA_DIR` for all paths. Never use relative paths — playwright-cli resolves them from its own working directory.

## Workflow

### 1. Target

Ask for the URL. Default tier is `{{ tier }}`.

| Tier | Scope |
|------|-------|
| Quick | Critical + high only |
| Standard | + medium |
| Exhaustive | + cosmetic |

### 2. Inventory

```bash
playwright-cli open --browser={{ browser }} <url>
playwright-cli snapshot
```

Write inventory to `$QA_DIR/inventory.md`:

```markdown
# QA Inventory

| Page | URL | Status | Issues |
|------|-----|--------|--------|
| Landing | / | pending | - |
```

### 3. Test each page

Navigate, snapshot for element refs, screenshot, check diagnostics:

```bash
playwright-cli goto <url>
playwright-cli snapshot
playwright-cli screenshot --filename="$QA_DIR/screenshots/<page-name>.png"
playwright-cli console
playwright-cli network
```

Screenshot naming: `landing.png`, `settings-form-empty.png`. Descriptive, no numbers.

Test interactions using refs from snapshot:

```bash
playwright-cli fill e3 "user@test.com"
playwright-cli click e5
playwright-cli snapshot
playwright-cli screenshot --filename="$QA_DIR/screenshots/<page-name>-after.png"
```

Per-page checks:
- **Critical**: loads, primary action works, auth correct
- **High**: forms validate, errors render, data loads
- **Medium**: responsive, loading states, empty states
- **Cosmetic**: spacing, contrast, typography

### 4. File bugs

Each bug: `$QA_DIR/bugs/<slug>.md`

```markdown
---
severity: critical | high | medium | cosmetic
page: /settings
screenshot: screenshots/settings-broken-form.png
---

# Form submit button unresponsive

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

`$QA_DIR/report.md`:

```markdown
# QA Report

**URL**: http://localhost:3000
**Run**: 2026-03-31T19-30
**Tier**: Standard
**Browser**: {{ browser }}
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
```

### 6. Close and update TODO

```bash
playwright-cli close
```

Append to `$(git rev-parse --show-toplevel)/.ship-session/todo.md`:

```markdown
- [ ] [Critical] Fix form submit on /settings — qa/{{ run_timestamp }}/bugs/form-submit.md
```

## Quality

Every bug needs a screenshot. Use descriptive filenames — they appear in the Studio Session artifacts tab.

## Auth

```bash
playwright-cli state-save "$QA_DIR/auth.json"
playwright-cli state-load "$QA_DIR/auth.json"
```

## Cross-browser

```bash
playwright-cli open --browser=firefox <url>
```

Supported: `chromium`, `firefox`, `webkit`, `msedge`.

## Studio trace viewer

```bash
playwright-cli tracing-start
# ... run session ...
playwright-cli tracing-stop
npx playwright show-trace <trace-file> --port 9323
# Studio iframes http://localhost:9323
```
