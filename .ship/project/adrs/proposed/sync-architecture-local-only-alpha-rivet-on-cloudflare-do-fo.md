<!-- 
  GENERATED FILE — DO NOT EDIT DIRECTLY
  This file is exported from the Ship SQLite database.
  Changes here will NOT be synchronized back to the database.
-->

+++
id = "xu3bE4js"
title = "Sync architecture — local-only alpha, Rivet on Cloudflare DO for cloud, self-hosted Rivet for enterprise"
date = "2026-03-03T16:26:46.304911579+00:00"
tags = []
+++

## Decision

Alpha ships local-only (SQLite, no sync). Cloud tier uses Rivet actors on Cloudflare Durable Objects via @rivetkit/cloudflare-workers with Hono as the BFF layer. Enterprise self-hosted tier uses the same Rivet actor code targeting a self-hosted Rivet cluster. TrailBase as the self-hosted single-tenant option. Personal multi-device sync is a tomorrow problem (Syncthing stopgap). All alternatives evaluated and rejected: cr-sqlite (dormant, would own it), Litestream/LiteFS (single-writer only, not real sync), PGlite (WASM overhead, still needs server Postgres), pg_embed (50-100MB bundle, hard no). See runtime primitives spec for sync engine hook model.
