+++
id = "bf2ac584-cd5f-4f30-8c34-6fbeb7cf87d9"
title = "AI provider profiles — Claude, Codex, Gemini CLI invocation in logic crate"
created = "2026-02-24T04:09:53.767256510Z"
updated = "2026-02-24T04:09:53.767257410Z"
tags = []
links = []
+++

## What
Add built-in provider profiles for Claude, Codex, and Gemini to the logic crate, with exact CLI invocation syntax for each. Implement child process execution so both the MCP server and Tauri UI can generate text without any API keys.

## Provider Profiles (built-in knowledge)

```rust
pub enum Provider {
    Claude,   // claude -p "prompt" [--model MODEL]
    Codex,    // codex exec "prompt"
    Gemini,   // gemini -p "prompt"
    Custom,   // arbitrary cli_path + args
}

impl Provider {
    pub fn default_binary(&self) -> &str {
        match self {
            Provider::Claude => "claude",
            Provider::Codex  => "codex",
            Provider::Gemini => "gemini",
            Provider::Custom => "",
        }
    }

    pub fn build_args(&self, prompt: &str, model: Option<&str>) -> Vec<String> {
        match self {
            Provider::Claude => {
                let mut args = vec!["-p".into(), prompt.into()];
                if let Some(m) = model { args.extend(["--model".into(), m.into()]); }
                args
            }
            Provider::Codex  => vec!["exec".into(), prompt.into()],
            Provider::Gemini => vec!["-p".into(), prompt.into()],
            Provider::Custom => vec![prompt.into()],
        }
    }
}
```

## Implementation: `logic/src/ai.rs`

```rust
pub struct AiGenerator {
    pub provider: Provider,
    pub cli_path: String,
    pub model: Option<String>,
}

impl AiGenerator {
    pub fn from_config(config: &AiConfig) -> Self { ... }

    /// Spawn CLI subprocess, capture stdout. Synchronous.
    pub fn generate(&self, prompt: &str) -> Result<String> {
        let output = std::process::Command::new(&self.cli_path)
            .args(self.provider.build_args(prompt, self.model.as_deref()))
            .output()?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    // Convenience prompts
    pub fn generate_issue_description(&self, title: &str, context: Option<&str>) -> Result<String>
    pub fn generate_adr(&self, problem: &str, constraints: Option<&str>) -> Result<String>
    pub fn brainstorm_issues(&self, topic: &str, count: u32) -> Result<String>
}
```

## Scope
- New `logic/src/ai.rs` module — no tokio dependency, pure `std::process`
- `AiConfig` already has `provider` and `cli_path` fields (added in previous commit)
- Export `AiGenerator` from `logic/src/lib.rs`
- Update MCP crate to use `AiGenerator` as fallback when sampling peer unavailable (currently returns error message — this is better)

## Acceptance
- `AiGenerator::generate("hello world")` with claude provider spawns `claude -p "hello world"` and returns stdout
- Works on macOS and Linux
- Returns meaningful error if binary not found (not a panic)