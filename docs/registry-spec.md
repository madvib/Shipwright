# Ship Registry & Resolver Specification

**Version:** 0.1-draft
**Date:** 2026-03-18
**Status:** draft
**Implements:** `registry-capabilities.json` v0.1 capabilities

---

## 1. Overview

Ship's package system makes git repos the registry. A package is any git repo
containing `.ship/ship.toml`. Versions are git tags. The resolver produces a
`ship.lock` that pins the exact commit + content hash of every dependency.
`ship install` fetches missing packages into a global content-addressed cache,
then compiles directly to provider-native files.

There is no central artifact store. Ship owns the **sum database** (integrity
anchor), the **proxy** (availability layer), and the **registry index**
(discovery). Packages themselves live in git.

---

## 2. Manifest Format — `ship.toml`

**Location:** `.ship/ship.toml`
**Format:** TOML only. No YAML or JSON alternates accepted.
**Purpose:** Module identity, dependencies, exports, compile targets, resolver config.

Presence of `.ship/ship.toml` makes the directory a valid Ship package.

```toml
# .ship/ship.toml — complete example

[module]
name        = "github.com/acme/agent-kit"   # required; namespaced path
version     = "1.2.0"                        # required; semver
description = "Reusable agents for ACME"    # optional
license     = "MIT"                          # optional; SPDX identifier

[dependencies]
# version constraint forms:
#   semver range:  "^1.0.0" | "~1.2.0" | "=1.2.3" | ">=1.0.0,<2.0.0"
#   branch pin:    "main"   (locked to HEAD commit at resolve time)
#   exact commit:  "abc1234def456..."  (40-char SHA)
#
# grant: explicit permission grants required by skills in this dep.
# Compiler refuses to compile if any required grant is missing.

"github.com/anthropics/skills"    = { version = "^2.0.0", grant = ["Bash(git:*)", "Read"] }
"github.com/acme/internal-rules"  = { version = "main" }
"github.com/owner/pinned"         = { version = "abc1234def456abc1234def456abc1234def456ab" }

[exports]
# Only what is listed here is resolvable as a dependency.
# Paths are relative to .ship/
# Omitting [exports] = no public surface (consume-only package).

skills = [
  "skills/git-helper",      # directory with SKILL.md
  "skills/code-review",
]
agents = [
  "agents/default.toml",
  "agents/reviewer.toml",
]

[compile]
# Provider targets for ship install / ship compile.
# Each listed provider receives compiled output.
providers = ["claude", "cursor", "codex"]

[compile.claude]
# Per-provider overrides. All fields optional.
model           = "claude-opus-4-6"
context_window  = 200000

[compile.cursor]
model = "claude-sonnet-4-6"

[resolver]
# sumdb: content hash verification endpoint.
# Default: sum.ship.dev
# Set to "" or "off" to disable (private/air-gapped; not recommended).
sumdb = "sum.ship.dev"

# proxy: resolver fetch order. Comma-separated.
# "direct" = fetch from git origin directly.
# Default: "proxy.ship.dev,direct"
proxy = "proxy.ship.dev,direct"

# nosumcheck: patterns excluded from sum DB verification.
# Glob patterns matched against package path. Use for private packages.
# Equivalent to GONOSUMCHECK.
nosumcheck = ["github.com/acme/private-*"]
```

### 2.1 Field Reference — `[module]`

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `name` | string | yes | Full namespaced path. Short names (no host) require registry.ship.dev registration. |
| `version` | string | yes | Semver (no `v` prefix in manifest; `v` prefix supported in dep constraints). |
| `description` | string | no | Max 1024 chars. |
| `license` | string | no | SPDX identifier. |

### 2.2 Field Reference — `[dependencies]`

Inline table per dependency. Key is the package path.

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `version` | string | yes | Semver range, branch name, or 40-char commit SHA. |
| `grant` | array of strings | no | Explicit permission grants for tools required by skills in this dep. Compiler validates. |

Shorthand (version only, no grants) is valid when the dep requires no tool permissions:
```toml
"github.com/owner/rules-only" = "^1.0.0"
```

### 2.3 Field Reference — `[exports]`

