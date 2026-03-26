# Skills — Surface Roadmap

Skills is an evergreen capability surface alongside Compiler, Studio, and Registry.

The goal: skills that are trustworthy, personalizable, and composable — good enough that inviting the community to build on them is an act of confidence, not hope.

---

## What shipped in 0.1.0

The foundation. Everything here is working and tested.

| Capability | Description |
|---|---|
| Template engine | MiniJinja (Jinja2) resolves `{{ var }}`, `{% if %}`, `{% for %}`, `{% else %}` in skill content at compile time. Pure WASM — no I/O. |
| `vars.json` schema | JSON-native variable schema with type, default, scope, values, label, description. Parsed via serde — no custom parser. |
| State files | Per-skill JSON state at `.ship/state/skills/{id}.json` (project) and `~/.ship/state/skills/{id}.json` (user). |
| Merge order | defaults → user state → project state (last wins). |
| Atomic writes | State files written via temp file + rename — no corruption on concurrent access. |
| `_meta` block | Every state file carries `{ v, skill, migrations }` for version tracking and migration awareness. |
| `ship vars` CLI | set / get / edit / append / reset commands with type validation and enum constraint enforcement. |
| Changes log | Append-only JSONL at `.ship/state/skills/{id}.changes.jsonl`. |
| `skill_id` validation | Path traversal rejected at the API boundary. Only `[a-z0-9][a-z0-9\-]*` accepted. |
| Hash exclusion | `state/` excluded from content hashes — state never affects publish identity. |

---

## 0.1.X — before the community builds on this

Breaking changes are free now. No one has written skills against the spec yet.

**Spec cleanup:**
- `storage-hint` field in `vars.json` replacing the hard-coded `user`/`project` scope model. Runtimes map hints to file locations; the spec stays storage-agnostic.
- `version` required in skill frontmatter, semver-validated. Currently optional text in metadata.
- `allowed-tools` structured: `{ required: [...], optional: [...], reason: "..." }`. Currently a flat space-delimited list with no enforcement.
- `min-runtime-version` field — skills using MiniJinja template syntax declare the minimum Ship version that understands them.
- `stable-id` in frontmatter — canonical identifier the registry uses, separate from directory name. Prevents state orphaning on skill rename.

**Runtime:**
- Declarative migrations: `migrations/` directory in skill, JSON array of ops (`rename`, `set_default`, `delete`, `change_type`). Applied on `ship install` and `ship update`. Tracked in `_meta.migrations`.
- User state moved to Ship's runtime SQLite DB (`~/.ship/platform.db`). No more scattered files in `~/.ship/state/`. Accessed via `ship vars` CLI and MCP tools.
- Project state consolidated: single `.ship/state.json` keyed by skill id instead of one file per skill.
- `ship install` and `ship update` apply pending migrations, create/update default state.
- `ship skill remove` cleans up state files.
- MCP tools: `get_skill_vars(skill_id)`, `set_skill_var(skill_id, key, value)`, `list_skill_vars()` — agents read and write vars through the same surface as everything else.

**Security:**
- `skill_id` validation already in (0.1.0). Remaining: enum validation enforced at compile time, not just at `ship vars set` time.
- MiniJinja environment locked to minimum: no file loader, no custom functions, chainable undefined behavior. Already implemented.

---

## Future — after the spec is stable and published

**WASM audit sandbox**

Ship's registry runs every published skill through a static analysis pipeline and sandboxed execution environment:
- Static scan: prompt injection patterns, credential harvesting, URL exfiltration, instructions that attempt to override safety
- Template analysis: all `{{ }}` markers validated against declared vars, no dangerous filter usage
- Migration analysis: ops validated against allowed set, no escape hatches
- Sandboxed execution: migration scripts run in a WASM host with no filesystem/network capabilities, 100ms timeout, 16MB memory
- Skill authors write normal bash. Ship's infrastructure wraps it. No WASM knowledge required.

Badge: "Verified by Ship" for registry-listed skills. The scanner runs client-side too (`ship audit <skill>`) — same WASM module, no network required.

**Studio skill editor**

`vars.json` drives a form UI in the skill detail view. Users see label, description, type, and current value for each var. Scope hint shown as "stored on your machine" / "stored in project". Saves via the same write path as `ship vars set`.

**Computed and dynamic vars**

- Environment variable injection at compile time: `{{ env.ANTHROPIC_MODEL }}`
- Git context: `{{ git.branch }}`, `{{ git.author }}`
- Agent-written state: agents set vars via MCP tools, creating an accumulation of learned preferences. The skill improves its own context over sessions.

**`crates/skill-vars` — open standard crate**

Extract the standard as a standalone crate:
- MiniJinja template resolution
- `vars.json` schema parsing (serde-based)
- `state.json` convention and `_meta` format
- Declarative migration application

No Ship-specific scope model. No CLI. Just the portable kernel that any tool can depend on.

Published to crates.io. The registry runs the same binary the skill editor runs. Trust comes from one implementation, not from N partial implementations.

---

## What this is not

Skills are not a plugin system, a scripting runtime, or a way to distribute executable code. They are markdown documents with typed configuration. The template engine renders text; it does not execute behavior. The WASM audit sandbox is Ship's infrastructure concern — skill authors never touch it.

The invitation to the community is: "here is a well-specified, trustworthy way to write skills that adapt to their context." Not: "here is an extension point, good luck."
