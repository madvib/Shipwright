use anyhow::{Context, Result, anyhow};
use runtime::{
    Rule, Skill,
    agents::{config::resolve_agent_config_with_mode_override, export as agent_export},
    get_effective_config, sync_workspace,
};
use ship_module_project::{Feature, FeatureEntry, Spec, SpecEntry, list_features, list_specs};
use std::collections::BTreeSet;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

const POST_CHECKOUT_HOOK_CONTENT: &str = "#!/usr/bin/env sh\nship git post-checkout \"$@\"\n";

const PRE_COMMIT_HOOK_CONTENT: &str = "\
#!/usr/bin/env sh
# ship pre-commit: block staging of generated agent config files.
# These are written by `ship git sync` / post-checkout and must never be committed.
BLOCKED=\"CLAUDE.md GEMINI.md AGENTS.md .mcp.json\"
for f in $BLOCKED; do
    if git diff --cached --name-only | grep -qx \"$f\"; then
        echo \"[ship] ERROR: '$f' is a generated file managed by Ship and must not be committed.\"
        echo \"[ship]        Add it to .gitignore and unstage it: git restore --staged $f\"
        exit 1
    fi
done
# Also block generated provider directories.
for dir in .claude .gemini .codex .agents; do
    if git diff --cached --name-only | grep -q \"^${dir}/\"; then
        echo \"[ship] ERROR: '$dir/' contains generated agent config managed by Ship.\"
        echo \"[ship]        Add '$dir/' to .gitignore and unstage: git restore --staged $dir/\"
        exit 1
    fi
done
exit 0
";

/// Generated file paths that must be in the root `.gitignore`.
pub const GENERATED_GITIGNORE_ENTRIES: &[&str] = &[
    "CLAUDE.md",
    "GEMINI.md",
    "AGENTS.md",
    ".mcp.json",
    ".claude/",
    ".gemini/",
    ".codex/",
    ".agents/",
];

pub fn install_hooks(git_dir: &Path) -> Result<()> {
    if !git_dir.exists() {
        return Ok(());
    }

    let hooks_dir = git_dir.join("hooks");
    fs::create_dir_all(&hooks_dir)
        .with_context(|| format!("Failed to create hooks directory: {}", hooks_dir.display()))?;

    install_hook(&hooks_dir, "post-checkout", POST_CHECKOUT_HOOK_CONTENT)?;
    install_hook(&hooks_dir, "pre-commit", PRE_COMMIT_HOOK_CONTENT)?;

    Ok(())
}

fn install_hook(hooks_dir: &Path, name: &str, content: &str) -> Result<()> {
    let path = hooks_dir.join(name);
    let should_write = fs::read_to_string(&path)
        .map(|existing| existing != content)
        .unwrap_or(true);

    if should_write {
        fs::write(&path, content)
            .with_context(|| format!("Failed to write git {} hook: {}", name, path.display()))?;
    }

    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&path)
            .with_context(|| format!("Failed to stat hook: {}", path.display()))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).with_context(|| {
            format!(
                "Failed to set executable permissions on hook: {}",
                path.display()
            )
        })?;
    }

    Ok(())
}

/// Append Ship's generated-file entries to the project root `.gitignore`.
/// Idempotent — skips entries already present.
pub fn write_root_gitignore(project_root: &Path) -> Result<()> {
    let gitignore_path = project_root.join(".gitignore");
    let existing = fs::read_to_string(&gitignore_path).unwrap_or_default();

    let mut additions = Vec::new();
    for entry in GENERATED_GITIGNORE_ENTRIES {
        // Match whole lines to avoid partial matches
        let already_present = existing.lines().any(|l| l.trim() == *entry);
        if !already_present {
            additions.push(*entry);
        }
    }

    if additions.is_empty() {
        return Ok(());
    }

    let mut content = existing;
    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str("\n# Ship — generated agent config (never commit these)\n");
    for entry in &additions {
        content.push_str(entry);
        content.push('\n');
    }

    fs::write(&gitignore_path, content)
        .with_context(|| format!("Failed to write {}", gitignore_path.display()))?;
    Ok(())
}

