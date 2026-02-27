use anyhow::{Context, Result, anyhow};
use runtime::{
    Feature, FeatureAgentConfig, IssueEntry, ProjectConfig, Skill, agent_export,
    get_effective_config, get_effective_skill, get_feature, list_issues_full,
};
use std::collections::HashSet;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

const POST_CHECKOUT_HOOK_CONTENT: &str = "#!/usr/bin/env sh\nship git post-checkout \"$@\"\n";

struct ResolvedFeatureAgent {
    mcp_server_ids: Vec<String>,
    skills: Vec<Skill>,
}

pub fn install_hooks(git_dir: &Path) -> Result<()> {
    if !git_dir.exists() {
        return Ok(());
    }

    let hooks_dir = git_dir.join("hooks");
    fs::create_dir_all(&hooks_dir)
        .with_context(|| format!("Failed to create hooks directory: {}", hooks_dir.display()))?;

    let post_checkout = hooks_dir.join("post-checkout");
    let should_write = fs::read_to_string(&post_checkout)
        .map(|existing| existing != POST_CHECKOUT_HOOK_CONTENT)
        .unwrap_or(true);

    if should_write {
        fs::write(&post_checkout, POST_CHECKOUT_HOOK_CONTENT).with_context(|| {
            format!(
                "Failed to write git post-checkout hook: {}",
                post_checkout.display()
            )
        })?;
    }

    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&post_checkout)
            .with_context(|| format!("Failed to stat hook: {}", post_checkout.display()))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&post_checkout, perms).with_context(|| {
            format!(
                "Failed to set executable permissions on hook: {}",
                post_checkout.display()
            )
        })?;
    }

    Ok(())
}

pub fn find_feature_for_branch(ship_dir: &Path, branch: &str) -> Result<Option<PathBuf>> {
    if branch.trim().is_empty() {
        return Ok(None);
    }

    let features_dir = runtime::project::features_dir(ship_dir);
    if !features_dir.exists() {
        return Ok(None);
    }

    let mut candidates = Vec::new();
    for entry in fs::read_dir(&features_dir)
        .with_context(|| format!("Failed to list features: {}", features_dir.display()))?
    {
        let path = entry?.path();
        if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("md") {
            let file_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            if file_name == "TEMPLATE.md" || file_name == "README.md" {
                continue;
            }
            candidates.push(path);
        }
    }
    candidates.sort();

    for path in candidates {
        let feature = get_feature(path.clone())
            .with_context(|| format!("Invalid feature: {}", path.display()))?;
        if feature.metadata.branch.as_deref() == Some(branch) {
            return Ok(Some(path));
        }
    }

    Ok(None)
}

pub fn on_post_checkout(ship_dir: &Path, new_branch: &str) -> Result<()> {
    let config = get_effective_config(Some(ship_dir.to_path_buf()))?;

    let Some(feature_path) = find_feature_for_branch(ship_dir, new_branch)? else {
        for provider in &config.providers {
            agent_export::teardown(ship_dir.to_path_buf(), provider)?;
        }
        return Ok(());
    };

    let feature = get_feature(feature_path)?;
    let resolved_agent = resolve_agent_config(ship_dir, &config, feature.metadata.agent.as_ref())?;

    // Effective providers: feature override wins, else project default.
    let providers = if let Some(agent) = &feature.metadata.agent {
        if !agent.providers.is_empty() {
            agent.providers.clone()
        } else {
            config.providers.clone()
        }
    } else {
        config.providers.clone()
    };

    let project_root = ship_dir
        .parent()
        .ok_or_else(|| anyhow!("Cannot determine project root from {}", ship_dir.display()))?;

    let mut open_issues = list_issues_full(ship_dir.to_path_buf())?;
    open_issues.retain(|issue| issue.status != "done");

    for provider in &providers {
        match provider.as_str() {
            "claude" => {
                generate_claude_md(project_root, &feature, &open_issues, &resolved_agent.skills)?;
                agent_export::export_to(ship_dir.to_path_buf(), "claude")?;
                ensure_required_mcp_servers(project_root, &resolved_agent.mcp_server_ids)?;
            }
            other => {
                agent_export::export_to(ship_dir.to_path_buf(), other)?;
            }
        }
    }

    println!("[ship] loaded feature '{}' for: {}", feature.metadata.title, providers.join(", "));
    Ok(())
}

