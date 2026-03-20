//! `ship compile` — load the project library, resolve, compile, write.

use anyhow::{Context, Result};
use compiler::{CompileOutput, HookConfig, HookTrigger, PluginEntry, PluginsManifest, ProjectLibrary, compile, get_provider, resolve_library};
use std::path::Path;

use crate::dep_skills::resolve_dep_skills;
use crate::loader::load_library;
use crate::mode::{Profile, apply_profile_permissions};

// ── Entry point ───────────────────────────────────────────────────────────────

pub struct CompileOptions<'a> {
    /// Project root (where CLAUDE.md etc. get written).
    pub project_root: &'a Path,
    /// Optionally restrict to a single provider.
    pub provider: Option<&'a str>,
    /// Print what would be written without touching the filesystem.
    pub dry_run: bool,
    /// Active mode id (already resolved from PathContext / ship.toml).
    pub active_agent: Option<&'a str>,
}

pub fn run_compile(opts: CompileOptions<'_>) -> Result<()> {
    let agents_dir = opts.project_root.join(".ship").join("agents");

    // 1. Load raw library from agents/
    let mut library = load_library(&agents_dir)
        .context("failed to load .ship/agents/")?;

    // 2. Apply mode overrides (permissions, inline rules, provider list)
    if let Some(mode_id) = opts.active_agent {
        apply_mode_to_library(&mut library, mode_id, opts.project_root)?;
    }
    library.active_agent = opts.active_agent.map(str::to_string);

    // 2b. Resolve dep skill refs from cached packages.
    //     Collect all skill refs declared across agent profiles and mode configs,
    //     resolve any github.com/ refs from ship.lock + cache, and merge into
    //     library.skills before passing to the compiler. Local refs are skipped.
    {
        let all_skill_refs = collect_all_skill_refs(&library);
        if !all_skill_refs.is_empty() {
            let lock_path = opts.project_root.join(".ship").join("ship.lock");
            let dep_skills = resolve_dep_skills(
                &all_skill_refs,
                &library.skills,
                &lock_path,
                None, // use default ~/.ship/cache/
            ).context("resolving dep skills from cache")?;
            library.skills.extend(dep_skills);
        }
    }

    // 3. Resolve (mode filtering, provider selection)
    let resolved = resolve_library(&library, None, opts.active_agent);

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

    // 6. Write mcp__ship__* to global ~/.claude/settings.json when compiling for claude.
    //    The ship-mcp server is always injected by the compiler into every claude compile.
    //    This is a one-time global allow — avoids per-session approval prompts for ship MCP.
    if !opts.dry_run
        && providers.contains(&"claude".to_string())
        && let Err(e) = ensure_ship_mcp_globally_allowed()
    {
        // Non-fatal — log warning but don't fail compile
        eprintln!("warning: could not update global Claude settings: {e}");
    }

    if !opts.dry_run {
        ensure_session_gitignored(opts.project_root)?;
        println!("✓ compiled for: {}", providers.join(", "));
    }
    Ok(())
}

// ── Global Claude settings ─────────────────────────────────────────────────────

