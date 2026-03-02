use crate::create_adr;
use anyhow::Result;
use runtime::{init_core_demo, log_action};
use std::path::PathBuf;

pub fn init_demo_project(base_dir: PathBuf) -> Result<PathBuf> {
    let project_dir = init_core_demo(base_dir)?;

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

    log_action(
        &project_dir,
        "adr create",
        "Created ADR: Use PostgreSQL as primary database",
    )?;

    Ok(project_dir)
}
