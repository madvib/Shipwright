# Skills — Surface Roadmap

Skills is an evergreen capability surface alongside Compiler, Studio, and Registry.

The goal: skills that are trustworthy, personalizable, and composable — good enough that inviting the community to build on them is an act of confidence, not hope.

---

## What shipped in 0.1.0

The foundation. Everything here is working and tested.

| Capability | Description |
|---|---|
| Template engine | MiniJinja (Jinja2) resolves `{{ var }}`, `{% if %}`, `{% for %}`, `{% else %}` in skill content at compile time. Pure WASM — no I/O, chainable undefined. |
| `assets/vars.json` schema | JSON variable schema with `type`, `default`, `storage-hint`, `values`, `label`, `description`. Parsed via serde — no custom parser. |
| `storage-hint` scopes | Three scopes: `global` (machine-wide KV), `local` (per-context KV, not shared), `project` (per-context KV, team-shared). |
| All state in platform.db | KV namespaces: `skill_vars:{id}` (global), `skill_vars.local:{ctx}:{id}` (local), `skill_vars.project:{ctx}:{id}` (project). No scattered files. |
| Context key | Stable 16-char hex token derived from project path — scopes local/project state without embedding the path in the key. |
| Merge order | defaults → global → local → project (last wins). |
| `stable-id` in frontmatter | Canonical state storage key; prevents orphaning when a skill directory is renamed. |
| `ship vars` CLI | `set / get / append / reset` with type validation and enum constraint enforcement. |
| `skill_id` validation | Path traversal rejected at the API boundary. Only `[a-z0-9][a-z0-9\-]*` accepted. |
| Directory-format skills | `assets/vars.json`, `references/docs/`, `evals/evals.json` — bundled resources alongside `SKILL.md`. |
| MiniJinja locked down | No file loader, no custom functions, chainable undefined. Template errors fall back to original content with a warning. |
| Hash exclusion | KV state never affects publish identity. |

---

## 0.1.X — before the community builds on this

Breaking changes are free now. No one has written skills against the spec yet.

**Spec:**
- `version` required in skill frontmatter, semver-validated. Currently optional in metadata.
- `allowed-tools` structured: `{ required: [...], optional: [...], reason: "..." }`. Currently a flat space-delimited list with no enforcement.
- `min-runtime-version` — skills using MiniJinja template syntax declare the minimum Ship version that understands them.
- Enum validation enforced at compile time in addition to `ship vars set`. Currently only at CLI.

**Runtime:**
- Declarative migrations: `migrations.json` in skill assets. JSON ops (`rename`, `set_default`, `delete`, `change_type`). Applied on `ship install`/`ship update`.
- `ship install` seeds default state for new skills, runs pending migrations.
- `ship skill remove` cleans up all KV state for the skill.
- MCP tools: `get_skill_vars(skill_id)`, `set_skill_var(skill_id, key, value)`, `list_skill_vars()` — agents read and write vars through the same surface as everything else.

**Evals:**
- `ship skill eval` tooling — runs `evals/evals.json` cases with/without skill, writes `{skill}-workspace/iteration-N/` output, produces `benchmark.json` delta.

---

## Future — after the spec is stable and published

**WASM audit sandbox**

Ship's registry runs every published skill through a static analysis pipeline and sandboxed execution environment:
- Static scan: prompt injection patterns, credential harvesting, URL exfiltration, instructions that attempt to override safety
- Template analysis: all `{{ }}` markers validated against declared vars, no dangerous filter usage
- Skill authors write normal skills. Ship's infrastructure wraps the audit. No WASM knowledge required.

Badge: "Verified by Ship" for registry-listed skills. The scanner runs client-side too (`ship audit <skill>`) — same WASM module, no network required.

**Studio skill editor**

`assets/vars.json` drives a form UI in the skill detail view. Users see label, description, type, and current value for each var. Scope shown as "stored on your machine" / "stored in project". Saves via the same write path as `ship vars set`. No JSON editing required.

**Documentation site**

`references/docs/` Markdoc files compiled into a browsable documentation site (Astro + Starlight). Skills become first-class documentation primitives — human-readable and agent-discoverable. `ship docs` serves locally; agentskills.io hosts the canonical site.

**Computed and dynamic vars**

- Environment variable injection at compile time: `{{ env.ANTHROPIC_MODEL }}`
- Git context: `{{ git.branch }}`, `{{ git.author }}`
- Agent-written state: agents set vars via MCP tools, accumulating learned preferences across sessions.

**`crates/skill-vars` — open standard crate**

Extract the core as a standalone crate:
- MiniJinja template resolution
- `assets/vars.json` schema parsing (serde-based)
- KV state merge logic (global → local → project)
- Declarative migration application

No Ship-specific assumptions. Published to crates.io. The same binary the skill editor uses.

---

## What this is not

Skills are not a plugin system, a scripting runtime, or a way to distribute executable code. They are markdown documents with typed configuration. The template engine renders text; it does not execute behavior. The WASM audit sandbox is Ship's infrastructure concern — skill authors never touch it.

The invitation to the community is: "here is a well-specified, trustworthy way to write skills that adapt to their context." Not: "here is an extension point, good luck."