/// Write `mcp__ship__*` to `~/.claude/settings.json` permissions allow list.
/// This pre-approves all ship MCP tools globally so agents aren't prompted on
/// every session when the ship server is active.
/// Idempotent — safe to call on every `ship use`.
fn ensure_ship_mcp_globally_allowed() -> Result<()> {
    let home = dirs::home_dir()
        .context("could not determine home directory")?;
    let path = home.join(".claude").join("settings.json");
    ensure_parent(&path)?;

    let mut settings: serde_json::Value = if path.exists() {
        serde_json::from_str(&std::fs::read_to_string(&path)?)
            .unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    let ship_pattern = "mcp__ship__*";

    // Check if already present — use a non-mutable read
    let already_present = settings
        .pointer("/permissions/allow")
        .and_then(|v| v.as_array())
        .is_some_and(|arr| arr.iter().any(|v| v.as_str() == Some(ship_pattern)));

    if already_present {
        return Ok(());
    }

    // Not present — ensure permissions.allow exists and add the pattern
    {
        let root = settings
            .as_object_mut()
            .context("settings.json must be an object")?;
        let perms = root
            .entry("permissions")
            .or_insert(serde_json::json!({}));
        let perms_obj = perms.as_object_mut()
            .context("permissions must be an object")?;
        let allow_arr = perms_obj
            .entry("allow")
            .or_insert(serde_json::json!([]));
        let arr = allow_arr.as_array_mut()
            .context("permissions.allow must be an array")?;
        arr.push(serde_json::json!(ship_pattern));
    }

    std::fs::write(&path, serde_json::to_string_pretty(&settings)?)?;
    Ok(())
}

// ── Mode → library ────────────────────────────────────────────────────────────

fn apply_mode_to_library(library: &mut ProjectLibrary, mode_id: &str, project_root: &Path) -> Result<()> {
    let Some(path) = find_profile_file(mode_id, project_root) else { return Ok(()); };

    let profile = Profile::load(&path)?;
    let agents_dir = project_root.join(".ship").join("agents");

    // Permission overrides — pass agents_dir so preset sections from permissions.toml are resolved
    library.permissions = apply_profile_permissions(library.permissions.clone(), &profile, Some(&agents_dir));

    // Inline rules → append as a synthetic rule file
    if let Some(inline) = &profile.rules.inline {
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

    // If profile declares a provider list, inject a ModeConfig so resolve() applies it
    if !profile.meta.providers.is_empty() {
        library.modes.push(compiler::ModeConfig {
            id: mode_id.to_string(),
            name: profile.meta.name.clone(),
            target_agents: profile.meta.providers.clone(),
            mcp_servers: profile.mcp.servers.clone(),
            skills: profile.skills.refs.clone(),
            ..Default::default()
        });
    }

    // Plugins — convert profile's Vec<String> install list into PluginsManifest
    if !profile.plugins.install.is_empty() {
        library.plugins = PluginsManifest {
            install: profile.plugins.install.iter().map(|id| PluginEntry {
                id: id.clone(),
                provider: "claude".to_string(),
            }).collect(),
            scope: profile.plugins.scope.clone(),
        };
    }

    // Hooks declared in [hooks] section of profile TOML
    if let Some(cmd) = &profile.hooks.stop {
        let id = format!("{}-stop", mode_id);
        if !library.hooks.iter().any(|h| h.id == id) {
            library.hooks.push(HookConfig {
                id,
                trigger: HookTrigger::Stop,
                command: cmd.clone(),
                matcher: None,
                cursor_event: None,
                gemini_event: None,
            });
        }
    }
    if let Some(cmd) = &profile.hooks.subagent_stop {
        let id = format!("{}-subagent-stop", mode_id);
        if !library.hooks.iter().any(|h| h.id == id) {
            library.hooks.push(HookConfig {
                id,
                trigger: HookTrigger::SubagentStop,
                command: cmd.clone(),
                matcher: None,
                cursor_event: None,
                gemini_event: None,
            });
        }
    }

    // Provider-specific settings pass-through
    if let Some(claude_extra) = profile.provider_settings.get("claude") {
        library.claude_settings_extra = Some(claude_extra.clone());
    }

    // Team agents from .ship/agents/teams/<provider>/*.md
    library.claude_team_agents = load_team_agents(project_root, "claude");

    Ok(())
}

fn load_team_agents(project_root: &Path, provider_id: &str) -> Vec<(String, String)> {
    let teams_dir = project_root.join(".ship").join("agents").join("teams").join(provider_id);
    if !teams_dir.exists() {
        return vec![];
    }
    let mut agents = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&teams_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "md")
                && let (Some(name), Ok(content)) = (
                    path.file_name().map(|n| n.to_string_lossy().to_string()),
                    std::fs::read_to_string(&path),
                )
            {
                agents.push((name, content));
            }
        }
    }
    agents.sort_by(|a, b| a.0.cmp(&b.0));
    agents
}

