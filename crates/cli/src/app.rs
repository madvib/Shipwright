use anyhow::Result;
use runtime::project::{get_global_dir, get_project_dir};
use runtime::workspace::set_workspace_active_mode;
use runtime::{
    CreateWorkspaceRequest, EndWorkspaceSessionRequest, WorkspaceStatus, WorkspaceType,
    activate_workspace, add_status, autodetect_providers, create_workspace, end_workspace_session,
    get_active_workspace_session, get_config, get_git_config, get_project_statuses, get_workspace,
    is_category_committed, list_mcp_servers, list_providers, list_workspace_sessions,
    list_workspaces, log_action, migrate_global_state, migrate_json_config_file,
    migrate_project_state, remove_status, set_category_committed, start_workspace_session,
    sync_workspace, transition_workspace_status,
};
use ship_module_git::{install_hooks, on_post_checkout, write_root_gitignore};
use ship_module_project::ops::adr::{create_adr, find_adr_path, list_adrs, move_adr};
use ship_module_project::ops::feature::{
    create_feature, feature_done, feature_start, get_feature_by_id, list_features, update_feature,
};
use ship_module_project::ops::issue::{
    create_issue, get_issue_by_id, list_issues, move_issue_with_from,
};
use ship_module_project::ops::note::{
    create_note, get_note_by_id, list_notes, update_note_content,
};
use ship_module_project::ops::release::{
    create_release, get_release_by_id, list_releases, update_release,
};
use ship_module_project::ops::spec::{create_spec, get_spec_by_id, list_specs};
use ship_module_project::{
    ADR, AdrStatus, FeatureStatus, ISSUE_STATUSES, IssueStatus, NoteScope, import_adrs_from_files,
    import_features_from_files, import_issues_from_files, import_notes_from_files,
    import_releases_from_files, import_specs_from_files, init_demo_project, init_project,
    list_registered_projects, register_project, rename_project, unregister_project,
};
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

use crate::surface::*;

pub fn handle_init_command(target: cli_framework::InitTarget) -> Result<()> {
    let ship_path = init_project(target.path.clone())?;
    if let Err(err) = install_hooks(&target.path.join(".git")) {
        eprintln!(
            "[ship] warning: failed to install git hooks in {}: {}",
            target.path.join(".git").display(),
            err
        );
    }
    if let Err(err) = write_root_gitignore(&target.path) {
        eprintln!("[ship] warning: failed to update root .gitignore: {}", err);
    }

    let tracked = match register_project(target.project_name.clone(), target.path.clone()) {
        Ok(()) => true,
        Err(err) => {
            eprintln!(
                "[ship] warning: initialized project but failed to register globally: {}",
                err
            );
            eprintln!(
                "[ship] run `ship projects track {} {}` later to add it to the global registry",
                target.project_name,
                target.path.display()
            );
            false
        }
    };

    if tracked {
        println!(
            "Initialized and tracked Ship project in {}",
            ship_path.display()
        );
    } else {
        println!("Initialized Ship project in {}", ship_path.display());
    }

    match autodetect_providers(&target.path) {
        Ok(found) if !found.is_empty() => {
            println!("Detected and connected providers: {}", found.join(", "));
        }
        Ok(_) => {}
        Err(err) => {
            eprintln!("[ship] warning: provider detection failed: {}", err);
        }
    }

    Ok(())
}

pub fn append_doctor_checks(report: &mut cli_framework::DoctorReport) -> Result<()> {
    match get_project_dir(None) {
        Ok(dir) => report.ok("Project directory", dir.display().to_string()),
        Err(err) => report.fail("Project directory", err.to_string()),
    }

    match get_global_dir() {
        Ok(dir) => report.ok("Global directory", dir.display().to_string()),
        Err(err) => report.fail("Global directory", err.to_string()),
    }

    match get_config(None) {
        Ok(_) => report.ok("Configuration", "valid"),
        Err(err) => report.fail("Configuration", err.to_string()),
    }

    if let Ok(dir) = get_project_dir(None) {
        if let Ok(providers) = list_providers(&dir) {
            let connected = providers.iter().filter(|provider| provider.enabled).count();
            let installed = providers
                .iter()
                .filter(|provider| provider.installed)
                .count();
            report.ok(
                "AI providers",
                format!(
                    "{} connected, {} installed (out of {} supported)",
                    connected,
                    installed,
                    providers.len()
                ),
            );

            if connected == 0 {
                report.warn(
                    "AI providers",
                    "No providers connected. Use `ship providers connect <id>`.",
                );
            }

            for provider in providers.iter().filter(|provider| provider.enabled) {
                if provider.installed {
                    let version = provider.version.as_deref().unwrap_or("unknown version");
                    report.ok(
                        format!("Provider {}", provider.id),
                        format!("connected and installed ({})", version),
                    );
                } else {
                    report.warn(
                        format!("Provider {}", provider.id),
                        format!(
                            "connected but binary '{}' is missing in PATH",
                            provider.binary
                        ),
                    );
                }
            }
        }

        if let Ok(servers) = list_mcp_servers(Some(dir)) {
            let ship_mcp = servers.iter().find(|server| server.id == "ship");
            match ship_mcp {
                Some(server)
                    if server.command == "ship"
                        && server.args.len() >= 1
                        && server.args[0] == "mcp" =>
                {
                    report.ok("MCP server ship", "registered with `ship mcp` command");
                }
                Some(server) => report.warn(
                    "MCP server ship",
                    format!(
                        "registration looks customized/outdated: {} {:?}",
                        server.command, server.args
                    ),
                ),
                None => report.warn("MCP server ship", "not registered in project MCP registry"),
            }
        }
    }

    Ok(())
}

