# Ship Package Registry

Ship packages are skills and agents distributed through Git repositories and indexed by the Ship registry at getship.dev.

## How it works

A Ship package is a Git repository with a `.ship/ship.jsonc` manifest declaring what it exports. The manifest lists skills (directories under `.ship/skills/`) and agents (files under `.ship/agents/`). Each export is individually installable.

The registry is an **index**, not a blob store. Package content lives in Git. The registry tracks metadata, versions, content hashes, and search indexes. When you run `ship install`, the CLI fetches directly from the Git host — the registry tells it where to look and what hash to expect.

## Packages, skills, and agents

A single repository can export multiple skills and agents. Each is a distinct installable unit:

```jsonc
// .ship/ship.jsonc
{
  "module": {
    "name": "github.com/acme/toolkit",
    "version": "1.0.0",
    "description": "Acme agent toolkit"
  },
  "exports": {
    "skills": ["skills/tdd", "skills/code-review", "skills/deploy"],
    "agents": ["agents/backend.jsonc", "agents/frontend.jsonc"]
  }
}
```

Users depend on the repository and use what they need:

```jsonc
{
  "dependencies": {
    "github.com/acme/toolkit": "^1.0.0"
  }
}
```

The compiler resolves skill and agent references from installed dependencies at compile time.

## Namespaces and aliases

Full repository paths (`github.com/owner/repo`) are the canonical format. Namespaces provide short, memorable aliases:

```
@ship/tdd           ->  github.com/madvib/ship  (skill: tdd)
@ship/commander     ->  github.com/madvib/ship  (agent: commander)
@better-auth/auth   ->  github.com/better-auth/skills
@gstack/qa          ->  github.com/garrytan/gstack
```

The `@scope` prefix maps to a repository. The part after the slash identifies the export. Namespace mappings are maintained in `registry/aliases.json` and mirrored in the registry API.

Three scopes exist in the registry:

| Scope | Meaning |
|-------|---------|
| **ship** | Published and maintained by the Ship team |
| **community** | Claimed by the repository owner, verified via GitHub |
| **curated** | Indexed by Ship, unclaimed — repo owner can claim at any time |

## Version resolution

Dependencies declare a version constraint. The CLI resolves it against Git tags:

| Constraint | Example | Resolves to |
|-----------|---------|-------------|
| Semver range | `"^1.0.0"` | Highest matching tag (e.g., `v1.2.3`) |
| Branch | `"main"` | HEAD commit of the branch |
| Exact commit | `"a1b2c3..."` | That commit, no resolution needed |

Resolution runs `git ls-remote` to list tags, matches against the constraint using semver, and selects the highest compatible version. The resolved commit SHA is pinned in the lockfile.

## Lockfile and integrity

`ship.lock` pins every dependency to an exact commit and content hash:

```
[[package]]
path = "github.com/acme/toolkit"
version = "1.2.3"
commit = "a1b2c3d4e5f6..."
hash = "sha256:..."

[package.export_hashes]
"skills/tdd" = "sha256:..."
"skills/code-review" = "sha256:..."
"agents/backend.jsonc" = "sha256:..."
```

On every install, the CLI fetches the package at the pinned commit, computes the SHA-256 hash of the fetched content, and compares it to the lockfile. If the hash doesn't match, the install fails. This catches:

- Corrupted downloads
- Tampered content (e.g., a force-pushed tag)
- Upstream changes between installs

`ship install --frozen` refuses to update the lockfile — if any dependency would change, it fails. Use this in CI.

## Content security

Two layers of scanning protect against malicious packages:

**On publish:** The registry scans every skill for prompt injection patterns — known attack phrases, encoded payloads, and obfuscation techniques. Skills that fail the scan are rejected.

**On install:** The CLI scans all downloaded files for hidden Unicode characters — bidirectional overrides, tag characters, zero-width joiners, and other invisible codepoints that LLMs tokenize but humans can't see. Critical findings block the install. Run `ship audit` to scan manually.

## Publishing

```bash
git tag v1.0.0
git push origin v1.0.0
ship publish
```

Publishing indexes your package in the registry. The CLI reads `.ship/ship.jsonc`, computes per-export content hashes, and sends them to the registry API. The registry fetches your manifest and skills from GitHub at the tagged commit, verifies content, and creates the index entry.

You must be authenticated (`ship login`) and the repository must be public on GitHub.

`ship publish --dry-run` previews what would be published without making any network requests.

## Fetching strategy

When installing a dependency, the CLI tries three strategies in order:

1. **GitHub tarball** — Downloads a compressed archive from GitHub's CDN. Fastest option, works for public GitHub repos.
2. **Sparse checkout** — Uses `git sparse-checkout` to fetch only `.ship/` from the remote. Works with any Git host that supports Git 2.25+.
3. **Full clone** — Shallow clone of the entire repository. Last resort, works everywhere.

Non-GitHub hosts (GitLab, Codeberg, Gitea) skip strategy 1 and use sparse checkout or clone directly.

## Cache

Fetched packages are cached locally at `~/.ship/cache/`. The cache is keyed by package path and commit SHA. Subsequent installs of the same version skip the network entirely.

## Quick reference

```bash
ship add github.com/owner/repo           # add dependency
ship add github.com/owner/repo@^1.0.0    # with version constraint
ship install                              # resolve and fetch all
ship install --frozen                     # CI mode — fail if lockfile would change
ship publish                              # publish to registry
ship publish --dry-run                    # preview without network
ship audit                                # scan for hidden Unicode
```
