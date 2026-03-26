# Handoff — job/skill-vars

Branch: `job/skill-vars`
Date: 2026-03-26
Status: implementation complete, ready for review and merge

---

## What was built

**Stateful Skills** — template variables for personalizable skill content. Skills that carry typed, scoped, versioned state.

### Compiler (`crates/core/compiler`)
- `src/vars.rs` — MiniJinja template resolver (replaces 373-line custom `%var%` parser). Standard Jinja2: `{{ var }}`, `{% if %}`, `{% else %}`, `{% for %}`. Pure WASM, no I/O, chainable undefined.
- `src/types/skill.rs` — `vars: HashMap<String, Value>` added to `Skill`.
- `src/compile/skills.rs` — resolves vars before emitting skill file content.
- `Cargo.toml` — `minijinja = "2"` added.

### CLI (`apps/ship-studio-cli`)
- `src/vars/schema.rs` — `VarDef`, `VarScope`, `VarType`, `parse_vars_json`, `load_vars_json`. Serde, no custom parser. `label` and `description` fields.
- `src/vars/state.rs` — state I/O: merge logic (defaults → user → project), atomic writes (temp + rename), `_meta` block (`v`, `skill`, `migrations`), `_meta` stripped on read, `validate_skill_id` path traversal guard.
- `src/vars/commands.rs` — `ship vars set/get/edit/append/reset`.
- `src/loader.rs` — loads `vars.json` + state for directory-format skills at compile time.
- `src/commands.rs` / `cli.rs` / `main.rs` — `ship vars` subcommand wired.

### Runtime (`crates/core/runtime`)
- `src/registry/hash.rs` — `state/` excluded from content hashes.

### Docs
- `docs/skill-vars.md` — full spec: vars.json format, MiniJinja syntax, state file format, CLI reference.
- `docs/skills-surface.md` — Skills as a surface, capability map 0.1.0 / 0.1.X / future.

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
| Atomic writes | Temp + rename prevents corruption |
| `_meta` in state | Version tracking; migration list for install/update logic |
| `skill_id` validation | Path traversal fixed at the API boundary |
| Declarative migrations | Shell scripts = supply chain risk; JSON ops are safe and inspectable |
| MiniJinja locked down | No file loader, no custom functions, chainable undefined |
| "Stateful Skills" | User's name; precise, no explanation required |
| Skills as own surface | Alongside Compiler, Studio, Registry |

---

## 0.1.X — next agent picks up here

Breaking changes — free to do before anyone publishes against the spec.

**Spec:**
- `storage-hint` replaces hard-coded `user`/`project` scope (runtime-agnostic)
- `version` required in frontmatter, semver-validated
- `allowed-tools` structured: `{ required, optional, reason }` with compile-time enforcement
- `min-runtime-version` field
- `stable-id` in frontmatter (canonical ID, rename-safe state linkage)

**Runtime:**
- Declarative `migrations.json` — JSON ops (rename, set_default, delete, change_type); applied by `ship install`/`ship update`; tracked in `_meta.migrations`
- User state → Ship's runtime SQLite DB (`~/.ship/platform.db`)
- Project state → single `.ship/state.json` keyed by skill id
- `ship install` seeds default state, runs pending migrations
- `ship update` runs only new migrations
- `ship skill remove` cleans up state
- MCP tools: `get_skill_vars`, `set_skill_var`, `list_skill_vars`
- Enum validation at compile time

**Standard:**
- Extract `crates/skill-vars` — resolver, schema parsing, state convention. Publish to crates.io.

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
3. **Registry signal for Stateful Skills** — implicit (presence of `vars.json`) or explicit registry field?
