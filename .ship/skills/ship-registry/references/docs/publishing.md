---
group: Registry
title: Publishing Packages
order: 2
---

# Publishing Packages

Publishing makes your skills and agents available for others to install. A package is a Git repository with `.ship/ship.jsonc`.

## Manifest requirements

The minimum manifest needs a `module` section and an `exports` section:

```jsonc
{
  "module": {
    "name": "github.com/yourorg/your-package",
    "version": "0.1.0",
    "description": "Skills for structured code review",
    "license": "MIT"
  },
  "exports": {
    "skills": ["skills/code-review"],
    "agents": ["agents/reviewer.jsonc"]
  }
}
```

### Module fields

| Field | Required | Description |
|-------|----------|-------------|
| `name` | yes | Package path. Doubles as the Git clone URL. |
| `version` | yes | Semver version. Must match a Git tag. |
| `description` | no | Human-readable summary |
| `license` | no | SPDX license identifier |

### Exports

Only items listed in `exports` are installed by consumers. Everything else in `.ship/` stays private.

- `skills` -- Skill directory paths relative to `.ship/` (e.g., `"skills/tdd"`)
- `agents` -- Agent file paths relative to `.ship/` (e.g., `"agents/reviewer.jsonc"`)

## Publishing commands

```bash
ship publish            # Publish to the registry
ship publish --dry-run  # Validate manifest, compute hash, print summary
ship publish --tag beta # Pre-release dist-tag
```

Publishing requires authentication via `ship login`. The `--dry-run` flag parses the manifest, computes the tree hash, and prints the package name, version, and hash without making network requests.

## Versioning

Package versions follow semver and map to Git tags:

| Git Tag | Resolved Version |
|---------|-----------------|
| `v0.1.0` | `0.1.0` |
| `v1.2.3` | `1.2.3` |

The `v` prefix is stripped during resolution. When consumers specify `^1.0.0`, Ship queries Git tags via `git ls-remote` and selects the best match.

## Content hashing

Every published version is content-addressed with SHA-256. The hash is recorded in the consumer's lock file and verified on subsequent installs to detect corruption or tampering.

### How the tree hash works

The hashing algorithm in `runtime/src/registry/hash.rs` produces a deterministic hash:

1. Walk the file tree recursively under the package root
2. Exclude `.git/`, `ship.lock`, `.ship/state/`, `.DS_Store`, `Thumbs.db`, and `.swp` files
3. Sort all file paths lexicographically (forward-slash separators for cross-platform determinism)
4. For each file, accumulate: `<relative-path>\0<byte-length>\0<sha256-of-file-content>`
5. SHA-256 the full accumulated string
6. Return `sha256:<lowercase-hex>`

### Per-export hashing

In addition to the full tree hash, Ship computes per-export content hashes. Skill exports (directories) use the tree hash algorithm. Agent exports (single files) use a single-file SHA-256 hash. A combined hash is derived from the sorted per-export hashes.

This means changing one skill's content only changes that skill's hash and the combined hash -- consumers can verify individual exports.

## Checklist before publishing

1. `ship.jsonc` has `module` with `name` and `version`
2. `exports` lists every skill and agent you want to share
3. Each exported skill has a `SKILL.md` with valid frontmatter
4. Each exported agent has a valid `.jsonc` profile
5. `ship validate` passes with no errors
6. Git tag matches the version in `ship.jsonc`
7. `ship publish --dry-run` prints the expected hash
