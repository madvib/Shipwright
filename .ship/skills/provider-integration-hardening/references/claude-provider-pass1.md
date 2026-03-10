# Claude Provider Pass 1 (Detection / Configuration / Import-Export)

Validated on: 2026-03-09  
Sources:
- https://docs.anthropic.com/en/docs/claude-code/settings
- https://docs.anthropic.com/en/docs/claude-code/hooks-guide

## 1) Detection

### Provider docs baseline
- Claude has multiple settings scopes, including shared and local project settings plus user settings.
- Hook events include core lifecycle events and additional events like `PreCompact`, `SessionEnd`, `Notification`, and others in current docs.

### Ship current behavior
- Provider registry is static (`claude`, `gemini`, `codex`): `core/runtime/src/agents/export/sections/provider_registry.rs`
- Binary detection uses `which`/PATH scan.
- Version detection runs `<binary> --version`.
- Model discovery is inferred from env vars, provider config files, and Ship `ai.model` preference.
- Autodetect enablement runs during project initialization flows.

### Gaps / notes
- Detection is binary/version/model aware, but not capability-aware (no doc-versioned provider capability snapshot).
- Provider listing currently requires an active project context.

## 2) Configuration

### Claude doc baseline (high signal)
- Claude supports layered settings scope.
- Claude MCP settings involve both project and user-level files.
- Hooks and permissions live in Claude settings, not in `.mcp.json`.

### Ship current mapping
- Provider descriptor:
  - project MCP config: `.mcp.json`
  - global MCP config: `.claude.json`
- Hook + tool permission export target:
  - `~/.claude/settings.json`
- Prompt output:
  - `CLAUDE.md` (managed separately)
- Skills output:
  - `.claude/skills/<id>/SKILL.md`

### Gaps / notes
- Claude has multiple settings files in docs; Ship currently maps to explicit files above.
- This is workable, but we should keep a compatibility matrix with explicit scope behavior and precedence.

## 3) Import / Export

### Export (Ship -> Claude)
- MCP: writes project `.mcp.json` with Ship-managed entries merged non-destructively.
- Hooks + tool permissions: writes `~/.claude/settings.json` when non-default overrides exist.
- Skills: writes provider-native skills directory.

### Import (Claude -> Ship)
- MCP import currently reads provider config paths and imports non-managed servers.
- Permissions import reads `~/.claude/settings.json` `permissions.allow/deny`.
- Import is non-destructive and skips Ship-managed servers.

### Important implementation detail
- Import path resolution now supports both existing project and global files (project first, then global).

## 4) Hooks and Default Tool Surface

### Claude hooks in docs
- Docs include a broad event surface beyond the minimal lifecycle subset.
- Matcher/output contracts are provider-specific and must be mapped intentionally.

### Ship status
- Ship hook trigger enum is provider-agnostic.
- Claude export maps supported triggers to Claude hook event keys.
- Unsupported triggers are intentionally dropped for Claude export.

### Default tools (docs snapshot)
- Current docs list core tools including shell/file/search/edit/web/task/planning tools.
- Ship should not hardcode this list as authoritative at runtime; use it for diagnostics UX and policy presets.

## 5) Recommended next changes (Claude-first)

1. Build a `ProviderDiagnosticsReport` for Claude with:
- detected binary/version/path
- settings files found + parse status
- export target writeability checks
- hooks event support matrix
- import source paths and precedence

2. Keep raw hooks as advanced/internal UI.
- Primary UI should expose policy toggles that compile to hooks/settings.

3. Keep a doc-anchored compatibility record.
- Update this file whenever Claude docs or Ship mapping changes.

