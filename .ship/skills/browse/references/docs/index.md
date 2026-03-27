---
title: Browse — Headless Browser Skill
description: Command reference and patterns for headless browser QA testing, screenshots, and site verification.
---

# Browse

Headless Chromium for QA testing and site verification. Persistent session — first call starts the browser (~3s), subsequent calls reuse it (~100ms each). Cookies, tabs, and login state persist between calls.

## Installation

Requires the gstack browse binary:

```bash
ship add github.com/garrytan/gstack
```

The binary installs to `.claude/skills/gstack/browse/dist/browse`. The skill auto-detects its location from the project root or home directory.

## Variables

| Variable | Type | Scope | Default | Description |
|----------|------|-------|---------|-------------|
| `screenshot_dir` | string | global | `.ship-session/screenshots` | Directory for screenshots and annotated images |

```bash
ship vars set browse screenshot_dir ./test-artifacts/screenshots
```

## Command Reference

### Navigation

| Command | Description | Example |
|---------|-------------|---------|
| `goto <url>` | Navigate to URL | `$B goto https://app.com` |
| `back` | Browser back | `$B back` |
| `forward` | Browser forward | `$B forward` |
| `reload` | Reload page | `$B reload` |
| `wait <sel>` | Wait for element | `$B wait ".loaded"` |
| `wait --networkidle` | Wait for network idle | `$B wait --networkidle` |

### Interaction

| Command | Description | Example |
|---------|-------------|---------|
| `click <sel>` | Click element | `$B click @e3` |
| `fill <sel> <val>` | Fill input field | `$B fill @e4 "user@test.com"` |
| `type <text>` | Type into focused element | `$B type "search query"` |
| `select <sel> <val>` | Select dropdown option | `$B select "#country" "US"` |
| `hover <sel>` | Hover over element | `$B hover @e1` |
| `press <key>` | Press keyboard key | `$B press Enter` |
| `scroll [sel]` | Scroll element into view | `$B scroll "#footer"` |
| `upload <sel> <file>` | Upload file to input | `$B upload "#avatar" photo.png` |

### Inspection

| Command | Description | Example |
|---------|-------------|---------|
| `text` | Get page text content | `$B text` |
| `html [sel]` | Get innerHTML | `$B html ".main"` |
| `links` | List all links on page | `$B links` |
| `console [--errors]` | Get console log | `$B console --errors` |
| `network [--clear]` | Get network requests | `$B network` |
| `cookies` | List all cookies | `$B cookies` |
| `js <expr>` | Execute JavaScript | `$B js "document.title"` |

### Screenshots and Snapshots

| Command | Description | Example |
|---------|-------------|---------|
| `screenshot [sel] [path]` | Take screenshot | `$B screenshot ./page.png` |
| `snapshot [flags]` | Accessibility tree | `$B snapshot -i` |
| `responsive [prefix]` | Multi-viewport shots | `$B responsive ./layouts` |
| `diff <url1> <url2>` | Visual diff two pages | `$B diff staging.app prod.app` |

### Assertions

| Command | Description | Example |
|---------|-------------|---------|
| `is visible <sel>` | Assert element visible | `$B is visible ".modal"` |
| `is hidden <sel>` | Assert element hidden | `$B is hidden ".loading"` |
| `is enabled <sel>` | Assert element enabled | `$B is enabled "#submit"` |
| `is disabled <sel>` | Assert element disabled | `$B is disabled "#submit"` |
| `is checked <sel>` | Assert checkbox checked | `$B is checked "#agree"` |

## Snapshot Flags

| Flag | Description |
|------|-------------|
| `-i` | Interactive elements only (buttons, links, inputs) with @e refs |
| `-c` | Compact output (no empty nodes) |
| `-d N` | Limit tree depth to N levels |
| `-s selector` | Scope to CSS selector |
| `-D` | Unified diff against previous snapshot |
| `-a` | Annotated screenshot with red overlay boxes |
| `-o path` | Output path for annotated screenshot |
| `-C` | Cursor-interactive elements with @c refs |

After `snapshot -i`, use @refs to interact: `$B click @e3`, `$B fill @e4 "value"`

## Common Patterns

### Login Flow Test

```bash
$B goto https://app.com/login
$B snapshot -i
$B fill @e3 "user@test.com"
$B fill @e4 "password123"
$B click @e5
$B wait ".dashboard"
$B is visible ".welcome-message"
$B screenshot ./evidence/login-success.png
```

### Before/After Diff

```bash
$B goto https://app.com/page
$B snapshot -i          # baseline
# ... make changes ...
$B reload
$B snapshot -i -D       # diff against baseline
```

### Responsive QA

```bash
$B goto https://app.com
$B responsive ./screenshots/responsive
# Creates: responsive-desktop.png, responsive-tablet.png, responsive-mobile.png
$B viewport 375x812    # iPhone dimensions
$B screenshot ./screenshots/iphone.png
```
