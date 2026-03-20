# Ship Skills Catalog

Skills are markdown instruction sets that extend agent behavior. They are compiled into provider config by `ship use` and loaded at agent startup alongside the active profile.

Skills live in `.ship/agents/skills/<skill-id>/SKILL.md`.

---

## Installing skills

```bash
ship skill add github.com/owner/repo      # from GitHub
ship skill add ./path/to/skill            # from local path
ship skill list                           # see what's installed
ship skill remove <id>                    # remove a skill
```

Skills are project-local by default (`--global` installs to `~/.ship/skills/`).

---

## Skill catalog

### `commander`

**Visibility:** private (project tooling)

Ship orchestrator skill. Defines the commander agent's full protocol: workspace activation, job claiming, pod coordination, risk tiers, gate protocol, session lifecycle, and multi-commander coordination.

**When to use:** Add this skill to agent profiles that act as orchestrators — routing jobs, managing worktrees, and surfacing work to the human. One commander per human. Multiple commanders (different providers, different machines) coordinate via the job queue.

**Key behaviors:**
- Workspace activation protocol (claim → worktree → compile → job spec → start agent)
- Risk tiers: `auto` (no review), `review` (gate agent), `human` (human inbox)
- Gate protocol: verifies acceptance criteria before marking a job done
- Multi-commander coordination via atomic job claiming
- Session lifecycle: get project info → check running jobs → human inbox → pending queue → stale worktrees

---

### `configure-agent`

**Visibility:** public

Guides setting up agent workspaces with the right permission tier, profile, and scope. Prevents the most common mistakes: over-restricting agents (kills productivity) and under-restricting them (safety risk).

**When to use:** Any time a commander or human is setting up a new specialist agent — selecting profile, permission tier, scope constraints, and worktree path.

**Key behaviors:**
- Permission tier selection: `ship-open` → `ship-standard` → `ship-guarded` → `ship-plan`
- Profile → tier mapping table (e.g. `rust-runtime` needs `ship-guarded`, most others use `ship-standard`)
- Worktree setup: canonical path from `~/.ship/config.toml [worktrees] dir`
- Scope constraint guidance: profile = capability, scope = authority
- Pre-flight checklist before starting any agent

---

### `find-skills`

**Visibility:** public

Helps discover and install skills from the open agent skills ecosystem at [skills.sh](https://skills.sh/).

**When to use:** When a user asks "how do I do X", "find a skill for X", or wants to extend agent capabilities.

**Key behaviors:**
- Check skills.sh leaderboard before running CLI search
- Use `npx skills find <query>` to discover; `ship skill add <owner/repo>` to install
- Verify install count (prefer 1K+) and source reputation before recommending
- Do **not** use `npx skills add` — installs to `.agents/skills/` which Ship doesn't compile

---

### `ship-coordination`

**Visibility:** private (project tooling)

Defines the lane coordination protocol for Ship's parallel development model. Covers when and how to log progress, leave cross-lane notes, and signal blockers.

**When to use:** Add to agent profiles that run in work lanes (cli-lane, server-lane, web-lane, rust-lane). Teaches agents what to log and when, and how to signal downstream dependencies.

**Key behaviors:**
- Session hooks: `start_session` → `log_progress` → `end_session`
- Cross-lane signals via `create_note` with `[UNBLOCKS <lane>]` prefix
- Lane dependency map (cli → web, server-auth → web-auth, etc.)
- Commit discipline: push per completed subtask, no AI attribution

---

### `spawn-agent`

**Visibility:** private (project tooling)

Full dispatch protocol for sending a job to a specialist agent in a git worktree. Step-by-step: read job → resolve worktree path → create worktree → compile config → write job spec → launch terminal → update job status.

**When to use:** Used by the commander when dispatching a job. Covers platform detection (WSL, macOS, iTerm2) and troubleshooting.

**Key behaviors:**
- Worktree creation: `git worktree add ~/dev/ship-worktrees/<job-id> -b job/<job-id>`
- Config compilation: `ship use <profile>` in the worktree
- Job spec template with scope, acceptance criteria, and constraints
- Platform-specific terminal launch (WSL Windows Terminal, macOS Terminal/iTerm2)
- `update_job(status="running")` after dispatch

---

### `write-adr`

**Visibility:** public

Guides writing an Architecture Decision Record. Ensures ADRs capture context, decision, alternatives (minimum 2), consequences, and measurement criteria — not just what was decided, but why.

**When to use:** Before calling `create_adr`. Use the `write-adr` skill to structure the content first. Do not use for minor implementation choices — only decisions with meaningful alternatives that constrain future work.

**Key behaviors:**
- Five-section structure: Context → Decision → Alternatives → Consequences → How to measure
- Quality gate: at least 2 alternatives with honest rejection reasoning
- Title format: verb + noun ("Use D1 for cloud state", not "Database decision")

---

## Public vs private

| Skill | Visibility | Reason |
|-------|-----------|--------|
| `commander` | private | Ship project orchestration protocol — specific to this repo's multi-lane model |
| `configure-agent` | **public** | Useful to any Ship user setting up agent workspaces |
| `find-skills` | **public** | Useful to any Ship user discovering the skills ecosystem |
| `ship-coordination` | private | Specific to Ship's parallel lane structure |
| `spawn-agent` | private | Ship project dispatch protocol — specific to this repo's worktree conventions |
| `write-adr` | **public** | Useful to any project recording architectural decisions |

Public skills are those that provide value outside this specific repository. Private skills encode project-specific conventions and are not useful in isolation.

---

## Exported skills

Public skills are listed in `.ship/ship.toml` under `[exports] skills`. When someone runs `ship install github.com/madvib/ship`, they get these skills.

See `.ship/ship.toml` for the current exports list.

---

## Writing a new skill

```bash
ship skill create my-skill --name "My Skill" --description "Does X"
```

This scaffolds `.ship/agents/skills/my-skill/SKILL.md` with the required frontmatter:

```markdown
---
name: My Skill
id: my-skill
version: 0.1.0
description: Does X
---

# My Skill

...instructions...
```

Skills are plain markdown. They are compiled verbatim into the provider's instruction format. Keep them focused: one skill, one concern. Reference MCP tools by their canonical names (e.g. `create_job`, not `createJob`).
