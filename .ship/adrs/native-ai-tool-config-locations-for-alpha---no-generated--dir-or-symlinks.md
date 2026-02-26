+++
id = "815c3a23-d01d-4532-bbb3-811fb59b3fb6"
title = "Native AI tool config locations for alpha — no generated/ dir or symlinks"
status = "accepted"
date = "2026-02-26"
tags = []
+++

## Decision

For alpha, generate AI tool configs at their native expected locations rather than consolidating under .ship/generated/ with symlinks.

**Context:** The directory structure spec called for all generated AI tool configs (CLAUDE.md, .mcp.json, .gemini/settings.json, .codex/config.toml) to live under .ship/generated/ with symlinks from their expected locations for tools that hardcode paths.

**Decision:** Generate at native locations for alpha. .ship/generated/ is the V1 target.

**Reasons:**
- Symlinks are fragile on Windows and in some git operations
- Claude Code, Gemini CLI, and Codex all have well-known config paths that work today
- "Dip our toes" approach: test the generated/ consolidation on a single tool first before committing
- Alpha goal is dogfooding the core loop, not solving config housekeeping
- Native locations are known-good; generated/ + symlinks is unverified in practice

**V1 path:** Add agentPolicy.generatedConfigStrategy = "ship-dir" | "native" flag. Default "native". Flip to "ship-dir" once verified across all three tools. Symlinks created only where a tool has no config path flag and only after testing.

**Gitignore:** CLAUDE.md, .mcp.json, .gemini/, .codex/ remain gitignored at project root regardless of this decision.