| Field | Type | Notes |
|-------|------|-------|
| `skills` | array of strings | Paths relative to `.ship/`. Each must be a directory containing `SKILL.md`. |
| `agents` | array of strings | Paths relative to `.ship/`. Each must be a `.toml` file. |

A dep reference to a non-exported path fails at resolve time with:
```
error: 'github.com/owner/pkg/skills/internal-only' is not exported by that package
```

---

## 3. Lockfile Format — `ship.lock`

**Location:** `.ship/ship.lock`
**Format:** TOML.
**Committed:** yes — alongside `ship.toml`.
**Owned by:** tooling only. Do not hand-edit.

```toml
# ship.lock — machine-generated, commit this file

version = 1

[[package]]
path    = "github.com/anthropics/skills"
version = "v2.1.0"
commit  = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2"
hash    = "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"

[[package]]
path    = "github.com/acme/internal-rules"
version = "main"
commit  = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef"
hash    = "sha256:abc123..."

# Transitive dependencies are also locked:
[[package]]
path    = "github.com/anthropics/base-rules"
version = "v1.0.0"
commit  = "cafecafecafecafecafecafecafecafecafecafe"
hash    = "sha256:def456..."
```

### 3.1 Field Reference

| Field | Type | Notes |
|-------|------|-------|
| `path` | string | Package path (no version suffix). |
| `version` | string | Resolved version string (semver tag with `v` prefix, branch name, or commit SHA). |
| `commit` | string | Exact 40-char git commit SHA. For tags and branches, this is the resolved HEAD at lock time. |
| `hash` | string | `sha256:` prefixed hex digest of the package content tree. See §6 (Content Hash). |

### 3.2 Invariants

- `ship install` with existing lockfile is **deterministic** — identical output every time.
- `ship install` without lockfile resolves from constraints and writes lockfile.
- Modifying `ship.toml` deps without running `ship update` produces a **lockfile mismatch error**:
  ```
  error: ship.toml and ship.lock are out of sync
    added in ship.toml:   github.com/new/dep
    run: ship install   to resolve and update the lockfile
  ```
- The lockfile includes **all transitive dependencies**, not just direct deps.

---

## 4. Skill Format

Skills conform to the **agentskills.io specification**. Ship is a conforming
implementation — it does not own this format.

Reference: https://agentskills.io/specification

A skill is a directory. The directory name is the skill name (lowercase,
hyphens only, max 64 chars). It must contain `SKILL.md` at minimum.

```
skills/
  git-helper/
    SKILL.md           ← required
    scripts/           ← optional; executable scripts
    references/        ← optional; reference documents
    assets/            ← optional; static assets
```

### 4.1 SKILL.md Structure

```markdown
---
name: git-helper
description: >
  Helps with git operations: branching, rebasing, conflict resolution.
  Designed for Claude Code and Cursor.
license: MIT
allowed-tools: Bash Read Glob
metadata:
  category: version-control
  provider-compat: claude cursor
---

# Git Helper

[Skill body — full instructions activated when this skill is invoked]
```

### 4.2 Frontmatter Fields

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `name` | string | yes | Must match directory name. Lowercase + hyphens only. Max 64 chars. |
| `description` | string | yes | Max 1024 chars. |
| `license` | string | no | SPDX identifier. |
| `allowed-tools` | string | no | Space-delimited list of tool permissions this skill requires. This is the permission surface. |
| `compatibility` | object | no | Provider compatibility hints. |
| `metadata` | object | no | Arbitrary key/value pairs. |

### 4.3 Progressive Disclosure

- `name` + `description` are loaded at startup (low cost).
- Body is loaded on skill activation.
- `scripts/`, `references/`, `assets/` are loaded on demand.

---

## 5. Agent Format

An agent is Ship's composition format: rules + skills + permissions + MCP + provider config.

**Location:** `.ship/agents/<name>.toml`
**Format:** TOML.