pub fn handle_cli(cli: Cli) -> Result<()> {
    let _ = ensure_user_notes_imported_once(false, false);
    if let Ok(project_dir) = get_project_dir(None) {
        let _ = ensure_project_imported_once(&project_dir, false, false);
    }

    match cli.command {
        Some(Commands::Init { .. } | Commands::Doctor | Commands::Version) => anyhow::bail!(
            "core command should be handled by cli-framework before app command dispatch"
        ),
        Some(Commands::Issue { action }) => {
            let project_dir = get_project_dir_cli()?;
            match action {
                IssueCommands::Create { title, description } => {
                    let issue = create_issue(
                        &project_dir,
                        &title,
                        &description,
                        IssueStatus::Backlog,
                        None,
                        None,
                        None,
                        None,
                    )?;
                    println!("Issue created: {} ({})", issue.file_name, issue.id);
                }
                IssueCommands::List => {
                    let issues = list_issues(&project_dir)?;
                    for issue in issues {
                        println!("[{}] {}", issue.status, issue.file_name);
                    }
                }
                IssueCommands::Move {
                    file_name,
                    from,
                    to,
                } => {
                    let from_status = from
                        .parse::<IssueStatus>()
                        .map_err(|_| anyhow::anyhow!("Invalid issue status: {}", from))?;
                    let to_status = to
                        .parse::<IssueStatus>()
                        .map_err(|_| anyhow::anyhow!("Invalid issue status: {}", to))?;
                    move_issue_with_from(&project_dir, &file_name, from_status, to_status)?;
                    println!("Moved {} from {} to {}", file_name, from, to);
                }
            }
        }
        Some(Commands::Adr { action }) => {
            let project_dir = get_project_dir_cli()?;
            match action {
                AdrCommands::Create { title, decision } => {
                    let entry = create_adr(&project_dir, &title, "", &decision, "proposed")?;
                    println!("ADR created: {} (id: {})", title, entry.id);
                }
                AdrCommands::List => {
                    let mut adrs = list_adrs(&project_dir)?;
                    adrs.sort_by(|a, b| b.file_name.cmp(&a.file_name));
                    if adrs.is_empty() {
                        println!("No ADRs found.");
                    } else {
                        for adr in adrs {
                            println!(
                                "[{}] {} ({})",
                                adr.status, adr.adr.metadata.title, adr.file_name
                            );
                        }
                    }
                }
                AdrCommands::Get { file_name } => {
                    let path = find_adr_path(&project_dir, &file_name)?;
                    let content = std::fs::read_to_string(path)?;
                    println!("{}", content);
                }
                AdrCommands::Move { file_name, status } => {
                    let new_status = status
                        .parse::<AdrStatus>()
                        .map_err(|_| anyhow::anyhow!("Invalid ADR status: {}", status))?;
                    // Find the ADR by reading the file and extracting its id.
                    let path = find_adr_path(&project_dir, &file_name)?;
                    let content = std::fs::read_to_string(&path)?;
                    let adr = ADR::from_markdown(&content)
                        .map_err(|_| anyhow::anyhow!("Could not parse ADR file: {}", file_name))?;
                    let entry = move_adr(&project_dir, &adr.metadata.id, new_status.clone())?;
                    println!("Moved {} to {} (id: {})", file_name, new_status, entry.id);
                }
            }
        }
        Some(Commands::Note { action }) => match action {
            NoteCommands::Create {
                title,
                content,
                scope,
            } => {
                let scope = parse_note_scope(&scope)?;
                let project_dir = match scope {
                    NoteScope::Project => Some(get_project_dir_cli()?),
                    NoteScope::User => None,
                };
                let body = content.unwrap_or_default();
                let note = create_note(scope, project_dir.as_deref(), &title, &body)?;
                println!("Note created: {} (id: {})", note.title, note.id);
            }
            NoteCommands::List { scope } => {
                let scope = parse_note_scope(&scope)?;
                let project_dir = match scope {
                    NoteScope::Project => Some(get_project_dir_cli()?),
                    NoteScope::User => None,
                };
                let notes = list_notes(scope, project_dir.as_deref())?;
                if notes.is_empty() {
                    println!("No notes found.");
                } else {
                    for note in notes {
                        println!("{} ({})", note.title, note.id);
                    }
                }
            }
            NoteCommands::Get { file_name, scope } => {
                let scope = parse_note_scope(&scope)?;
                let project_dir = match scope {
                    NoteScope::Project => Some(get_project_dir_cli()?),
                    NoteScope::User => None,
                };
                let note = get_note_by_id(scope, project_dir.as_deref(), &file_name)?;
                println!("{}", note.content);
            }
            NoteCommands::Update {
                file_name,
                content,
                scope,
            } => {
                let scope = parse_note_scope(&scope)?;
                let project_dir = match scope {
                    NoteScope::Project => Some(get_project_dir_cli()?),
                    NoteScope::User => None,
                };
                let note =
                    update_note_content(scope, project_dir.as_deref(), &file_name, &content)?;
                println!("Updated note: {}", note.title);
            }
        },
        Some(Commands::Skill { action }) => {
            let project_dir = get_project_dir(None).ok();
            let action = match action {
                SkillCommands::Install {
                    source,
                    id,
                    git_ref,
                    repo_path,
                    scope,
                    force,
                } => cli_framework::SkillAction::Install {
                    source,
                    id,
                    git_ref,
                    repo_path,
                    scope: cli_framework::parse_skill_write_scope(&scope)?,
                    force,
                },
                SkillCommands::Create {
                    id,
                    name,
                    content,
                    scope,
                } => cli_framework::SkillAction::Create {
                    id,
                    name,
                    content,
                    scope: cli_framework::parse_skill_write_scope(&scope)?,
                },
                SkillCommands::List { scope } => cli_framework::SkillAction::List {
                    scope: cli_framework::parse_skill_read_scope(&scope)?,
                },
                SkillCommands::Get { id, scope } => cli_framework::SkillAction::Get {
                    id,
                    scope: cli_framework::parse_skill_read_scope(&scope)?,
                },
                SkillCommands::Update {
                    id,
                    name,
                    content,
                    scope,
                } => cli_framework::SkillAction::Update {
                    id,
                    name,
                    content,
                    scope: cli_framework::parse_skill_write_scope(&scope)?,
                },
                SkillCommands::Delete { id, scope } => cli_framework::SkillAction::Delete {
                    id,
                    scope: cli_framework::parse_skill_write_scope(&scope)?,
                },
            };
            cli_framework::handle_skill_action(action, project_dir.as_deref())?;
        }
        Some(Commands::Spec { action }) => {
            let project_dir = get_project_dir_cli()?;
            match action {
                SpecCommands::Create {
                    title,
                    content,
                    workspace,
                } => {
                    let body = content.unwrap_or_default();
                    let spec = create_spec(&project_dir, &title, &body, workspace.as_deref())?;
                    println!("Spec created: {} ({})", spec.file_name, spec.id);
                }
                SpecCommands::List => {
                    let mut specs = list_specs(&project_dir)?;
                    specs.sort_by(|a, b| b.spec.metadata.updated.cmp(&a.spec.metadata.updated));
                    if specs.is_empty() {
                        println!("No specs found.");
                    } else {
                        for spec in specs {
                            println!(
                                "[{}] {} ({})",
                                spec.status, spec.spec.metadata.title, spec.file_name
                            );
                        }
                    }
                }
                SpecCommands::Get { file_name } => {
                    let spec = get_spec_by_id(&project_dir, &file_name)?;
                    println!("{}", spec.spec.to_markdown()?);
                }
            }
        }
        Some(Commands::Release { action }) => {
            let project_dir = get_project_dir_cli()?;
            match action {
                ReleaseCommands::Create { version, content } => {
                    let body = content.unwrap_or_default();
                    let entry = create_release(&project_dir, &version, &body)?;
                    println!("Release created: {}", entry.path);
                }
                ReleaseCommands::List => {
                    let releases = list_releases(&project_dir)?;
                    if releases.is_empty() {
                        println!("No releases found.");
                    } else {
                        for release in releases {
                            println!(
                                "[{}] {} ({})",
                                release.status, release.version, release.file_name
                            );
                        }
                    }
                }
                ReleaseCommands::Get { file_name } => {
                    let version = file_name.trim_end_matches(".md");
                    let entry = get_release_by_id(&project_dir, version)
                        .map_err(|_| anyhow::anyhow!("Release not found: {}", file_name))?;
                    let release_path = {
                        let primary =
                            runtime::project::releases_dir(&project_dir).join(&entry.file_name);
                        if primary.exists() {
                            primary
                        } else {
                            let legacy = runtime::project::upcoming_releases_dir(&project_dir)
                                .join(&entry.file_name);
                            if legacy.exists() {
                                legacy
                            } else {
                                anyhow::bail!("Release file not found: {}", entry.file_name);
                            }
                        }
                    };
                    let content = std::fs::read_to_string(release_path)?;
                    println!("{}", content);
                }
                ReleaseCommands::Update { file_name, content } => {
                    let version = file_name.trim_end_matches(".md");
                    let mut entry = get_release_by_id(&project_dir, version)?;
                    entry.release.body = content;
                    update_release(&project_dir, version, entry.release)?;
                    println!("Updated release: {}", file_name);
                }
            }
        }
        Some(Commands::Feature { action }) => {
            let project_dir = get_project_dir_cli()?;
            match action {
                FeatureCommands::Create {
                    title,
                    content,
                    release_id,
                    spec_id,
                    branch,
                } => {
                    let body = content.unwrap_or_default();
                    let entry = create_feature(
                        &project_dir,
                        &title,
                        &body,
                        release_id.as_deref(),
                        spec_id.as_deref(),
                        branch.as_deref(),
                    )?;
                    println!("Feature created: {}", entry.path);
                }
                FeatureCommands::List { status } => {
                    let features = list_features(&project_dir)?;
                    let filtered: Vec<_> = if let Some(s) = status {
                        let target_status = s.parse::<FeatureStatus>().unwrap_or_default();
                        features
                            .into_iter()
                            .filter(|f| f.status == target_status)
                            .collect()
                    } else {
                        features
                    };

                    if filtered.is_empty() {
                        println!("No features found.");
                    } else {
                        for entry in filtered {
                            println!(
                                "[{}] {} ({}) id={}",
                                entry.status,
                                entry.feature.metadata.title,
                                entry.file_name,
                                entry.id
                            );
                        }
                    }
                }
                FeatureCommands::Get { id } => {
                    let entry = get_feature_by_id(&project_dir, &id)?;
                    println!("{}", entry.feature.to_markdown()?);
                }
                FeatureCommands::Update { id, content } => {
                    let mut entry = get_feature_by_id(&project_dir, &id)?;
                    entry.feature.body = content;
                    update_feature(&project_dir, &id, entry.feature)?;
                    println!("Updated feature: {}", id);
                }
                FeatureCommands::Start { id, branch } => {
                    let mut entry = get_feature_by_id(&project_dir, &id)?;
                    let branch_name = branch.unwrap_or_else(|| {
                        let base =
                            runtime::project::sanitize_file_name(&entry.feature.metadata.title);
                        format!("feature/{}", base)
                    });

                    // Create the branch if it doesn't exist
                    let branch_exists = std::process::Command::new("git")
                        .args(["rev-parse", "--verify", &branch_name])
                        .current_dir(&project_dir)
                        .output()
                        .map(|o| o.status.success())
                        .unwrap_or(false);

                    if !branch_exists {
                        let result = std::process::Command::new("git")
                            .args(["checkout", "-b", &branch_name])
                            .current_dir(&project_dir)
                            .status()?;
                        if !result.success() {
                            anyhow::bail!("Failed to create branch: {}", branch_name);
                        }
                    } else {
                        let result = std::process::Command::new("git")
                            .args(["checkout", &branch_name])
                            .current_dir(&project_dir)
                            .status()?;
                        if !result.success() {
                            anyhow::bail!("Failed to checkout branch: {}", branch_name);
                        }
                    }

                    entry.feature.metadata.branch = Some(branch_name);
                    update_feature(&project_dir, &id, entry.feature)?;
                    feature_start(&project_dir, &id)?;
                    println!("Feature started: {}", id);
                }
                FeatureCommands::Done { id } => {
                    feature_done(&project_dir, &id)?;
                    println!("Feature marked as implemented: {}", id);
                }
            }
        }
        Some(Commands::Workspace { action }) => {
            let project_dir = get_project_dir_cli()?;
            let project_root = project_dir.parent().unwrap_or(&project_dir).to_path_buf();
            match action {
                WorkspaceCommands::List => {
                    let workspaces = list_workspaces(&project_dir)?;
                    if workspaces.is_empty() {
                        println!("No workspaces found.");
                    } else {
                        for workspace in workspaces {
                            println!("{}", format_workspace_summary(&workspace));
                        }
                    }
                }
                WorkspaceCommands::Sync { branch } => {
                    let cwd = env::current_dir()?;
                    let branch = match branch {
                        Some(value) => value,
                        None => current_branch(&cwd)?,
                    };
                    let workspace = sync_workspace(&project_dir, &branch)?;
                    let context_root = resolve_workspace_context_root(&project_root, &workspace);
                    on_post_checkout(&project_dir, &workspace.branch, &context_root)?;
                    println!(
                        "Workspace synced: {} [{}]",
                        workspace.branch, workspace.status
                    );
                }
                WorkspaceCommands::Switch { branch, mode } => {
                    let existing_workspace = get_workspace(&project_dir, &branch)?;
                    let switch_targets_worktree = existing_workspace
                        .as_ref()
                        .map(|workspace| workspace.is_worktree)
                        .unwrap_or(false);

                    if !switch_targets_worktree {
                        let result = ProcessCommand::new("git")
                            .args(["checkout", &branch])
                            .current_dir(&project_root)
                            .status()?;
                        if !result.success() {
                            anyhow::bail!("Failed to checkout branch: {}", branch);
                        }
                    } else if let Some(workspace) = existing_workspace.as_ref() {
                        let context_root = resolve_workspace_context_root(&project_root, workspace);
                        if !context_root.exists() {
                            anyhow::bail!(
                                "Workspace '{}' is a worktree, but path does not exist: {}",
                                branch,
                                context_root.display()
                            );
                        }
                    }
                    let mut workspace = activate_workspace(&project_dir, &branch)?;
                    if let Some(mode_id) = mode.as_deref() {
                        workspace =
                            set_workspace_active_mode(&project_dir, &branch, Some(mode_id))?;
                    }
                    // Ensure context + provider config are regenerated after any
                    // workspace-level mode override is applied.
                    let context_root = resolve_workspace_context_root(&project_root, &workspace);
                    on_post_checkout(&project_dir, &workspace.branch, &context_root)?;
                    println!(
                        "Workspace active: {} [{}]",
                        workspace.branch, workspace.status
                    );
                }
                WorkspaceCommands::Create {
                    branch,
                    workspace_type,
                    feature,
                    feature_title,
                    spec,
                    release,
                    mode,
                    activate,
                    checkout,
                    worktree,
                    worktree_path,
                } => {
                    if worktree && checkout {
                        anyhow::bail!("--worktree and --checkout cannot be used together");
                    }
                    if worktree_path.is_some() && !worktree {
                        anyhow::bail!("--worktree-path requires --worktree");
                    }

                    let parsed_workspace_type = workspace_type
                        .as_deref()
                        .map(str::parse::<WorkspaceType>)
                        .transpose()?;
                    let is_feature_workspace = parsed_workspace_type
                        == Some(WorkspaceType::Feature)
                        || (parsed_workspace_type.is_none() && branch.starts_with("feature/"));
                    let resolved_worktree_path = if worktree {
                        Some(worktree_path.unwrap_or_else(|| {
                            let b = branch
                                .trim_start_matches("feature/")
                                .trim_start_matches("hotfix/");
                            format!("../{}", b)
                        }))
                    } else {
                        None
                    };

                    if worktree {
                        let path = resolved_worktree_path
                            .as_deref()
                            .ok_or_else(|| anyhow::anyhow!("Worktree path resolution failed"))?;
                        let exists = ProcessCommand::new("git")
                            .args(["rev-parse", "--verify", &branch])
                            .current_dir(&project_root)
                            .output()
                            .map(|output| output.status.success())
                            .unwrap_or(false);

                        let mut args = vec!["worktree", "add"];
                        if !exists {
                            args.push("-b");
                            args.push(&branch);
                            args.push(path);
                        } else {
                            args.push(path);
                            args.push(&branch);
                        }

                        let status = ProcessCommand::new("git")
                            .args(args)
                            .current_dir(&project_root)
                            .status()?;
                        if !status.success() {
                            if !exists {
                                let _ = ProcessCommand::new("git")
                                    .args(["branch", "-D", &branch])
                                    .current_dir(&project_root)
                                    .status();
                            }
                            anyhow::bail!("Failed to create git worktree: {}", branch);
                        }
                    } else if checkout {
                        let exists = ProcessCommand::new("git")
                            .args(["rev-parse", "--verify", &branch])
                            .current_dir(&project_root)
                            .output()
                            .map(|output| output.status.success())
                            .unwrap_or(false);
                        let checkout_status = if exists {
                            ProcessCommand::new("git")
                                .args(["checkout", &branch])
                                .current_dir(&project_root)
                                .status()?
                        } else {
                            ProcessCommand::new("git")
                                .args(["checkout", "-b", &branch])
                                .current_dir(&project_root)
                                .status()?
                        };
                        if !checkout_status.success() {
                            anyhow::bail!("Failed to create/switch branch: {}", branch);
                        }
                    }

                    let resolved_feature_id = resolve_workspace_feature_link(
                        &project_dir,
                        &branch,
                        feature,
                        feature_title,
                        is_feature_workspace,
                    )?;

                    let desired_status = if activate && !checkout && !worktree {
                        Some(WorkspaceStatus::Active)
                    } else {
                        None
                    };
                    let mut workspace = create_workspace(
                        &project_dir,
                        CreateWorkspaceRequest {
                            branch: branch.clone(),
                            workspace_type: parsed_workspace_type,
                            status: desired_status,
                            feature_id: resolved_feature_id,
                            spec_id: spec,
                            release_id: release,
                            active_mode: mode,
                            is_worktree: Some(worktree),
                            worktree_path: resolved_worktree_path,
                            ..CreateWorkspaceRequest::default()
                        },
                    )?;

                    if worktree || checkout {
                        workspace = sync_workspace(&project_dir, &branch)?;
                    } else if activate {
                        workspace = activate_workspace(&project_dir, &branch)?;
                    }

                    if worktree || checkout || workspace.status == WorkspaceStatus::Active {
                        let context_root =
                            resolve_workspace_context_root(&project_root, &workspace);
                        on_post_checkout(&project_dir, &workspace.branch, &context_root)?;
                    }

                    println!(
                        "Workspace {}: {} [{}]",
                        if workspace.status == WorkspaceStatus::Active {
                            "active"
                        } else {
                            "created"
                        },
                        workspace.branch,
                        workspace.status
                    );
                }
                WorkspaceCommands::Archive { branch } => {
                    let workspace = transition_workspace_status(
                        &project_dir,
                        &branch,
                        WorkspaceStatus::Archived,
                    )?;
                    println!(
                        "Workspace archived: {} [{}]",
                        workspace.branch, workspace.status
                    );
                }
                WorkspaceCommands::Session { action } => match action {
                    WorkspaceSessionCommands::Start {
                        branch,
                        goal,
                        mode,
                        provider,
                    } => {
                        let branch = branch.unwrap_or(current_branch(&project_root)?);
                        let session =
                            start_workspace_session(&project_dir, &branch, goal, mode, provider)?;
                        println!(
                            "Session started: {} [{}] branch={}{}{}{}",
                            session.id,
                            session.status,
                            session.workspace_branch,
                            session
                                .mode_id
                                .as_ref()
                                .map(|mode| format!(" mode={}", mode))
                                .unwrap_or_default(),
                            session
                                .primary_provider
                                .as_ref()
                                .map(|provider| format!(" provider={}", provider))
                                .unwrap_or_default(),
                            session
                                .goal
                                .as_ref()
                                .map(|goal| format!(" goal=\"{}\"", goal))
                                .unwrap_or_default()
                        );
                    }
                    WorkspaceSessionCommands::End {
                        branch,
                        summary,
                        updated_feature,
                        updated_spec,
                    } => {
                        let branch = branch.unwrap_or(current_branch(&project_root)?);
                        let session = end_workspace_session(
                            &project_dir,
                            &branch,
                            EndWorkspaceSessionRequest {
                                summary,
                                updated_feature_ids: updated_feature,
                                updated_spec_ids: updated_spec,
                            },
                        )?;
                        println!(
                            "Session ended: {} [{}] branch={}{}",
                            session.id,
                            session.status,
                            session.workspace_branch,
                            session
                                .summary
                                .as_ref()
                                .map(|summary| format!(" summary=\"{}\"", summary))
                                .unwrap_or_default()
                        );
                    }
                    WorkspaceSessionCommands::Status { branch } => {
                        let branch = branch.unwrap_or(current_branch(&project_root)?);
                        match get_active_workspace_session(&project_dir, &branch)? {
                            Some(session) => println!("{}", format_workspace_session(&session)),
                            None => println!("No active session for workspace '{}'.", branch),
                        }
                    }
                    WorkspaceSessionCommands::List { branch, limit } => {
                        let sessions =
                            list_workspace_sessions(&project_dir, branch.as_deref(), limit)?;
                        if sessions.is_empty() {
                            println!("No workspace sessions found.");
                        } else {
                            for session in sessions {
                                println!("{}", format_workspace_session(&session));
                            }
                        }
                    }
                },
                WorkspaceCommands::Open { branch, editor } => {
                    let branch = branch.unwrap_or(current_branch(&project_root)?);
                    let target_path =
                        resolve_workspace_open_path(&project_dir, &project_root, &branch)?;
                    let path_env = env::var("PATH").ok();
                    let selected_editor =
                        resolve_workspace_editor(editor.as_deref(), path_env.as_deref())?;
                    let status = ProcessCommand::new(selected_editor.binary)
                        .arg(&target_path)
                        .status()?;
                    if !status.success() {
                        anyhow::bail!(
                            "Failed to open {} with editor '{}'",
                            target_path.display(),
                            selected_editor.id
                        );
                    }
                    println!(
                        "Opened {} in {} ({})",
                        branch,
                        selected_editor.id,
                        target_path.display()
                    );
                }
            }
        }
        Some(Commands::Event { action }) => {
            let project_dir = get_project_dir_cli()?;
            let action = match action {
                EventCommands::List { since, limit } => {
                    cli_framework::EventAction::List { since, limit }
                }
                EventCommands::Ingest => cli_framework::EventAction::Ingest,
                EventCommands::Export { output } => cli_framework::EventAction::Export { output },
            };
            cli_framework::handle_event_action(action, &project_dir)?;
        }
        Some(Commands::Projects { action }) => match action {
            ProjectCommands::List => {
                let projects = list_registered_projects()?;
                for p in projects {
                    println!("- {} ({})", p.name, p.path.display());
                }
            }
            ProjectCommands::Track { name, path } => {
                register_project(name.clone(), path.clone())?;
                println!("Now tracking project: {} ({})", name, path.display());
            }
            ProjectCommands::Rename { path, name } => {
                rename_project(path.clone(), name.clone())?;
                println!("Renamed project at {} to {}", path.display(), name);
            }
            ProjectCommands::Untrack { path } => {
                unregister_project(path.clone())?;
                println!("Stopped tracking project: {}", path.display());
            }
        },
        Some(Commands::Demo { path }) => {
            let abs = std::fs::canonicalize(&path)
                .unwrap_or_else(|_| env::current_dir().unwrap_or_default().join(&path));
            let project_dir = init_demo_project(abs.clone())?;
            println!("Demo project ready at {}", project_dir.display());
            println!(
                "Point Ship at it with: SHIP_DIR={} ship issue list",
                project_dir.display()
            );
        }
        Some(Commands::Git { action }) => {
            let project_dir = get_project_dir_cli()?;
            match action {
                GitCommands::Status => {
                    let git = get_git_config(&project_dir)?;
                    let cats = [
                        "issues",
                        "releases",
                        "features",
                        "adrs",
                        "specs",
                        "notes",
                        "agents",
                        "ship.toml",
                        "templates",
                    ];
                    println!("Ship git commit settings:");
                    for cat in cats {
                        let state = if is_category_committed(&git, cat) {
                            "committed"
                        } else {
                            "local only"
                        };
                        println!("  {:<14} {}", cat, state);
                    }
                    println!("\n.gitignore: {}", project_dir.join(".gitignore").display());
                }
                GitCommands::Include { category } => {
                    set_category_committed(&project_dir, &category, true)?;
                    println!("{} will now be committed to git.", category);
                    println!(".ship/.gitignore updated.");
                }
                GitCommands::Exclude { category } => {
                    set_category_committed(&project_dir, &category, false)?;
                    println!("{} will now be local only (gitignored).", category);
                    println!(".ship/.gitignore updated.");
                }
                GitCommands::InstallHooks => {
                    let project_root = project_dir
                        .parent()
                        .ok_or_else(|| anyhow::anyhow!("Could not resolve project root"))?;
                    install_hooks(&project_root.join(".git"))?;
                    println!(
                        "Installed git hooks in {}",
                        project_root.join(".git/hooks").display()
                    );
                }
                GitCommands::PostCheckout { old, new, flag } => {
                    // Use CWD as project_root so worktrees write CLAUDE.md to the
                    // worktree directory, not the main repo root.
                    let cwd = env::current_dir()?;
                    let old_ref = old
                        .or_else(|| env::var("SHIP_GIT_OLD_REF").ok())
                        .or_else(|| env::var("GIT_OLD_REF").ok());
                    let _new_ref = new
                        .or_else(|| env::var("SHIP_GIT_NEW_REF").ok())
                        .or_else(|| env::var("GIT_NEW_REF").ok());
                    let _checkout_flag = flag
                        .or_else(|| env::var("SHIP_GIT_CHECKOUT_FLAG").ok())
                        .or_else(|| env::var("GIT_CHECKOUT_FLAG").ok());

                    let branch = if _new_ref.is_none() && _checkout_flag.is_none() {
                        old_ref
                    } else {
                        None
                    }
                    .or_else(|| env::var("SHIP_GIT_BRANCH").ok())
                    .unwrap_or(current_branch(&cwd)?);

                    on_post_checkout(&project_dir, &branch, &cwd)?;
                }
                GitCommands::Sync => {
                    let cwd = env::current_dir()?;
                    let branch = current_branch(&cwd)?;
                    on_post_checkout(&project_dir, &branch, &cwd)?;
                }
            }
        }
        Some(Commands::Ghost { action }) => {
            let project_dir = get_project_dir_cli()?;
            ensure_builtin_plugin_namespaces(&project_dir)?;
            match action {
                GhostCommands::Scan { dir } => {
                    let root = dir.unwrap_or_else(|| {
                        // project_dir is .ship/; go up one level to the repo root
                        project_dir.parent().unwrap_or(&project_dir).to_path_buf()
                    });
                    println!("Scanning {}...", root.display());
                    let result = ghost_issues::scan(&project_dir, &root)?;
                    let unpromoted = result.issues.iter().filter(|g| !g.promoted).count();
                    println!(
                        "Found {} ghost issue{} in {} file{}.",
                        unpromoted,
                        if unpromoted == 1 { "" } else { "s" },
                        {
                            let files: std::collections::HashSet<_> =
                                result.issues.iter().map(|g| &g.file).collect();
                            files.len()
                        },
                        if result.issues.len() == 1 { "" } else { "s" }
                    );
                    for g in result.issues.iter().filter(|g| !g.promoted).take(10) {
                        println!("  {}", g.display());
                    }
                    if unpromoted > 10 {
                        println!(
                            "  ... and {} more. Run `ship ghost report` for full list.",
                            unpromoted - 10
                        );
                    }
                }
                GhostCommands::Report => {
                    let report = ghost_issues::generate_report(&project_dir)?;
                    println!("{}", report);
                }
                GhostCommands::Promote { file, line } => {
                    let found = ghost_issues::mark_promoted(&project_dir, &file, line)?;
                    if found {
                        println!("Marked {}:{} as promoted.", file, line);
                        // Optionally create an issue
                        if let Ok(Some(scan)) = ghost_issues::load_last_scan(&project_dir) {
                            if let Some(g) = scan
                                .issues
                                .iter()
                                .find(|g| g.file == file && g.line == line)
                            {
                                let title = g.suggested_title();
                                let desc = format!(
                                    "Promoted from `{}:{}` ({}).\n\nOriginal comment: {}",
                                    g.file,
                                    g.line,
                                    g.kind.as_str(),
                                    g.text.trim()
                                );
                                let path = create_issue(
                                    &project_dir,
                                    &title,
                                    &desc,
                                    IssueStatus::Backlog,
                                    None,
                                    None,
                                    None,
                                    None,
                                )?;
                                println!("Created issue: {}", path.file_name);
                            }
                        }
                    } else {
                        println!(
                            "Ghost issue not found at {}:{}. Run `ship ghost scan` first.",
                            file, line
                        );
                    }
                }
            }
        }
        Some(Commands::Config { action }) => {
            let project_dir = get_project_dir(None).ok();
            match action {
                ConfigCommands::Status { action } => match action {
                    StatusCommands::List => {
                        let statuses = get_project_statuses(project_dir)?;
                        println!("Issue statuses:");
                        for s in statuses {
                            println!("  - {}", s);
                        }
                    }
                    StatusCommands::Add { name } => {
                        if let Some(p_dir) = project_dir.as_ref() {
                            log_action(&p_dir, "config status add", &name)?;
                        }
                        add_status(project_dir, &name)?;
                        println!("Added status: {}", name.to_lowercase().replace(' ', "-"));
                    }
                    StatusCommands::Remove { name } => {
                        remove_status(project_dir, &name)?;
                        println!("Removed status: {}", name);
                    }
                },
                ConfigCommands::Export { target } => {
                    let dir = project_dir.ok_or_else(|| {
                        anyhow::anyhow!("No Ship project found in current directory")
                    })?;
                    runtime::agents::export::export_to(dir, &target)?;
                    println!("Exported MCP server registry to {} config.", target);
                }
                ConfigCommands::Ai => {
                    let cfg = get_config(project_dir)?;
                    let ai = cfg.ai.unwrap_or_default();
                    println!("AI provider : {}", ai.effective_provider());
                    if let Some(path) = &ai.cli_path {
                        println!("CLI path    : {}", path);
                    } else {
                        println!("CLI path    : (default — uses provider name on PATH)");
                    }
                }
            }
        }
        Some(Commands::Mode { action }) => {
            let project_dir = get_project_dir(None).ok();
            let action = match action {
                ModeCommands::List => cli_framework::ModeAction::List,
                ModeCommands::Add { id, name } => cli_framework::ModeAction::Add { id, name },
                ModeCommands::Remove { id } => cli_framework::ModeAction::Remove { id },
                ModeCommands::Set { id } => cli_framework::ModeAction::Set { id },
                ModeCommands::Clear => cli_framework::ModeAction::Clear,
                ModeCommands::Get => cli_framework::ModeAction::Get,
            };
            cli_framework::handle_mode_action(action, project_dir)?;
        }
        Some(Commands::Time { action }) => {
            let project_dir = get_project_dir_cli()?;
            ensure_builtin_plugin_namespaces(&project_dir)?;
            handle_time_command(action, &project_dir)?;
        }
        Some(Commands::Mcp { action }) => {
            match action {
                None | Some(McpCommands::Serve) => {
                    // Handled by the main unitary binary as it requires async
                }
                Some(McpCommands::List) => {
                    let project_dir = get_project_dir_cli()?;
                    cli_framework::handle_mcp_action(cli_framework::McpAction::List, &project_dir)?;
                }
                Some(McpCommands::Export { target }) => {
                    let project_dir = get_project_dir_cli()?;
                    cli_framework::handle_mcp_action(
                        cli_framework::McpAction::Export { target },
                        &project_dir,
                    )?;
                }
                Some(McpCommands::Import { provider }) => {
                    let project_dir = get_project_dir_cli()?;
                    cli_framework::handle_mcp_action(
                        cli_framework::McpAction::Import { provider },
                        &project_dir,
                    )?;
                }
                Some(McpCommands::Add {
                    id,
                    name,
                    url,
                    disabled,
                }) => {
                    let project_dir = get_project_dir_cli()?;
                    cli_framework::handle_mcp_action(
                        cli_framework::McpAction::Add {
                            id,
                            name,
                            url,
                            disabled,
                        },
                        &project_dir,
                    )?;
                }
                Some(McpCommands::AddStdio {
                    id,
                    name,
                    command,
                    args,
                }) => {
                    let project_dir = get_project_dir_cli()?;
                    cli_framework::handle_mcp_action(
                        cli_framework::McpAction::AddStdio {
                            id,
                            name,
                            command,
                            args,
                        },
                        &project_dir,
                    )?;
                }
                Some(McpCommands::Remove { id }) => {
                    let project_dir = get_project_dir_cli()?;
                    cli_framework::handle_mcp_action(
                        cli_framework::McpAction::Remove { id },
                        &project_dir,
                    )?;
                }
            }
        }
        Some(Commands::Providers { action }) => {
            let project_dir = get_project_dir_cli()?;
            let action = match action {
                ProviderCommands::List => cli_framework::ProviderAction::List,
                ProviderCommands::Connect { id } => cli_framework::ProviderAction::Connect { id },
                ProviderCommands::Disconnect { id } => {
                    cli_framework::ProviderAction::Disconnect { id }
                }
                ProviderCommands::Detect => cli_framework::ProviderAction::Detect,
                ProviderCommands::Models { id } => cli_framework::ProviderAction::Models { id },
                ProviderCommands::Import { id } => cli_framework::ProviderAction::Import { id },
            };
            cli_framework::handle_provider_action(action, &project_dir)?;
        }
        Some(Commands::Dev { action }) => match action {
            DevCommands::Migrate { force } => {
                handle_migrate_command(force)?;
            }
        },
        None => match get_project_dir(None) {
            Ok(project_dir) => {
                println!("{}", render_workspace_home(&project_dir)?);
            }
            Err(_) => {
                println!("No Ship project found.");
                println!("Start here:");
                println!("  ship init .");
                println!(
                    "  ship workspace create feature/<name> --type feature --feature-title \"<Feature Title>\" --checkout --activate"
                );
            }
        },
    }

    Ok(())
}

