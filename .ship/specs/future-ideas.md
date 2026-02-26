# Future Ideas Backlog

**Status:** Active  
**Last Updated:** 2026-02-25

Purpose: capture valuable post-alpha concepts without polluting alpha scope.

---

## Candidate Ideas

## Graph / Topology Views (Post-Alpha)

Why:

- Visualize links across vision, features, specs, issues, and ADRs.
- Improve navigation and impact analysis for large projects.

Not alpha because:

- Requires stable link semantics and strong data integrity first.
- UI graph experience is expensive to polish and easy to overbuild.

Prerequisites:

1. Canonical typed-link model across all document primitives.
2. Reliable backlink/index projection.
3. Workflow semantics finalized and test-covered.

---

## Auto Mode Switching By Feature (Post-Alpha)

Why:

- Reduce manual mode management during execution phases.

Not alpha because:

- Needs mature policy engine and clear override rules.
- Risky to hide behavior from users during early adoption.

Prerequisites:

1. Feature lifecycle states formalized.
2. Mode-policy mapping spec finalized.
3. Explicit UX for opt-in automation and manual override.
