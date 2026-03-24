---
name: design-workflow
description: Use when designing and implementing UI. Full pipeline — brainstorm HTML mockups, preview in browser, extract design spec, dispatch implementation agent. Goes from idea to shipped code.
tags: [design, ui, workflow, browser]
authors: [ship]
---

# Design Workflow

From idea to shipped UI in one session. Brainstorm → preview → spec → implement → verify.

## Phase 1: Brainstorm (visual-brainstorm)

Generate a self-contained HTML mockup at `.ship-session/mockup.html`:
- Tailwind via CDN, no build step
- Realistic data (not lorem ipsum)
- Full visual fidelity — this should look shippable

Serve it:
```bash
python3 -m http.server 8765 --directory .ship-session &
```

**Browse it:**
```
/browse goto http://localhost:8765/mockup.html
/browse screenshot
```

Show the user the screenshot. Iterate on feedback — update the HTML, refresh, re-screenshot. Each round takes seconds.

## Phase 2: Responsive Check

Once the design is approved at desktop:
```
/browse responsive http://localhost:8765/mockup.html
```

This captures mobile + tablet + desktop viewports. Save to `.ship-session/screenshots/`. Fix any responsive issues in the mockup before moving on.

## Phase 3: Design Spec (visual-spec)

Extract the approved mockup into a structured spec:

```markdown
<!-- .ship-session/design-spec/spec.md -->
# Design Spec: <name>

## Screenshots
![Desktop](screenshots/desktop.png)
![Mobile](screenshots/mobile.png)

## Component Breakdown
- Header: sticky, logo left, nav right, auth state
- Hero: full-width, gradient bg, CTA button
- Card grid: 3-col desktop, 1-col mobile, hover shadow

## Design Tokens
- Primary: oklch(0.65 0.15 250)
- Surface: oklch(0.15 0.02 260)
- Text: oklch(0.95 0 0)
- Font: Inter, -apple-system, sans-serif
- Radius: 8px cards, 6px buttons, 4px inputs
- Spacing: 4px base, multiples of 4

## Interaction Notes
- Card hover: translateY(-2px) + shadow transition 150ms
- Mobile nav: hamburger → slide-in panel
- CTA: scale(1.02) on hover, ring on focus

## Implementation Checklist
- [ ] Header with auth-aware nav
- [ ] Hero section with gradient and CTA
- [ ] Card grid with responsive layout
- [ ] Hover states and transitions
- [ ] Mobile navigation
```

## Phase 4: Dispatch Implementation

Create the job spec from the design spec:

```bash
# Write the implementation job
cat > .ship-session/jobs/impl-<feature>.md << 'SPEC'
---
status: ready
agent: web-lane
scope: apps/web/src/
---

# Implement: <feature>

Open `.ship-session/design-spec/spec.md` — that is your implementation brief.

## Acceptance Criteria
- [ ] Matches desktop screenshot
- [ ] Matches mobile screenshot
- [ ] All checklist items from spec complete
- [ ] Uses project design tokens (not hardcoded values)
- [ ] Components are in the right directory per project conventions
SPEC

# Dispatch
bash scripts/dispatch.sh \
  --slug impl-<feature> \
  --agent web-lane \
  --spec .ship-session/jobs/impl-<feature>.md
```

## Phase 5: Verify

After the implementation agent completes:

```bash
# Start the dev server in the implementation worktree
(cd ~/dev/ship-worktrees/impl-<feature> && pnpm dev &)

# Browse and compare
/browse goto http://localhost:3000/<page>
/browse screenshot

# Compare with the approved mockup
# Side-by-side: mockup screenshot vs implementation screenshot
```

Check:
- Layout matches mockup
- Responsive behavior correct
- Interactions work (hover, click, transition)
- No visual regressions on other pages

## The Mechanical Advantage

This workflow is fast because every step produces an artifact the next step consumes:
1. Mockup HTML → browser renders it → screenshot
2. Screenshot → spec extracts tokens/components → implementation brief
3. Brief → agent implements → dev server → browser verifies

No handwaving. No "make it look like the Figma." Every step is verifiable.