fn handle_migrate_command(force: bool) -> Result<()> {
    let project_dir = get_project_dir_cli()?;
    let global_dir = get_global_dir()?;
    let global = migrate_global_state(&global_dir)?;
    let project = migrate_project_state(&project_dir)?;
    let issues = import_issues_from_files(&project_dir)?;
    let specs = import_specs_from_files(&project_dir)?;
    let config = migrate_json_config_file(&project_dir)?;
    let cleared_project_markers = runtime::clear_project_migration_meta(&project_dir)?;
    let cleared_global_markers = runtime::clear_global_migration_meta()?;
    ensure_user_notes_imported_once(true, true)?;
    ensure_project_imported_once(&project_dir, true, true)?;
    println!(
        "Migration complete{}:\n- file namespace copies: copied={} skipped={} conflicts={}\n- project DB: {} (applied {})\n- global DB: {} (applied {})\n- registry: {} -> {} entries (normalized {})\n- app_state paths normalized: {}\n- startup import markers reset: {} project marker{}, {} global marker{}\n- imported docs: {} issue{}, {} spec{}{}.",
        if force { " (forced)" } else { "" },
        project.files.copied_files,
        project.files.skipped_identical_files,
        project.files.conflict_files,
        project.db.db_path.display(),
        project.db.applied_migrations,
        global.db.db_path.display(),
        global.db.applied_migrations,
        global.registry_entries_before,
        global.registry_entries_after,
        global.normalized_paths,
        global.app_state_paths_normalized,
        cleared_project_markers,
        if cleared_project_markers == 1 {
            ""
        } else {
            "s"
        },
        cleared_global_markers,
        if cleared_global_markers == 1 { "" } else { "s" },
        issues,
        if issues == 1 { "" } else { "s" },
        specs,
        if specs == 1 { "" } else { "s" },
        if config {
            ", config.json → ship.toml"
        } else {
            ""
        },
    );
    Ok(())
}

