---
name: browse
stable-id: browse
description: Headless browser for QA testing and site verification. Navigate, interact, screenshot, diff, assert element states. ~100ms per command.
tags: [qa, testing, browser, screenshots, headless]
authors: [ship, gstack]
attribution: Based on gstack browse (MIT) by Garry Tan — github.com/garrytan/gstack
compatibility: Requires the gstack browse binary. Install via ship add github.com/garrytan/gstack
---

# Browse

Persistent headless Chromium. First call auto-starts (~3s), then ~100ms per command. State persists between calls (cookies, tabs, login sessions).

## Setup

Requires the gstack browse binary. Install: `ship add github.com/garrytan/gstack`

```bash
_ROOT=$(git rev-parse --show-toplevel 2>/dev/null)
B=""
[ -n "$_ROOT" ] && [ -x "$_ROOT/.claude/skills/gstack/browse/dist/browse" ] && B="$_ROOT/.claude/skills/gstack/browse/dist/browse"
[ -z "$B" ] && B=~/.claude/skills/gstack/browse/dist/browse
[ -x "$B" ] && echo "READY: $B" || echo "NEEDS_SETUP: run ship add github.com/garrytan/gstack"
```

## Core Patterns

### Verify a page loads
```bash
$B goto https://yourapp.com
$B text
$B console
$B network
$B is visible ".main-content"
```

### Test a user flow
```bash
$B goto https://app.com/login
$B snapshot -i
$B fill @e3 "user@test.com"
$B fill @e4 "password"
$B click @e5
$B snapshot -D
$B is visible ".dashboard"
```

### Visual evidence
```bash
$B snapshot -i -a -o {{ screenshot_dir }}/annotated.png
$B screenshot {{ screenshot_dir }}/page.png
$B console
```

### Responsive layouts
```bash
$B responsive {{ screenshot_dir }}/layout
$B viewport 375x812
$B screenshot {{ screenshot_dir }}/mobile.png
```

### Assert element states
```bash
$B is visible ".modal"
$B is enabled "#submit-btn"
$B is disabled "#submit-btn"
$B is checked "#agree-checkbox"
```

### Diff environments
```bash
$B diff https://staging.app.com https://prod.app.com
```

## Snapshot Flags

```
-i    Interactive elements only (buttons, links, inputs) with @e refs
-c    Compact (no empty nodes)
-d N  Limit tree depth
-s s  Scope to CSS selector
-D    Unified diff against previous snapshot
-a    Annotated screenshot with red overlay boxes
-o p  Output path for annotated screenshot
-C    Cursor-interactive elements (@c refs)
```

After snapshot, use @refs: `$B click @e3`, `$B fill @e4 "value"`, `$B hover @e1`

## Commands

| Command | Description |
|---------|-------------|
| `goto <url>` | Navigate |
| `back` / `forward` / `reload` | History |
| `text` | Page text |
| `html [sel]` | innerHTML |
| `links` | All links |
| `click <sel>` | Click |
| `fill <sel> <val>` | Fill input |
| `type <text>` | Type into focused element |
| `select <sel> <val>` | Dropdown |
| `hover <sel>` | Hover |
| `press <key>` | Keypress |
| `scroll [sel]` | Scroll into view |
| `upload <sel> <file>` | File upload |
| `wait <sel\|--networkidle>` | Wait for element or idle |
| `screenshot [sel] [path]` | Screenshot |
| `snapshot [flags]` | Accessibility tree |
| `responsive [prefix]` | Multi-viewport screenshots |
| `diff <url1> <url2>` | Cross-page diff |
| `js <expr>` | Run JavaScript |
| `console [--errors]` | Console log |
| `network [--clear]` | Network requests |
| `cookies` | All cookies |
| `is <prop> <sel>` | State assert (visible/hidden/enabled/disabled/checked) |

All screenshots and artifacts go to `{{ screenshot_dir }}/`.
