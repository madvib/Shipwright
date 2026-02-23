+++
title = "MCP sampling for AI generation in UI"
created = "2026-02-22T07:02:04.476929543Z"
updated = "2026-02-22T07:28:14.517669591Z"
tags = []
links = []
+++

Implement AI generation features using MCP sampling (server calls back to Claude Code via peer.create_message()). Tools: generate_issue_description, generate_adr, brainstorm_issues. Tauri UI gets AI generate buttons on issue/ADR creation forms. Fallback to direct Anthropic API when sampling not available (standalone mode). Requires anthropic_api_key in global config.

## Implementation

Added MCP sampling + Anthropic API fallback to crates/mcp/src/lib.rs:
- generate_issue_description, generate_adr, brainstorm_issues tools accept Peer<RoleServer>
- generate_with_sampling(): tries peer.create_message() first, falls back to direct Anthropic API
- call_anthropic_api() uses reqwest with rustls-tls
- Fixed: SamplingMessage::user_text() (not new_user_text), removed enable_sampling() from ServerCapabilities
- Added reqwest + rustls-tls to mcp/Cargo.toml