fn current_branch(project_root: &std::path::Path) -> Result<String> {
    let output = ProcessCommand::new("git")
        .args(["branch", "--show-current"])
        .current_dir(project_root)
        .output()?;
    if !output.status.success() {
        anyhow::bail!("Failed to determine current git branch");
    }
    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if branch.is_empty() {
        anyhow::bail!("Current HEAD is detached; cannot map to a feature branch");
    }
    Ok(branch)
}

fn branch_to_feature_title(branch: &str) -> String {
    let core = branch
        .trim()
        .strip_prefix("feature/")
        .unwrap_or(branch)
        .trim();
    let words: Vec<String> = core
        .split(['/', '-', '_'])
        .filter(|segment| !segment.trim().is_empty())
        .map(|segment| {
            let mut chars = segment.chars();
            match chars.next() {
                Some(first) => {
                    let mut title = first.to_uppercase().to_string();
                    title.push_str(chars.as_str());
                    title
                }
                None => String::new(),
            }
        })
        .filter(|segment| !segment.is_empty())
        .collect();

    if words.is_empty() {
        "Workspace Feature".to_string()
    } else {
        words.join(" ")
    }
}

fn resolve_workspace_feature_link(
    project_dir: &Path,
    branch: &str,
    feature_id: Option<String>,
    feature_title: Option<String>,
    is_feature_workspace: bool,
) -> Result<Option<String>> {
    if let Some(feature_id) = feature_id {
        if let Ok(feature) = get_feature_by_id(project_dir, &feature_id)
            && feature
                .feature
                .metadata
                .branch
                .as_deref()
                .map(|value| value != branch)
                .unwrap_or(true)
        {
            let mut updated = feature.feature;
            updated.metadata.branch = Some(branch.to_string());
            update_feature(project_dir, &feature_id, updated)?;
        }
        return Ok(Some(feature_id));
    }

    if !is_feature_workspace {
        return Ok(None);
    }

    let existing = list_features(project_dir)?
        .into_iter()
        .find(|entry| entry.feature.metadata.branch.as_deref() == Some(branch));
    if let Some(existing) = existing {
        return Ok(Some(existing.id));
    }

    let title = feature_title
        .as_deref()
        .map(str::trim)
        .filter(|title| !title.is_empty())
        .map(|title| title.to_string())
        .unwrap_or_else(|| branch_to_feature_title(branch));
    let created = create_feature(project_dir, &title, "", None, None, Some(branch))?;
    println!(
        "Feature created and linked: {} ({})",
        created.feature.metadata.title, created.id
    );
    Ok(Some(created.id))
}

