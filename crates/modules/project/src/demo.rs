use crate::{create_adr, create_feature};
use anyhow::Result;
use runtime::log_action;
use std::path::PathBuf;

pub fn init_demo_project(base_dir: PathBuf) -> Result<PathBuf> {
    let project_dir = crate::project::init_project(base_dir)?;

    // Sample feature
    let feature_title = "Pre-defined Agent Modes";
    create_feature(
        &project_dir,
        feature_title,
        "## Overview\n\nDefine a unified agent config layer for provider/model, instruction skills, context, rules, skills, MCP servers, and modes.\n\n## Goals\n\n- One global and project-scoped config model\n- Pass-through generation via claude/codex/gemini CLIs\n- Clear mode semantics tied to workflow policy\n",
        None,
        None,
    )?;

    // Seed ADRs
    let adrs = vec![
        (
            "Use PostgreSQL as primary database",
            "After evaluating SQLite, MySQL, and PostgreSQL, we chose PostgreSQL for its JSONB support, strong consistency guarantees, and ecosystem maturity.",
            "accepted",
        ),
        (
            "Adopt trunk-based development",
            "We will use trunk-based development with short-lived feature branches instead of gitflow. This reduces merge conflicts and speeds up integration.",
            "accepted",
        ),
        (
            "Evaluate GraphQL for API layer",
            "Considering GraphQL to replace the REST API for more flexible client queries. Still under evaluation — decision pending performance benchmarks.",
            "proposed",
        ),
    ];

    for (title, decision, status) in adrs {
        let adr_path = runtime::project::adrs_dir(&project_dir);
        let slug = runtime::project::sanitize_file_name(title);
        let exists = std::fs::read_dir(&adr_path)
            .map(|entries| {
                entries.flatten().any(|e| {
                    e.file_name()
                        .to_string_lossy()
                        .contains(&slug[..slug.len().min(20)])
                })
            })
            .unwrap_or(false);
        if !exists {
            create_adr(&project_dir, title, "", decision, status)?;
        }
    }

    // Seed log with a few entries
    log_action(
        &project_dir,
        "demo init",
        "Initialized core demo project data",
    )?;
    log_action(
        &project_dir,
        "adr create",
        "Created ADR: Use PostgreSQL as primary database",
    )?;
    log_action(&project_dir, "demo init", "Core demo project initialized")?;

    Ok(project_dir)
}
