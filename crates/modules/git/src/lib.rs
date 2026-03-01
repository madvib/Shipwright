use anyhow::{Context, Result, anyhow};
use runtime::{
    Feature, IssueEntry, Rule, Skill, agent_config::resolve_agent_config, agent_export,
    get_effective_config, get_feature, get_spec, list_issues_full,
};
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

const POST_CHECKOUT_HOOK_CONTENT: &str = "#!/usr/bin/env sh\nship git post-checkout \"$@\"\n";

const PRE_COMMIT_HOOK_CONTENT: &str = "\
#!/usr/bin/env sh
# ship pre-commit: block staging of generated agent config files.
# These are written by `ship git sync` / post-checkout and must never be committed.
BLOCKED=\"CLAUDE.md GEMINI.md .mcp.json\"
for f in $BLOCKED; do
    if git diff --cached --name-only | grep -qx \"$f\"; then
        echo \"[ship] ERROR: '$f' is a generated file managed by Ship and must not be committed.\"
        echo \"[ship]        Add it to .gitignore and unstage it: git restore --staged $f\"
        exit 1
    fi
done
# Also block .claude/, .gemini/, .codex/ directories
for dir in .claude .gemini .codex; do
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
    ".mcp.json",
    ".claude/",
    ".gemini/",
    ".codex/",
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

pub fn find_feature_for_branch(ship_dir: &Path, branch: &str) -> Result<Option<PathBuf>> {
    if branch.trim().is_empty() {
        return Ok(None);
    }

    let features = runtime::list_features(ship_dir.to_path_buf(), None)?;
    for feat in features {
        if feat.branch.as_deref() == Some(branch) {
            return Ok(Some(PathBuf::from(feat.path)));
        }
    }

    Ok(None)
}

/// Which document is associated with the checked-out branch.
pub enum BranchDocument {
    Feature(PathBuf),
    Spec(PathBuf),
}

fn find_spec_for_branch(ship_dir: &Path, branch: &str) -> Result<Option<PathBuf>> {
    let specs_dir = runtime::project::specs_dir(ship_dir);
    if !specs_dir.exists() {
        return Ok(None);
    }

    let mut candidates = Vec::new();
    for entry in fs::read_dir(&specs_dir)
        .with_context(|| format!("Failed to list specs: {}", specs_dir.display()))?
    {
        let path = entry?.path();
        if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("md") {
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if file_name == "TEMPLATE.md" || file_name == "README.md" {
                continue;
            }
            candidates.push(path);
        }
    }
    candidates.sort();

    for path in candidates {
        let spec =
            get_spec(path.clone()).with_context(|| format!("Invalid spec: {}", path.display()))?;
        if spec.metadata.branch.as_deref() == Some(branch) {
            return Ok(Some(path));
        }
    }

    Ok(None)
}

fn find_feature_by_uuid(ship_dir: &Path, uuid: &str) -> Option<PathBuf> {
    if let Ok(features) = runtime::list_features(ship_dir.to_path_buf(), None) {
        for feat in features {
            if let Ok(f) = get_feature(PathBuf::from(&feat.path)) {
                if f.metadata.id == uuid {
                    return Some(PathBuf::from(feat.path));
                }
            }
        }
    }
    None
}

fn find_spec_by_uuid(ship_dir: &Path, uuid: &str) -> Option<PathBuf> {
    let dir = runtime::project::specs_dir(ship_dir);
    fs::read_dir(&dir).ok()?.flatten().find_map(|e| {
        let path = e.path();
        if path.extension().and_then(|x| x.to_str()) != Some("md") {
            return None;
        }
        let spec = get_spec(path.clone()).ok()?;
        if spec.metadata.id == uuid {
            Some(path)
        } else {
            None
        }
    })
}

/// Find which document (feature or spec) is associated with the given branch.
/// Checks the DB index first (O(1)), then falls back to a frontmatter file scan.
pub fn find_document_for_branch(ship_dir: &Path, branch: &str) -> Result<Option<BranchDocument>> {
    if branch.trim().is_empty() {
        return Ok(None);
    }

    // Fast path: DB index stores document UUID populated by `feature_start`
    if let Ok(Some((doc_type, doc_uuid))) = runtime::state_db::get_branch_doc(ship_dir, branch) {
        match doc_type.as_str() {
            "feature" => {
                if let Some(path) = find_feature_by_uuid(ship_dir, &doc_uuid) {
                    return Ok(Some(BranchDocument::Feature(path)));
                }
            }
            "spec" => {
                if let Some(path) = find_spec_by_uuid(ship_dir, &doc_uuid) {
                    return Ok(Some(BranchDocument::Spec(path)));
                }
            }
            _ => {}
        }
    }

    // Fallback: scan frontmatter of all features then specs
    if let Some(path) = find_feature_for_branch(ship_dir, branch)? {
        return Ok(Some(BranchDocument::Feature(path)));
    }
    if let Some(path) = find_spec_for_branch(ship_dir, branch)? {
        return Ok(Some(BranchDocument::Spec(path)));
    }
    Ok(None)
}