```toml
# .ship/agents/default.toml — complete example

[agent]
name        = "default"
description = "General-purpose coding agent"

# Rules: markdown files that become the system prompt backbone.
# Local paths are relative to .ship/
# Dep references: "dep-path/rules/file.md"
rules = [
  "rules/core.md",
  "rules/commit-style.md",
  "github.com/acme/internal-rules/rules/security.md",
]

# Skills: directories conforming to agentskills.io spec.
# Local paths are relative to .ship/
# Dep references: "dep-path/skills/skill-name"
skills = [
  "skills/git-helper",
  "github.com/anthropics/skills/skills/code-review",
]

# Permissions: explicit grants covering all required allowed-tools
# from all referenced skills. Compiler refuses if any are missing.
# Syntax mirrors Claude's allowed-tools format.
[permissions]
allow = [
  "Bash(git:*)",
  "Bash(npm:*)",
  "Read",
  "Write",
  "Glob",
]

# MCP server declarations
[[mcp]]
id      = "github"
command = "npx"
args    = ["-y", "@modelcontextprotocol/server-github"]
env     = { GITHUB_TOKEN = "${GITHUB_TOKEN}" }

# Provider targets + per-provider overrides
[providers]
targets = ["claude", "cursor"]

[providers.claude]
model          = "claude-opus-4-6"
context_window = 200000

[providers.cursor]
model = "claude-sonnet-4-6"
```

### 5.1 Field Reference

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `[agent].name` | string | yes | Matches filename (no `.toml`). |
| `[agent].description` | string | no | Human-readable description. |
| `rules` | array | no | Local paths or dep references. |
| `skills` | array | no | Local paths or dep references. |
| `[permissions].allow` | array | no | Must cover all `allowed-tools` from all referenced skills. |
| `[[mcp]]` | array | no | MCP server declarations. |
| `[providers].targets` | array | no | Provider compile targets. Overrides `[compile].providers` in ship.toml. |

An agent with only rules and no skills is valid.
An agent with only skills and no rules is valid.

> **Terminology note:** "Profile" and "Mode" are not used as user-facing type
> names. The unit of publication is an **agent**. Internally, modes remain
> valid for workspace-scoped filtering.

---

## 6. Content Hash

**Algorithm:** SHA-256 over a deterministic serialization of the package tree.

**Tree hash construction:**

1. Collect all files in the package directory (recursively), excluding:
   - `.git/`
   - `.DS_Store`, `Thumbs.db`
   - Files matched by `.shipignore` (if present)
2. Sort file paths lexicographically.
3. For each file: concatenate `<path>\0<length>\0<sha256(content)>`.
4. SHA-256 the concatenated result.
5. Encode as lowercase hex with `sha256:` prefix.

This is identical to what the sum DB records. The resolver verifies every
package against the sum DB before accepting it.

---

## 7. Resolver

### 7.1 Algorithm

Input: `ship.toml` `[dependencies]` block.
Output: `ship.lock` with all packages (direct + transitive) pinned.

```
resolve(manifest):
  queue = direct deps from manifest
  locked = {}

  while queue not empty:
    dep = queue.pop()

    if dep.path in locked:
      // Diamond dependency — check compatibility
      existing = locked[dep.path]
      if version_compatible(existing.version, dep.constraint):
        continue  // unified — use already-resolved version
      else:
        error("Incompatible versions for {dep.path}:\n"
              "  {existing.source} requires {existing.constraint}\n"
              "  {dep.source} requires {dep.constraint}\n"
              "  resolved: {existing.version} (incompatible with {dep.constraint})")

    commit = resolve_version(dep.path, dep.constraint)
    hash = fetch_and_hash(dep.path, commit)
    verify_sumdb(dep.path, commit, hash)
    locked[dep.path] = { commit, hash, version: dep.constraint }

    // Recurse into transitive deps
    transitive = parse_manifest(dep.path, commit)
    queue.extend(transitive.dependencies)

  return locked
```

### 7.2 Version Resolution

| Constraint form | Resolution |
|-----------------|-----------|
| `^1.0.0`, `~1.2.0`, `=1.2.3`, `>=1.0.0,<2.0.0` | List tags via proxy/git; select highest matching semver tag. |
| `main`, `develop`, branch name | Resolve to current HEAD commit of that branch at resolve time. |
| 40-char hex SHA | Use verbatim — no resolution needed. |

Semver tags with and without `v` prefix are both accepted (`v1.0.0` == `1.0.0`
when comparing; lockfile stores the tag as-is including `v` prefix).

### 7.3 Diamond Dependencies

- **Compatible** (e.g., both require `^1.x`, resolved `v1.2.0` satisfies both): unified.
  Only one copy is fetched and compiled.
