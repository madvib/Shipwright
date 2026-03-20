# Agent Session Directory

`.ship-session/` is the per-worktree scratch space for ephemeral agent artifacts. It is gitignored by the compiler and never committed.

## Rule

**All ephemeral working artifacts go in `.ship-session/`. Nothing ephemeral goes in the project root, `.ship/`, or source directories.**

## What belongs here

| Artifact | Path |
|----------|------|
| Job specification | `.ship-session/job-spec.md` |
| UI mockups | `.ship-session/mockup.html` |
| Design screenshots | `.ship-session/design-spec/screenshots/` |
| Design spec | `.ship-session/design-spec/spec.md` |
| Any other working files an agent generates during a job | `.ship-session/<name>` |

## What does NOT belong here

- Source code changes — commit those to the branch
- `.ship/` config — that is the package source of truth, never write working artifacts there
- `design-spec/` or `mockup.html` at project root — always prefix with `.ship-session/`

## Rationale

- `.ship/` stays pure: package manifests, profiles, skills, rules only
- Working artifacts don't pollute git history or diffs
- Consistent location means agents can find each other's outputs reliably
- `.ship-session/` is provisioned automatically by `ship use` via gitignore injection
