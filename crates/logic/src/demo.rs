use crate::{create_adr, create_issue, init_project, log_action};
use anyhow::Result;
use std::path::PathBuf;

/// Initialize a demo project at `base_dir` and seed it with sample data.
/// Safe to call on an existing project — won't overwrite existing issues.
pub fn init_demo_project(base_dir: PathBuf) -> Result<PathBuf> {
    let project_dir = init_project(base_dir)?;

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

    for (title, desc, status) in issues {
        let path = project_dir
            .join("issues")
            .join(status)
            .join(format!("{}.md", crate::sanitize_file_name(title)));
        if !path.exists() {
            create_issue(project_dir.clone(), title, desc, status)?;
        }
    }

    // Sample ADRs
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
        let adr_path = project_dir.join("adrs");
        // Check if an ADR with this title already exists (approximate)
        let slug = crate::sanitize_file_name(title);
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
            create_adr(project_dir.clone(), title, decision, status)?;
        }
    }

    // Seed log with a few entries
    log_action(
        project_dir.clone(),
        "demo init",
        "Demo project initialized with sample data",
    )?;
    log_action(
        project_dir.clone(),
        "issue move",
        "Moved update-dependencies to done",
    )?;
    log_action(
        project_dir.clone(),
        "adr create",
        "Created ADR: Use PostgreSQL as primary database",
    )?;

    Ok(project_dir)
}