pub fn generate_claude_md(
    project_root: &Path,
    feature: &Feature,
    open_issues: &[IssueEntry],
    skills: &[Skill],
) -> Result<()> {
    let mut content = String::new();
    content.push_str(&format!("# [ship] {}\n\n", feature.metadata.title));
    content.push_str(
        "> Auto-generated by ship on branch checkout. Do not edit manually - re-run `ship git sync` to regenerate.\n\n",
    );

    content.push_str("## Feature Spec\n\n");
    if feature.body.trim().is_empty() {
        content.push_str("_No feature body provided._\n\n");
    } else {
        content.push_str(feature.body.trim());
        content.push_str("\n\n");
    }

    content.push_str("## Open Issues\n\n");
    if open_issues.is_empty() {
        content.push_str("_No open issues._\n\n");
    } else {
        let mut ordered: Vec<&IssueEntry> = open_issues.iter().collect();
        ordered.sort_by(|a, b| {
            a.status
                .cmp(&b.status)
                .then_with(|| a.file_name.cmp(&b.file_name))
        });
        for issue in ordered {
            content.push_str(&format!(
                "- [ ] {} (`{}/{}`)\n",
                issue.issue.metadata.title, issue.status, issue.file_name
            ));
        }
        content.push('\n');
    }

    content.push_str("## Skills\n\n");
    if skills.is_empty() {
        content.push_str("_No skills configured._\n\n");
    } else {
        for skill in skills {
            content.push_str(&format!("### {} (`{}`)\n\n", skill.name, skill.id));
            content.push_str(skill.content.trim());
            content.push_str("\n\n");
        }
    }

    let branch = feature.metadata.branch.as_deref().unwrap_or("unassigned");
    let feature_id = if feature.metadata.id.is_empty() {
        "unknown"
    } else {
        feature.metadata.id.as_str()
    };
    content.push_str("---\n");
    content.push_str(&format!("_Branch: {} | Feature: {}_\n", branch, feature_id));

    let claude_md = project_root.join("CLAUDE.md");
    fs::write(&claude_md, content)
        .with_context(|| format!("Failed to write {}", claude_md.display()))?;
    Ok(())
}

fn resolve_agent_config(
    ship_dir: &Path,
    project_config: &ProjectConfig,
    feature_agent: Option<&FeatureAgentConfig>,
) -> Result<ResolvedFeatureAgent> {
    let configured_servers = &project_config.mcp_servers;
    let configured_server_ids: HashSet<&str> = configured_servers
        .iter()
        .map(|server| server.id.as_str())
        .collect();

    let mcp_server_ids = if let Some(agent) = feature_agent {
        if !agent.mcp_servers.is_empty() {
            agent
                .mcp_servers
                .iter()
                .map(|server| server.id.clone())
                .collect::<Vec<_>>()
        } else {
            configured_servers
                .iter()
                .filter(|server| !server.disabled)
                .map(|server| server.id.clone())
                .collect::<Vec<_>>()
        }
    } else {
        configured_servers
            .iter()
            .filter(|server| !server.disabled)
            .map(|server| server.id.clone())
            .collect::<Vec<_>>()
    };

    for id in &mcp_server_ids {
        if !configured_server_ids.contains(id.as_str()) {
            return Err(anyhow!("Feature references unknown MCP server id '{}'", id));
        }
        if let Some(server) = configured_servers.iter().find(|server| server.id == *id) {
            if server.disabled {
                return Err(anyhow!(
                    "Feature references disabled MCP server id '{}'",
                    id
                ));
            }
        }
    }

    let skill_ids = if let Some(agent) = feature_agent {
        if !agent.skills.is_empty() {
            agent
                .skills
                .iter()
                .map(|skill| skill.id.clone())
                .collect::<Vec<_>>()
        } else {
            project_config.agent.skills.clone()
        }
    } else {
        project_config.agent.skills.clone()
    };

    let mut seen = HashSet::new();
    let mut skills = Vec::new();
    for skill_id in skill_ids {
        if !seen.insert(skill_id.clone()) {
            continue;
        }
        let skill = get_effective_skill(ship_dir, &skill_id)
            .with_context(|| format!("Feature references unknown skill id '{}'", skill_id))?;
        skills.push(skill);
    }

    Ok(ResolvedFeatureAgent {
        mcp_server_ids,
        skills,
    })
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
    fn install_hooks_writes_post_checkout() -> Result<()> {
        let tmp = tempdir()?;
        let git_dir = tmp.path().join(".git");
        fs::create_dir_all(git_dir.join("hooks"))?;

        install_hooks(&git_dir)?;
        install_hooks(&git_dir)?;

        let hook_path = git_dir.join("hooks").join("post-checkout");
        let hook = fs::read_to_string(&hook_path)?;
        assert_eq!(hook, POST_CHECKOUT_HOOK_CONTENT);
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

        generate_claude_md(tmp.path(), &feature, &[], &[])?;
        let content = fs::read_to_string(tmp.path().join("CLAUDE.md"))?;
        assert!(content.contains("# [ship] Feature Title"));
        assert!(content.contains("## Feature Spec"));
        assert!(content.contains("Feature body"));
        Ok(())
    }
}
