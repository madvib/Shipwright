---
name: ship-cloud
stable-id: ship-cloud
description: Use when working with Ship Cloud — the hosted platform for auth, billing, registry hosting, GitHub integration, and remote workspace sync.
tags: [ship, cloud, platform]
authors: [ship]
---

# Ship Cloud

Ship Cloud is the hosted platform layer. It provides the services that require accounts, persistence, and external integrations — things the local CLI and Studio cannot do alone.

Cloud lives in a private repo. This doc skill tracks capabilities and interfaces that the public codebase needs to be aware of.

## What Cloud owns

- **Authentication** — Better Auth login, OAuth providers, user/org management
- **Billing** — Stripe subscriptions, usage metering, plan limits
- **Registry API** — package hosting, publish endpoints, search, security review queue
- **GitHub App** — auto-publish webhooks, PR integration, repo linking
- **Remote sync** — workspace state sync for teams, cloud-backed session storage
- **Landing page** — marketing site at getship.dev

## What Cloud does NOT own

- Local CLI (`ship` binary)
- Local daemon (`shipd`)
- Studio config IDE (workspace/agent/skill editing)
- Compiler (`.ship/` → provider configs)
- Registry browser UI (client-side, reads from registry API)

## Interfaces

Studio talks to Cloud through:
- Registry search/install API (`/api/registry/*`)
- Auth endpoints (login, session validation)
- GitHub webhook receiver

The CLI talks to Cloud through:
- `ship publish` → registry publish API
- `ship login` → auth flow
- `ship add` → registry resolve API
