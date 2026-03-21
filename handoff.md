# Handoff — v0.1.0 Session 2026-03-21 (continued)

## Branch: `v0.1.0`

## What happened this session

### Committed (6 commits on v0.1.0)
- `02ada1f` feat: split DB schema, enrich events, add session metrics and gate tracking
- `5f247e1` feat: add file ownership claims for job conflict detection
- `ce99d54` feat: add update_target MCP tool with tests
- `7238d76` feat: TUI events tab, detail views, scrolling, and CLI audit
- `145ed46` feat: help agent with tutorial skill, fix permission presets
- `2d3dece` docs: update handoff for v0.1.0 session

### Uncommitted on v0.1.0 (ready to commit)
- **`ship config get/set/list/path`** — user preferences CLI for `~/.ship/config.toml`
  - Keys: terminal.program, dispatch.confirm, worktrees.dir, defaults.provider, identity.name, etc.
  - Validation on set (terminal.program must be known value)
  - 4 new tests, help topic added (`ship help config`)
  - Files: config.rs, cli.rs (ConfigCommands), main.rs (dispatch_config), help_topics.rs
- **`dispatch.sh`** — idempotent job dispatch script in spawn-agent skill
  - Creates worktree, writes job spec, compiles agent, opens terminal — one command
  - Confirm flow: shows spec summary, y/n/e(dit) before launching agent
  - Terminal auto-detection: wt (WSL2), iterm, tmux, gnome, vscode, manual
  - Resolution: flag > env var > `ship config get` > default
  - Files: .ship/agents/skills/spawn-agent/scripts/dispatch.sh, SKILL.md rewrite
- **spawn-agent SKILL.md rewrite** — documents dispatch.sh, env vars, batch dispatch, confirm flow

### Pruned worktrees
- `A39uK8JX` — force removed (was merged)
- `agent-rename` — force removed (stale)
- `tutorial` — force removed (replaced by help agent)

### Work streams specced and dispatched
6 spec agents ran in parallel, produced detailed job specs with acceptance criteria.
6 worktrees created with job specs written to `.ship-session/job-spec.md`:

| Worktree | Branch | Agent | Spec status |
|----------|--------|-------|-------------|
| jsonc-config | job/jsonc-config | rust-compiler | Ready — JSONC migration, JSON Schema |
| registry | job/registry | rust-lane | Ready — publish, namespace, auth, signing |
| docs | job/docs | default | Ready — README (#1), getting started, refs, arch |
| skills-library | job/skills-library | default | Ready — audit, curate, fix, export |
| commander-demo | job/commander-demo | default | Ready — docs + config only |
| tui-interactive | job/tui-interactive | cli-lane | Ready — status cycling, launch, filter |

### Worktrees still pending
- `mLaHiccr` — registry rework (4 files), needs gate review before registry job starts

### Tests: 312 runtime, 192 CLI (incl 4 new config), 3 MCP — all passing

### CLI installed with `ship config` command

## Next steps

### Immediate
1. **Commit** uncommitted work (config command + dispatch script + SKILL.md)
2. **Gate review mLaHiccr** — registry job depends on this
3. **Launch work streams** — use dispatch.sh:
   ```bash
   bash .ship/agents/skills/spawn-agent/scripts/dispatch.sh \
     --slug docs --agent default \
     --spec ~/dev/ship-worktrees/docs/.ship-session/job-spec.md
   ```
   Repeat for each worktree. Or launch manually with the commands in each worktree.

### Dependency ordering
- **Parallel now**: docs, skills-library, commander-demo, tui-interactive
- **Gate first**: mLaHiccr → then registry can start
- **JSONC first**: registry published spec depends on config format decision

### User config set (in ~/.ship/config.toml)
```toml
[worktrees]
dir = "/home/madvib/dev/ship-worktrees"

[terminal]
program = "wt"

[dispatch]
confirm = true
```

### Design decisions made this session
- **Config format**: `~/.ship/config.toml` for user prefs. TOML for now — will migrate to JSONC with the rest when gNCTZBBs lands.
- **Dispatch UX**: Spec agents write specs to disk (not inline). dispatch.sh does worktree + spec + agent + terminal in one idempotent call. Confirm flow (y/n/e) available via `ship config set dispatch.confirm true`.
- **Preferences over env vars**: `ship config get/set` is the primary interface. Env vars override config. Scripts call `ship config get` — no JSONC parser needed in bash.
- **Terminal detection**: Explicit preference (`ship config set terminal.program wt`) > env var (`SHIP_DEFAULT_TERMINAL`) > auto-detect > manual fallback.

### Known issues
- `ship use` from wrong CWD writes output to wrong location
- Plugin uninstall warnings when switching agents (cosmetic)
- Spec agents returned inline — wasted context. Next time: have them write to `.ship-session/specs/` directly.
