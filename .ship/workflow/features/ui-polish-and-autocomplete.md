+++
id = "feat-ui-polish-and-autocomplete"
title = "UI Polish and Autocomplete"
status = "active"
created = "2026-02-27T00:59:11Z"
updated = "2026-02-27T00:59:11Z"
owner = "ship"
release = "v0.1.0-alpha"
spec = "ui-vision---production-roadmap.md"
adrs = []
tags = ["alpha", "ui", "polish", "autocomplete"]
+++

## Why

The alpha UI needs a focused polish pass so daily dogfooding feels fast, predictable, and low-friction.

## Acceptance Criteria

- [ ] Primary app layout and navigation have consistent hierarchy, spacing, and visual affordances.
- [ ] Core relational inputs support autocomplete (issues, specs, releases, features, tags where applicable).
- [ ] Forms provide keyboard-first interaction and inline validation/microcopy.
- [ ] UI behavior for these flows is covered by targeted tests.

## Delivery Todos

- [ ] Define exact polish scope for pass #1 and lock it to this branch.
- [ ] Implement reusable autocomplete field component(s) and wire into existing forms.
- [ ] Add keyboard navigation and selection support in autocomplete dropdowns.
- [ ] Add frontend test coverage for happy path + no-result/error states.
- [ ] Capture any remaining gaps as follow-on issues for post-alpha.

## Notes

This feature intentionally prioritizes practical alpha UX improvements over broad refactors.