- **Incompatible** (e.g., `^1.0.0` vs `^2.0.0`): hard error with the conflict path shown.
  Ship does not support multiple versions of the same package — agent configs
  are not code; there is no isolated `node_modules` equivalent.

### 7.4 Sum Database Verification

Default: every fetched package is verified against `sum.ship.dev`.

```
GET https://sum.ship.dev/lookup/github.com/owner/pkg@v1.2.0
→ 200 { "hash": "sha256:abc...", "recorded_at": "2026-01-01T00:00:00Z" }
```

- If the hash matches the fetched content: proceed.
- If the hash mismatches: **hard error**. Do not install.
  ```
  error: content hash mismatch for github.com/owner/pkg@v1.2.0
    expected (sum.ship.dev): sha256:abc...
    actual (fetched):        sha256:def...
  ```
- If the package is not yet in the sum DB (new version): record it on first install.
- Packages matching `[resolver].nosumcheck` patterns skip sum DB verification.

---

## 8. Content-Addressed Cache

**Root:** `~/.ship/cache/`
**Scope:** Global — shared across all projects on the machine.

```
~/.ship/cache/
  objects/
    sha256/
      e3b0c44298fc1c149afbf4c8996fb924.../
        SKILL.md
        scripts/
        ...
  index/
    github.com/
      anthropics/
        skills@v2.1.0   → sha256:e3b0c44...
```

### 8.1 Invariants

- Two packages with identical content share exactly one cache entry (no duplication).
- `ship install` with warm cache and unchanged lockfile makes **zero network requests**.
- Cache corruption (hash mismatch between stored content and index entry) causes **re-fetch**, not silent use of bad content.
- `ship cache clean` removes entries not referenced by any `ship.lock` found in known projects.

### 8.2 Cache Operations

```
fetch(path, commit):
  hash = index_lookup(path, commit)
  if hash and objects_exist(hash):
    return cache_path(hash)   // warm hit

  content = proxy_or_git_fetch(path, commit)
  hash = tree_hash(content)
  verify_sumdb(path, commit, hash)  // must pass before writing
  objects_write(hash, content)
  index_write(path, commit, hash)
  return cache_path(hash)
```

---

## 9. Permissions Model

### 9.1 Declaration

Skills declare required tool permissions in `SKILL.md` frontmatter:

```yaml
allowed-tools: Bash(git:*) Read Glob
```

### 9.2 Grant

The consuming agent must explicitly grant each required permission:

```toml
[permissions]
allow = ["Bash(git:*)", "Read", "Glob"]
```

Or per-dependency in `ship.toml`:

```toml
[dependencies]
"github.com/owner/skills" = { version = "^1.0.0", grant = ["Bash(git:*)", "Read"] }
```

### 9.3 Compiler Enforcement

The compiler validates at compile time:

1. Collect all `allowed-tools` from all skills referenced by the agent.
2. Check that each required tool is listed in the agent's `[permissions].allow`.
3. If any required tool is missing: **compilation error**.
   ```
   error: agent 'default' uses skill 'git-helper' which requires Bash(git:*)
     but this permission is not granted
     add to .ship/agents/default.toml [permissions] allow:
       "Bash(git:*)"
   ```

### 9.4 No Transitive Escalation

Skill A depends on Skill B (which has `allowed-tools: Bash`). Skill A does **not**
inherit Bash unless Skill A also declares it in its own `allowed-tools` and the
consuming agent explicitly grants it.

### 9.5 Permission Syntax

Follows Claude's `allowed-tools` format. Examples:

| String | Meaning |
|--------|---------|
| `Bash` | Full Bash access |
| `Bash(git:*)` | Bash restricted to `git *` commands |
| `Bash(npm:install)` | Bash restricted to `npm install` only |
| `Read` | File read access |
| `Write` | File write access |
| `Glob` | File glob access |
| `WebFetch` | HTTP fetch |
| `*` | All tools (use sparingly, requires explicit acknowledgment) |

---

## 10. Public/Private Exports

Everything in `.ship/` is **private by default**. Only what is explicitly
listed in `[exports]` is resolvable as a dependency.

