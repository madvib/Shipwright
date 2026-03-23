# Registry

This guide is maintained as the `ship-publishing` skill. Agents get it in their compiled context automatically.

For the full guide — package structure, version constraints, resolution flow, cache, and publishing — see [.ship/skills/ship-publishing/SKILL.md](../.ship/skills/ship-publishing/SKILL.md).

## Quick reference

```bash
ship add github.com/owner/repo           # add dependency
ship add github.com/owner/repo@^1.0.0    # with version constraint
ship install                              # resolve and fetch all
ship install --frozen                     # CI mode
ship publish                              # publish your package
ship publish --dry-run                    # preview
```

## Package naming

| Format | Example | Use case |
|--------|---------|----------|
| Canonical | `github.com/owner/repo` | Direct GitHub reference |
| Scoped | `@owner/package` | Short alias |
| Unofficial | `@unofficial/package` | Community-seeded, claimable |

## Content integrity

Every dependency is pinned in `ship.lock` with an exact commit SHA and SHA-256 tree hash. `ship install --frozen` enforces reproducibility in CI.
