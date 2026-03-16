+++
title = "Studio"
owners = ["apps/web/"]
profile_hint = "web-lane"
+++

# Studio

Ship's web UI. Profile builder, provider matrix, account management. Deployed on Cloudflare Workers.

## Actual
- [x] Basic compiler UI (import, compile)
- [x] Wrangler deployment config

## Aspirational
- [ ] Polished component library — production-quality, consistent design system
- [ ] Profile builder — visual editor for `.ship/agents/presets/` TOML
- [ ] Provider matrix UI — per-provider feature flags, surfaced and togglable
- [ ] Skill browser — discover, preview, add skills from Studio
- [ ] Account-connected — sign in, save profiles to account, share with team
- [ ] Import flow — paste repo URL → generate .ship scaffold → download or push via GitHub App
- [ ] CLI install from Studio — one-click with pre-configured profile
- [ ] Live preview — show compiled CLAUDE.md / .mcp.json output as you edit
