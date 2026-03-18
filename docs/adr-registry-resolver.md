# ADR: Git-Native Package Registry for Skills and Agents

**Status:** proposed
**Date:** 2026-03-18
**Supersedes:** partial — replaces R2/D1 artifact storage model in SPEC.md; ship.lock format; "profile" terminology

---

## Context

Ship is a compiler and runtime for agent configuration. It currently operates locally: `.ship/` holds your authored configs, `ship compile` emits provider-native files. There is no distribution mechanism. Teams copy configs manually. Versions don't exist. Dependencies don't exist.

Three converging pressures make this the right moment to add a package layer:

1. **The `.agents/` convention is forming.** The community is standardizing on skill directories. agentskills.io has a published SKILL.md specification. If Ship doesn't have a package story when this solidifies, we become a local tool that happens to compile things — not the platform.

2. **Agent configs are composite.** A real agent needs rules, skills, MCP config, and provider targeting. None of these tools have a way to express dependencies between configs or version them. Someone has to build this.

3. **We are currently the only multi-provider compiler.** The window to establish Ship as the resolution and compilation layer — before any single provider builds it themselves — is measured in months.

The decision is: what is the package system, how is it structured, and what are the security properties that prevent it from being forked away from us?

---

## Decision

### 1. Git is the registry. We own the infrastructure on top of it.

Packages live in git repos. A git tag is a version. There is no central artifact store we maintain (no R2 upload, no blob storage for user content).

We own three layers on top of git:

- **Sum database** (`sum.ship.dev`) — append-only, publicly auditable log of `package@version → content hash`. Once a version is recorded, it is immutable. Every resolver verifies against it. This is the cryptographic anchor of the ecosystem and our primary moat against compatible competitors.
- **Proxy** (`proxy.ship.dev`) — caches git content for availability and performance. Solves left-pad. Resolvers default to proxy first, direct git second. Private proxy with access control = paid tier.
- **Registry index** (`registry.ship.dev`) — discovery only. Crawls git repos. No storage. Points to git.

A "pship" can implement the CLI. They cannot replicate the sum DB without ecosystem fragmentation — their packages won't verify against ours, and ours won't verify against theirs. They become a client of our infrastructure or they fork the trust model.

### 2. Two publishable types: Skills and Agents

**Skills** conform to the agentskills.io specification. Ship does not own this format. A skill is a directory with a `SKILL.md` (YAML frontmatter + markdown body). Required fields: `name` (matches directory name), `description`. Optional: `license`, `compatibility`, `metadata`, `allowed-tools`.

Ship is a conforming implementation of the agentskills.io spec. We cite it, we don't redefine it.

**Agents** are Ship's format. An agent is a composition:
- Rules: markdown files (local paths or dependency references)
- Skills: local skill directories or dependency references
- Permissions: explicit grants for each required skill permission
- MCP: server declarations
- Provider config: compile targets + per-provider overrides

"Profile" and "Mode" are retired as user-facing terms. An agent is the unit of publication.

### 3. `.ship/` remains the canonical directory. ship.toml lives inside it.

Ship is a runtime and cloud OS, not just a package manager. The config complexity justifies a directory, not a root-level dotfile. This mirrors `.github/` — the tool's namespace is inside a folder, not sprawled at the repo root.

The manifest is `.ship/ship.toml`. Source files are declared explicitly under `[exports]`. Nothing is published by convention — the public API of a package is what `[exports]` declares.

```
.ship/
  ship.toml         ← manifest: module identity, deps, exports
  ship.lock         ← resolved commits + checksums (committed)
  agents/           ← your agent definitions
  skills/           ← your skill directories (agentskills.io conformant)
  rules/            ← markdown guidelines
  mcp.toml          ← MCP declarations
```

### 4. Content-addressed cache. pnpm model, not npm.