pub fn find_feature_for_branch(ship_dir: &Path, branch: &str) -> Result<Option<FeatureEntry>> {
    if branch.trim().is_empty() {
        return Ok(None);
    }

    let features = list_features(ship_dir)?;
    for feat in features {
        if feat.feature.metadata.branch.as_deref() == Some(branch) {
            return Ok(Some(feat));
        }
    }

    Ok(None)
}

/// Which linked entity is associated with the checked-out branch.
pub enum BranchLinkedEntity {
    Feature(FeatureEntry),
    Spec(SpecEntry),
}

fn persist_branch_link(ship_dir: &Path, branch: &str, linked: &BranchLinkedEntity) -> Result<()> {
    match linked {
        BranchLinkedEntity::Feature(entry) => {
            runtime::set_branch_link(ship_dir, branch, "feature", &entry.id)?;
        }
        BranchLinkedEntity::Spec(entry) => {
            let content = fs::read_to_string(&entry.path)?;
            let spec = Spec::from_markdown(&content)?;
            runtime::set_branch_link(ship_dir, branch, "spec", &spec.metadata.id)?;
        }
    }
    Ok(())
}

fn find_spec_for_branch(ship_dir: &Path, branch: &str) -> Result<Option<SpecEntry>> {
    let specs = list_specs(ship_dir)?;
    for entry in specs {
        if entry.spec.metadata.branch.as_deref() == Some(branch) {
            return Ok(Some(entry));
        }
    }
    Ok(None)
}

fn find_feature_by_uuid(ship_dir: &Path, uuid: &str) -> Option<FeatureEntry> {
    if let Ok(features) = list_features(ship_dir) {
        for feat in features {
            if feat.feature.metadata.id == uuid {
                return Some(feat);
            }
        }
    }
    None
}

fn find_spec_by_uuid(ship_dir: &Path, uuid: &str) -> Option<SpecEntry> {
    list_specs(ship_dir)
        .ok()?
        .into_iter()
        .find(|entry| entry.id == uuid)
}

/// Find which linked entity (feature or spec) is associated with the given branch.
/// Checks the DB index first (O(1)), then falls back to a frontmatter file scan.
pub fn find_linked_entity_for_branch(
    ship_dir: &Path,
    branch: &str,
) -> Result<Option<BranchLinkedEntity>> {
    if branch.trim().is_empty() {
        return Ok(None);
    }

    // Fast path: DB index stores linked entity ids.
    if let Ok(Some((link_type, link_id))) = runtime::state_db::get_branch_link(ship_dir, branch) {
        match link_type.as_str() {
            "feature" => {
                if let Some(path) = find_feature_by_uuid(ship_dir, &link_id) {
                    return Ok(Some(BranchLinkedEntity::Feature(path)));
                }
            }
            "spec" => {
                if let Some(path) = find_spec_by_uuid(ship_dir, &link_id) {
                    return Ok(Some(BranchLinkedEntity::Spec(path)));
                }
            }
            _ => {}
        }
    }

    // Fallback: scan frontmatter of all features then specs
    if let Some(path) = find_feature_for_branch(ship_dir, branch)? {
        return Ok(Some(BranchLinkedEntity::Feature(path)));
    }
    if let Some(path) = find_spec_for_branch(ship_dir, branch)? {
        return Ok(Some(BranchLinkedEntity::Spec(path)));
    }
    Ok(None)
}

/// Legacy alias retained for compatibility.
pub fn find_document_for_branch(
    ship_dir: &Path,
    branch: &str,
) -> Result<Option<BranchLinkedEntity>> {
    find_linked_entity_for_branch(ship_dir, branch)
}

