//! `ship compile` — load the project library, resolve, compile, write.

use anyhow::{Context, Result};
use compiler::{CompileOutput, ProjectLibrary, compile, get_provider, resolve_library};
use std::path::Path;

use crate::loader::load_library;
use crate::mode::{Mode, apply_mode_permissions};

// ── Entry point ───────────────────────────────────────────────────────────────

pub struct CompileOptions<'a> {
    /// Project root (where CLAUDE.md etc. get written).
    pub project_root: &'a Path,
    /// Optionally restrict to a single provider.
    pub provider: Option<&'a str>,
    /// Print what would be written without touching the filesystem.
    pub dry_run: bool,
    /// Active mode id (already resolved from PathContext / ship.toml).
    pub active_mode: Option<&'a str>,
}

pub fn run_compile(opts: CompileOptions<'_>) -> Result<()> {
    let agents_dir = opts.project_root.join(".ship").join("agents");

    // 1. Load raw library from agents/
    let mut library = load_library(&agents_dir)
        .context("failed to load .ship/agents/")?;

    // 2. Apply mode overrides (permissions, inline rules, provider list)
    if let Some(mode_id) = opts.active_mode {
        apply_mode_to_library(&mut library, mode_id, opts.project_root)?;
    }
    library.active_mode = opts.active_mode.map(str::to_string);

    // 3. Resolve (mode filtering, provider selection)
    let resolved = resolve_library(&library, None, opts.active_mode);

    // 4. Determine providers to compile for
    let providers: Vec<String> = match opts.provider {
        Some(p) => vec![p.to_string()],
        None    => resolved.providers.clone(),
    };

    if providers.is_empty() {
        println!("No providers configured. Add providers to .ship/ship.toml or mode file.");
        return Ok(());
    }

    // 5. Compile + write
    for provider_id in &providers {
        let Some(output) = compile(&resolved, provider_id) else {
            eprintln!("warning: unknown provider '{}', skipping", provider_id);
            continue;
        };
        if opts.dry_run {
            print_dry_run(provider_id, &output);
        } else {
            write_output(opts.project_root, provider_id, &output)
                .with_context(|| format!("failed to write {} output", provider_id))?;
        }
    }

    if !opts.dry_run {
        println!("✓ compiled for: {}", providers.join(", "));
    }
    Ok(())
}

// ── Mode → library ────────────────────────────────────────────────────────────

fn apply_mode_to_library(library: &mut ProjectLibrary, mode_id: &str, project_root: &Path) -> Result<()> {
    let mode_file = find_mode_file(mode_id, project_root);
    let Some(path) = mode_file else { return Ok(()); };

    let mode = Mode::load(&path)?;

    // Provider list from mode (only if non-empty)
    // (The resolver picks this up via ProjectLibrary.modes[] — for now we skip ModeConfig
    // injection and just apply permissions + inline rules directly)

    // Permission overrides
    library.permissions = apply_mode_permissions(library.permissions.clone(), &mode);

    // Inline rules → append as a synthetic rule file
    if let Some(inline) = &mode.rules.inline {
        let trimmed = inline.trim();
        if !trimmed.is_empty() {
            library.rules.push(compiler::Rule {
                file_name: format!("{}.md", mode_id),
                content: trimmed.to_string(),
                always_apply: true,
                globs: vec![],
                description: None,
            });
        }
    }

    // If mode declares a provider list, inject a ModeConfig so resolve() applies it
    if !mode.meta.providers.is_empty() {
        library.modes.push(compiler::ModeConfig {
            id: mode_id.to_string(),
            name: mode.meta.name.clone(),
            target_agents: mode.meta.providers.clone(),
            mcp_servers: mode.mcp.servers.clone(),
            skills: mode.skills.refs.clone(),
            ..Default::default()
        });
    }

    Ok(())
}

fn find_mode_file(mode_id: &str, project_root: &Path) -> Option<std::path::PathBuf> {
    let p = project_root.join(".ship").join("modes").join(format!("{}.toml", mode_id));
    if p.exists() { return Some(p); }
    let g = dirs::home_dir()?.join(".ship").join("modes").join(format!("{}.toml", mode_id));
    if g.exists() { return Some(g); }
    None
}

// ── File writer ───────────────────────────────────────────────────────────────

