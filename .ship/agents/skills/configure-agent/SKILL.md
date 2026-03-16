---
name: Configure Agent
description: Use when setting up a workspace for a specialist agent — selecting preset, permission tier, scope, and worktree path. Ensures agents are productive (not over-restricted) and safe (not under-restricted).
---

## Permission Tier Selection

Pick the tier that matches the blast radius of the work, not your anxiety level.
Over-restricting kills productivity — every approval interruption breaks flow.

| Tier | default_mode | Use for |
|------|-------------|---------|
| `ship-open` | `bypassPermissions` | CI agents, service workers, fully trusted automated contexts |
| `ship-standard` | `acceptEdits` | Most specialist workers — web, Rust, docs, tests |
| `ship-guarded` | `default` | DB migrations, infra changes, anything touching prod config |
| `ship-plan` | `plan` | Reviewer/gate agents, analysis-only, no writes |

**Default to `ship-standard`.** Most agents doing scoped feature work should be on `acceptEdits` — file edits never interrupt, only genuinely dangerous bash patterns ask.

## The Compound Command Problem

Claude Code matches permission patterns against the FULL command string.
`Bash(ship exec*)` will NOT match `cd .target/bin && ship exec blah`.

Rules:
- Never try to allowlist compound commands with patterns — it won't work
- Use `default_mode` to set the baseline, `tools_ask` only for specific destructive ops
- If a legitimate workflow always chains commands, document the full pattern OR grant the tier that doesn't need to ask

```toml
# Wrong: trying to pattern-match compound commands
tools_ask = ["Bash(ship exec*)"]   # misses: cd dir && ship exec

# Right: set mode, guard only what's genuinely dangerous
default_mode = "acceptEdits"
tools_ask = ["Bash(rm -rf*)", "Bash(*--force*)", "Bash(*drop table*)"]
```

## Preset → Tier Mapping

| Preset | Recommended tier | Rationale |
|--------|-----------------|-----------|
| react-architect | `ship-standard` | Feature work, no infra |
| react-designer | `ship-standard` | UI only |
| better-auth | `ship-standard` | App layer, not DB migrations |
| rust-runtime | `ship-guarded` | Owns DB migrations exclusively |
| rust-compiler | `ship-standard` | Pure transforms, WASM only |
| cloudflare | `ship-guarded` | Infra + deploy commands |
| commander | `ship-standard` | Orchestration, not execution |

## Worktree Setup

Canonical path: read `~/.ship/config.toml [worktrees] dir`.
Default if not set: `~/dev/<project-name>-worktrees/`

```bash
# worktree path
WORKTREE_BASE=$(ship config get worktrees.dir 2>/dev/null || echo ~/dev/ship-worktrees)
WORKTREE_PATH="$WORKTREE_BASE/<job-id>"

git worktree add "$WORKTREE_PATH" -b job/<job-id>
cd "$WORKTREE_PATH"
ship use <preset>
```

Never use `../<branch>` — always use the configured path.
Never use Claude Code's native worktree UI — it won't respect the config.

## Scope Constraints

Always set explicit file scope in the job spec. The preset constrains capability
(what the agent knows how to do). Scope constrains authority (what it's allowed to touch).

```
Scope: apps/web/src/routes/, apps/web/src/lib/auth*
Off-limits: crates/, apps/mcp/, apps/ship-studio-cli/
```

Agents WILL respect explicit scope instructions. Don't rely on preset alone.

## Checklist Before Starting an Agent

- [ ] Correct preset for the domain?
- [ ] Permission tier matched to blast radius (default to ship-standard)?
- [ ] Worktree created at canonical path?
- [ ] `ship use <preset>` run IN the worktree?
- [ ] Job spec includes scope + off-limits?
- [ ] Acceptance criteria written (not just "make it work")?
- [ ] Dependencies noted (what must already be true)?
