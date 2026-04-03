---
name: web-qa
stable-id: web-qa
description: Use when testing a web application — navigate pages, interact, screenshot, file bugs, write a report. Each run is timestamped. Outputs to .ship-session/{{ stable_id }}/ for live viewing in Ship Studio.
tags: [qa, testing, browser, playwright]
authors: [ship]
license: MIT
compatibility: Requires Playwright (@playwright/test) with browsers installed, and system libs libnspr4 + libnss3 on Linux
artifacts: [image, markdown]
allowed-tools: Bash(playwright*) Bash(npx playwright*) Bash(pnpm exec playwright*)
---

# Web QA

Systematic QA using Playwright. Each run is isolated in a timestamped folder under `.ship-session/{{ stable_id }}/`. Results are viewable live in Ship Studio's Session page.

## Setup

```bash
QA_ROOT="$(git rev-parse --show-toplevel)/.ship-session/{{ stable_id }}"
QA_RUN="$(date -u +%Y-%m-%dT%H-%M)"
QA_DIR="$QA_ROOT/$QA_RUN"
mkdir -p "$QA_DIR/screenshots" "$QA_DIR/bugs"

# Discover playwright — check PATH, npx, then project node_modules
if command -v playwright &>/dev/null; then
  PW="playwright"
elif npx --no-install playwright --version &>/dev/null 2>&1; then
  PW="npx playwright"
else
  # Walk up from repo root looking for node_modules/.bin/playwright
  _root="$(git rev-parse --show-toplevel)"
  _found=""
  for _candidate in "$_root"/node_modules/.bin/playwright "$_root"/apps/*/node_modules/.bin/playwright "$_root"/packages/*/node_modules/.bin/playwright; do
    if [ -x "$_candidate" ]; then _found="$_candidate"; break; fi
  done
  if [ -n "$_found" ]; then
    PW="$_found"
  else
    echo "Playwright not found. Install: npm install -D @playwright/test" >&2
    exit 1
  fi
fi
$PW --version
```

Use `$PW` for all playwright commands and `$QA_DIR` for all paths. Never use relative paths — Playwright resolves them from its own working directory.

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
$PW open --browser={{ browser }} <url>
$PW snapshot
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
$PW goto <url>
$PW snapshot
$PW screenshot --filename="$QA_DIR/screenshots/<page-name>.png"
$PW console
$PW network
```

Screenshot naming: `landing.png`, `settings-form-empty.png`. Descriptive, no numbers.

Test interactions using refs from snapshot:

```bash
$PW fill e3 "user@test.com"
$PW click e5
$PW snapshot
$PW screenshot --filename="$QA_DIR/screenshots/<page-name>-after.png"
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
$PW close
```

Append to `$(git rev-parse --show-toplevel)/.ship-session/todo.md`:

```markdown
- [ ] [Critical] Fix form submit on /settings — qa/{{ run_timestamp }}/bugs/form-submit.md
```

## Quality

Every bug needs a screenshot. Use descriptive filenames — they appear in the Studio Session artifacts tab.

## Auth

```bash
$PW state-save "$QA_DIR/auth.json"
$PW state-load "$QA_DIR/auth.json"
```

## Cross-browser

```bash
$PW open --browser=firefox <url>
```

Supported: `chromium`, `firefox`, `webkit`, `msedge`.

## Studio trace viewer

```bash
$PW tracing-start
# ... run session ...
$PW tracing-stop
$PW show-trace <trace-file> --port 9323
# Studio iframes http://localhost:9323
```