fn get_project_dir_cli() -> Result<PathBuf> {
    get_project_dir(None)
}

fn resolve_workspace_context_root(
    project_root: &Path,
    workspace: &runtime::workspace::Workspace,
) -> PathBuf {
    if workspace.is_worktree
        && let Some(path) = workspace.worktree_path.as_deref()
    {
        let candidate = PathBuf::from(path);
        if candidate.is_absolute() {
            candidate
        } else {
            project_root.join(candidate)
        }
    } else {
        project_root.to_path_buf()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WorkspaceEditor {
    id: &'static str,
    binary: &'static str,
    aliases: &'static [&'static str],
}

const WORKSPACE_EDITORS: &[WorkspaceEditor] = &[
    WorkspaceEditor {
        id: "cursor",
        binary: "cursor",
        aliases: &["cursor"],
    },
    WorkspaceEditor {
        id: "vscode",
        binary: "code",
        aliases: &["vscode", "code", "vs-code"],
    },
    WorkspaceEditor {
        id: "zed",
        binary: "zed",
        aliases: &["zed"],
    },
];

fn normalize_workspace_editor_id(raw: &str) -> Option<&'static str> {
    let normalized = raw.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return None;
    }
    WORKSPACE_EDITORS
        .iter()
        .find(|editor| {
            editor.id.eq_ignore_ascii_case(&normalized)
                || editor
                    .aliases
                    .iter()
                    .any(|alias| alias.eq_ignore_ascii_case(&normalized))
        })
        .map(|editor| editor.id)
}

