+++
title = "Compiler"
owners = ["crates/core/compiler/", "apps/ship-studio-cli/src/compile.rs", "apps/ship-studio-cli/src/mode.rs"]
profile_hint = "rust-compiler"
+++

# Compiler

Transforms `.ship/` source of truth into native provider config. Same inputs + state = same outputs. Deterministic, explicit failures, no silent fallbacks.

## Actual
- [x] Profile resolution — TOML → `ResolvedConfig`
- [x] Claude output — CLAUDE.md, .mcp.json, .claude/settings.json
- [x] `provider_settings` pass-through — arbitrary provider flags
- [x] Team agents — `.ship/agents/teams/<provider>/` → `.<provider>/agents/`
- [x] Skill injection — refs resolved, content merged into context
- [x] Permissions — tiers (ship-standard, ship-guarded, read-only, full-access), deny/ask patterns
- [x] Rules — inline + rules/*.md concatenated
- [x] MCP server config — servers filtered by profile refs

## Aspirational
- [ ] Dry-run diff — what will change, before writing
- [ ] Validation — schema errors with actionable messages, not panics
- [ ] Gemini output — GEMINI.md, .gemini/ config
- [ ] Codex output — AGENTS.md, codex config
- [ ] Cursor/Windsurf output — .cursor/rules, .windsurfrules
- [ ] Provider feature matrix — per-provider capability flags compiled into config
- [ ] Multi-profile merge — base profile + overlay (inheritance)
- [ ] Watch mode — recompile on `.ship/` changes
