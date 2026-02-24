+++
id = "fdef05b2-9d0e-4882-988b-12e719dff2e8"
title = "AI pass-through Tauri commands — generate via CLI subprocess"
created = "2026-02-24T04:10:46.029637934Z"
updated = "2026-02-24T04:10:46.029638734Z"
tags = []
links = []
+++

## What
Wire up the `AiGenerator` from the logic crate into three Tauri commands, making AI generation available in the UI without API keys. Uses the configured provider CLI (claude, codex, gemini) as a child process.

## Commands

```rust
#[tauri::command]
#[specta::specta]
fn generate_issue_description_cmd(
    title: String,
    context: Option<String>,
    state: State<AppState>,
) -> Result<String, String>

#[tauri::command]
#[specta::specta]
fn generate_adr_cmd(
    problem: String,
    constraints: Option<String>,
    state: State<AppState>,
) -> Result<String, String>

#[tauri::command]
#[specta::specta]
fn brainstorm_issues_cmd(
    topic: String,
    count: Option<u32>,
    state: State<AppState>,
) -> Result<Vec<String>, String>
```

Each command reads `AiConfig` from the active project config, builds an `AiGenerator`, and calls the appropriate method. These are synchronous subprocess calls — Tauri runs them on a blocking thread via `spawn_blocking`.

## UI Integration

- **NewIssueModal**: Add "Generate" button next to description field — calls `generate_issue_description_cmd` with title as input, populates description textarea
- **NewAdrModal**: "Generate from problem" — calls `generate_adr_cmd`
- Loading state (spinner) while subprocess runs
- Error toast if provider binary not found or returns non-zero exit

## Error Handling

```rust
pub enum AiError {
    BinaryNotFound(String),     // "claude not found — install Claude Code or set cli_path in settings"
    NonZeroExit(i32, String),   // exit code + stderr
    EmptyResponse,
}
```

## Acceptance
- Click "Generate" in NewIssueModal with a title → populated description within ~5s
- Works with any of the three providers if installed
- Graceful error if no provider configured