fn command_exists_in_path(binary: &str, path_env: Option<&str>) -> bool {
    let Some(path_env) = path_env else {
        return false;
    };
    for dir in env::split_paths(path_env) {
        let path = dir.join(binary);
        if path.is_file() {
            return true;
        }
        #[cfg(windows)]
        {
            let path_exe = dir.join(format!("{}.exe", binary));
            if path_exe.is_file() {
                return true;
            }
        }
    }
    false
}

fn available_workspace_editors(path_env: Option<&str>) -> Vec<WorkspaceEditor> {
    WORKSPACE_EDITORS
        .iter()
        .copied()
        .filter(|editor| command_exists_in_path(editor.binary, path_env))
        .collect()
}

fn resolve_workspace_editor(
    requested: Option<&str>,
    path_env: Option<&str>,
) -> Result<WorkspaceEditor> {
    let available = available_workspace_editors(path_env);
    if let Some(raw) = requested {
        let normalized = normalize_workspace_editor_id(raw).ok_or_else(|| {
            anyhow::anyhow!("Unknown editor '{}'. Use cursor, vscode, or zed", raw)
        })?;
        return available
            .into_iter()
            .find(|editor| editor.id == normalized)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Editor '{}' is not installed in PATH. Available: {}",
                    normalized,
                    WORKSPACE_EDITORS
                        .iter()
                        .map(|editor| editor.id)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            });
    }

    available.into_iter().next().ok_or_else(|| {
        anyhow::anyhow!(
            "No supported editors found in PATH (looked for: {})",
            WORKSPACE_EDITORS
                .iter()
                .map(|editor| editor.binary)
                .collect::<Vec<_>>()
                .join(", ")
        )
    })
}

fn resolve_workspace_open_path(
    project_dir: &Path,
    project_root: &Path,
    branch: &str,
) -> Result<PathBuf> {
    if let Some(workspace) = get_workspace(project_dir, branch)?
        && workspace.is_worktree
        && let Some(path) = workspace.worktree_path.as_deref()
    {
        let candidate = PathBuf::from(path);
        let resolved = if candidate.is_absolute() {
            candidate
        } else {
            project_root.join(candidate)
        };
        if !resolved.exists() {
            return Err(anyhow::anyhow!(
                "Workspace worktree path does not exist: {}",
                resolved.display()
            ));
        }
        return Ok(resolved);
    }

    Ok(project_root.to_path_buf())
}

fn ensure_project_imported_once(project_dir: &Path, force: bool, strict: bool) -> Result<()> {
    run_project_import(project_dir, "adr", "ADRs", force, strict, || {
        import_adrs_from_files(project_dir)
    })?;
    run_project_import(
        project_dir,
        "note_project",
        "project notes",
        force,
        strict,
        || import_notes_from_files(NoteScope::Project, Some(project_dir)),
    )?;
    run_project_import(project_dir, "feature", "features", force, strict, || {
        import_features_from_files(project_dir)
    })?;
    run_project_import(project_dir, "release", "releases", force, strict, || {
        import_releases_from_files(project_dir)
    })?;
    Ok(())
}

fn ensure_user_notes_imported_once(force: bool, strict: bool) -> Result<()> {
    if !force && runtime::migration_meta_complete_global("note_user")? {
        return Ok(());
    }

    match import_notes_from_files(NoteScope::User, None) {
        Ok(count) => {
            runtime::mark_migration_meta_complete_global("note_user", count)?;
            if count > 0 {
                println!(
                    "[ship] Imported {} global notes from files to SQLite",
                    count
                );
            }
            Ok(())
        }
        Err(err) if strict => Err(err),
        Err(err) => {
            eprintln!(
                "[ship] warning: failed to import global notes from files: {}",
                err
            );
            Ok(())
        }
    }
}

fn run_project_import<F>(
    project_dir: &Path,
    entity_type: &str,
    label: &str,
    force: bool,
    strict: bool,
    importer: F,
) -> Result<()>
where
    F: FnOnce() -> Result<usize>,
{
    if !force && runtime::migration_meta_complete_project(project_dir, entity_type)? {
        return Ok(());
    }

    match importer() {
        Ok(count) => {
            runtime::mark_migration_meta_complete_project(project_dir, entity_type, count)?;
            if count > 0 {
                println!("[ship] Imported {} {} from files to SQLite", count, label);
            }
            Ok(())
        }
        Err(err) if strict => Err(err),
        Err(err) => {
            eprintln!(
                "[ship] warning: failed to import {} from files: {}",
                label, err
            );
            Ok(())
        }
    }
}

fn parse_note_scope(raw: &str) -> Result<NoteScope> {
    raw.parse::<NoteScope>()
}

fn format_workspace_summary(workspace: &runtime::workspace::Workspace) -> String {
    let mut parts = vec![format!(
        "[{}] {} ({})",
        workspace.status, workspace.branch, workspace.workspace_type
    )];

    if let Some(feature_id) = workspace.feature_id.as_deref() {
        parts.push(format!("feature={}", feature_id));
    }
    if let Some(spec_id) = workspace.spec_id.as_deref() {
        parts.push(format!("spec={}", spec_id));
    }
    if let Some(release_id) = workspace.release_id.as_deref() {
        parts.push(format!("release={}", release_id));
    }
    if let Some(mode) = workspace.active_mode.as_deref() {
        parts.push(format!("mode={}", mode));
    }
    if workspace.is_worktree {
        if let Some(path) = workspace.worktree_path.as_deref() {
            parts.push(format!("worktree={}", path));
        } else {
            parts.push("worktree=true".to_string());
        }
    }

    parts.join(" ")
}

fn format_workspace_session(session: &runtime::WorkspaceSession) -> String {
    let mut parts = vec![format!(
        "[{}] {} workspace={} started={}",
        session.status, session.id, session.workspace_branch, session.started_at
    )];

    if let Some(ended_at) = session.ended_at.as_ref() {
        parts.push(format!("ended={}", ended_at));
    }
    if let Some(mode_id) = session.mode_id.as_deref() {
        parts.push(format!("mode={}", mode_id));
    }
    if let Some(provider) = session.primary_provider.as_deref() {
        parts.push(format!("provider={}", provider));
    }
    if let Some(goal) = session.goal.as_deref() {
        parts.push(format!("goal=\"{}\"", goal));
    }
    if let Some(summary) = session.summary.as_deref() {
        parts.push(format!("summary=\"{}\"", summary));
    }
    if !session.updated_feature_ids.is_empty() {
        parts.push(format!(
            "features=[{}]",
            session.updated_feature_ids.join(",")
        ));
    }
    if !session.updated_spec_ids.is_empty() {
        parts.push(format!("specs=[{}]", session.updated_spec_ids.join(",")));
    }
    if let Some(compiled_at) = session.compiled_at.as_ref() {
        parts.push(format!("compiled_at={}", compiled_at));
    }
    if let Some(compile_error) = session.compile_error.as_deref() {
        parts.push(format!("compile_error=\"{}\"", compile_error));
    }

    parts.join(" ")
}

