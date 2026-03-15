# Ship — Sprint Board

> Three parallel lanes. Each agent works in its own worktree on a dedicated branch.
> Status: `[ ]` todo · `[~]` in progress · `[x]` done · `[!]` blocked
>
> **Constraint:** CLI must be solid before Web/Server ship the GitHub PR flow.
> **Key invariant:** `.ship/` is source of truth. Provider files (CLAUDE.md, .mcp.json, etc.)
> are generated artifacts — gitignored, never committed.
> **Distribution:** agentskills.io compliance = automatic presence in all marketplaces.
> The PR flow IS distribution. Goal: `.ship/` in every repo on GitHub.

---

## Lane 1 — CLI (`feat/cli-*`)

The CLI is the linchpin. The PR flow is useless without it.

### feat/cli-init
- [ ] `ship log <message>` — write timestamped note to `.ship/coordination.log` (coordination primitive, do first)
- [ ] `ship init` — scaffold `.ship/agents/` in current project, write `ship.toml`
- [ ] Detect existing provider files and offer to import them
- [ ] Write provider files to `.gitignore` (CLAUDE.md, .mcp.json, AGENTS.md, .cursor/, .gemini/)
- [ ] Print next steps: `ship use <preset>`

### feat/cli-use
- [ ] `ship use <preset-id>` — fetch preset from registry, set as active, emit provider files
- [ ] `ship use` (no args) — re-emit current preset
- [ ] `ship use --list` — list available presets (local + registry)
- [ ] `ship use <preset>` installs required skills, respects ship.lock
- [ ] `ship use` triggers plugin activation if preset declares `[plugins]`
- [ ] Exit nonzero with actionable error if no preset and no arg
- [ ] `ship status` — show active preset, last compiled, providers configured

### feat/cli-import
- [ ] `ship import` — detect and import existing provider configs into `.ship/agents/`
- [ ] Support: CLAUDE.md, .mcp.json, .cursor/rules/, AGENTS.md, .gemini/
- [ ] Deduplicate rules across providers into shared library format
- [ ] Output summary of what was imported

### feat/cli-branch-preset *(after cli-use is stable)*
- [ ] DB stores `active_preset` per workspace, keyed to branch name
- [ ] `ship workspace activate <branch>` — look up workspace, run `ship use <preset>` for that branch
- [ ] Git post-checkout hook: calls `ship workspace activate` on branch switch
- [ ] `ship init` installs the hook automatically
- [ ] Goal: switching branches auto-switches your agent config

### feat/cli-plugin *(after cli-use is stable)*
- [ ] `ship plugin install <id>` — install a Claude Code plugin from registry
- [ ] `ship plugin list` — list installed plugins
- [ ] Ship plugin package: bundles `ship mcp` + branch-switch hook + ship-workflow skill
- [ ] Plugin published to `~/.claude/plugins/ship/` on install
- [ ] No slash commands — MCP tools + hooks only

---

## Lane 2 — Server (`feat/server-*`)

### feat/server-auth
- [ ] Better Auth setup on Cloudflare Workers (D1 adapter)
- [ ] GitHub OAuth provider (user identity, not GitHub App)
- [ ] D1 schema: `users`, `sessions`, `orgs`, `org_members`
- [ ] `GET /api/me` — current user + orgs
- [ ] Auth middleware for protected routes

### feat/server-persistence
- [ ] D1 schema: `libraries`, `library_versions`, `presets`, `preset_versions`
- [ ] `POST /api/library` — save library (requires auth)
- [ ] `GET /api/library/:id` — load library (public if flagged)
- [ ] `POST /api/preset` — publish preset to registry (requires auth)
- [ ] `GET /api/registry` — list public presets (paginated, filterable)
- [ ] `GET /api/registry/:id` — get preset by id

### feat/server-github
- [ ] Register GitHub App (repo read, PR write, contents write)
- [ ] GitHub App OAuth flow (separate from user OAuth — grants repo access)
- [ ] `POST /api/github/import` — given a public repo URL, fetch + extract provider configs
- [ ] `POST /api/github/pr` — create PR adding `.ship/` to a user's repo
  - PR body includes: what Ship is, install instructions, `ship compile` quickstart
  - `.gitignore` patch included in PR

---

## Lane 3 — Web / Studio (`feat/web-*`)

### feat/web-import
- [ ] GitHub URL input in Studio header/landing
- [ ] Call `POST /api/github/import` → populate library from extracted configs
- [ ] Show import preview: rules found, MCP servers found, skills found
- [ ] "Open in Studio" — load imported library into compiler UI
- [ ] Works unauthenticated (public repos only)

### feat/web-auth
- [ ] "Sign in with GitHub" button (Better Auth client)
- [ ] Auth state in app (user context, session)
- [ ] "Save Library" gated behind auth
- [ ] "My Libraries" page

### feat/web-pr
- [ ] "Add to repo" flow — requires GitHub App OAuth (repo access)
- [ ] Repo picker (list user's repos via GitHub API)
- [ ] Preview what the PR will contain (`.ship/` scaffold + gitignore patch)
- [ ] Call `POST /api/github/pr` → link to created PR
- [ ] Post-PR: instructions to install CLI + run `ship compile`

### feat/web-registry
- [ ] Preset registry browse panel in Studio
- [ ] Search + filter presets
- [ ] One-click "use this preset" → loads into Studio

---

## Worktree conventions

```bash
git worktree add ../ship-<task> -b feat/<lane>-<task>
# e.g.
git worktree add ../ship-cli-init -b feat/cli-init
git worktree add ../ship-server-auth -b feat/server-auth
git worktree add ../ship-web-import -b feat/web-import
```

Each agent: read `ARCHITECTURE.md` + `SPEC.md` first, then this file.
PR back to `main` when done. No cross-lane dependencies within a lane.

## Cross-lane dependencies

```
cli-init ─────────────────────────────► web-pr (PR needs working CLI)
server-auth ──────────────────────────► web-auth
server-github (import endpoint) ──────► web-import
server-github (PR endpoint) ──────────► web-pr
server-persistence ───────────────────► web-auth (save/load)
```

CLI tasks are independent of each other. Server tasks are mostly independent.
Web-import can start immediately (calls server endpoint, degrades gracefully if not up).

---

## Completed (foundation)

- [x] Archive desktop, site, plugins, examples, docs, cli, modules/git
- [x] Trim MCP to platform-only tools
- [x] Rename binary to `ship`, clean CLI surface
- [x] Remove stale git hooks
- [x] ARCHITECTURE.md, SPEC.md, TASKS.md written
- [x] Devcontainer + rust-toolchain.toml
- [x] vite-plus / oxlint wired, zero lint warnings
- [x] pnpm workspace cleaned up
- [x] Clean state on main
