use crate::{IssueStatus, create_adr, create_issue, create_spec};
use anyhow::Result;
use runtime::log_action;
use std::path::PathBuf;
use std::str::FromStr;

pub fn init_demo_project(base_dir: PathBuf) -> Result<PathBuf> {
    let project_dir = crate::project::init_project(base_dir)?;

    // Sample issues across all statuses
    let issues = vec![
        (
            "Design authentication flow",
            "Define the OAuth2 flow, token storage strategy, and session management for the new auth system.",
            "backlog",
        ),
        (
            "Set up CI/CD pipeline",
            "Configure GitHub Actions for automated testing and deployment to staging and production.",
            "backlog",
        ),
        (
            "Migrate legacy API endpoints",
            "Port the remaining v1 endpoints to the new v2 format. See ADR-001 for the versioning strategy.",
            "in-progress",
        ),
        (
            "Fix memory leak in worker pool",
            "The worker pool accumulates goroutines when tasks are cancelled. Reproduce with the stress test suite.",
            "blocked",
        ),
        (
            "Update dependencies",
            "Bump all deps to latest patch versions. Run security audit after.",
            "done",
        ),
        (
            "Write onboarding docs",
            "Create a getting-started guide for new team members. Include local setup, key concepts, and first PR guide.",
            "done",
        ),
    ];

    for (title, desc, status_str) in issues {
        let status = IssueStatus::from_str(status_str).unwrap();
        let path = runtime::project::issues_dir(&project_dir)
            .join(status.to_string())
            .join(format!(
                "{}.md",
                runtime::project::sanitize_file_name(title)
            ));
        if !path.exists() {
            create_issue(&project_dir, title, desc, status, None, None, None, None)?;
        }
    }

    // Sample specs
    let spec_title = "Agent Configuration and Modes";
    let spec_slug = runtime::project::sanitize_file_name(spec_title);
    let spec_file = runtime::project::specs_dir(&project_dir).join(format!("{}.md", spec_slug));
    if !spec_file.exists() {
        runtime::create_workspace(
            &project_dir,
            runtime::CreateWorkspaceRequest {
                branch: "feature/agent-configuration-and-modes".to_string(),
                status: Some(runtime::WorkspaceStatus::Active),
                ..Default::default()
            },
        )?;
        create_spec(
            &project_dir,
            spec_title,
            "## Overview\n\nDefine a unified agent config layer for provider/model, instruction skills, context, rules, skills, MCP servers, and modes.\n\n## Goals\n\n- One global and project-scoped config model\n- Pass-through generation via claude/codex/gemini CLIs\n- Clear mode semantics tied to workflow policy\n\n## Non-Goals\n\n- Full workflow customization engine in alpha\n\n## Approach\n\nBuild release/feature/spec primitives and wire them through CLI, MCP, and UI.\n\n## Open Questions\n\n- How to best express mode overrides per checked-out feature?\n",
            None,
        )?;
    }

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
        "issue move",
        "Moved update-dependencies to done",
    )?;
    log_action(
        &project_dir,
        "adr create",
        "Created ADR: Use PostgreSQL as primary database",
    )?;
    log_action(&project_dir, "demo init", "Core demo project initialized")?;

    Ok(project_dir)
}
