---
name: Spawn Agent
id: spawn-agent
version: 0.1.0
description: Dispatch a job to a specialist agent in a git worktree. Creates the worktree, compiles the agent config, writes the job spec, and opens a new terminal session. Use this from the commander when dispatching jobs.
tags: [commander, orchestration, worktree, dispatch]
authors: [ship]
---

# Spawn Agent

Dispatch a job to a specialist agent. Follow this sequence exactly — no shortcuts.

## Prerequisites

- `ANTHROPIC_API_KEY` must be in `~/.bashrc` or `~/.zshrc` (not just the current session — spawned terminals don't inherit session exports)
- `ship` and `claude` CLIs must be on `$PATH`
- Job must exist in the queue with a description and profile hint

## Sequence

### 1. Read the job

Get the full job payload: title, description, acceptance criteria, scope, profile hint.

```
python3 -c "
import sqlite3, os, json
db = os.path.expanduser('~/.ship/state/ship-hrvmuz4p/platform.db')
conn = sqlite3.connect(db)
cur = conn.cursor()
cur.execute('SELECT payload_json FROM job WHERE id=?', ('<JOB_ID>',))
row = cur.fetchone()
print(json.dumps(json.loads(row[0]), indent=2))
conn.close()
"
```

> Note: replace `ship-hrvmuz4p` with the actual slug for this project. Run `ls ~/.ship/state/` and look for the entry matching this repo.

### 2. Resolve the worktree path

Default: `~/dev/ship-worktrees/<job-id>`

Check `~/.ship/config.toml` for a `[worktrees] dir` override:

```toml
[worktrees]
dir = "~/dev/my-worktrees"
```

If set, use that base dir. Otherwise use `~/dev/ship-worktrees/`.

### 3. Create the worktree

```bash
git worktree add ~/dev/ship-worktrees/<job-id> -b job/<job-id>
```

If the branch already exists (resuming a stalled job):

```bash
git worktree add ~/dev/ship-worktrees/<job-id> job/<job-id>
```

### 4. Compile the agent config

```bash
cd ~/dev/ship-worktrees/<job-id> && ship use <profile>
```

Profile comes from the job payload (`preset_hint` field) or this table:

| Work type | Profile |
|-----------|---------|
| Rust runtime / DB / platform | `rust-runtime` |
| Rust compiler / CLI | `rust-compiler` |
| Web / React / Studio | `web-lane` |
| Cloudflare Workers / infra | `cloudflare` |
| Auth / Better Auth | `better-auth` |
| Default / mixed | `default` |

### 5. Write the job spec

Write `job-spec.md` to the worktree root. This is the agent's opening context — they should read it on start.

```markdown
# Job <JOB_ID> — <title>

## What
<description from payload>

## Scope
File scope: <file_scope from payload>
Profile: <profile>

## Acceptance Criteria
<acceptance_criteria checklist>

## Constraints
- Stay within declared file scope
- Append touched files to job log via MCP: `append_job_log(id, "touched: path/to/file")`
- Mark done via MCP: `update_job(id, status="complete")` — commander runs the gate

## Context
Branch: job/<job-id>
Worktree: ~/dev/ship-worktrees/<job-id>
```

### 6. Launch the terminal

Detect platform and open a new terminal session in the worktree directory.

**WSL (Windows Terminal):**
```bash
cmd.exe /c start wt.exe -d "$(wslpath -w ~/dev/ship-worktrees/<job-id>)"
```

**macOS:**
```bash
osascript -e 'tell application "Terminal" to do script "cd ~/dev/ship-worktrees/<job-id> && claude ."'
```

**iTerm2 (macOS):**
```bash
osascript << 'EOF'
tell application "iTerm2"
  create window with default profile
  tell current session of current window
    write text "cd ~/dev/ship-worktrees/<job-id> && claude ."
  end tell
end tell
EOF
```

**Fallback (unknown platform):**
```
echo "Open a new terminal and run:"
echo "  cd ~/dev/ship-worktrees/<job-id> && claude ."
```

**Platform detection:**
```bash
if grep -qi microsoft /proc/version 2>/dev/null; then
  echo "wsl"
elif [[ "$OSTYPE" == "darwin"* ]]; then
  echo "macos"
else
  echo "linux"
fi
```

### 7. Update job status

```
update_job(id="<job-id>", status="running")
```

If the job has a `claimed_by` field in the payload, set it to the current provider ID (e.g. `"claude-main"`).

### 8. Tell the human

```
Dispatched [<job-id>] <title>
→ worktree: ~/dev/ship-worktrees/<job-id>
→ profile: <profile>
→ terminal: opened (or: "open manually — see above")
```

## Opening Message for the Agent

When you open the terminal manually, paste this as the opening message:

```
Read job-spec.md in this directory. That is your full context — scope, acceptance criteria, constraints. Start working. Report progress via append_job_log. Mark done via update_job when acceptance criteria are met.
```

## Stale Worktree Cleanup

After a job completes and the gate passes:

```bash
git worktree remove ~/dev/ship-worktrees/<job-id>
git branch -d job/<job-id>
```

Or use `list_stale_worktrees()` from the commander to find idle worktrees > 24h.

## Troubleshooting

**`wt.exe` not found:** Windows Terminal not installed, or not on PATH from WSL. Install from Microsoft Store or use the fallback.

**`claude .` opens but has no MCP tools:** The `.mcp.json` wasn't generated. Run `ship use <profile>` again in the worktree, then restart the Claude session in that directory.

**Agent can't see job queue:** MCP server not running or wrong project path. Confirm `ship-mcp` is installed (`which ship-mcp`) and `.mcp.json` points to it correctly.

**`ANTHROPIC_API_KEY` missing in spawned terminal:** Add `export ANTHROPIC_API_KEY=...` to `~/.bashrc` and reload: `source ~/.bashrc`.