```toml
[exports]
skills = ["skills/public-skill"]
agents = ["agents/public-agent.toml"]
```

- No `[exports]` section = package has no public surface (consume-only project — valid).
- `[exports]` with empty arrays = same as no section.
- A dep reference to a non-exported path fails at **resolve time**:
  ```
  error: 'github.com/owner/pkg/skills/internal' is not in [exports] for that package
  ```
- `ship publish` validates that all exported paths exist and are spec-valid before tagging.

---

## 11. Compilation

`ship install` = resolve + fetch + compile.

**Input sources:**
1. Local `.ship/` definitions (agents, skills, rules, MCP, permissions).
2. All resolved + fetched dependencies (from cache).

**Output:** Provider-native files only.

| Provider | Output files |
|----------|-------------|
| `claude` | `CLAUDE.md`, `.claude/agents/*.md`, `.claude/settings.json`, `.mcp.json` |
| `cursor` | `.cursor/rules/*.mdc`, `.cursor/mcp.json` |
| `codex` | `AGENTS.md` |
| `gemini` | `GEMINI.md` |

**Invariants:**
- No ship-specific intermediate files in output. Output is indistinguishable from hand-authored provider config.
- `.agents/` is written only when a provider explicitly expects it as input (i.e., when a provider adopts `.agents/` as a compile target, Ship outputs there — not before).
- Compilation is **deterministic**: same lockfile + same local files = identical output every time.

**Dry run:**
```
ship compile --dry-run
```
Prints what would be written without writing.

---

## 12. CLI Commands

All commands exit non-zero on error with an actionable message.
All commands are idempotent where semantically appropriate.

### `ship install`

```
ship install [--frozen]
```

1. Check `ship.toml` and `ship.lock` are in sync. If not, error with instructions.
2. For each package in `ship.lock`: verify cache hit. Fetch missing packages.
3. Verify all hashes against sum DB (unless `nosumcheck` matched).
4. Compile to all declared providers.

`--frozen`: Fail if lockfile would change. For CI.

### `ship add <path>[@version]`

```
ship add github.com/anthropics/skills
ship add github.com/anthropics/skills@^2.0.0
ship add github.com/anthropics/skills@main
```

1. Add to `[dependencies]` in `ship.toml`.
2. Resolve the new dep (and its transitive deps).
3. Update `ship.lock`.
4. Compile.

### `ship update [dep]`

```
ship update
ship update github.com/anthropics/skills
```

Re-resolves dep(s) against their constraint. Updates `ship.lock`. Compiles.
`ship update` with no args re-resolves all deps.

### `ship publish`

```
ship publish [--version <semver>]
```

1. Validate all exported paths exist and are spec-valid (`ship validate` runs first).
2. Check working tree is clean.
3. Write/confirm version in `[module].version`.
4. Create git tag `v<version>`.
5. Push tag to origin.

### `ship audit`

```
ship audit
ship audit --violations
```

Prints the full permission tree: which agent uses which skill, what tools it requires, and whether the grant is present.

`--violations`: Print only missing grants (exits non-zero if any found).

### `ship validate`

```
ship validate
ship validate --fix
```

Validates `.ship/` structure:
- `ship.toml` parses correctly.
- All exported paths exist.
- All exported skills have valid `SKILL.md` with required frontmatter.
- All exported agents have valid TOML.
- All agent permissions cover their skill requirements.

`--fix`: Auto-fix minor issues (e.g., add missing `name` from directory).

### `ship cache clean`

```
ship cache clean [--dry-run]
```

Removes cache entries not referenced by any `ship.lock` found in known projects.
`--dry-run`: Print what would be removed without removing.

---

## 13. Sum Database Protocol

**Endpoint:** `sum.ship.dev`

### Lookup

```
GET /lookup/<path>@<version>

200 OK
{
  "path": "github.com/owner/pkg",
  "version": "v1.2.0",
  "hash": "sha256:abc...",
  "recorded_at": "2026-01-01T12:00:00Z"
}

404 Not Found  → package not yet recorded
```

### Record (write)

```
POST /record
Authorization: Bearer <publisher-token>
{
  "path": "github.com/owner/pkg",
  "version": "v1.2.0",
  "commit": "abc...",
  "hash": "sha256:abc..."
}

201 Created
409 Conflict  → already recorded with different hash (tamper detection)
```

