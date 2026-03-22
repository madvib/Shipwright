//! File writing — emitting compiler output to the filesystem.

use anyhow::Result;
use compiler::{CompileOutput, get_provider};
use std::path::Path;

// ── File writer ──────────────────────────────────────────────────────────────

pub fn write_output(root: &Path, provider_id: &str, output: &CompileOutput) -> Result<()> {
    let desc = get_provider(provider_id).expect("provider validated earlier");

    // Context file (CLAUDE.md, GEMINI.md, AGENTS.md)
    if let (Some(content), Some(file_name)) =
        (&output.context_content, desc.context_file.file_name())
    {
        let path = root.join(file_name);
        std::fs::write(&path, content)?;
        println!("  {} {}", provider_id, file_name);
    }

    // MCP config file
    if let Some(ref mcp_path) = output.mcp_config_path {
        // Gemini: settings.json gets mcp_servers + hooks merged together
        if provider_id == "gemini" {
            let path = root.join(mcp_path);
            ensure_parent(&path)?;
            let mut patch =
                serde_json::json!({ desc.mcp_key.as_str(): &output.mcp_servers });
            if let Some(hooks) = &output.gemini_settings_patch {
                merge_json(&mut patch, hooks);
            }
            merge_json_file(&path, &patch)?;
            println!("  {} {}", provider_id, mcp_path);
        } else if provider_id == "codex" {
            // Codex: TOML patch written separately — skip JSON write
        } else {
            let path = root.join(mcp_path);
            ensure_parent(&path)?;
            let content = serde_json::to_string_pretty(
                &serde_json::json!({ desc.mcp_key.as_str(): &output.mcp_servers }),
            )?;
            std::fs::write(&path, content)?;
            println!("  {} {}", provider_id, mcp_path);
        }
    }

    // Skill files
    for (rel_path, content) in &output.skill_files {
        let path = root.join(rel_path);
        ensure_parent(&path)?;
        std::fs::write(&path, content)?;
    }

    // Rule files (Cursor .mdc)
    for (rel_path, content) in &output.rule_files {
        let path = root.join(rel_path);
        ensure_parent(&path)?;
        std::fs::write(&path, content)?;
        println!("  {} {}", provider_id, rel_path);
    }

    // Claude settings patch → .claude/settings.json
    if let Some(patch) = &output.claude_settings_patch {
        let path = root.join(".claude/settings.json");
        ensure_parent(&path)?;
        merge_json_file(&path, patch)?;
        println!("  {} .claude/settings.json", provider_id);
    }

    // Provider agent files (e.g. .claude/agents/*.md for teams)
    for (rel_path, content) in &output.agent_files {
        let path = root.join(rel_path);
        ensure_parent(&path)?;
        std::fs::write(&path, content)?;
        println!("  {} {}", provider_id, rel_path);
    }

    // Codex TOML patch → .codex/config.toml
    if let Some(toml_str) = &output.codex_config_patch {
        let path = root.join(".codex/config.toml");
        ensure_parent(&path)?;
        std::fs::write(&path, toml_str)?;
        println!("  {} .codex/config.toml", provider_id);
    }

    // Gemini policy → .gemini/policies/ship.toml
    if let Some(policy) = &output.gemini_policy_patch {
        let path = root.join(".gemini/policies/ship.toml");
        ensure_parent(&path)?;
        std::fs::write(&path, policy)?;
        println!("  {} .gemini/policies/ship.toml", provider_id);
    }

    // Cursor hooks → .cursor/hooks.json
    if let Some(hooks) = &output.cursor_hooks_patch {
        let path = root.join(".cursor/hooks.json");
        ensure_parent(&path)?;
        std::fs::write(&path, serde_json::to_string_pretty(hooks)?)?;
        println!("  {} .cursor/hooks.json", provider_id);
    }

    // Cursor CLI permissions → .cursor/cli.json
    if let Some(perms) = &output.cursor_cli_permissions {
        let path = root.join(".cursor/cli.json");
        ensure_parent(&path)?;
        std::fs::write(&path, serde_json::to_string_pretty(perms)?)?;
        println!("  {} .cursor/cli.json", provider_id);
    }

    Ok(())
}

/// Dry-run: print what would be written.
pub(crate) fn print_dry_run(provider_id: &str, output: &CompileOutput) {
    println!("[dry-run] provider: {}", provider_id);
    if let Some(f) = get_provider(provider_id).and_then(|d| d.context_file.file_name())
        && output.context_content.is_some()
    {
        println!("  would write {}", f);
    }
    if let Some(ref p) = output.mcp_config_path {
        println!("  would write {}", p);
    }
    for path in output.skill_files.keys() {
        println!("  would write {}", path);
    }
    for path in output.rule_files.keys() {
        println!("  would write {}", path);
    }
    if output.claude_settings_patch.is_some() {
        println!("  would merge .claude/settings.json");
    }
    for path in output.agent_files.keys() {
        println!("  would write {}", path);
    }
    if output.codex_config_patch.is_some() {
        println!("  would write .codex/config.toml");
    }
    if output.gemini_policy_patch.is_some() {
        println!("  would write .gemini/policies/ship.toml");
    }
    if output.cursor_hooks_patch.is_some() {
        println!("  would write .cursor/hooks.json");
    }
    if output.cursor_cli_permissions.is_some() {
        println!("  would write .cursor/cli.json");
    }
}

// ── JSON merge helpers ───────────────────────────────────────────────────────

/// Recursively merge `patch` into `base` (patch wins on scalar conflict).
pub(crate) fn merge_json(base: &mut serde_json::Value, patch: &serde_json::Value) {
    match (base, patch) {
        (serde_json::Value::Object(b), serde_json::Value::Object(p)) => {
            for (k, v) in p {
                merge_json(
                    b.entry(k.clone()).or_insert(serde_json::Value::Null),
                    v,
                );
            }
        }
        (base, patch) => *base = patch.clone(),
    }
}

/// Read an existing JSON file (or start with `{}`), merge `patch` in, write back.
pub(crate) fn merge_json_file(path: &Path, patch: &serde_json::Value) -> Result<()> {
    let mut existing: serde_json::Value = if path.exists() {
        serde_json::from_str(&std::fs::read_to_string(path)?).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };
    merge_json(&mut existing, patch);
    std::fs::write(path, serde_json::to_string_pretty(&existing)?)?;
    Ok(())
}

pub(crate) fn ensure_parent(path: &Path) -> Result<()> {
    if let Some(p) = path.parent() {
        std::fs::create_dir_all(p)?;
    }
    Ok(())
}

// ── Session scratch space ────────────────────────────────────────────────────

/// Ensure `.ship-session/` is listed in the root `.gitignore`.
/// Called once per `ship use` — idempotent.
pub(crate) fn ensure_session_gitignored(root: &Path) -> Result<()> {
    const ENTRY: &str = ".ship-session/";
    let path = root.join(".gitignore");
    let existing = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    if existing.lines().any(|l| l.trim() == ENTRY) {
        return Ok(());
    }
    let updated = if existing.is_empty() || existing.ends_with('\n') {
        format!("{}{}\n", existing, ENTRY)
    } else {
        format!("{}\n{}\n", existing, ENTRY)
    };
    std::fs::write(&path, updated)?;
    Ok(())
}