pub fn on_post_checkout(ship_dir: &Path, new_branch: &str, project_root: &Path) -> Result<()> {
    // Ensure generated agent files are gitignored regardless of branch type.
    let _ = write_root_gitignore(project_root);

    let linked = find_linked_entity_for_branch(ship_dir, new_branch)?;
    match &linked {
        Some(doc) => {
            if let Err(error) = persist_branch_link(ship_dir, new_branch, doc) {
                eprintln!(
                    "[ship] branch-context sync warning for branch '{}': {}",
                    new_branch, error
                );
            }
        }
        None => {
            if let Err(error) = runtime::state_db::clear_branch_link(ship_dir, new_branch) {
                eprintln!(
                    "[ship] branch-context clear warning for branch '{}': {}",
                    new_branch, error
                );
            }
        }
    }

    // Workspace state is owned by runtime. Git hook is the adapter that
    // reconciles current branch -> active workspace.
    let workspace_mode_override = match sync_workspace(ship_dir, new_branch) {
        Ok(workspace) => workspace.active_mode,
        Err(error) => {
            eprintln!(
                "[ship] workspace sync warning for branch '{}': {}",
                new_branch, error
            );
            None
        }
    };

    let config = get_effective_config(Some(ship_dir.to_path_buf()))?;

    let Some(doc) = linked else {
        // Teardown must include:
        // 1) currently configured providers, and
        // 2) providers that have previously-exported Ship-managed state.
        //
        // This prevents stale context/config when a feature-level provider override
        // (e.g. codex) differs from project-level providers.
        let mut teardown_targets: BTreeSet<String> = config.providers.iter().cloned().collect();
        for provider in runtime::list_providers(ship_dir)? {
            let (managed_servers, last_mode) =
                runtime::get_managed_state_db(ship_dir, &provider.id).unwrap_or_default();
            if !managed_servers.is_empty() || last_mode.is_some() {
                teardown_targets.insert(provider.id);
            }
        }

        for provider in teardown_targets {
            agent_export::teardown(ship_dir.to_path_buf(), &provider)?;
        }
        return Ok(());
    };

    match doc {
        BranchLinkedEntity::Feature(entry) => {
            let feature = entry.feature;
            let agent_cfg = resolve_agent_config_with_mode_override(
                ship_dir,
                feature.metadata.agent.as_ref(),
                workspace_mode_override.as_deref(),
            )?;

            let mcp_server_ids: Vec<String> =
                agent_cfg.mcp_servers.iter().map(|s| s.id.clone()).collect();

            let feature_server_filter = feature
                .metadata
                .agent
                .as_ref()
                .filter(|a| !a.mcp_servers.is_empty())
                .map(|_| mcp_server_ids.as_slice());

            let context = build_feature_context(&feature, &agent_cfg.skills, &agent_cfg.rules);

            for provider in &agent_cfg.providers {
                agent_export::write_context(project_root, provider, &context)?;
                agent_export::export_to_filtered_with_mode_override(
                    ship_dir.to_path_buf(),
                    provider,
                    feature_server_filter,
                    workspace_mode_override.as_deref(),
                )?;
                if provider == "claude" {
                    ensure_required_mcp_servers(project_root, &mcp_server_ids)?;
                }
            }

            println!(
                "[ship] loaded feature '{}' for: {}",
                feature.metadata.title,
                agent_cfg.providers.join(", ")
            );
        }
        BranchLinkedEntity::Spec(spec_entry) => {
            let spec = spec_entry.spec;
            let agent_cfg = resolve_agent_config_with_mode_override(
                ship_dir,
                None,
                workspace_mode_override.as_deref(),
            )?;

            let context = build_spec_context(&spec, &agent_cfg.skills, &agent_cfg.rules);

            for provider in &agent_cfg.providers {
                agent_export::write_context(project_root, provider, &context)?;
                agent_export::export_to_with_mode_override(
                    ship_dir.to_path_buf(),
                    provider,
                    workspace_mode_override.as_deref(),
                )?;
            }

            println!(
                "[ship] loaded spec '{}' for: {}",
                spec.metadata.title,
                agent_cfg.providers.join(", ")
            );
        }
    }

    Ok(())
}

// ─── Context content builders ─────────────────────────────────────────────────

