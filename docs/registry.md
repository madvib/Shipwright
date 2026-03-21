# Ship Registry

The Ship registry is a package distribution layer for agent configurations — skills, profiles, and MCP server definitions. Packages are content-addressed and verified by SHA-256 tree hash.

## Package Names and Namespaces

Ship supports three naming formats:

### Canonical: `host.tld/owner/repo`

The canonical form uses the git host URL path. This is the primary format for packages sourced from git repositories.

```toml
[module]
name = "github.com/acme/agent-toolkit"
```

Any git host works: `github.com`, `gitlab.com`, `sr.ht`, self-hosted instances.

### Scoped: `@scope/name`

Short, human-friendly names for organizations and curated collections. Scopes must be claimed before publishing.

```toml
[module]
name = "@acme/toolkit"
```

### Unofficial: `@unofficial/$package`

Packages seeded from public repositories that have not been claimed by their original author. These are community-contributed wrappers.

```toml
[module]
name = "@unofficial/better-auth"
```

When the original author claims the package, they may publish under their own scope and the `@unofficial/` version is marked deprecated.

## Name Validation

All package names must match `^[a-z0-9._/@-]+$`:
- Lowercase ASCII letters and digits
- Dots (`.`), hyphens (`-`), underscores (`_`)
- Forward slashes (`/`) for path segments
- `@` for scoped names

Uppercase letters, spaces, and other characters are rejected at parse time.

## Claiming Rules

1. **Canonical names** — ownership is verified through the git repository. The publisher must have push access to the repository matching the package name.
2. **Scoped names** — a scope (`@org`) is claimed by the first authenticated user to publish under it. Transfer requires manual intervention.
3. **Unofficial names** — anyone can publish under `@unofficial/`. If the canonical author later claims the package, their version takes precedence.

## Version Resolution

Dependencies declare a version constraint (semver range, branch name, or commit SHA). The resolver:

1. Queries the git repository for available tags.
2. Selects the highest tag satisfying the constraint.
3. Records the resolved version and commit in `ship.lock`.

Branch names (e.g. `main`) resolve to the latest commit on that branch.

## Content Integrity

### SHA-256 Tree Hashing (v0.1.0)

Every package is content-addressed using a deterministic SHA-256 tree hash:

1. Collect all files recursively, excluding `.git/`, `.DS_Store`, `Thumbs.db`, `*.swp`, and `ship.lock`.
2. Sort file paths lexicographically (forward-slash separators).
3. For each file: `"<rel-path>\0<byte-length>\0<sha256-of-content>"`.
4. SHA-256 of the accumulated string.
5. Stored as `sha256:<lowercase-hex>`.

The lockfile records the expected hash. On `ship install`, the cache hash is verified against the lockfile. A mismatch aborts the install.

### Cryptographic Signing (deferred to v0.2)

v0.1.0 relies on SHA-256 content hashing for integrity verification. This prevents accidental corruption and detects tampering when combined with the lockfile.

Full cryptographic package signing (Ed25519 or similar) is planned for v0.2. The rationale for deferring:

- SHA-256 tree hashing already provides content verification.
- Key management UX (key generation, distribution, revocation) requires careful design.
- v0.1.0 focuses on establishing the registry protocol; signing layers on top without breaking changes.

No `ship sign` or `ship verify` commands exist in v0.1.0.

## Publishing

```sh
# Preview what would be published (no network)
ship publish --dry-run

# Publish to the registry
ship publish

# Publish with a pre-release tag
ship publish --tag beta
```

Publishing requires authentication (`ship login`). The `--dry-run` flag parses the manifest, computes the tree hash, and prints the package name, version, and hash without making any network requests.

## Install Flow

```sh
# Install all dependencies from ship.toml
ship install

# Add a new dependency
ship add github.com/owner/repo@^1.0.0

# Fail if the lockfile would change (CI mode)
ship install --frozen
```

`ship install` resolves, fetches, and caches packages in `~/.ship/cache/objects/<hash>/`. The lockfile (`ship.lock`) records exact versions and hashes for reproducible installs.
