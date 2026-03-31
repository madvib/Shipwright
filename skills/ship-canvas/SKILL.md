---
name: ship-canvas
stable-id: ship-canvas
description: Use when the conversation needs a visual surface — mockups, data visualizations, animated demos, or any idea better expressed as a rendered artifact than as text.
tags: [design, canvas, visualization, animation]
authors: [ship]
artifacts: [html]
---

# Ship Canvas

Visual language between human and agent. You produce self-contained HTML artifacts. The human sees them live in Ship Studio's canvas, draws on them, annotates, sketches alternatives — you iterate from their visual feedback.

Not chat with pictures. A shared thinking surface.

## What to make

- **Mockups** — UI layouts, pages, components, design explorations
- **Data visualizations** — charts, graphs, dashboards, metrics
- **Animated demos** — flows, sequences, motion studies, concept animations
- **Diagrams** — architecture, relationships, timelines, processes

## Style

{% if style == "bare" %}
**Bare canvas.** No Ship tokens. Write all CSS from scratch. The base template structure (fonts, CDN scripts) is still a useful starting point, but replace the `:root` token block entirely.
{% elif style == "neutral" %}
**Neutral style.** Use Ship layout primitives (`.card`, grid, spacing, fonts) but replace `--primary`, `--accent`, and status colors with neutral grays. Do not let the Ship palette show — this canvas should look like it belongs to a different product's visual language.
{% else %}
**Ship style (default).** Start every canvas from `assets/base-template.html`. It includes Ship design tokens, Syne + DM Sans, Tailwind CDN, GSAP, and Chart.js wired for both light and dark.
{% endif %}

## Layout

{% if density == "focused" %}
**One concept per canvas.** 2–3 sections maximum. Use whitespace as a design element — a half-empty canvas that makes one thing clear is better than a dense one that makes five things muddy. If the topic has more to cover, make a second canvas rather than crowding this one.
{% else %}
**Rich layout.** Comprehensive coverage is appropriate here — full dashboards, multiple data sections, detailed specs. Still keep a clear visual hierarchy.
{% endif %}

## Rendering rules

- `data-theme="dark"` on `<html>` by default. Support both via CSS vars.
- Tailwind for layout/spacing. Ship CSS vars for all colors — never hardcode hex.
- Realistic content. No lorem ipsum, no "Chart Title", no "Label".
- One file. No external assets beyond the CDN scripts in the template.

## Animations

{% if animate %}
Include GSAP entrance animations. Entrance and emphasis patterns below.

Entrance — fade + translate on load:
```js
gsap.from('#hero', { opacity: 0, y: 16, duration: 0.5, ease: 'power2.out' })
gsap.from('.card', { opacity: 0, y: 12, stagger: 0.08, duration: 0.4, delay: 0.2 })
```

Data reveal — counters after a short delay:
```js
gsap.from(counter, { textContent: 0, duration: 1.2, ease: 'power2.out', snap: { textContent: 1 }, delay: 0.5 })
```

Emphasis — pulse when pointing something out:
```js
gsap.to(target, { scale: 1.03, duration: 0.15, yoyo: true, repeat: 1, ease: 'power1.inOut' })
```
{% else %}
Skip GSAP animations. Static canvas — no entrance tweens, no counters. Load the GSAP script only if you have a specific non-decorative use for it.
{% endif %}

## Theme awareness

CSS (backgrounds, borders, text) updates automatically when Studio toggles the theme — `data-theme` is set by the injected bridge before your code runs.

JS-driven colors (Chart.js datasets, GSAP color tweens) must be reinitialized on theme change:

```js
let _chart = null
function buildChart() {
  const dark = document.documentElement.getAttribute('data-theme') === 'dark'
  const primary = dark ? 'oklch(0.77 0.16 70)' : 'oklch(0.67 0.16 58)'
  if (_chart) _chart.destroy()
  _chart = new Chart(el, { /* use dark/primary inline */ })
}
buildChart()

window.addEventListener('message', (e) => {
  if (e.data?.type !== 'theme') return
  buildChart() // data-theme already updated by the time this fires
})
```

For GSAP timelines with theme-dependent colors: kill, re-read, restart inside the same `message` handler.

## Chart patterns

Always wrap Chart.js creation in a `buildChart()` function as shown above. Never resolve colors outside that function — they must be read fresh on every call.

## Iteration

Save each revision as `canvas-{slug}-v2.html`. Never overwrite — the history is the conversation. On approval, output the final path and a paragraph: layout decisions, color rationale, what visual feedback drove it.

Path: `.ship-session/ship-canvas/canvas-{slug}-v{n}.html`

## Quality bar

Every canvas should look finished. Production typography, contrast, spacing. You are a senior designer who codes. If the human drew something rough, interpret the intent — don't reproduce the roughness.
