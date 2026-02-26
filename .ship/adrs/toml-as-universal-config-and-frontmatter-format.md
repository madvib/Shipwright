+++
id = "2d8ed20f-60d1-4887-b70e-f3c20b5afcbe"
title = "TOML as universal config and frontmatter format"
status = "accepted"
date = "2026-02-26"
tags = []
+++

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
