---
name: write-adr
stable-id: write-adr
description: Use when committing to a technical or product decision that future contributors — human or agent — need to understand. Captures context, decision, alternatives, consequences, and how to measure success. Do NOT use for minor implementation choices — only decisions with meaningful alternatives that constrain future work.
tags: [architecture, decisions, documentation]
authors: [ship]
---

## When to write an ADR

- You're choosing between two or more real approaches
- The decision will be hard to reverse or costly to change later
- Future agents/contributors will wonder "why not X?" — this answers that
- A capability or architectural boundary is being established

## Structure

Work through each section before calling `create_adr`. Don't skip alternatives — that's the most valuable part.

### 1. Context
What situation, constraint, or requirement forced this decision? Be specific. Include:
- What problem you were solving
- Any constraints (performance, compatibility, team capability, timeline)
- What triggered the decision now vs later

### 2. Decision
One clear statement. "We will X." Not "we might" or "we could consider."

### 3. Alternatives considered
For each alternative you seriously evaluated:
- What it was
- Why it was attractive (be honest — bad options don't get seriously considered)
- Why you rejected it
- At least 2 alternatives. If you can't name any, you haven't decided — you've defaulted.

### 4. Consequences
What this enables. What this constrains. What becomes harder. What technical debt this creates (if any). Be direct about trade-offs.

### 5. How to measure (optional but valuable)
How will you know in 3 months this was the right call? What would make you revisit it?
- A metric that should improve
- A test that would fail if the decision was wrong
- A condition that would trigger revisiting ("if we need X, we'd revisit Y")

## Format for `create_adr`

```
title: <verb + noun> — "Use D1 for cloud workspace state" not "Database decision"

decision: (combine all sections into flowing prose — context first, then decision,
then alternatives with reasons rejected, then consequences, then measurement if relevant)
```

## Quality checks before submitting

- [ ] Someone reading this in 6 months would understand WHY, not just WHAT
- [ ] At least 2 alternatives named with honest rejection reasoning
- [ ] Consequences include at least one constraint (what this makes harder)
- [ ] Title is specific enough to search for ("Use D1" not "Database")

## What ADRs are NOT

- Implementation notes (use a comment or note instead)
- Meeting summaries
- Decisions that can be changed freely with no cost
- Choices between things that are equivalent (coin-flip decisions don't need ADRs)
