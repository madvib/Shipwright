+++
title = "Infra"
owners = ["apps/web/wrangler.jsonc", "crates/core/mcp-server/"]
profile_hint = "cloudflare"
+++

# Infra

Cloudflare-hosted runtime — Workers, D1, Durable Objects (via Rivet). Enables Studio hosting, cloud job queue, multi-device sync, and async notifications.

Cross-cutting: enables Studio (hosting), Platform (cloud queue), MCP (remote access).

## Actual
- [x] Studio deployed on Cloudflare Workers (wrangler.jsonc)

## Aspirational
- [ ] D1 database — cloud job queue, account storage, profile sync
- [ ] Durable Objects (Rivet actors) — presence, real-time session coordination
- [ ] KV — session cache, skill registry cache
- [ ] Auth worker — JWT validation, GitHub OAuth callback
- [ ] Provider monitor worker — cron job crawling provider config docs, triggers matrix update
- [ ] Ship API — REST/RPC layer over D1 for CLI auth bridge and Studio account features
- [ ] Environments — staging (`.dev.vars`) and production separate, no shared credentials