fn render_workspace_home(project_dir: &Path) -> Result<String> {
    let workspaces = list_workspaces(project_dir)?;
    let mut out = String::new();

    out.push_str("Workspace Home\n");
    out.push_str("--------------\n");

    if workspaces.is_empty() {
        out.push_str("No workspaces found.\n");
        out.push_str("Start here:\n");
        out.push_str(
            "  ship workspace create feature/<name> --type feature --feature-title \"<Feature Title>\" --checkout --activate\n",
        );
        out.push_str("  ship workspace list\n");
        return Ok(out);
    }

    out.push_str("Workspaces:\n");
    for workspace in &workspaces {
        out.push_str(&format!("  {}\n", format_workspace_summary(workspace)));
    }

    out.push_str("\nNext:\n");
    out.push_str("  ship workspace switch <branch>\n");
    out.push_str("  ship workspace open --branch <branch> --editor <cursor|vscode|zed>\n");
    out.push_str(
        "  ship workspace session start --branch <branch> --goal \"<goal>\" --provider <id>\n",
    );
    out.push_str("  ship workspace session end --branch <branch> --summary \"<summary>\"\n");
    out.push_str("  ship workspace sync --branch <branch>\n");

    Ok(out)
}

fn ensure_builtin_plugin_namespaces(project_dir: &PathBuf) -> Result<()> {
    let mut registry = runtime::PluginRegistry::new();
    registry.register_with_project(project_dir, Box::new(ghost_issues::GhostIssues))?;
    registry.register_with_project(project_dir, Box::new(time_tracker::TimeTracker))?;
    Ok(())
}