pub fn on_post_checkout(ship_dir: &Path, new_branch: &str, project_root: &Path) -> Result<()> {
    // Ensure generated agent files are gitignored regardless of branch type.
    let _ = write_root_gitignore(project_root);

    let config = get_effective_config(Some(ship_dir.to_path_buf()))?;

    let Some(doc) = find_document_for_branch(ship_dir, new_branch)? else {
        for provider in &config.providers {
            agent_export::teardown(ship_dir.to_path_buf(), provider)?;
        }
        return Ok(());
    };

    let mut open_issues = list_issues_full(ship_dir.to_path_buf())?;
    open_issues.retain(|issue| issue.status != "done");

    match doc {
        BranchDocument::Feature(feature_path) => {
            let feature = get_feature(feature_path)?;
            let agent_cfg =
                resolve_agent_config(ship_dir, feature.metadata.agent.as_ref())?;

            let mcp_server_ids: Vec<String> =
                agent_cfg.mcp_servers.iter().map(|s| s.id.clone()).collect();

            let feature_server_filter = feature
                .metadata
                .agent
                .as_ref()
                .filter(|a| !a.mcp_servers.is_empty())
                .map(|_| mcp_server_ids.as_slice());

            let context = build_feature_context(
                &feature,
                &open_issues,
                &agent_cfg.skills,
                &agent_cfg.rules,
            );

            for provider in &agent_cfg.providers {
                agent_export::write_context(project_root, provider, &context)?;
                agent_export::export_to_filtered(
                    ship_dir.to_path_buf(),
                    provider,
                    feature_server_filter,
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
        BranchDocument::Spec(spec_path) => {
            let spec = get_spec(spec_path)?;
            let agent_cfg = resolve_agent_config(ship_dir, None)?;

            let context = build_spec_context(
                &spec,
                &open_issues,
                &agent_cfg.skills,
                &agent_cfg.rules,
            );

            for provider in &agent_cfg.providers {
                agent_export::write_context(project_root, provider, &context)?;
                agent_export::export_to(ship_dir.to_path_buf(), provider)?;
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
pub fn build_feature_context(
    feature: &Feature,
    open_issues: &[IssueEntry],
    skills: &[Skill],
    rules: &[Rule],
) -> String {
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

    append_issues_section(&mut c, open_issues);
    append_skills_section(&mut c, skills);
    append_rules_section(&mut c, rules);

    let branch = feature.metadata.branch.as_deref().unwrap_or("unassigned");
    let fid = if feature.metadata.id.is_empty() { "unknown" } else { &feature.metadata.id };
    c.push_str(&format!("---\n_Branch: {} | Feature: {}_\n", branch, fid));
    c
}

/// Build provider-agnostic Markdown context for a spec branch.
pub fn build_spec_context(
    spec: &runtime::Spec,
    open_issues: &[IssueEntry],
    skills: &[Skill],
    rules: &[Rule],
) -> String {
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

    append_issues_section(&mut c, open_issues);
    append_skills_section(&mut c, skills);
    append_rules_section(&mut c, rules);

    let branch = spec.metadata.branch.as_deref().unwrap_or("unassigned");
    let sid = if spec.metadata.id.is_empty() { "unknown" } else { &spec.metadata.id };
    c.push_str(&format!("---\n_Branch: {} | Spec: {}_\n", branch, sid));
    c
}

fn append_issues_section(c: &mut String, open_issues: &[IssueEntry]) {
    c.push_str("## Open Issues\n\n");
    if open_issues.is_empty() {
        c.push_str("_No open issues._\n\n");
    } else {
        let mut ordered: Vec<&IssueEntry> = open_issues.iter().collect();
        ordered.sort_by(|a, b| a.status.cmp(&b.status).then_with(|| a.file_name.cmp(&b.file_name)));
        for issue in ordered {
            c.push_str(&format!(
                "- [ ] {} (`{}/{}`)\n",
                issue.issue.metadata.title, issue.status, issue.file_name
            ));
        }
        c.push('\n');
    }
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
        let name = rule.file_name
            .trim_end_matches(".md")
            .replace('-', " ");
        c.push_str(&format!("### {}\n\n", name));
        c.push_str(rule.content.trim());
        c.push_str("\n\n");
    }
}

// Kept for test compatibility — writes Claude.md only.
pub fn generate_claude_md(
    project_root: &Path,
    feature: &Feature,
    open_issues: &[IssueEntry],
    skills: &[Skill],
    rules: &[Rule],
) -> Result<()> {
    let content = build_feature_context(feature, open_issues, skills, rules);
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
    use runtime::{create_feature, get_feature, init_project};
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
        let feature_path = create_feature(
            ship_dir.clone(),
            "Auth",
            "body",
            None,
            None,
            Some("feature/auth"),
        )?;

        let found = find_feature_for_branch(&ship_dir, "feature/auth")?;
        assert_eq!(found, Some(feature_path));
        Ok(())
    }

    #[test]
    fn generate_claude_md_writes_expected_sections() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = init_project(tmp.path().to_path_buf())?;
        let feature_path = create_feature(
            ship_dir.clone(),
            "Feature Title",
            "Feature body",
            None,
            None,
            Some("feature/title"),
        )?;
        let feature = get_feature(feature_path)?;

        generate_claude_md(tmp.path(), &feature, &[], &[], &[])?;
        let content = fs::read_to_string(tmp.path().join("CLAUDE.md"))?;
        assert!(content.contains("# [ship] Feature Title"));
        assert!(content.contains("## Feature Spec"));
        assert!(content.contains("Feature body"));
        Ok(())
    }
}
