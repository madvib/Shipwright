+++
id = "xMHxyPGF"
title = "TOML as universal config and frontmatter format"
date = "2026-02-28"
spec_id = ""
supersedes_id = ""
tags = []
+++

## Context

Shipwright uses frontmatter in markdown documents and standalone config files (`ship.toml`, `agents/mcp.toml`, `agents/permissions.toml`). Two formats were evaluated for alpha: JSONC (with `$schema` for editor autocomplete via LSP) and TOML. The directory structure spec had initially called for JSONC + a published schema URL.

## Decision

Use TOML for all config files and document frontmatter throughout the alpha.

**Context:** Evaluated JSONC (with $schema for editor autocomplete) vs TOML for config files. The directory structure spec called for JSONC + published schema.

**Decision:** TOML everywhere for alpha.

**Reasons:**
- AI agents read and write TOML more reliably — less punctuation noise than JSON
- Consistency: document frontmatter is already TOML, one mental model throughout
- The schema is still changing too fast to publish and maintain a JSONC schema URL
- The UI provides autocomplete for config, making $schema less critical for hand-editing
- Can add taplo-based JSON schema for TOML in editors later with zero format migration
- Zero migration cost from current codebase

**Deferred:** JSONC + published schema at a stable URL is a V1 polish item, once the schema has stabilised and we want external tooling (LSPs, third-party editors) to validate Shipwright config files without the app installed.

## Consequences

### Positive
- Single format everywhere: one mental model for agents, humans, and tooling
- Agents write TOML more reliably than JSON — less punctuation noise, fewer quoting errors
- No migration cost from current codebase
- TOML has a taplo LSP for editor support when needed

### Negative
- TOML is less widely known than JSON; some developers will need to learn it
- No $schema autocompletion until a taplo schema is published (V1)
- TOML multiline strings and arrays are verbose for complex nested config