Records are **immutable once written**. No update, no delete.

### Transparency Log Tiles

```
GET /tile/<level>/<index>
```

Implements the [certificate transparency tile protocol](https://c2sp.org/tlog-tiles).
Enables third-party auditing.

---

## 14. Proxy Protocol

**Endpoint:** `proxy.ship.dev` (v0.2)

```
GET /{path}/@v/list
→ v1.0.0\nv1.1.0\nv2.0.0\n

GET /{path}/@v/{version}.info
→ { "Version": "v1.2.0", "Time": "2026-01-01T00:00:00Z", "Commit": "abc..." }

GET /{path}/@v/{version}.zip
→ zip archive of package content
```

Resolution order: proxy first, then direct git (configurable via `[resolver].proxy`).

`SHIP_PROXY=direct` bypasses proxy entirely (air-gapped environments).

---

## 15. Error Reference

| Code | Message | Resolution |
|------|---------|-----------|
| `E001` | `ship.toml and ship.lock are out of sync` | Run `ship install` |
| `E002` | `content hash mismatch for <pkg>@<v>` | Possible tamper; do not install; report |
| `E003` | `incompatible versions for <pkg>: <src1> requires <c1>, <src2> requires <c2>` | Pin to compatible version in `ship.toml` |
| `E004` | `'<path>' is not exported by <pkg>` | Use an exported path; ask publisher to export it |
| `E005` | `agent '<n>' uses skill '<s>' which requires <tool> but this permission is not granted` | Add tool to `[permissions].allow` |
| `E006` | `not logged in` | Run `ship login` |
| `E007` | `working tree is not clean` | Commit or stash changes before `ship publish` |

---

## 16. .shipignore

Optional file at `.ship/.shipignore`. Same syntax as `.gitignore`.
Files matched are excluded from the content hash and from package archives.

Default excludes (always applied, no `.shipignore` needed):
```
.git/
.DS_Store
Thumbs.db
*.swp
ship.lock
```

`ship.lock` is excluded from content hashes — it describes dependencies, not published content.

---

## Appendix A — Directory Layout Reference

```
.ship/
  ship.toml         ← manifest (module identity, deps, exports, compile, resolver)
  ship.lock         ← resolved dep graph (committed)
  .shipignore       ← optional; files excluded from content hash
  agents/
    default.toml    ← agent definitions
    reviewer.toml
  skills/
    git-helper/
      SKILL.md
    code-review/
      SKILL.md
      scripts/
  rules/
    core.md
    commit-style.md
  mcp.toml          ← MCP server declarations (project-wide)

~/.ship/
  credentials       ← auth token (not committed)
  config.toml       ← local config (cloud.base_url, worktrees.dir)
  cache/
    objects/
      sha256/
        <hash>/     ← package content, keyed by content hash
    index/
      <path>@<version>  → <hash>  (symlink or file)
```

## Appendix B — ship.toml vs .shiprc vs eslintrc

`ship.toml` is a **manifest**, not a tool config file. It defines:
- What this package **is** (identity, version).
- What it **depends on**.
- What it **publishes** (exports).
- How it should be **compiled**.

It is analogous to `Cargo.toml` / `go.mod` / `package.json`, not to `.eslintrc`.

The reason it lives at `.ship/ship.toml` (not repo root) is that Ship is additive
to existing projects. The `.ship/` namespace keeps Ship's config out of the repo
root, analogous to `.github/`. A project that already has a `package.json` at
its root is not burdened by a second manifest-like file at that level.

## Appendix C — Versioning Constraints Grammar

```ebnf
constraint    = range | branch | commit_sha
range         = comparator ("," comparator)*
comparator    = op version_core
op            = "^" | "~" | "=" | ">=" | "<=" | ">" | "<"
version_core  = digit+ "." digit+ "." digit+ pre_release?
pre_release   = "-" alphanumeric+
branch        = letter (letter | digit | "-" | "/")*
commit_sha    = hex{40}
```

Caret (`^`) semantics follow npm/Cargo: `^1.2.3` = `>=1.2.3 <2.0.0`.
Tilde (`~`) semantics: `~1.2.3` = `>=1.2.3 <1.3.0`.
