+++
id = "05f4b09a-a2ce-419d-97da-e23aba7a0850"
title = "Per-provider agent config export architecture"
status = "accepted"
date = "2026-03-01"
tags = []
+++

## Decision

## Context

Ship supports multiple AI agent providers (Claude Code, Gemini CLI, Codex CLI) and will support editor integrations (Cursor, Windsurf) as additional providers. Each provider has a different native location for skills, different MCP config syntax, different permission models, and different context file formats. The initial implementation left Gemini and Codex without feature context, rules, or skills — only MCP config was wired up. Rules (`agents/rules/*.md`) were never exported to any provider.

## Decision

Extend the `ProviderDescriptor` abstraction to capture the full per-provider export surface. Each provider export consists of five layers, each with provider-specific syntax:

### 1. MCP servers → already done per provider
- Claude: `.mcp.json` (JSON, `mcpServers` key, inline `_ship` managed marker)
- Gemini: `.gemini/settings.json` (JSON, `mcpServers` key, no `type` field)
- Codex: `.codex/config.toml` (TOML, `mcp_servers` key)
- Cursor (future): `.cursor/mcp.json` (JSON, same as Claude format)

### 2. Context file → feature spec + open issues + rule references + skill map
The context file is the "stitcher" — it tells the agent what it's working on and where to find its tools. Written by the git module on branch checkout, not by `agent_export`.

- Claude: `CLAUDE.md` (already feature-aware ✅)
- Gemini: `GEMINI.md` — must be made feature-aware (currently only written if `prompt_id` set ❌)
- Codex: `instructions` key in `.codex/config.toml` — must be made feature-aware ❌
- Cursor (future): `.cursor/rules/ship.mcd`

Content is provider-agnostic Markdown. The git module's `generate_claude_md` is refactored to `generate_context(dest, feature, issues, skills, rules)` and called for every provider with a context file destination.

### 3. Skills → provider-specific native locations where they exist; inline in context file otherwise
- Claude: `.claude/commands/<id>.md` — native slash commands ✅
- Gemini: No native skill system → inline skill content in `GEMINI.md` under `## Skills` section with invocation instructions
- Codex: No native skill system → inline in `instructions` field
- Cursor (future): No native skill system → inline in `.cursor/rules/ship.mcd`

`SkillsOutput` enum gains a variant `InlineInContext` for providers without native skill locations.

### 4. Rules → always-active instructions, inlined in every provider's context file
Rules (`agents/rules/*.md`) are short, always-active instructions. No provider has a native "rules directory" outside Claude's ecosystem. Decision: inline rule content in every provider's context file under a `## Rules` section. They are not written to any separate provider-specific file.

Exception: Claude `settings.json` `hooks` field — hooks are not rules and remain a separate concept.

### 5. Permissions → provider-specific, degrade gracefully
- Claude: `~/.claude/settings.json` permissions block (`allow`/`deny` arrays) ✅
- Gemini: No equivalent permission model in Gemini CLI → skip, no warning needed
- Codex: Codex CLI has a `policy` field (`auto`/`full`) and optional `allow` list in `.codex/config.toml` → map our `Permissions.network.policy` to Codex `policy`; map `Permissions.commands.allow` to Codex `allow`
- Cursor (future): Cursor has no programmatic permission config → skip

### Adding a new provider (editor or agent)
1. Add a `ProviderDescriptor` entry in `PROVIDERS` (10–15 lines)
2. Add a `PromptOutput` variant if the context file path is novel
3. Add a `SkillsOutput` variant if the provider has a native skill location
4. Handle the new variants in `export_to_inner` (context write) and `teardown`
5. No architectural changes required

### SHIPWRIGHT.md
Removed. Was never implemented, only referenced. Ill-conceived as a second context file. The single per-provider context file (CLAUDE.md, GEMINI.md, etc.) is the right abstraction.

## Consequences

### Positive
- All three launch providers get equivalent feature context, rules, and skills on checkout
- New providers (editors) slot in without rearchitecting
- Context file content is provider-agnostic — maintain in one place
- Rules always reach the agent regardless of provider

### Negative
- Gemini and Codex skill content is inlined in the context file rather than accessible as slash commands — less ergonomic for those providers
- Codex permission mapping is lossy (only `network.policy` and `commands.allow` translate)
- `generate_context` in the git module must handle TOML write for Codex (updating a key in an existing TOML file rather than writing a standalone markdown file)
