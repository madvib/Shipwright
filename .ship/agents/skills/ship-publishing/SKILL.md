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

The `name` field is the package path — it doubles as the Git clone URL (`https://<name>.git`). Use the `host/owner/repo` convention. The `version` field follows semver and corresponds to Git tags (`v0.1.0`).

## Declaring Exports

The `[exports]` section declares what the package provides to consumers:

```toml
[exports]
skills = [
    "agents/skills/my-skill",
    "agents/skills/another-skill",
]
agents = [
    "agents/profiles/my-agent.toml",
]
```

- **skills** — paths to skill directories (each containing a `SKILL.md`). These are installed into the consumer's skill library and become available to their agents.
- **agents** — paths to agent profile TOML files. These are installed as available profiles the consumer can activate.

Anything not listed in `[exports]` stays private — it works locally but is not installed when someone depends on your package.

## Declaring Dependencies

The `[dependencies]` section lists packages your project consumes:

```toml
[dependencies]
"github.com/better-auth/skills" = "main"
"github.com/garrytan/gstack" = "main"
"github.com/acme/toolkit" = "^1.0.0"
"github.com/acme/pinned" = { version = "main", grant = ["Bash"] }
```

Each key is a package path. The value is either a version string or an inline table with `version` and optional `grant` (tool permissions granted to that dependency's skills).

## Version Constraints

Three constraint types are supported:

| Syntax | Type | Behavior |
|---|---|---|
| `^1.0.0` | Semver caret | Compatible updates: `>=1.0.0, <2.0.0` |
| `~1.2.0` | Semver tilde | Patch-level updates: `>=1.2.0, <1.3.0` |
| `>=1.0.0` | Semver range | Any matching semver range expression |
| `1.2.3` | Exact semver | Exactly version `1.2.3` |
| `main` | Branch | Tracks the branch tip (re-resolved on each install) |
| `feat/my-feature` | Branch | Any non-semver, non-SHA string is a branch name |
| `abc123...` (40 hex chars) | Commit SHA | Pinned to an exact commit, no network call needed |

Semver constraints match against Git tags. A tag like `v1.0.0` is normalized by stripping the `v` prefix before comparison. The resolver picks the highest matching version.

Branch constraints always re-resolve to the current tip via `git ls-remote`, so `ship install` may update the lockfile even if `ship.toml` hasn't changed.

## Consuming Packages

### `ship add <package>[@version]`

Add a dependency and install it in one step:

```bash
ship add github.com/better-auth/skills          # defaults to @main
ship add github.com/acme/toolkit@^1.0.0         # semver constraint
ship add github.com/acme/repo@v2.1.0            # exact tag
ship add github.com/acme/repo@abc123...def       # pinned commit
```

What happens:
1. Parses the spec — splits on the last `@` to separate path from version.
2. Validates the dep is not already in `ship.toml`.
3. Appends the entry to `[dependencies]` (creates the section if absent).
4. Resolves and fetches only the new dependency.
5. Updates `ship.lock`.
6. Recompiles provider targets.

On failure after modifying `ship.toml`, the file is restored from a backup.

### `ship install [--frozen]`

Resolve all dependencies and populate the cache:

```bash
ship install           # resolve, fetch, lock, compile
ship install --frozen  # fail if ship.lock would change (CI mode)
```

`--frozen` is for CI pipelines: it ensures the lockfile matches `ship.toml` exactly and fails rather than updating it.

## How Resolution Works

The full pipeline for each dependency:

```
ship.toml constraint
    |
    v
parse_constraint() --> Semver / Branch / Commit
    |
    v
git ls-remote --tags --heads https://<path>.git
    |
    v
resolve_version() --> ResolvedVersion { tag, commit }
    |
    v
cache.get(path, tag)  -- hit? verify hash, return
    |                      miss or corrupt?
    v
fetch_package_content(url, commit, tmpdir)
    |  tries git archive first, falls back to shallow clone
    v
cache.store(path, tag, commit, tmpdir) --> CachedPackage
    |  computes SHA-256 tree hash, writes to objects/ + index/
    v
LockedPackage { path, version, commit, hash }
    |
    v
ship.lock written atomically (sorted, deterministic)
```

Key behaviors:
- **Incremental resolution**: When a lockfile exists and is in sync, no network calls are made — packages are served from cache. Only newly added or removed deps trigger resolution.
- **Hash verification**: On cache hit, the tree hash is recomputed and compared. Corrupt entries are evicted and re-fetched.
- **Atomic writes**: Both the cache index and `ship.lock` use write-to-temp-then-rename to prevent partial writes.

## The Lock File

`ship.lock` pins every dependency to an exact commit and content hash:

```toml
version = 1

[[package]]
path = "github.com/better-auth/skills"
version = "main"
commit = "6a1636950a1d7fc53602639ce7505a4a5d39c797"
hash = "sha256:83fb025b015f9472ea8504cbdf8c8e042eff86f87cb0f69757bb00fbacd5acb9"

[[package]]
path = "github.com/garrytan/gstack"
version = "main"
commit = "1f4b6fd7a2a349dfc6f04d158b8b7778b5b74232"
hash = "sha256:0952a3319f662cfb8a0e8e7420135b49fb9dc118e45e735fa6e5ef14d1104895"
```

Fields:
- **path** — the dependency's package path (matches the key in `[dependencies]`).
- **version** — the resolved tag or branch name.
- **commit** — the exact 40-character Git commit SHA.
- **hash** — `sha256:<hex>` content hash of the package tree (excludes `.git/`, `ship.lock`, `.DS_Store`, `*.swp`, `Thumbs.db`).

**When to commit it**: Always. The lockfile ensures reproducible installs across machines and CI. Use `ship install --frozen` in CI to enforce this.

**Output is deterministic**: packages are sorted by path, fields in fixed order. Two runs with the same inputs produce byte-identical output.

## Cache

Resolved packages are stored in `~/.ship/cache/` using content-addressed storage:

```
~/.ship/cache/
  objects/<sha256-hex>/       # package files, keyed by content hash
  index/<encoded-path>@<ver>  # maps dep+version to hash+commit
```

- **objects/** — the actual package files. Directory name is the SHA-256 hex of the content tree. Two packages with identical content share the same object directory.
- **index/** — lightweight pointer files. Each contains `<sha256-hex>\n<commit>\n`. The key is the URL-encoded dep path + `@` + URL-encoded version.

The cache is global (shared across all projects). It is safe to delete `~/.ship/cache/` — `ship install` will repopulate it.

## Publishing

`ship publish` does not exist yet. In v0.1, publishing means pushing your `.ship/ship.toml` to a public Git repository. Consumers add your package path as a dependency and Ship resolves it directly via Git.

To make your package consumable:
1. Add a `[module]` section with `name` matching your repo path and a semver `version`.
2. Declare exports in `[exports]` — only listed skills and agents are installed by consumers.
3. Tag releases with semver tags (`v1.0.0`, `v1.1.0`) so consumers can use caret/tilde constraints.
4. Push to a publicly accessible Git remote.

### Planned

- **`ship publish`** — push package metadata to the Ship registry API at `getship.dev`, enabling discovery, search, and verified ownership.
- **Registry API** — centralized package index with search, download counts, and namespace management. Git remains the source of truth for content.
- **Content signing** — packages signed with author keys, verified on install.
- **Namespace ownership** — verified ownership of `github.com/org/*` namespaces via OAuth, preventing impersonation.
- **Transitive dependencies** — the current resolver is flat (one level). Planned: recursive resolution with conflict detection.