fn handle_time_command(action: TimeCommands, project_dir: &PathBuf) -> Result<()> {
    use time_tracker::{
        format_duration, generate_report, get_active_timer, list_entries, log_time, start_timer,
        stop_timer,
    };

    match action {
        TimeCommands::Start { issue, note } => {
            // issue can be a filename or a path — normalise to just the filename
            let issue_path = PathBuf::from(&issue);
            let issue_file = issue_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or(issue.clone());

            // Try to read the issue title from the file
            let issue_title = {
                let path = if issue_path.is_absolute() {
                    issue_path.clone()
                } else {
                    // Search through statuses
                    let mut found = None;
                    for status in ISSUE_STATUSES {
                        let p = runtime::project::issues_dir(project_dir)
                            .join(status)
                            .join(&issue_file);
                        if p.exists() {
                            found = Some(p);
                            break;
                        }
                    }
                    found.unwrap_or(issue_path)
                };
                if path.exists() {
                    get_issue_by_id(project_dir, &issue_file)
                        .ok()
                        .map(|i| i.issue.metadata.title)
                        .unwrap_or_else(|| issue_file.clone())
                } else {
                    issue_file.clone()
                }
            };

            let timer = start_timer(project_dir, &issue_file, &issue_title, note)?;
            println!(
                "Timer started: {} ({})",
                timer.issue_title,
                timer.started_at.format("%H:%M")
            );
        }
        TimeCommands::Stop { note } => {
            let entry = stop_timer(project_dir, note)?;
            println!(
                "Timer stopped: {} — {}",
                entry.issue_title,
                format_duration(entry.duration_minutes)
            );
        }
        TimeCommands::Status => match get_active_timer(project_dir)? {
            Some(t) => {
                let elapsed = (chrono::Utc::now() - t.started_at).num_minutes().max(0) as u64;
                println!(
                    "Running: {} (started {}, elapsed {})",
                    t.issue_title,
                    t.started_at.format("%H:%M"),
                    format_duration(elapsed)
                );
            }
            None => println!("No timer running."),
        },
        TimeCommands::Log {
            issue,
            minutes,
            note,
        } => {
            let issue_path = PathBuf::from(&issue);
            let issue_file = issue_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or(issue.clone());
            let entry = log_time(project_dir, &issue_file, &issue_file, minutes, note)?;
            println!(
                "Logged {} for {}",
                format_duration(entry.duration_minutes),
                entry.issue_title
            );
        }
        TimeCommands::List { issue } => {
            let entries = list_entries(project_dir, issue.as_deref())?;
            if entries.is_empty() {
                println!("No time entries.");
            } else {
                for e in &entries {
                    println!(
                        "[{}] {} — {}{}",
                        e.started_at.format("%Y-%m-%d"),
                        e.issue_title,
                        format_duration(e.duration_minutes),
                        e.note
                            .as_deref()
                            .map(|n| format!(" ({})", n))
                            .unwrap_or_default()
                    );
                }
            }
        }
        TimeCommands::Report => {
            let report = generate_report(project_dir)?;
            println!("{}", report);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use tempfile::tempdir;

    fn write_feature_file(ship_dir: &Path, id: &str, title: &str, file_name: &str) -> Result<()> {
        let path = runtime::project::features_dir(ship_dir)
            .join("planned")
            .join(file_name);
        std::fs::create_dir_all(path.parent().unwrap())?;
        std::fs::write(
            path,
            format!(
                "+++\nid = \"{}\"\ntitle = \"{}\"\ncreated = \"2026-01-01T00:00:00Z\"\nupdated = \"2026-01-01T00:00:00Z\"\ntags = []\n+++\n\nbody\n",
                id, title
            ),
        )?;
        Ok(())
    }

    #[test]
    fn ensure_project_imported_once_skips_after_marker_and_force_reimports() -> Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;

        write_feature_file(
            &project_dir,
            "feature-startup-1",
            "Startup Import One",
            "startup-import-one.md",
        )?;

        ensure_project_imported_once(&project_dir, false, true)?;
        assert!(runtime::migration_meta_complete_project(
            &project_dir,
            "feature"
        )?);
        assert_eq!(list_features(&project_dir)?.len(), 1);

        write_feature_file(
            &project_dir,
            "feature-startup-2",
            "Startup Import Two",
            "startup-import-two.md",
        )?;

        // Marker is already set, so regular startup import should skip re-scan.
        ensure_project_imported_once(&project_dir, false, true)?;
        assert_eq!(list_features(&project_dir)?.len(), 1);

        ensure_project_imported_once(&project_dir, true, true)?;
        assert_eq!(list_features(&project_dir)?.len(), 2);
        Ok(())
    }

    #[test]
    fn render_workspace_home_empty_guides_creation() -> Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;

        let rendered = render_workspace_home(&project_dir)?;
        assert!(rendered.contains("No workspaces found."));
        assert!(rendered.contains("ship workspace create"));
        Ok(())
    }

    #[test]
    fn render_workspace_home_lists_existing_workspaces() -> Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &project_dir,
            CreateWorkspaceRequest {
                branch: "feature/workspace-home".to_string(),
                workspace_type: Some(WorkspaceType::Feature),
                ..CreateWorkspaceRequest::default()
            },
        )?;
        activate_workspace(&project_dir, "feature/workspace-home")?;

        let rendered = render_workspace_home(&project_dir)?;
        assert!(rendered.contains("[active] feature/workspace-home (feature)"));
        assert!(rendered.contains("ship workspace switch <branch>"));
        Ok(())
    }

    #[test]
    fn branch_to_feature_title_normalizes_branch_name() {
        assert_eq!(
            branch_to_feature_title("feature/workspace-first-launch"),
            "Workspace First Launch"
        );
        assert_eq!(
            branch_to_feature_title("feature/nested/workspace_flow"),
            "Nested Workspace Flow"
        );
    }

    #[test]
    fn cli_parses_projects_rename_subcommand() {
        let cli = Cli::try_parse_from(["ship", "projects", "rename", "/tmp/project", "ship-core"])
            .expect("projects rename should parse");

        match cli.command {
            Some(Commands::Projects {
                action: ProjectCommands::Rename { path, name },
            }) => {
                assert_eq!(path, PathBuf::from("/tmp/project"));
                assert_eq!(name, "ship-core");
            }
            other => panic!("unexpected parse result: {:?}", other),
        }
    }

    #[test]
    fn cli_parses_skill_install_subcommand() {
        let cli = Cli::try_parse_from([
            "ship",
            "skill",
            "install",
            "vercel-labs/agent-skills",
            "vercel-react-best-practices",
            "--scope",
            "user",
            "--repo-path",
            "skills",
            "--git-ref",
            "main",
            "--force",
        ])
        .expect("skill install should parse");

        match cli.command {
            Some(Commands::Skill {
                action:
                    SkillCommands::Install {
                        source,
                        id,
                        git_ref,
                        repo_path,
                        scope,
                        force,
                    },
            }) => {
                assert_eq!(source, "vercel-labs/agent-skills");
                assert_eq!(id, "vercel-react-best-practices");
                assert_eq!(git_ref, "main");
                assert_eq!(repo_path, "skills");
                assert_eq!(scope, "user");
                assert!(force);
            }
            other => panic!("unexpected parse result: {:?}", other),
        }
    }

    #[test]
    fn cli_parses_spec_create_with_workspace_override() {
        let cli = Cli::try_parse_from([
            "ship",
            "spec",
            "create",
            "Execution Plan",
            "--workspace",
            "feature/execution-plan",
        ])
        .expect("spec create with workspace should parse");

        match cli.command {
            Some(Commands::Spec {
                action:
                    SpecCommands::Create {
                        title, workspace, ..
                    },
            }) => {
                assert_eq!(title, "Execution Plan");
                assert_eq!(workspace.as_deref(), Some("feature/execution-plan"));
            }
            other => panic!("unexpected parse result: {:?}", other),
        }
    }

    #[test]
    fn cli_parses_workspace_create_with_feature_title() {
        let cli = Cli::try_parse_from([
            "ship",
            "workspace",
            "create",
            "feature/workspace-first",
            "--type",
            "feature",
            "--feature-title",
            "Workspace First",
            "--checkout",
            "--activate",
        ])
        .expect("workspace create with feature title should parse");

        match cli.command {
            Some(Commands::Workspace {
                action:
                    WorkspaceCommands::Create {
                        branch,
                        workspace_type,
                        feature_title,
                        checkout,
                        activate,
                        ..
                    },
            }) => {
                assert_eq!(branch, "feature/workspace-first");
                assert_eq!(workspace_type.as_deref(), Some("feature"));
                assert_eq!(feature_title.as_deref(), Some("Workspace First"));
                assert!(checkout);
                assert!(activate);
            }
            other => panic!("unexpected parse result: {:?}", other),
        }
    }

    #[test]
    fn cli_parses_workspace_session_start() {
        let cli = Cli::try_parse_from([
            "ship",
            "workspace",
            "session",
            "start",
            "--branch",
            "feature/session-flow",
            "--goal",
            "Implement parser",
            "--mode",
            "code",
            "--provider",
            "claude",
        ])
        .expect("workspace session start should parse");

        match cli.command {
            Some(Commands::Workspace {
                action:
                    WorkspaceCommands::Session {
                        action:
                            WorkspaceSessionCommands::Start {
                                branch,
                                goal,
                                mode,
                                provider,
                            },
                    },
            }) => {
                assert_eq!(branch.as_deref(), Some("feature/session-flow"));
                assert_eq!(goal.as_deref(), Some("Implement parser"));
                assert_eq!(mode.as_deref(), Some("code"));
                assert_eq!(provider.as_deref(), Some("claude"));
            }
            other => panic!("unexpected parse result: {:?}", other),
        }
    }

    #[test]
    fn cli_parses_workspace_session_end_with_updates() {
        let cli = Cli::try_parse_from([
            "ship",
            "workspace",
            "session",
            "end",
            "--summary",
            "Done",
            "--updated-feature",
            "feat-auth",
            "--updated-feature",
            "feat-session",
            "--updated-spec",
            "spec-auth",
        ])
        .expect("workspace session end should parse");

        match cli.command {
            Some(Commands::Workspace {
                action:
                    WorkspaceCommands::Session {
                        action:
                            WorkspaceSessionCommands::End {
                                branch,
                                summary,
                                updated_feature,
                                updated_spec,
                            },
                    },
            }) => {
                assert!(branch.is_none());
                assert_eq!(summary.as_deref(), Some("Done"));
                assert_eq!(updated_feature, vec!["feat-auth", "feat-session"]);
                assert_eq!(updated_spec, vec!["spec-auth"]);
            }
            other => panic!("unexpected parse result: {:?}", other),
        }
    }

    #[test]
    fn cli_parses_workspace_open_command() {
        let cli = Cli::try_parse_from([
            "ship",
            "workspace",
            "open",
            "--branch",
            "feature/session-flow",
            "--editor",
            "vscode",
        ])
        .expect("workspace open should parse");

        match cli.command {
            Some(Commands::Workspace {
                action: WorkspaceCommands::Open { branch, editor },
            }) => {
                assert_eq!(branch.as_deref(), Some("feature/session-flow"));
                assert_eq!(editor.as_deref(), Some("vscode"));
            }
            other => panic!("unexpected parse result: {:?}", other),
        }
    }

    #[test]
    fn normalize_workspace_editor_id_supports_aliases() {
        assert_eq!(normalize_workspace_editor_id("code"), Some("vscode"));
        assert_eq!(normalize_workspace_editor_id("VSCode"), Some("vscode"));
        assert_eq!(normalize_workspace_editor_id("cursor"), Some("cursor"));
        assert_eq!(normalize_workspace_editor_id("zed"), Some("zed"));
        assert_eq!(normalize_workspace_editor_id(""), None);
        assert_eq!(normalize_workspace_editor_id("ghost"), None);
    }

    #[test]
    fn resolve_workspace_editor_prefers_installed_priority_and_honors_requested_id() -> Result<()> {
        let tmp = tempdir()?;
        std::fs::write(tmp.path().join("cursor"), "")?;
        std::fs::write(tmp.path().join("code"), "")?;
        let path_env = tmp.path().to_string_lossy().to_string();

        let auto = resolve_workspace_editor(None, Some(&path_env))?;
        assert_eq!(auto.id, "cursor");

        let vscode = resolve_workspace_editor(Some("code"), Some(&path_env))?;
        assert_eq!(vscode.id, "vscode");

        let err = resolve_workspace_editor(Some("zed"), Some(&path_env)).unwrap_err();
        assert!(
            err.to_string()
                .contains("Editor 'zed' is not installed in PATH")
        );
        Ok(())
    }

    #[test]
    fn resolve_workspace_open_path_prefers_workspace_worktree() -> Result<()> {
        let tmp = tempdir()?;
        let project_root = tmp.path().join("repo");
        std::fs::create_dir_all(&project_root)?;
        let ship_dir = init_project(project_root.clone())?;
        let worktree_dir = tmp.path().join("wt-feature");
        std::fs::create_dir_all(&worktree_dir)?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/open-worktree".to_string(),
                status: Some(WorkspaceStatus::Active),
                is_worktree: Some(true),
                worktree_path: Some(worktree_dir.to_string_lossy().to_string()),
                ..CreateWorkspaceRequest::default()
            },
        )?;

        let resolved =
            resolve_workspace_open_path(&ship_dir, &project_root, "feature/open-worktree")?;
        assert_eq!(resolved, worktree_dir);
        Ok(())
    }

    #[test]
    fn cli_parses_providers_import_subcommand() {
        let with_id = Cli::try_parse_from(["ship", "providers", "import", "codex"])
            .expect("providers import <id> should parse");
        match with_id.command {
            Some(Commands::Providers {
                action: ProviderCommands::Import { id },
            }) => assert_eq!(id.as_deref(), Some("codex")),
            other => panic!("unexpected parse result: {:?}", other),
        }

        let no_id = Cli::try_parse_from(["ship", "providers", "import"])
            .expect("providers import should parse");
        match no_id.command {
            Some(Commands::Providers {
                action: ProviderCommands::Import { id },
            }) => assert!(id.is_none()),
            other => panic!("unexpected parse result: {:?}", other),
        }
    }

    #[test]
    fn cli_parses_mcp_export_subcommand() {
        let cli = Cli::try_parse_from(["ship", "mcp", "export", "--target", "codex"])
            .expect("mcp export should parse");
        match cli.command {
            Some(Commands::Mcp {
                action: Some(McpCommands::Export { target }),
            }) => assert_eq!(target, "codex"),
            other => panic!("unexpected parse result: {:?}", other),
        }
    }

    #[test]
    fn cli_parses_mcp_import_subcommand() {
        let cli = Cli::try_parse_from(["ship", "mcp", "import", "gemini"])
            .expect("mcp import should parse");
        match cli.command {
            Some(Commands::Mcp {
                action: Some(McpCommands::Import { provider }),
            }) => assert_eq!(provider, "gemini"),
            other => panic!("unexpected parse result: {:?}", other),
        }
    }

    #[test]
    fn cli_parses_dev_migrate_subcommand() {
        let cli = Cli::try_parse_from(["ship", "dev", "migrate", "--force"])
            .expect("dev migrate should parse");
        match cli.command {
            Some(Commands::Dev {
                action: DevCommands::Migrate { force },
            }) => assert!(force),
            other => panic!("unexpected parse result: {:?}", other),
        }
    }

    #[test]
    fn cli_rejects_legacy_top_level_migrate_command() {
        let err = Cli::try_parse_from(["ship", "migrate"])
            .expect_err("top-level migrate command should be rejected");
        let message = err.to_string();
        assert!(
            message.contains("unrecognized subcommand"),
            "unexpected clap error: {}",
            message
        );
    }
}
