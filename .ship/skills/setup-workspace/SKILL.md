---
name: setup-workspace
stable-id: setup-workspace
description: Use when starting a multi-agent work session to configure the terminal environment. Detects iTerm2, tmux, Warp, or VS Code and sets up panes/tabs for each agent in the pod. Customizable per user.
tags: [environment, terminal, workflow, setup]
authors: [ship]
---

# Workspace Setup

Configure your terminal for multi-agent development. One pane per agent, each in its own worktree with the right agent compiled.

## On Activation

1. Detect terminal: check `$TERM_PROGRAM`, `$TMUX`, `$WT_SESSION`
2. Read `.ship-session/pod.md` if it exists (user's saved layout)
3. If no saved layout, ask: "Which agents do you want in your pod today?"
4. Set up the environment

## Terminal Detection

```bash
if [ -n "$TMUX" ]; then
  TERMINAL="tmux"
elif [ "$TERM_PROGRAM" = "iTerm.app" ]; then
  TERMINAL="iterm"
elif [ "$TERM_PROGRAM" = "WarpTerminal" ]; then
  TERMINAL="warp"
elif [ "$TERM_PROGRAM" = "vscode" ]; then
  TERMINAL="vscode"
else
  TERMINAL="manual"
fi
```

## iTerm2 Setup

iTerm2 supports AppleScript for pane management:

```bash
# Create a new tab for each agent
osascript -e '
tell application "iTerm2"
  tell current window
    # Main tab — Mission Control
    tell current session
      write text "cd '"$PROJECT_ROOT"' && ship use mission-control"
      set name to "🎯 control"
    end tell

    # For each specialist agent:
    set newTab to (create tab with default profile)
    tell current session of newTab
      write text "cd '"{{ worktree_dir }}/<slug>"' && ship use <agent>"
      set name to "<emoji> <agent-name>"
    end tell
  end tell
end tell'
```

Tab naming convention:
- 🎯 control — Mission Control (main tab)
- 🦀 rust — Rust specialist
- 🌐 web — Web/React specialist
- ☁️ cloud — Cloudflare specialist
- 🔒 auth — Auth specialist
- 🧪 test — Test writer
- 👀 review — Reviewer

## tmux Setup

```bash
SESSION="ship"
tmux new-session -d -s "$SESSION" -n "control"
tmux send-keys -t "$SESSION:control" "cd $PROJECT_ROOT && ship use mission-control" Enter

# For each agent:
tmux new-window -t "$SESSION" -n "<agent>"
tmux send-keys -t "$SESSION:<agent>" "cd {{ worktree_dir }}/<slug> && ship use <agent>" Enter

# Layout: tiled view of all windows
tmux select-layout -t "$SESSION" tiled
tmux attach -t "$SESSION"
```

## VS Code Setup

VS Code multi-root workspace with worktree folders:

```bash
# Generate .code-workspace file
cat > .ship-session/ship.code-workspace << 'WS'
{
  "folders": [
    { "path": ".", "name": "🎯 control" },
    { "path": "<worktree-path>/<slug>", "name": "<emoji> <agent>" }
  ],
  "settings": {
    "terminal.integrated.tabs.title": "${process}",
  }
}
WS

code .ship-session/ship.code-workspace
```

## Warp Setup

Warp supports launch configurations:

```bash
# Each agent gets its own Warp tab
for agent in "${AGENTS[@]}"; do
  open -a Warp --args --working-directory "{{ worktree_dir }}/$agent"
done
```

## Manual Fallback

When terminal can't be detected, print launch commands:

```
Your pod is ready. Open these in separate terminals:

  Tab 1 (control):  cd /project && ship use mission-control
  Tab 2 (rust):     cd {{ worktree_dir }}/rust-work && ship use rust-runtime
  Tab 3 (web):      cd {{ worktree_dir }}/web-work && ship use web-lane
```

## Saving Your Layout

After setup, save the pod configuration:

```markdown
<!-- .ship-session/pod.md -->
# Pod Layout

terminal: iterm
agents:
  - name: control
    agent: mission-control
    path: .
    emoji: 🎯
  - name: rust
    agent: rust-runtime
    path: {{ worktree_dir }}/rust-work
    emoji: 🦀
  - name: web
    agent: web-lane
    path: {{ worktree_dir }}/web-work
    emoji: 🌐
```

Next session, this skill reads `pod.md` and recreates the exact same layout. Users customize once, reuse forever.

## User preferences

Terminal: **{{ terminal }}**, worktree base: **{{ worktree_dir }}**, tab emoji: **{{ pod_emoji }}**

To change:
```bash
ship vars set setup-workspace terminal <auto|iterm|tmux|warp|vscode|manual>
ship vars set setup-workspace worktree_dir <path>
ship vars set setup-workspace pod_emoji false
```

## Teardown

When the session ends:

```bash
# Close agent tabs (tmux)
tmux kill-session -t ship

# Or just close the terminal tabs manually
# Worktrees persist — they're cleaned up by the gate or manually
```