`~/.ship/cache/` stores packages by SHA256 content hash. Same file content across 100 packages = one disk entry. No per-project copies. `ship install` with a warm cache and unchanged lockfile = zero network, pure compilation.

### 5. Provider targets are "node_modules"

There is no intermediate `.agents/` output directory. `ship install` resolves deps, populates cache, then compiles directly to provider-native locations: `CLAUDE.md`, `.claude/agents/`, `.cursor/rules/`, `agents.json`, etc. The package system is invisible after compilation. Providers read their native files.

When a provider adopts `.agents/` as a compile target, Ship outputs there. Not before.

### 6. Permissions are explicit grants, enforced by the compiler

A skill's `allowed-tools` field declares what it requires. The consuming agent must explicitly grant each permission in `ship.toml`. The compiler refuses to compile if required permissions are not granted. Transitive permissions do not escalate — a skill that depends on a skill with `Bash(*)` does not inherit that grant.

```toml
# .ship/ship.toml
[dependencies."github.com/anthropics/tools"]
version = "v2.1.0"
grant   = ["Bash(git:*)", "Read"]   # explicit, required
```

### 7. TOML is the format

TOML for `.ship/ship.toml` and agent definitions. Rationale: no implicit type coercion (unlike YAML), comments supported (unlike JSON), Cargo.toml precedent for exactly this dependency declaration pattern, language-agnostic parsers available everywhere. One format. No alternates.

---

## Alternatives considered

**R2/D1 as artifact store (current SPEC.md model)**
Requires Ship to be the storage layer. Publishers upload to us. We own the content. This creates storage costs, DMCA exposure, and positions us as a SaaS rather than infrastructure. Abandoned in favor of git-native with proxy caching.

**Top-level `ship.toml` (package.json model)**
Clean for pure package managers. Wrong for Ship because Ship is additive to an existing project — the config belongs in a namespaced directory, not mixed with the project's root files.

**Convention-based publishing (any SKILL.md in skills/ is published)**
Fragile. Accidentally publishes internal configs. Constrains repo layout. Explicit `[exports]` is the TypeScript `exports` field pattern — declare your public API, everything else is private.

**Open spec, encourage others to implement**
Mis-framed. We don't want providers implementing the resolver — we want providers being compilation targets. Openness is about distribution (anyone can publish, anyone can consume) not about making our tooling easy to replicate.

---

## Consequences

**Positive:**
- Ship becomes infrastructure, not a SaaS. Infrastructure is harder to displace.
- Distribution is solved for free: publish to git, reference by path. Zero onboarding friction.
- The sum DB creates a moat that scales with ecosystem size, not with our engineering investment.
- Strict permissions model is a genuine security contribution. Enterprises care about this.
- "Profile" being renamed "Agent" aligns terminology with where the industry is going.

**Negative / risks:**
- SPEC.md needs significant revision. R2/D1 artifact model is replaced.
- `ship.lock` format defined here supersedes the existing format — migration required.
- Sum DB must be operational before the resolver ships publicly. A resolver without sum DB verification is incomplete.
- If we don't get to critical mass of packages before a well-funded competitor, the sum DB moat doesn't form.

**What this does NOT change:**
- The compiler architecture (provider emitters, TOML parsing, WASM target)
- Auth, workspace, session, job queue — SaaS layer remains for identity + runtime
- The Studio (authoring interface, drafts) — Studio drafts are pre-publish state, backed by D1
- CLI commands that aren't package-related

---

## Measuring success

- [ ] `ship install` on a repo with a `[dependencies]` block resolves, caches, and compiles correctly
- [ ] `ship.lock` round-trips: install → lock → fresh install from lock produces identical compilation output
- [ ] Sum DB records every published version with a verifiable hash
- [ ] Compiler refuses to compile a dep whose required permissions are not explicitly granted
- [ ] Zero files written outside provider-native targets on `ship install`
- [ ] A second implementation of the resolver verifies correctly against sum.ship.dev
