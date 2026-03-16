+++
title = "CLI"
owners = ["apps/ship-studio-cli/"]
profile_hint = "cli-lane"
+++

# CLI

The `ship` binary. Transport layer over runtime operations — business logic stays in the runtime.

## Actual
- [x] `ship use [profile]` — compile and activate provider config
- [x] `ship skill add <repo>` — install skills from GitHub into `.ship/agents/skills/`
- [x] `ship job` — create, list, update jobs
- [x] `ship profile` — list profiles
- [x] Worktree path from config

## Aspirational
- [ ] `ship init` — scaffold `.ship/` in any existing repo (interactive)
- [ ] `ship login` / `ship logout` — auth against Ship account, store token in `~/.ship/credentials`
- [ ] `ship profile push` / `pull` — sync profiles to/from account
- [ ] `ship install <url>` — install Ship from a web URL (self-extracting, sets up PATH)
- [ ] `ship update` — self-update binary
- [ ] `ship doctor` — diagnose config issues, missing deps, stale locks
- [ ] Shell completions — bash, zsh, fish