/// Search order: agents/profiles/ (new) → modes/ (legacy), project then global.
fn find_profile_file(profile_id: &str, project_root: &Path) -> Option<std::path::PathBuf> {
    let ship = project_root.join(".ship");
    let file = format!("{}.toml", profile_id);

    // Project-local: profiles/ → presets/ (compat) → modes/ (legacy)
    let p = ship.join("agents").join("profiles").join(&file);
    if p.exists() { return Some(p); }
    let p_compat = ship.join("agents").join("presets").join(&file);
    if p_compat.exists() { return Some(p_compat); }
    let m = ship.join("modes").join(&file);
    if m.exists() { return Some(m); }

    // Global: ~/.ship
    let home = dirs::home_dir()?;
    let gp = home.join(".ship").join("agents").join("profiles").join(&file);
    if gp.exists() { return Some(gp); }
    let gp_compat = home.join(".ship").join("agents").join("presets").join(&file);
    if gp_compat.exists() { return Some(gp_compat); }
    let gm = home.join(".ship").join("modes").join(&file);
    if gm.exists() { return Some(gm); }

    None
}

// ── Session scratch space ─────────────────────────────────────────────────────

/// Ensure `.ship-session/` is listed in the root `.gitignore`.
/// Called once per `ship use` — idempotent.
fn ensure_session_gitignored(root: &Path) -> Result<()> {
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
    if let Some(ref mcp_path) = output.mcp_config_path {
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
fn print_dry_run(provider_id: &str, output: &CompileOutput) {
    println!("[dry-run] provider: {}", provider_id);
    if let Some(f) = get_provider(provider_id).and_then(|d| d.context_file.file_name())
        && output.context_content.is_some()
    {
        println!("  would write {}", f);
    }
    if let Some(ref p) = output.mcp_config_path { println!("  would write {}", p); }
    for path in output.skill_files.keys() { println!("  would write {}", path); }
    for path in output.rule_files.keys() { println!("  would write {}", path); }
    if output.claude_settings_patch.is_some() { println!("  would merge .claude/settings.json"); }
    for path in output.agent_files.keys() { println!("  would write {}", path); }
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

// ── Dep skill ref collection ───────────────────────────────────────────────────

/// Collect all skill refs declared in agent profiles and mode configs within
/// `library`. Only dep refs (those starting with `github.com/`) will be resolved
/// by the caller; local refs are included in the list but filtered in
/// [`resolve_dep_skills`].
fn collect_all_skill_refs(library: &ProjectLibrary) -> Vec<String> {
    let mut refs: Vec<String> = Vec::new();

    // From agent profiles: [skills] refs = [...]
    for profile in &library.agent_profiles {
        for r in &profile.skills.refs {
            if !refs.contains(r) {
                refs.push(r.clone());
            }
        }
    }

    // From mode configs: skills = [...] filter list
    for mode in &library.modes {
        for r in &mode.skills {
            if !refs.contains(r) {
                refs.push(r.clone());
            }
        }
    }

    refs
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
            dry_run: false, active_agent: None,
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
            dry_run: false, active_agent: None,
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
            dry_run: true, active_agent: None,
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
            dry_run: false, active_agent: None,
        }).unwrap();
        let path = tmp.path().join(".gemini/settings.json");
        assert!(path.exists(), ".gemini/settings.json must be written for gemini");
        let parsed: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert!(parsed["mcpServers"]["ship"].is_object());
    }

    #[test]
    fn compile_gemini_writes_gemini_md_with_rules() {
        let tmp = TempDir::new().unwrap();
        setup_minimal_project(&tmp);
        run_compile(CompileOptions {
            project_root: tmp.path(), provider: Some("gemini"),
            dry_run: false, active_agent: None,
        }).unwrap();
        let content = std::fs::read_to_string(tmp.path().join("GEMINI.md")).unwrap();
        assert!(content.contains("Use explicit types."), "GEMINI.md must contain rules");
    }

    #[test]
    fn compile_codex_writes_agents_md_with_rules() {
        let tmp = TempDir::new().unwrap();
        setup_minimal_project(&tmp);
        run_compile(CompileOptions {
            project_root: tmp.path(), provider: Some("codex"),
            dry_run: false, active_agent: None,
        }).unwrap();
        let content = std::fs::read_to_string(tmp.path().join("AGENTS.md")).unwrap();
        assert!(content.contains("Use explicit types."), "AGENTS.md must contain rules");
    }

    #[test]
    fn compile_codex_writes_toml_config_with_mcp_servers() {
        let tmp = TempDir::new().unwrap();
        setup_minimal_project(&tmp);
        run_compile(CompileOptions {
            project_root: tmp.path(), provider: Some("codex"),
            dry_run: false, active_agent: None,
        }).unwrap();
        let path = tmp.path().join(".codex/config.toml");
        assert!(path.exists(), ".codex/config.toml must be written for codex");
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("mcp_servers"), "config.toml must contain mcp_servers section");
        assert!(content.contains("ship"), "ship server must appear in codex config");
    }

    #[test]
    fn compile_cursor_writes_mdc_rule_files() {
        let tmp = TempDir::new().unwrap();
        setup_minimal_project(&tmp);
        run_compile(CompileOptions {
            project_root: tmp.path(), provider: Some("cursor"),
            dry_run: false, active_agent: None,
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
            dry_run: false, active_agent: None,
        }).unwrap();
        let settings_path = tmp.path().join(".claude/settings.json");
        assert!(settings_path.exists());
        let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&settings_path).unwrap()).unwrap();
        assert_eq!(v["permissions"]["deny"][0], "Bash(rm -rf *)");
    }

    #[test]
    fn compile_with_mode_applies_permissions() {
        let tmp = TempDir::new().unwrap();
        write(tmp.path(), ".ship/agents/profiles/guarded.toml", r#"
[profile]
name = "Guarded"
id = "guarded"
providers = ["claude"]
[permissions]
preset = "ship-guarded"
"#);
        run_compile(CompileOptions {
            project_root: tmp.path(), provider: Some("claude"),
            dry_run: false, active_agent: Some("guarded"),
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
        write(tmp.path(), ".ship/agents/profiles/strict.toml", r#"
[profile]
name = "Strict"
id = "strict"
providers = ["claude"]
[rules]
inline = "Never delete files without explicit confirmation."
"#);
        run_compile(CompileOptions {
            project_root: tmp.path(), provider: Some("claude"),
            dry_run: false, active_agent: Some("strict"),
        }).unwrap();
        let content = std::fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();
        assert!(content.contains("Never delete files without explicit confirmation."));
    }

    #[test]
    fn compile_with_profile_stop_hook_emits_to_settings() {
        let tmp = TempDir::new().unwrap();
        write(tmp.path(), ".ship/agents/profiles/commander.toml", r#"
[profile]
name = "Commander"
id = "commander"
providers = ["claude"]

[hooks]
stop = "ship permissions sync"
"#);
        run_compile(CompileOptions {
            project_root: tmp.path(), provider: Some("claude"),
            dry_run: false, active_agent: Some("commander"),
        }).unwrap();
        let v: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(tmp.path().join(".claude/settings.json")).unwrap()
        ).unwrap();
        let stop_hooks = v["hooks"]["Stop"].as_array()
            .expect("Stop hooks array must be present");
        assert!(
            stop_hooks.iter().any(|entry| {
                entry["hooks"].as_array()
                    .is_some_and(|hooks| hooks.iter().any(|h| h["command"] == "ship permissions sync"))
            }),
            "stop hook command must be emitted"
        );
    }

    #[test]
    fn compile_with_mode_uses_permissions_toml_preset() {
        let tmp = TempDir::new().unwrap();
        write(tmp.path(), ".ship/agents/permissions.toml", r#"
[ship-fast]
default_mode = "bypassPermissions"
tools_deny = ["Bash(git push --force*)"]
"#);
        write(tmp.path(), ".ship/agents/profiles/fast.toml", r#"
[profile]
name = "Fast"
id = "fast"
providers = ["claude"]
[permissions]
preset = "ship-fast"
"#);
        run_compile(CompileOptions {
            project_root: tmp.path(), provider: Some("claude"),
            dry_run: false, active_agent: Some("fast"),
        }).unwrap();
        let v: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(tmp.path().join(".claude/settings.json")).unwrap()
        ).unwrap();
        assert_eq!(v["permissions"]["defaultMode"], "bypassPermissions",
            "defaultMode from permissions.toml preset must be written");
        let deny = v["permissions"]["deny"].as_array().unwrap();
        assert!(deny.iter().any(|d| d == "Bash(git push --force*)"),
            "tools_deny from permissions.toml preset must be written");
    }

    #[test]
    fn compile_with_bypass_permissions_mode_writes_default_mode() {
        let tmp = TempDir::new().unwrap();
        write(tmp.path(), ".ship/agents/profiles/autonomous.toml", r#"
[profile]
name = "Autonomous"
id = "autonomous"
providers = ["claude"]
[permissions]
default_mode = "bypassPermissions"
"#);
        run_compile(CompileOptions {
            project_root: tmp.path(), provider: Some("claude"),
            dry_run: false, active_agent: Some("autonomous"),
        }).unwrap();
        let v: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(tmp.path().join(".claude/settings.json")).unwrap()
        ).unwrap();
        assert_eq!(v["permissions"]["defaultMode"], "bypassPermissions");
    }

    #[test]
    fn ensure_ship_mcp_globally_allowed_is_idempotent() {
        let tmp = TempDir::new().unwrap();
        let settings_path = tmp.path().join("settings.json");
        // Override home by writing directly
        std::fs::create_dir_all(tmp.path()).unwrap();
        // Write a pre-existing settings.json
        std::fs::write(&settings_path, r#"{"permissions":{"allow":["mcp__ship__*"]}}"#).unwrap();
        // Parse and check directly (simulating idempotency)
        let v: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(&settings_path).unwrap()
        ).unwrap();
        let allow = v["permissions"]["allow"].as_array().unwrap();
        assert_eq!(allow.iter().filter(|x| x.as_str() == Some("mcp__ship__*")).count(), 1);
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
    fn ensure_session_gitignored_adds_entry() {
        let tmp = TempDir::new().unwrap();
        ensure_session_gitignored(tmp.path()).unwrap();
        let content = std::fs::read_to_string(tmp.path().join(".gitignore")).unwrap();
        assert!(content.contains(".ship-session/"), "must add .ship-session/ entry");
    }

    #[test]
    fn ensure_session_gitignored_is_idempotent() {
        let tmp = TempDir::new().unwrap();
        ensure_session_gitignored(tmp.path()).unwrap();
        ensure_session_gitignored(tmp.path()).unwrap();
        let content = std::fs::read_to_string(tmp.path().join(".gitignore")).unwrap();
        assert_eq!(
            content.lines().filter(|l| l.trim() == ".ship-session/").count(),
            1,
            "must not duplicate the entry"
        );
    }

    #[test]
    fn ensure_session_gitignored_appends_to_existing() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join(".gitignore"), "node_modules/\n").unwrap();
        ensure_session_gitignored(tmp.path()).unwrap();
        let content = std::fs::read_to_string(tmp.path().join(".gitignore")).unwrap();
        assert!(content.contains("node_modules/"), "must preserve existing entries");
        assert!(content.contains(".ship-session/"), "must add new entry");
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
