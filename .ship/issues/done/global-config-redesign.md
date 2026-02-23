+++
title = "Global config redesign"
created = "2026-02-22T07:02:04.503388475Z"
updated = "2026-02-22T07:31:01.363020323Z"
tags = []
links = []
+++

Extend Config with: anthropic_api_key (for direct API calls), model (default claude-haiku-4-5), and a ModelConfig sub-struct. Expose in UI settings panel. Needed as foundation for AI generation features.

## Implementation
Added AiConfig to crates/logic/src/config.rs: anthropic_api_key, model, max_tokens fields with resolve_api_key() (config → ANTHROPIC_API_KEY env var fallback), effective_model() defaults to claude-haiku-4-5-20251001. Config struct gains optional ai field.
