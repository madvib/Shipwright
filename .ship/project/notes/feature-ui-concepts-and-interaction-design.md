+++
id = "386b85ef-ff19-4faf-944d-ccc07dabf9ab"
title = "Feature UI — Concepts and Interaction Design"
created = "2026-02-27T16:08:55.071029689Z"
updated = "2026-02-27T16:08:55.071029689Z"
tags = []
+++

# Feature UI — Concepts and Interaction Design

_Alpha scoping: Roadmap + Feature Detail. Catalog view is post-alpha._

---

## Core Principle

Features are the product record — not work items. The UI should feel like a **product catalog with a project management layer**, not the other way around. Linear is the reference for issue UX. Notion is the reference for feature UX (but leaner, no blocks).

---

## View 1: Roadmap (primary feature view)

Horizontal swim lanes by status. Fixed columns, not drag-configurable in alpha.

```
┌─────────────────────────────────────────────────────────────────┐
│  Features                                      [+ New Feature]  │
├──────────────┬──────────────┬──────────────────┬────────────────┤
│   PLANNED    │  IN-PROGRESS │   IMPLEMENTED    │  DEPRECATED    │
│   (3)        │  (2)         │   (12)           │  (1)           │
├──────────────┼──────────────┼──────────────────┼────────────────┤
│ ┌──────────┐ │ ┌──────────┐ │ ┌──────────────┐ │                │
│ │Auth SSO  │ │ │Billing   │ │ │Issue Kanban  │ │                │
│ │          │ │ │Module    │ │ │v0.1.0-alpha  │ │                │
│ │v0.2.0    │ │ │          │ │ └──────────────┘ │                │
│ └──────────┘ │ │feature/  │ │ ┌──────────────┐ │                │
│ ┌──────────┐ │ │billing ●│ │ │MCP Server    │ │                │
│ │Dark Mode │ │ └──────────┘ │ │v0.1.0-alpha  │ │                │
│ └──────────┘ │ ┌──────────┐ │ └──────────────┘ │                │
│              │ │Git Hooks │ │                  │                │
│              │ │feature/  │ │  [show all 12]   │                │
│              │ │hooks ●  │ │                  │                │
│              │ └──────────┘ │                  │                │
└──────────────┴──────────────┴──────────────────┴────────────────┘
  ● = branch active (worktree or checkout)
```

**Card design:**
- Title (bold)
- Release tag (if set): `v0.1.0-alpha` chip
- Branch indicator: `feature/billing` + green dot if currently checked out
- Linked spec: small icon
- No issue count in alpha (too noisy)

**Interactions:**
- Click card → Feature Detail panel slides in from right (no nav, no page change)
- `+ New Feature` → inline title input in the PLANNED column, then detail opens
- Drag within column to reorder (cosmetic only in alpha)
- Drag across columns → status change (writes file)

---

## View 2: Feature Detail (right panel or full page)

```
┌───────────────────────────────────────────────────────┐
│  ← Back to Roadmap                       [Edit] [···] │
├───────────────────────────────────────────────────────┤
│  Billing Module                                        │
│                                                        │
│  Status  ┌─────────────┐  Release  ┌───────────────┐  │
│          │ in-progress ▾│           │ v0.1.0-alpha ▾│  │
│          └─────────────┘           └───────────────┘  │
│                                                        │
│  Branch   feature/billing  ●                           │
│  Spec     billing-spec.md  ↗                           │
│  Version  (not yet shipped)                            │
│  Tags     [billing] [payments]                         │
│                                                        │
│ ─── Description ──────────────────────────────────── │
│  Stripe integration for subscription billing.          │
│  Supports monthly/annual plans with proration.         │
│  (one paragraph — used in docs/changelog generation)  │
│                                                        │
│ ─── Acceptance Criteria ──────────────────────────── │
│  ☑ Stripe webhook integration                          │
│  ☐ Invoice generation                                  │
│  ☐ Trial period handling                               │
│                                                        │
│ ─── Delivery Todos ───────────────────────────────── │
│  ☐ Wire Stripe customer portal                         │
│                                                        │
│ ─── Implementation Notes ─────────────────────────── │
│  Using stripe-rust crate. Webhooks handled in...      │
│                                                        │
│ ─── History ──────────────────────────────────────── │
│  2026-02-27 Created                                    │
│  2026-02-28 Moved to in-progress (branch created)     │
└───────────────────────────────────────────────────────┘
```

**Key interactions:**
- Status dropdown → writes frontmatter, emits event
- Branch field with ● indicator: click → opens terminal to that branch (or triggers `ship feature start`)
- "Supersedes" field appears when status = deprecated
- Version populates automatically when `ship feature done` is called

---

## View 3: Catalog (post-alpha, but design it now)

Implemented features only. Searchable. Description-first layout.

```
┌─────────────────────────────────────────────────────────────┐
│  Product Catalog  [v0.1.0-alpha ▾]           [🔍 Search]   │
├─────────────────────────────────────────────────────────────┤
│  Issue Kanban                            v0.1.0-alpha        │
│  Drag-and-drop status board. Columns from config. File-      │
│  based — moving a card moves a file.                         │
│  [kanban] [issues]                                          │
├─────────────────────────────────────────────────────────────┤
│  MCP Server                              v0.1.0-alpha        │
│  Persistent project memory across agent sessions.            │
│  TypeScript MCP protocol over stdio.                         │
│  [mcp] [agents]                                             │
├─────────────────────────────────────────────────────────────┤
│  Git Hooks                               v0.1.0-alpha        │
│  post-checkout generates CLAUDE.md + .mcp.json. pre-commit   │
│  blocks staging generated files.                             │
│  [git] [agents]                                             │
└─────────────────────────────────────────────────────────────┘
```

This view is what you'd paste into marketing copy or hand to a new contributor as "what does this product do?"

---

## CLI surface (what's missing)

```bash
ship feature list                     # all, any status
ship feature list --status planned    # filter
ship feature start <id>               # creates branch, checks out, links UUID
ship feature done <id>                # marks implemented, sets version from active release
ship feature catalog                  # list implemented only, description-first
ship feature changelog                # group by version, emit markdown
```

---

## Agent / MCP tools (what's missing)

```
get_feature_catalog          # returns Vec<{title, description, version, tags}> for implemented
ship_feature_start <id>      # branch + checkout via MCP
ship_feature_done <id>       # mark implemented
```

`get_feature_catalog` is the unlock for "agent knows what the product does without being told." Include brief summaries in CLAUDE.md for implemented features — agents stop suggesting features that already exist.

---

## Open design questions

- **Feature → Issue linking**: should features have a list of related issues? Or is branch + spec enough?
- **Multiple branches per feature**: e.g. `feature/billing-api` + `feature/billing-ui`. Allow array of branches, or separate features?
- **Feature creation flow**: does `ship feature start` require a planned feature to exist, or can it create one inline? Recommend: allow inline for speed, but encourage planned-first.
- **Supersedes UX**: when marking deprecated, auto-prompt "does this supersede another feature?" — would keep catalog clean.
