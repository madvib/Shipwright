# Pending Dependency Resolution

Skills removed from `.ship/agents/skills/` as part of v0.1.0 ship_trim (job XWxQJPtd).

## Resolved — Added to [dependencies] in ship.toml

These skills have confirmed public git repositories and are declared as dependencies:

| Skill | Source | ship.toml key |
|-------|--------|---------------|
| gstack | https://github.com/garrytan/gstack | `"github.com/garrytan/gstack" = "main"` |
| better-auth | https://github.com/better-auth/skills | `"github.com/better-auth/skills" = "main"` |

## Resolved — Provided by Plugin

These skills are available via the `superpowers@claude-plugins-official` plugin, which is already
installed in `ship.lock`. No dependency entry needed.

| Skill | Plugin |
|-------|--------|
| cloudflare | superpowers@claude-plugins-official |
| workers-best-practices | superpowers@claude-plugins-official |
| durable-objects | superpowers@claude-plugins-official |
| agents-sdk | superpowers@claude-plugins-official |
| building-ai-agent-on-cloudflare | superpowers@claude-plugins-official |
| building-mcp-server-on-cloudflare | superpowers@claude-plugins-official |
| web-perf | superpowers@claude-plugins-official |
| hono | superpowers@claude-plugins-official |
| wrangler | superpowers@claude-plugins-official |
| tanstack-query | superpowers@claude-plugins-official |
| tanstack-router | superpowers@claude-plugins-official |
| tanstack-start | superpowers@claude-plugins-official |
| tanstack-integration | superpowers@claude-plugins-official |

## Action Required

The agent session that prepared these changes does not have shell access. Run the following
sequence from the repo root to complete the trim:

```bash
# Step 1: Remove third-party skills from git and filesystem
bash trim-skills.sh

# Step 2: Commit the removal
git add -A
git commit -m "chore: trim third-party skills from .ship/agents/skills/"

# Step 3: Verify
ls .ship/agents/skills/
# Should show only: commander  configure-agent  find-skills  ship-coordination  spawn-agent  write-adr

git diff HEAD~1 --stat
# Should show deletions for all removed skill directories

# Step 4: Activate (ship install already ran inside trim-skills.sh)
ship use web-lane
cargo test -p ship-studio-cli
```

Note: `ship use web-lane` requires `ship install` to have populated `ship.lock` with the registry
format first. The `trim-skills.sh` script runs `ship install` automatically. If you skip the script,
run `ship install` manually before `ship use web-lane`.
