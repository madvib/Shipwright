use crate::{create_feature, create_issue, create_release, create_spec, init_project, log_action};
use anyhow::Result;
use std::path::PathBuf;

/// Initialize a demo project with core primitives (Issues, Features, Specs, Releases).
/// Safe to call on an existing project — won't overwrite existing issues.
pub fn init_core_demo(base_dir: PathBuf) -> Result<PathBuf> {
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
        let path = crate::project::issues_dir(&project_dir)
            .join(status)
            .join(format!("{}.md", crate::sanitize_file_name(title)));
        if !path.exists() {
            create_issue(project_dir.clone(), title, desc, status)?;
        }
    }

    // Sample release
    let release_version = "v0.1.0-alpha";
    let release_slug = crate::sanitize_file_name(release_version);
    let release_file =
        crate::project::releases_dir(&project_dir).join(format!("{}.md", release_slug));
    if !release_file.exists() {
        create_release(
            project_dir.clone(),
            release_version,
            "## Goal\n\nShip alpha with core runtime, MCP, CLI, and UI agent configuration.\n\n## Scope\n\n- [x] Runtime + MCP bridge\n- [x] CLI + project primitives\n- [ ] Agent module UX polish\n\n## Included Features\n\n- [ ] Unified agent config panel\n- [ ] Feature delivery flow\n\n## Notes\n\nFocus on architecture confidence and test coverage.\n",
        )?;
    }

    // Sample specs
    let spec_title = "Agent Configuration and Modes";
    let spec_slug = crate::sanitize_file_name(spec_title);
    let spec_file = crate::project::specs_dir(&project_dir).join(format!("{}.md", spec_slug));
    if !spec_file.exists() {
        create_spec(
            project_dir.clone(),
            spec_title,
            "## Overview\n\nDefine a unified agent config layer for provider/model, prompts, context, rules, skills, MCP servers, and modes.\n\n## Goals\n\n- One global and project-scoped config model\n- Pass-through generation via claude/codex/gemini CLIs\n- Clear mode semantics tied to workflow policy\n\n## Non-Goals\n\n- Full workflow customization engine in alpha\n\n## Approach\n\nBuild release/feature/spec primitives and wire them through CLI, MCP, and UI.\n\n## Open Questions\n\n- How to best express mode overrides per checked-out feature?\n",
            "active",
        )?;
    }

    // Sample features
    let features = vec![
        (
            "Unified Agent Configuration UI",
            "v0-1-0-alpha.md",
            "agent-configuration-and-modes.md",
        ),
        (
            "Project Workflow Primitives",
            "v0-1-0-alpha.md",
            "agent-configuration-and-modes.md",
        ),
    ];
    for (title, release, spec) in features {
        let slug = crate::sanitize_file_name(title);
        let path = crate::project::features_dir(&project_dir).join(format!("{}.md", slug));
        if !path.exists() {
            create_feature(
                project_dir.clone(),
                title,
                "",
                Some(release),
                Some(spec),
                None,
            )?;
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
    log_action(&project_dir, "demo init", "Core demo project initialized")?;

    Ok(project_dir)
}
