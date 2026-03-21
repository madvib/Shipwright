---
name: ship-publishing
description: How to publish and consume Ship packages — skills, agents, and workflows on the registry. Use when users want to publish their skills, add dependencies, understand versioning, or learn how the registry works. Covers ship.toml manifest, ship install, ship add, ship publish, and the dependency resolution flow.
tags: [guide, registry, publishing, documentation]
authors: [ship]
---

# Ship Publishing

Ship packages are Git repositories with a `.ship/ship.toml` manifest. The registry resolves dependencies directly from Git remotes — no central server required in v0.1.

## Package Structure

A publishable package is any Git repository containing `.ship/ship.toml` with a `[module]` section:

```toml
[module]
name = "github.com/yourorg/your-package"
version = "0.1.0"
description = "What this package provides"
license = "MIT"
```

The `name` field is the package path — it doubles as the Git clone URL (`https://<name>.git`). The `version` field follows semver and corresponds to Git tags (`v0.1.0`).

## Declaring Exports

The `[exports]` section declares what the package provides to consumers:

```toml
[exports]
skills = ["agents/skills/my-skill"]
agents = ["agents/profiles/my-agent.toml"]
```

Anything not listed in `[exports]` stays private — it works locally but is not installed when someone depends on your package.

## Declaring Dependencies

```toml
[dependencies]
"github.com/better-auth/skills" = "main"
"github.com/acme/toolkit" = "^1.0.0"
"github.com/acme/pinned" = { version = "main", grant = ["Bash"] }
```

Each key is a package path. The value is a version string or inline table with `version` and optional `grant` (tool permissions).

## Version Constraints

| Syntax | Type | Behavior |
|---|---|---|
| `^1.0.0` | Semver caret | `>=1.0.0, <2.0.0` |
| `~1.2.0` | Semver tilde | `>=1.2.0, <1.3.0` |
| `1.2.3` | Exact semver | Exactly `1.2.3` |
| `main` | Branch | Tracks tip (re-resolved on each install) |
| 40-char hex | Commit SHA | Pinned to exact commit |

Semver constraints match against Git tags. Tag `v1.0.0` is normalized by stripping the `v` prefix.

## Consuming Packages

### `ship add <package>[@version]`

```bash
ship add github.com/better-auth/skills          # defaults to @main
ship add github.com/acme/toolkit@^1.0.0         # semver constraint
```

Parses the spec, validates, appends to `[dependencies]`, resolves, updates `ship.lock`, recompiles. Restores `ship.toml` on failure.

### `ship install [--frozen]`

```bash
ship install           # resolve, fetch, lock, compile
ship install --frozen  # fail if ship.lock would change (CI mode)
```

## How Resolution Works

```
ship.toml constraint → parse → git ls-remote → resolve version
  → cache lookup (hit? verify hash) → fetch if miss → store in cache
  → write ship.lock atomically (sorted, deterministic)
```

Key behaviors:
- **Incremental**: existing lockfile entries skip network calls
- **Hash verification**: cache hits are integrity-checked; corrupt entries re-fetched
- **Atomic writes**: temp-file-then-rename for both cache and lockfile

## The Lock File

`ship.lock` pins every dependency to an exact commit and content hash. Fields: `path`, `version`, `commit` (40-char SHA), `hash` (`sha256:<hex>`).

**Always commit it.** Use `ship install --frozen` in CI to enforce reproducibility.

## Cache

Resolved packages stored at `~/.ship/cache/` with content-addressed storage:
- `objects/<sha256>/` — package files keyed by content hash
- `index/<path>@<ver>` — pointer files mapping dep+version to hash+commit

Safe to delete — `ship install` repopulates.

## Publishing

In v0.1, publishing means pushing `.ship/ship.toml` to a public Git repository:
1. Add `[module]` with `name` matching repo path and semver `version`
2. Declare exports in `[exports]`
3. Tag releases with semver tags (`v1.0.0`)
4. Push to a publicly accessible Git remote

### Planned

- `ship publish` — push metadata to the Ship registry API
- Content signing and namespace ownership verification
- Transitive dependency resolution
