+++
id = "Jd7ztt92"
title = "Native AI tool config locations for alpha — no generated/ dir or symlinks"
date = "2026-02-28"
spec_id = ""
supersedes_id = ""
tags = []
+++

## Context

The original directory structure spec called for all generated AI tool configs to live under `.ship/generated/` with symlinks from the tool-expected locations. This would keep the repo root clean and make all generated files easy to find and gitignore with one entry. However, symlinks have known failure modes on Windows and in some git operations, and the approach was untested in practice.

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

## Consequences

### Positive
- Works immediately on all platforms — no symlink fragility
- No unverified infrastructure to debug during alpha dogfooding
- Native tool paths are well-documented and stable
- Derisks the generated/ approach by deferring it until it can be tested tool by tool

### Negative
- Generated files spread across the repo root instead of consolidated under .ship/
- Each tool needs its own gitignore entry rather than one wildcard for generated/
- The V1 migration to generated/ will require updating gitignore and potentially breaking existing setups
