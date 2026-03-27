# Handoff — job/skill-vars

Branch: `job/skill-vars`
Date: 2026-03-26
Status: implementation complete, ready for review and merge

---

## What was built

**Smart Skills** — template variables for personalizable skill content. Skills that carry typed, scoped configuration resolved at compile time via MiniJinja.

### Compiler (`crates/core/compiler`)
- `src/vars.rs` — MiniJinja template resolver (replaces 373-line custom `%var%` parser). Standard Jinja2: `{{ var }}`, `{% if %}`, `{% else %}`, `{% for %}`. Pure WASM, no I/O, chainable undefined.
- `src/types/skill.rs` — `vars: HashMap<String, Value>` added to `Skill`.
- `src/compile/skills.rs` — resolves vars before emitting skill file content.
- `Cargo.toml` — `minijinja = "2"` added.

### CLI (`apps/ship-studio-cli`)
- `src/vars/schema.rs` — `VarDef`, `StorageHint` (Global/Local/Project), `VarType`, `parse_vars_json`, `load_vars_json`. Serde, no custom parser. `label` and `description` fields.
- `src/vars/state.rs` — `validate_skill_id` path traversal guard, `append_to_array` helper.
- `src/vars/commands.rs` — `ship vars set/get/append/reset`. `find_vars_json` resolves `assets/vars.json`.
- `src/loader.rs` — loads `assets/vars.json` + KV state for directory-format skills at compile time.
- `src/commands.rs` / `cli.rs` / `main.rs` — `ship vars` subcommand wired.

### Runtime (`crates/core/runtime`)
- `src/skill_vars.rs` — all state in `platform.db` KV. Three namespaces: `skill_vars:{id}` (global), `skill_vars.local:{ctx}:{id}` (local), `skill_vars.project:{ctx}:{id}` (project). Context key = 16-char hex of `DefaultHasher(ship_dir)`.
- `src/lib.rs` — exports `get_skill_vars`, `set_skill_var`, `list_skill_vars`, `reset_skill_vars`.

### Skills
- `.ship/skills/commit/` — canonical Smart Skill with MiniJinja template, `assets/vars.json`, `evals/evals.json`.
- All 27 authored skills updated: `stable-id` in frontmatter, 8 skills have `assets/vars.json`, 6 skills have MiniJinja template markers.

### Docs
- `docs/smart-skills.md` — full spec: directory layout, vars.json schema, KV storage model, stable-id, template syntax, CLI, references/docs/, evals format.
- `docs/skills-surface.md` — Skills surface roadmap: 0.1.0 shipped / 0.1.X next / future.

---

## Test counts (all passing)
- compiler: 391 | runtime: 379 | CLI: 263
- **Total: 1,043**

---

## Key decisions

| Decision | Rationale |
|---|---|
| MiniJinja | Browser support for skill editor; Jinja2 is known; `{% else %}` included; one parser to maintain |
| `vars.json` not `vars.yaml` | JSON native; serde; no custom parser |
| All state in platform.db KV | No scattered state files; three namespaces give global/local/project scoping |
| `assets/vars.json` path | Schema belongs with assets, not peer to SKILL.md |
| Context key from path hash | Stable, no path embedded in key, no collisions |
| `stable-id` in frontmatter | Rename-safe state linkage; orphaning prevented |
| `skill_id` validation | Path traversal fixed at the API boundary |
| MiniJinja locked down | No file loader, no custom functions, chainable undefined |
| `evals/evals.json` | First-class eval loop per agentskills.io spec |
| "Smart Skills" | Skills that adapt to user and context at compile time |
| Skills as own surface | Alongside Compiler, Studio, Registry |

---

## 0.1.X — next agent picks up here

Breaking changes — free to do before anyone publishes against the spec.

**Spec:**
- `version` required in frontmatter, semver-validated
- `allowed-tools` structured: `{ required, optional, reason }` with compile-time enforcement
- `min-runtime-version` field
- Enum validation at compile time (currently only at `ship vars set`)

**Runtime:**
- Declarative `migrations.json` — JSON ops (rename, set_default, delete, change_type); applied by `ship install`/`ship update`
- `ship install` seeds default KV state for new skills, runs pending migrations
- `ship skill remove` cleans up all KV state
- MCP tools: `get_skill_vars`, `set_skill_var`, `list_skill_vars`

**Evals:**
- `ship skill eval` tooling — runs `evals/evals.json` with/without skill, writes `{skill}-workspace/iteration-N/`, produces `benchmark.json`

**Standard:**
- Extract `crates/skill-vars` — resolver, schema parsing, KV merge logic. Publish to crates.io.

---

## Future

- WASM audit sandbox on registry publish (static scan + sandboxed execution; `ship audit` runs same thing client-side)
- Studio skill editor — form UI from `vars.json`
- Computed vars — env injection, git context at compile time
- Agent-written state — skills accumulate learned preferences via MCP
- Skills surface added to platform cap map

---

## Open questions

1. **agentskills.io v1 publication** — minimum spec before publishing? Does it wait for `storage-hint` and `stable-id`?
2. **User state migration** — path from `~/.ship/state/` files to SQLite needs care for existing users
3. **Registry signal for Smart Skills** — implicit (presence of `assets/vars.json`) or explicit registry field?
