+++
title = "MCP Server"
owners = ["crates/core/mcp-server/", "crates/packages/"]
profile_hint = "rust-runtime"
+++

# MCP Server

Ship's own MCP interface — the coordination layer commanders and agents talk to. Thin transport over platform runtime.

## Actual
- [x] Core workflow tools: jobs, workspaces, sessions, notes, ADRs
- [x] ship:// resources
- [x] Project auto-detection

## Aspirational
- [ ] Extended tools behind mode gates — not all tools always active
- [ ] `claim_file` / `get_file_owner` — file ownership protocol
- [ ] `assign_job` — set assigned_to, risk_level
- [ ] Cloud backend — tools route to D1/DO when cloud queue is configured
- [ ] Streaming job logs — real-time log tail via MCP resource
- [ ] Webhook / push notification — surface human-action jobs to external systems