pub fn write_output(root: &Path, provider_id: &str, output: &CompileOutput) -> Result<()> {
    let desc = get_provider(provider_id).expect("provider validated earlier");

    // Context file (CLAUDE.md, GEMINI.md, AGENTS.md)
    if let (Some(content), Some(file_name)) = (&output.context_content, desc.context_file.file_name()) {
        let path = root.join(file_name);
        std::fs::write(&path, content)?;
        println!("  {} {}", provider_id, file_name);
    }

    // MCP config file
    if let Some(mcp_path) = output.mcp_config_path {
        // Gemini: settings.json gets mcp_servers + hooks merged together
        if provider_id == "gemini" {
            let path = root.join(mcp_path);
            ensure_parent(&path)?;
            let mut patch = serde_json::json!({ desc.mcp_key.as_str(): &output.mcp_servers });
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
                &serde_json::json!({ desc.mcp_key.as_str(): &output.mcp_servers })
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
fn print_dry_run(provider_id: &str, output: &CompileOutput) {
    println!("[dry-run] provider: {}", provider_id);
    if let Some(f) = get_provider(provider_id).and_then(|d| d.context_file.file_name()) {
        if output.context_content.is_some() { println!("  would write {}", f); }
    }
    if let Some(p) = output.mcp_config_path { println!("  would write {}", p); }
    for path in output.skill_files.keys() { println!("  would write {}", path); }
    for path in output.rule_files.keys() { println!("  would write {}", path); }
    if output.claude_settings_patch.is_some() { println!("  would merge .claude/settings.json"); }
    if output.codex_config_patch.is_some() { println!("  would write .codex/config.toml"); }
    if output.gemini_policy_patch.is_some() { println!("  would write .gemini/policies/ship.toml"); }
    if output.cursor_hooks_patch.is_some() { println!("  would write .cursor/hooks.json"); }
    if output.cursor_cli_permissions.is_some() { println!("  would write .cursor/cli.json"); }
}

// ── JSON merge helpers ────────────────────────────────────────────────────────

/// Recursively merge `patch` into `base` (patch wins on scalar conflict).
fn merge_json(base: &mut serde_json::Value, patch: &serde_json::Value) {
    match (base, patch) {
        (serde_json::Value::Object(b), serde_json::Value::Object(p)) => {
            for (k, v) in p {
                merge_json(b.entry(k.clone()).or_insert(serde_json::Value::Null), v);
            }
        }
        (base, patch) => *base = patch.clone(),
    }
}

/// Read an existing JSON file (or start with `{}`), merge `patch` in, write back.
fn merge_json_file(path: &Path, patch: &serde_json::Value) -> Result<()> {
    let mut existing: serde_json::Value = if path.exists() {
        serde_json::from_str(&std::fs::read_to_string(path)?).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };
    merge_json(&mut existing, patch);
    std::fs::write(path, serde_json::to_string_pretty(&existing)?)?;
    Ok(())
}

fn ensure_parent(path: &Path) -> Result<()> {
    if let Some(p) = path.parent() { std::fs::create_dir_all(p)?; }
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write(dir: &Path, rel: &str, content: &str) {
        let p = dir.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, content).unwrap();
    }

    fn setup_minimal_project(tmp: &TempDir) {
        write(tmp.path(), ".ship/agents/rules/style.md", "Use explicit types.");
        write(tmp.path(), ".ship/agents/mcp.toml", r#"
[[servers]]
id = "github"
command = "npx"
args = ["-y", "@mcp/github"]
"#);
    }

    #[test]
    fn compile_writes_claude_md() {
        let tmp = TempDir::new().unwrap();
        setup_minimal_project(&tmp);
        run_compile(CompileOptions {
            project_root: tmp.path(), provider: Some("claude"),
            dry_run: false, active_mode: None,
        }).unwrap();
        let content = std::fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();
        assert!(content.contains("Use explicit types."));
    }

    #[test]
    fn compile_writes_mcp_json() {
        let tmp = TempDir::new().unwrap();
        setup_minimal_project(&tmp);
        run_compile(CompileOptions {
            project_root: tmp.path(), provider: Some("claude"),
            dry_run: false, active_mode: None,
        }).unwrap();
        let content = std::fs::read_to_string(tmp.path().join(".mcp.json")).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(parsed["mcpServers"]["ship"].is_object(), "ship server must be in .mcp.json");
        assert!(parsed["mcpServers"]["github"].is_object());
    }

    #[test]
    fn compile_dry_run_writes_nothing() {
        let tmp = TempDir::new().unwrap();
        setup_minimal_project(&tmp);
        run_compile(CompileOptions {
            project_root: tmp.path(), provider: Some("claude"),
            dry_run: true, active_mode: None,
        }).unwrap();
        assert!(!tmp.path().join("CLAUDE.md").exists(), "dry-run must not write files");
        assert!(!tmp.path().join(".mcp.json").exists());
    }

    #[test]
    fn compile_gemini_writes_settings_json_with_mcp_and_context() {
        let tmp = TempDir::new().unwrap();
        setup_minimal_project(&tmp);
        run_compile(CompileOptions {
            project_root: tmp.path(), provider: Some("gemini"),
            dry_run: false, active_mode: None,
        }).unwrap();
        let path = tmp.path().join(".gemini/settings.json");
        assert!(path.exists(), ".gemini/settings.json must be written for gemini");
        let parsed: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert!(parsed["mcpServers"]["ship"].is_object());
    }

    #[test]
    fn compile_cursor_writes_mdc_rule_files() {
        let tmp = TempDir::new().unwrap();
        setup_minimal_project(&tmp);
        run_compile(CompileOptions {
            project_root: tmp.path(), provider: Some("cursor"),
            dry_run: false, active_mode: None,
        }).unwrap();
        let mdc = tmp.path().join(".cursor/rules/style.mdc");
        assert!(mdc.exists(), ".cursor/rules/style.mdc must be written");
        let content = std::fs::read_to_string(&mdc).unwrap();
        assert!(content.contains("Use explicit types."));
        assert!(content.starts_with("---\n"), "must have frontmatter");
    }

    #[test]
    fn compile_with_deny_writes_claude_settings() {
        let tmp = TempDir::new().unwrap();
        write(tmp.path(), ".ship/agents/permissions.toml", r#"
[tools]
deny = ["Bash(rm -rf *)"]
"#);
        run_compile(CompileOptions {
            project_root: tmp.path(), provider: Some("claude"),
            dry_run: false, active_mode: None,
        }).unwrap();
        let settings_path = tmp.path().join(".claude/settings.json");
        assert!(settings_path.exists());
        let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&settings_path).unwrap()).unwrap();
        assert_eq!(v["permissions"]["deny"][0], "Bash(rm -rf *)");
    }

    #[test]
    fn compile_with_mode_applies_permissions() {
        let tmp = TempDir::new().unwrap();
        write(tmp.path(), ".ship/modes/guarded.toml", r#"
[mode]
name = "Guarded"
id = "guarded"
providers = ["claude"]
[permissions]
preset = "ship-guarded"
"#);
        run_compile(CompileOptions {
            project_root: tmp.path(), provider: Some("claude"),
            dry_run: false, active_mode: Some("guarded"),
        }).unwrap();
        let v: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(tmp.path().join(".claude/settings.json")).unwrap()
        ).unwrap();
        let deny = v["permissions"]["deny"].as_array().unwrap();
        assert!(deny.iter().any(|d| d == "mcp__*__delete*"));
    }

    #[test]
    fn compile_with_mode_inline_rules_adds_to_context() {
        let tmp = TempDir::new().unwrap();
        write(tmp.path(), ".ship/modes/strict.toml", r#"
[mode]
name = "Strict"
id = "strict"
providers = ["claude"]
[rules]
inline = "Never delete files without explicit confirmation."
"#);
        run_compile(CompileOptions {
            project_root: tmp.path(), provider: Some("claude"),
            dry_run: false, active_mode: Some("strict"),
        }).unwrap();
        let content = std::fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();
        assert!(content.contains("Never delete files without explicit confirmation."));
    }

    #[test]
    fn merge_json_deep_merge() {
        let mut base = serde_json::json!({ "a": { "x": 1 }, "b": 2 });
        let patch = serde_json::json!({ "a": { "y": 2 }, "c": 3 });
        merge_json(&mut base, &patch);
        assert_eq!(base["a"]["x"], 1, "existing key must survive");
        assert_eq!(base["a"]["y"], 2, "patch key must be added");
        assert_eq!(base["c"], 3);
    }

    #[test]
    fn merge_json_file_creates_if_missing() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("settings.json");
        merge_json_file(&path, &serde_json::json!({ "model": "claude-opus-4-6" })).unwrap();
        let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(v["model"], "claude-opus-4-6");
    }
}
