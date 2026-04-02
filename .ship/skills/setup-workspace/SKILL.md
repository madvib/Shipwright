---
name: setup-workspace
stable-id: setup-workspace
description: Configure a multi-agent terminal environment. Reads pod.md layout, opens one tab/window per agent in their worktree.
tags: [environment, terminal, workflow, setup]
authors: [ship]
---

# Workspace Setup

Launch your agent pod in one command. Reads `.ship-session/pod.md` and opens a
terminal tab for each agent in its worktree.

## Usage

```bash
bash .ship/skills/setup-workspace/scripts/setup.sh
```

Reads `.ship-session/pod.md` by default. Pass `--pod <file>` to use a different layout.

```bash
bash .ship/skills/setup-workspace/scripts/setup.sh --pod .ship-session/pod.md
bash .ship/skills/setup-workspace/scripts/setup.sh --dry-run  # preview without opening
```

## Pod layout file

Create `.ship-session/pod.md` to define your agent pod. The file is gitignored.

```yaml
agents:
  - name: rust
    agent: rust-runtime
    path: ~/dev/ship-worktrees/rust-work
  - name: web
    agent: web-lane
    path: ~/dev/ship-worktrees/web-work
  - name: auth
    agent: better-auth
    path: ~/dev/ship-worktrees/auth-work
```

`path` defaults to `$SHIP_WORKTREE_DIR/<name>` if omitted.

Save the file once, reuse every session.

## Environment variables

Set in your shell profile — personal config, not project files.

| Variable | What |
|----------|------|
| `SHIP_DEFAULT_TERMINAL` | Force terminal: `tmux`, `wt`, `iterm`, `vscode`, `warp`, `manual` |
| `SHIP_WORKTREE_DIR` | Base directory for worktrees (default: `~/dev/ship-worktrees`) |

## Terminal support

| Terminal | Detection | Behavior |
|----------|-----------|----------|
| tmux | `$TMUX` set | `tmux new-window -d` per agent |
| Windows Terminal (WSL) | `$WT_SESSION` set | `wt.exe -w 0 nt` per agent |
| iTerm2 | `$TERM_PROGRAM=iTerm.app` | AppleScript tab per agent |
| manual | fallback | Prints launch commands |

Set `SHIP_DEFAULT_TERMINAL` to override detection. Useful when `WT_SESSION` is
set inside tmux (WSL) and you want tmux windows instead.

## Teardown

```bash
# tmux
tmux kill-session -t ship

# or close tabs manually — worktrees persist until gate passes or manual cleanup
git worktree remove ~/dev/ship-worktrees/<slug>
git branch -d job/<slug>
```