/// Build provider-agnostic Markdown context for a feature branch.
pub fn build_feature_context(feature: &Feature, skills: &[Skill], rules: &[Rule]) -> String {
    let mut c = String::new();
    c.push_str(&format!("# [ship] {}\n\n", feature.metadata.title));
    c.push_str("> Auto-generated by ship on branch checkout. Do not edit manually - re-run `ship git sync` to regenerate.\n\n");

    c.push_str("## Feature Spec\n\n");
    if feature.body.trim().is_empty() {
        c.push_str("_No feature body provided._\n\n");
    } else {
        c.push_str(feature.body.trim());
        c.push_str("\n\n");
    }

    append_skills_section(&mut c, skills);
    append_rules_section(&mut c, rules);

    let branch = feature.metadata.branch.as_deref().unwrap_or("unassigned");
    let fid = if feature.metadata.id.is_empty() {
        "unknown"
    } else {
        &feature.metadata.id
    };
    c.push_str(&format!("---\n_Branch: {} | Feature: {}_\n", branch, fid));
    c
}

/// Build provider-agnostic Markdown context for a spec branch.
pub fn build_spec_context(spec: &Spec, skills: &[Skill], rules: &[Rule]) -> String {
    let mut c = String::new();
    c.push_str(&format!("# [ship] {}\n\n", spec.metadata.title));
    c.push_str("> Auto-generated by ship on branch checkout. Do not edit manually - re-run `ship git sync` to regenerate.\n\n");

    c.push_str("## Spec\n\n");
    if spec.body.trim().is_empty() {
        c.push_str("_No spec body provided._\n\n");
    } else {
        c.push_str(spec.body.trim());
        c.push_str("\n\n");
    }

    append_skills_section(&mut c, skills);
    append_rules_section(&mut c, rules);

    let branch = spec.metadata.branch.as_deref().unwrap_or("unassigned");
    let sid = if spec.metadata.id.is_empty() {
        "unknown"
    } else {
        &spec.metadata.id
    };
    c.push_str(&format!("---\n_Branch: {} | Spec: {}_\n", branch, sid));
    c
}

fn append_skills_section(c: &mut String, skills: &[Skill]) {
    c.push_str("## Skills\n\n");
    if skills.is_empty() {
        c.push_str("_No skills configured._\n\n");
    } else {
        for skill in skills {
            c.push_str(&format!("### {} (`{}`)\n\n", skill.name, skill.id));
            c.push_str(skill.content.trim());
            c.push_str("\n\n");
        }
    }
}

fn append_rules_section(c: &mut String, rules: &[Rule]) {
    if rules.is_empty() {
        return;
    }
    c.push_str("## Rules\n\n");
    for rule in rules {
        // Derive a display name from the filename (strip .md, replace dashes with spaces)
        let name = rule.file_name.trim_end_matches(".md").replace('-', " ");
        c.push_str(&format!("### {}\n\n", name));
        c.push_str(rule.content.trim());
        c.push_str("\n\n");
    }
}

// Kept for test compatibility — writes Claude.md only.
pub fn generate_claude_md(
    project_root: &Path,
    feature: &Feature,
    skills: &[Skill],
    rules: &[Rule],
) -> Result<()> {
    let content = build_feature_context(feature, skills, rules);
    agent_export::write_context(project_root, "claude", &content)
}

