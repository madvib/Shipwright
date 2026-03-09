---
name: provider-integration-hardening
description: Add or audit agent-provider integrations end-to-end with a strict 4-pass workflow (doc accuracy, hooks/security, UX/reactivity, reusable provider extension output). Use this whenever a new provider is added, provider config is refactored, hook support changes, model selection logic is touched, or settings UX must stay provider-aware and non-static.
compatibility: Requires repo write access and ability to run runtime tests.
allowed-tools:
  - Read
  - Edit
  - Bash
metadata:
  display_name: Provider Integration Hardening
  source: custom
---

# Provider Integration Hardening

Run these passes in order. Do not skip validation.

## Pass 1: Doc-Accurate Audit

1. Read latest official docs for config keys, MCP schema, model config behavior, hooks, permissions.
2. Compare docs to runtime export/import + UI surfaces.
3. Capture mismatches with exact file/field references.
4. Fix correctness and security mismatches first.

Required audit output format:
- `Mismatch`
- `Expected (docs)`
- `Current (repo)`
- `Fix`
- `Verification`

## Pass 2: Hooks + Security Depth

1. Keep Ship hook model provider-agnostic.
2. Map hook triggers to provider-native events safely.
3. Export hooks only where the provider schema supports them.
4. Preserve structured hook payloads (grouping, matcher, timeout, description).
5. Never pretend support exists: explicitly skip unsupported providers.

Security baseline:
- Pre-exec policy checks for shell/tool calls.
- Post-exec telemetry/conflict detection.
- Session-start prompt/context injection.

## Pass 3: UX + Reactive Settings

1. Add a dedicated settings section for hook/model/provider behavior when needed.
2. Keep model suggestions dynamic (provider config + env + project config).
3. Avoid hardcoded model IDs in runtime and UI.
4. Make options reactive to currently enabled providers.
5. Show unsupported capability states in plain language.

## Pass 4: Reusable Provider Extension Pack

1. Capture provider-specific mapping notes in `references/`.
2. Produce a concrete checklist for onboarding the next provider.
3. Record dogfood results for Claude/Gemini/Codex in docs.
4. Keep implementation guidance operational and copy/paste-ready.

## Examples

### Example A: Provider supports hooks

Input task:
- "Add provider X with native hooks and MCP export"

Expected approach:
1. Audit provider docs.
2. Add trigger mapping for supported lifecycle events.
3. Add export test proving hook shape in provider-native config.
4. Add/adjust settings UI so users can configure hooks.
5. Update docs + dogfood report.

### Example B: Provider does not support hooks

Input task:
- "Add provider Y; no hooks in schema"

Expected approach:
1. Keep hooks in Ship internal config.
2. Skip native hook export for provider Y.
3. Surface "no native hooks" in settings UX.
4. Add explicit assessment note in docs with source link.

## Output Contract

A complete run produces all of:
1. Runtime code changes (export/import/config mapping).
2. UI changes (reactive provider-aware settings).
3. Tests for new provider behavior and hook/model edge cases.
4. Docs update (`docs/agent-configuration.md` + dogfood notes).

## Validation Checklist

- Runtime tests pass for changed provider export paths.
- Hook payloads match provider schema exactly.
- No static model IDs in provider registry logic.
- Settings route and navigation updated when new section added.
- Codex-like unsupported capabilities are explicitly called out.

See detailed provider notes:
- `references/provider-matrix.md`
- `references/hook-mapping.md`
