---
group: Registry
title: Installing Packages
order: 3
---

# Installing Packages

Install skills and agents from other Ship packages into your project.

## Adding a dependency

```bash
ship add github.com/better-auth/skills          # Tracks the main branch
ship add github.com/acme/toolkit@^1.0.0         # Semver constraint
ship add github.com/acme/pinned@v1.2.3          # Exact version
```

`ship add` performs these steps in order:

1. Parses the package spec and version constraint
2. Validates the source is reachable via `git ls-remote`
3. Appends the dependency to `ship.jsonc`
4. Resolves all dependencies (including transitive)
5. Updates `ship.lock`
6. Recompiles the active agent

If resolution fails, `ship.jsonc` is restored to its previous state.

## Version constraints

| Syntax | Type | Behavior |
|--------|------|----------|
| `^1.0.0` | Semver caret | `>=1.0.0, <2.0.0` |
| `~1.2.0` | Semver tilde | `>=1.2.0, <1.3.0` |
| `1.2.3` | Exact semver | Exactly `1.2.3` |
| `main` | Branch | Tracks branch tip, re-resolved on each install |
| 40-char hex | Commit SHA | Pinned to exact commit |

Semver constraints match against Git tags. The `v` prefix on tags is stripped automatically.

## Dependency configuration

Dependencies live in `ship.jsonc`. The value can be a simple version string or an object with additional options:

```jsonc
{
  "dependencies": {
    "github.com/better-auth/skills": "main",
    "github.com/acme/toolkit": "^1.0.0",
    "github.com/acme/pinned": {
      "version": "main",
      "grant": ["Bash"]
    }
  }
}
```

The `grant` array explicitly allows tool permissions for that dependency's skills.

## Installing all dependencies

```bash
ship install           # Resolve, fetch, lock, compile
ship install --frozen  # Fail if ship.lock would change (CI mode)
```

`--frozen` is for CI pipelines. It ensures reproducible builds by failing if the lock file would need any changes.

## The lock file

`ship.lock` is a TOML file that pins every dependency to an exact commit and content hash:

```toml
version = 1

[[package]]
path = "github.com/better-auth/skills"
version = "main"
commit = "6a1636950a1d7fc53602639ce7505a4a5d39c797"
hash = "sha256:83fb025b015f9472ea8504cbdf8c8e042eff86f87cb0f69757bb00fbacd5acb9"
```

| Field | Description |
|-------|-------------|
| `path` | Package path (matches the dependency key) |
| `version` | Resolved version string or branch name |
| `commit` | 40-character Git SHA |
| `hash` | `sha256:<hex>` content hash of the package tree |

{% aside type="tip" %}
Always commit `ship.lock` to your repository. Use `ship install --frozen` in CI to enforce that dependencies match exactly.
{% /aside %}

## Resolution algorithm

```
ship.jsonc constraint
  --> parse version spec
  --> git ls-remote (query tags/branches)
  --> resolve best matching version
  --> check local cache (verify content hash if hit)
  --> fetch from remote if cache miss or hash mismatch
  --> write ship.lock atomically (temp file, then rename)
```

Key behaviors:

- **Incremental** -- Existing lock file entries skip network calls
- **Hash verification** -- Cache hits are integrity-checked against their SHA-256 hash; corrupt entries are re-fetched
- **Atomic writes** -- Temp-file-then-rename for both cache and lock file

## Cache

Resolved packages are stored in `~/.ship/cache/` with content-addressed storage:

- `objects/<sha256>/` -- Package files keyed by content hash
- `index/<path>@<ver>` -- Pointer files mapping dependency + version to hash + commit

The cache is safe to delete. `ship install` repopulates it on the next run.

## Removing a dependency

```bash
ship skills remove <skill-id>
```

This removes the skill reference from agent profiles and cleans up installed files.