fn ensure_required_mcp_servers(project_root: &Path, required_ids: &[String]) -> Result<()> {
    if required_ids.is_empty() {
        return Ok(());
    }

    let mcp_json_path = project_root.join(".mcp.json");
    let raw = fs::read_to_string(&mcp_json_path)
        .with_context(|| format!("Expected {} to exist", mcp_json_path.display()))?;
    let root: serde_json::Value = serde_json::from_str(&raw)
        .with_context(|| format!("Failed to parse {}", mcp_json_path.display()))?;

    for id in required_ids {
        let present = root
            .get("mcpServers")
            .and_then(|servers| servers.get(id))
            .is_some();
        if !present {
            return Err(anyhow!("Expected .mcp.json to contain MCP server '{}'", id));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ship_module_project::create_feature;
    use ship_module_project::init_project;
    use tempfile::tempdir;

    #[test]
    fn install_hooks_writes_post_checkout_and_pre_commit() -> Result<()> {
        let tmp = tempdir()?;
        let git_dir = tmp.path().join(".git");
        fs::create_dir_all(git_dir.join("hooks"))?;

        install_hooks(&git_dir)?;
        // idempotent
        install_hooks(&git_dir)?;

        let post_checkout = fs::read_to_string(git_dir.join("hooks/post-checkout"))?;
        assert_eq!(post_checkout, POST_CHECKOUT_HOOK_CONTENT);

        let pre_commit = fs::read_to_string(git_dir.join("hooks/pre-commit"))?;
        assert!(pre_commit.contains("CLAUDE.md"));
        assert!(pre_commit.contains(".mcp.json"));
        assert!(pre_commit.contains(".claude"));
        assert!(pre_commit.starts_with("#!/usr/bin/env sh"));
        Ok(())
    }

    #[test]
    fn install_hooks_skips_missing_git_dir() -> Result<()> {
        let tmp = tempdir()?;
        // .git doesn't exist — should be a no-op, not an error
        install_hooks(&tmp.path().join(".git"))?;
        Ok(())
    }

    #[test]
    fn write_root_gitignore_appends_generated_entries() -> Result<()> {
        let tmp = tempdir()?;
        write_root_gitignore(tmp.path())?;

        let content = fs::read_to_string(tmp.path().join(".gitignore"))?;
        for entry in GENERATED_GITIGNORE_ENTRIES {
            assert!(content.contains(entry), "missing entry: {}", entry);
        }
        Ok(())
    }

    #[test]
    fn write_root_gitignore_is_idempotent() -> Result<()> {
        let tmp = tempdir()?;
        write_root_gitignore(tmp.path())?;
        write_root_gitignore(tmp.path())?;

        let content = fs::read_to_string(tmp.path().join(".gitignore"))?;
        // Each entry should appear exactly once
        assert_eq!(content.matches("CLAUDE.md").count(), 1);
        assert_eq!(content.matches(".mcp.json").count(), 1);
        Ok(())
    }

    #[test]
    fn write_root_gitignore_preserves_existing_entries() -> Result<()> {
        let tmp = tempdir()?;
        fs::write(tmp.path().join(".gitignore"), "node_modules/\n.env\n")?;

        write_root_gitignore(tmp.path())?;

        let content = fs::read_to_string(tmp.path().join(".gitignore"))?;
        assert!(content.contains("node_modules/"));
        assert!(content.contains(".env"));
        assert!(content.contains("CLAUDE.md"));
        Ok(())
    }

    #[test]
    fn write_root_gitignore_skips_already_present_entries() -> Result<()> {
        let tmp = tempdir()?;
        // Pre-populate with some of the entries
        fs::write(tmp.path().join(".gitignore"), "CLAUDE.md\n.mcp.json\n")?;

        write_root_gitignore(tmp.path())?;

        let content = fs::read_to_string(tmp.path().join(".gitignore"))?;
        // These should still appear exactly once (not duplicated)
        assert_eq!(content.matches("CLAUDE.md").count(), 1);
        assert_eq!(content.matches(".mcp.json").count(), 1);
        // But the others should now be present
        assert!(content.contains(".claude/"));
        Ok(())
    }

    #[test]
    fn find_feature_for_branch_returns_matching_feature() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = init_project(tmp.path().to_path_buf())?;
        let entry = create_feature(&ship_dir, "Auth", "body", None, None, Some("feature/auth"))?;

        let found = find_feature_for_branch(&ship_dir, "feature/auth")?;
        assert_eq!(found.map(|f| f.id), Some(entry.id));
        Ok(())
    }

    #[test]
    fn generate_claude_md_writes_expected_sections() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = init_project(tmp.path().to_path_buf())?;
        let entry = create_feature(
            &ship_dir,
            "Feature Title",
            "Feature body",
            None,
            None,
            Some("feature/title"),
        )?;
        let feature = entry.feature;

        generate_claude_md(tmp.path(), &feature, &[], &[])?;
        let content = fs::read_to_string(tmp.path().join("CLAUDE.md"))?;
        assert!(content.contains("# [ship] Feature Title"));
        assert!(content.contains("## Feature Spec"));
        assert!(content.contains("Feature body"));
        Ok(())
    }
}
