use anyhow::Result;
use runtime::project::{get_global_dir, get_project_dir, ship_dir_from_path};
use runtime::workspace::set_workspace_active_mode;
use runtime::{
    CreateWorkspaceRequest, EndWorkspaceSessionRequest, ShipWorkspaceKind, WorkspaceStatus,
    activate_workspace, add_status, autodetect_providers, create_workspace, end_workspace_session,
    get_active_workspace_session, get_config, get_git_config, get_project_statuses, get_workspace,
    is_category_committed, list_mcp_servers, list_providers, list_workspace_sessions,
    list_workspaces, log_action, migrate_global_state, migrate_json_config_file,
    migrate_project_state, record_workspace_session_progress, remove_status, repair_workspace,
    set_category_committed, start_workspace_session, sync_workspace, transition_workspace_status,
};
use ship_module_git::{install_hooks, on_post_checkout, write_root_gitignore};
use ship_module_project::ops::adr::{create_adr, find_adr_path, list_adrs, move_adr};
use ship_module_project::ops::feature::{
    create_feature, delete_feature, ensure_feature_documentation, feature_done, feature_start,
    get_feature_by_id, get_feature_documentation, list_features, sync_feature_docs_after_session,
    update_feature, update_feature_documentation,
};
use ship_module_project::ops::note::{
    create_note, get_note_by_id, list_notes, update_note_content,
};
use ship_module_project::ops::release::{
    create_release, get_release_by_id, list_releases, update_release,
};
use ship_module_project::ops::spec::{create_spec, get_spec_by_id, list_specs, update_spec};
use ship_module_project::{
    ADR, AdrStatus, FeatureDocStatus, FeatureStatus, NoteScope, import_adrs_from_files,
    import_features_from_files, import_notes_from_files, import_releases_from_files,
    import_specs_from_files, init_demo_project, init_project, list_registered_projects,
    register_project, rename_project, unregister_project,
};
use std::collections::HashMap;
use std::env;
use std::io::{self, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

use crate::surface::*;

pub fn handle_init_command(target: cli_framework::InitTarget) -> Result<()> {
    let mut project_name = target.project_name.clone();
    if io::stdin().is_terminal() && io::stdout().is_terminal() {
        project_name = prompt_with_default("Project name", &project_name)?;
    }

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

    let tracked = match register_project(project_name.clone(), target.path.clone()) {
        Ok(()) => true,
        Err(err) => {
            eprintln!(
                "[ship] warning: initialized project but failed to register globally: {}",
                err
            );
            eprintln!(
                "[ship] run `ship projects track {} {}` later to add it to the global registry",
                project_name,
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
    // Keep file import off hot paths where project data isn't needed.
    let skip_auto_import = matches!(
        &cli.command,
        None | Some(
            Commands::Init { .. }
                | Commands::Doctor
                | Commands::Version
                | Commands::Ui { .. }
                | Commands::Projects { .. }
                | Commands::Providers { .. }
                | Commands::Mcp { .. }
                | Commands::Hooks { .. }
        )
    );
    if !skip_auto_import {
        let _ = ensure_user_notes_imported_once(false, false);
        if let Ok(project_dir) = get_project_dir(None) {
            let _ = ensure_project_imported_once(&project_dir, false, false);
        }
    }

    match cli.command {
        Some(Commands::Init { .. } | Commands::Doctor | Commands::Version) => anyhow::bail!(
            "core command should be handled by cli-framework before app command dispatch"
        ),
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
                SpecCommands::Update { file_name, content } => {
                    let entry = get_spec_by_id(&project_dir, &file_name)?;
                    let body = match content {
                        Some(c) => c,
                        None => {
                            use std::io::Read;
                            let mut buf = String::new();
                            std::io::stdin().read_to_string(&mut buf)?;
                            buf
                        }
                    };
                    let mut spec = entry.spec.clone();
                    spec.body = body;
                    update_spec(&project_dir, &entry.id, spec)?;
                    println!("Spec updated: {}", file_name);
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
                    let release_path =
                        runtime::project::releases_dir(&project_dir).join(&entry.file_name);
                    if !release_path.exists() {
                        anyhow::bail!("Release file not found: {}", entry.file_name);
                    }
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
                    let m = &entry.feature.metadata;
                    println!("id = \"{}\"", m.id);
                    println!("title = \"{}\"", m.title);
                    println!("status = \"{}\"", entry.status);
                    if let Some(ref branch) = m.branch {
                        println!("branch = \"{}\"", branch);
                    }
                    if let Some(ref release_id) = m.release_id {
                        println!("release_id = \"{}\"", release_id);
                    }
                    if let Some(ref spec_id) = m.spec_id {
                        println!("spec_id = \"{}\"", spec_id);
                    }
                    println!("updated = \"{}\"", m.updated);
                    if !entry.feature.body.trim().is_empty() {
                        println!("\n---\n\n{}", entry.feature.body.trim());
                    }
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
                FeatureCommands::Delete { id } => {
                    delete_feature(&project_dir, &id)?;
                    println!("Feature deleted: {}", id);
                }
                FeatureCommands::Docs { action } => match action {
                    FeatureDocCommands::EnsureAll => {
                        let features = list_features(&project_dir)?;
                        let mut ensured = 0usize;
                        for feature in features {
                            ensure_feature_documentation(&project_dir, &feature.id)?;
                            ensured += 1;
                        }
                        println!("Ensured documentation for {} feature(s).", ensured);
                    }
                    FeatureDocCommands::Get { id } => {
                        let doc = get_feature_documentation(&project_dir, &id)?;
                        println!("{}", doc.content);
                    }
                    FeatureDocCommands::Update {
                        id,
                        content,
                        status,
                        verify,
                    } => {
                        let parsed_status = status
                            .as_deref()
                            .map(|value| value.parse::<FeatureDocStatus>())
                            .transpose()?;
                        let updated = update_feature_documentation(
                            &project_dir,
                            &id,
                            content,
                            parsed_status,
                            verify,
                            Some("cli"),
                        )?;
                        println!(
                            "Feature docs updated: {} status={} revision={}",
                            updated.feature_id, updated.status, updated.revision
                        );
                    }
                    FeatureDocCommands::Status { id } => {
                        let doc = get_feature_documentation(&project_dir, &id)?;
                        println!(
                            "feature={} status={} revision={} updated={}{}",
                            doc.feature_id,
                            doc.status,
                            doc.revision,
                            doc.updated_at,
                            doc.last_verified_at
                                .as_ref()
                                .map(|value| format!(" verified={}", value))
                                .unwrap_or_default()
                        );
                    }
                },
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
                    environment_id,
                    spec,
                    release,
                    mode,
                    activate,
                    checkout,
                    worktree,
                    worktree_path,
                    start_session,
                    goal,
                    provider,
                    session_mode,
                    no_input,
                } => {
                    if worktree && checkout {
                        anyhow::bail!("--worktree and --checkout cannot be used together");
                    }
                    if worktree_path.is_some() && !worktree {
                        anyhow::bail!("--worktree-path requires --worktree");
                    }

                    let parsed_workspace_type = workspace_type
                        .as_deref()
                        .map(str::parse::<ShipWorkspaceKind>)
                        .transpose()?;
                    let is_feature_workspace = parsed_workspace_type
                        == Some(ShipWorkspaceKind::Feature)
                        || (parsed_workspace_type.is_none() && branch.starts_with("feature/"));
                    let resolved_worktree_path = if worktree {
                        Some(worktree_path.unwrap_or_else(|| {
                            let b = branch
                                .trim_start_matches("feature/")
                                .trim_start_matches("patch/");
                            format!("../{}", b)
                        }))
                    } else {
                        None
                    };
                    let mut spec = spec;
                    let mut release = release;
                    let mut start_session = start_session;
                    let mut goal = goal;
                    let mut provider = provider;
                    let mut session_mode = session_mode;

                    let interactive =
                        !no_input && io::stdin().is_terminal() && io::stdout().is_terminal();
                    if interactive {
                        if release.is_none() {
                            let releases = list_releases(&project_dir)?;
                            if !releases.is_empty() {
                                println!("Available releases:");
                                for entry in releases.iter().take(8) {
                                    println!(
                                        "  - {} ({})",
                                        entry.release.metadata.version, entry.id
                                    );
                                }
                                if releases.len() > 8 {
                                    println!("  ... and {} more", releases.len() - 8);
                                }
                                release = prompt_optional("Attach release id (Enter to skip): ")?;
                            }
                        }

                        if spec.is_none() && is_feature_workspace {
                            let create_now =
                                prompt_yes_no("Create a starter spec now? [Y/n]: ", true)?;
                            if create_now {
                                let default_title =
                                    format!("{} Spec", branch_to_feature_title(&branch));
                                let title = prompt_with_default("Spec title", &default_title)?;
                                let spec_goal = prompt_optional("Spec goal (optional): ")?;
                                let body = format!(
                                    "## Goal\n{}\n\n## Scope\n- \n\n## Acceptance Criteria\n- [ ] \n",
                                    spec_goal.clone().unwrap_or_else(|| {
                                        "Define outcome and boundaries for this workspace."
                                            .to_string()
                                    })
                                );
                                let created_spec =
                                    create_spec(&project_dir, &title, &body, Some(&branch))?;
                                println!(
                                    "Spec created and linked: {} ({})",
                                    created_spec.file_name, created_spec.id
                                );
                                spec = Some(created_spec.id);
                                if goal.is_none() {
                                    goal = spec_goal;
                                }
                            }
                        }

                        if !start_session
                            && goal.is_none()
                            && provider.is_none()
                            && session_mode.is_none()
                        {
                            start_session = prompt_yes_no("Start a session now? [Y/n]: ", true)?;
                        }

                        if start_session
                            || goal.is_some()
                            || provider.is_some()
                            || session_mode.is_some()
                        {
                            if goal.is_none() {
                                goal = prompt_optional("Session goal (optional): ")?;
                            }
                            if provider.is_none() {
                                let connected: Vec<String> = list_providers(&project_dir)?
                                    .into_iter()
                                    .filter(|entry| entry.enabled)
                                    .map(|entry| entry.id)
                                    .collect();
                                if !connected.is_empty() {
                                    println!("Connected providers: {}", connected.join(", "));
                                }
                                provider = prompt_optional("Session provider (Enter for auto): ")?;
                            }
                            if session_mode.is_none() {
                                session_mode =
                                    prompt_optional("Session mode id (Enter to skip): ")?;
                            }
                        }
                    }

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
                            environment_id,
                            feature_id: resolved_feature_id,
                            spec_id: spec,
                            release_id: release,
                            active_mode: mode,
                            is_worktree: Some(worktree),
                            worktree_path: resolved_worktree_path,
                            ..CreateWorkspaceRequest::default()
                        },
                    )?;

                    let should_start_session = start_session
                        || goal.as_ref().is_some()
                        || provider.as_ref().is_some()
                        || session_mode.as_ref().is_some();

                    if worktree || checkout {
                        workspace = sync_workspace(&project_dir, &branch)?;
                    } else if activate {
                        workspace = activate_workspace(&project_dir, &branch)?;
                    }

                    if should_start_session && workspace.status != WorkspaceStatus::Active {
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

                    if should_start_session {
                        let session = start_workspace_session(
                            &project_dir,
                            &workspace.branch,
                            goal,
                            session_mode,
                            provider,
                        )?;
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
                                .unwrap_or_default(),
                        );
                    } else {
                        println!(
                            "Next: ship workspace session start --branch {} --goal \"<goal>\" --provider <id>",
                            workspace.branch
                        );
                    }
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
                WorkspaceCommands::Reconcile { apply } => {
                    let report = reconcile_workspace_links(&project_dir, apply)?;
                    println!(
                        "Workspace link reconcile ({})",
                        if apply { "applied" } else { "dry-run" }
                    );
                    println!("  workspace updates: {}", report.workspace_updates);
                    println!("  spec metadata updates: {}", report.spec_updates);
                    println!(
                        "  ambiguous feature links: {}",
                        report.ambiguous_feature_links
                    );
                    for detail in &report.ambiguous_feature_details {
                        println!("    - {}", detail);
                    }
                    println!("  ambiguous spec links: {}", report.ambiguous_spec_links);
                    for detail in &report.ambiguous_spec_details {
                        println!("    - {}", detail);
                    }
                }
                WorkspaceCommands::Repair { branch, dry_run } => {
                    let branch = branch.unwrap_or(current_branch(&project_root)?);
                    let report = repair_workspace(&project_dir, &branch, dry_run)?;
                    println!(
                        "Workspace repair ({}) branch={} status={}",
                        if report.dry_run { "dry-run" } else { "applied" },
                        report.workspace_branch,
                        report.status
                    );
                    if let Some(mode_id) = report.mode_id.as_deref() {
                        println!("  mode: {}", mode_id);
                    }
                    if let Some(error) = report.resolution_error.as_deref() {
                        println!("  resolution_error: {}", error);
                    }
                    println!(
                        "  providers_expected: {}",
                        if report.providers_expected.is_empty() {
                            "none".to_string()
                        } else {
                            report.providers_expected.join(",")
                        }
                    );
                    println!(
                        "  missing_provider_configs: {}",
                        if report.missing_provider_configs.is_empty() {
                            "none".to_string()
                        } else {
                            report.missing_provider_configs.join(",")
                        }
                    );
                    println!("  had_compile_error: {}", report.had_compile_error);
                    println!("  needs_recompile: {}", report.needs_recompile);
                    println!("  reapplied_compile: {}", report.reapplied_compile);
                    if !report.actions.is_empty() {
                        println!("  actions:");
                        for action in &report.actions {
                            println!("    - {}", action);
                        }
                    }
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
                        let summary_for_docs = summary.clone();
                        let session = end_workspace_session(
                            &project_dir,
                            &branch,
                            EndWorkspaceSessionRequest {
                                summary,
                                updated_feature_ids: updated_feature,
                                updated_spec_ids: updated_spec,
                            },
                        )?;
                        if !session.updated_feature_ids.is_empty() {
                            let updated_docs = sync_feature_docs_after_session(
                                &project_dir,
                                &session.updated_feature_ids,
                                summary_for_docs.as_deref(),
                            )?;
                            if !updated_docs.is_empty() {
                                println!(
                                    "Feature docs synced for {} feature(s).",
                                    updated_docs.len()
                                );
                            }
                        }
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
                    WorkspaceSessionCommands::Note { note, branch } => {
                        let branch = branch.unwrap_or(current_branch(&project_root)?);
                        match get_active_workspace_session(&project_dir, &branch)? {
                            None => anyhow::bail!(
                                "No active session for '{}'. Run `ship workspace session start` first.",
                                branch
                            ),
                            Some(_) => {
                                record_workspace_session_progress(&project_dir, &branch, &note)?;
                                println!("Logged session note: {}", note);
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
                "Point Ship at it with: SHIP_DIR={} ship feature list",
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
        Some(Commands::Hooks { action }) => match action {
            HooksCommands::Run { provider } => {
                run_hooks_runtime(provider)?;
            }
        },
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
        Some(Commands::Ui {
            dev,
            watch,
            release,
        }) => {
            launch_ui_command(dev, watch, release)?;
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
                println!("  ship ui");
                println!("Automation/API flow:");
                println!(
                    "  ship workspace create feature/<name> --type feature --checkout --activate --start-session --goal \"<goal>\" --provider <id> --no-input"
                );
            }
        },
    }

    Ok(())
}

fn run_hooks_runtime(provider_hint: Option<String>) -> Result<()> {
    let mut raw = String::new();
    if io::stdin().read_to_string(&mut raw).is_err() {
        return Ok(());
    }

    let payload = serde_json::from_str::<serde_json::Value>(&raw)
        .unwrap_or_else(|_| serde_json::json!({ "raw": raw.trim() }));
    let event = extract_hook_event_name(&payload);
    let cwd = payload
        .get("cwd")
        .and_then(|value| value.as_str())
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let ship_dir = resolve_ship_dir_for_hook_runtime(&cwd);
    let provider = detect_hook_provider(provider_hint.as_deref(), &event, &payload);
    let hook_response = build_hook_response(provider, &event, &payload, ship_dir.as_deref());

    append_hook_event_log(
        provider.as_str(),
        ship_dir.as_deref(),
        &cwd,
        &event,
        &payload,
        hook_response.as_ref(),
    );

    if let Some(response) = hook_response {
        print!("{}", serde_json::to_string(&response)?);
    }

    Ok(())
}

fn extract_hook_event_name(payload: &serde_json::Value) -> String {
    payload
        .get("hook_event_name")
        .and_then(|value| value.as_str())
        .or_else(|| payload.get("event").and_then(|value| value.as_str()))
        .or_else(|| payload.get("hook").and_then(|value| value.as_str()))
        .unwrap_or("unknown")
        .to_string()
}

fn resolve_ship_dir_for_hook_runtime(cwd: &Path) -> Option<PathBuf> {
    if cwd
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == ".ship")
    {
        return Some(cwd.to_path_buf());
    }
    if cwd.join(".ship").is_dir() {
        return Some(cwd.join(".ship"));
    }
    ship_dir_from_path(cwd)
}

fn append_hook_event_log(
    provider_hint: Option<&str>,
    ship_dir: Option<&Path>,
    cwd: &Path,
    event: &str,
    payload: &serde_json::Value,
    response: Option<&serde_json::Value>,
) {
    let Some(log_path) = hook_telemetry_log_path() else {
        return;
    };
    let Some(parent) = log_path.parent() else {
        return;
    };
    if std::fs::create_dir_all(parent).is_err() {
        return;
    }
    let mut file = match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        Ok(file) => file,
        Err(_) => return,
    };

    let line = serde_json::json!({
        "ts": chrono::Utc::now().to_rfc3339(),
        "event": event,
        "provider": provider_hint,
        "cwd": cwd.to_string_lossy(),
        "ship_dir": ship_dir.map(|path| path.to_string_lossy().to_string()),
        "payload": payload,
        "response": response,
        "decision": response.and_then(extract_hook_decision),
    });
    let _ = writeln!(file, "{}", line);
}

fn hook_telemetry_log_path() -> Option<PathBuf> {
    let global_dir = get_global_dir().ok()?;
    Some(
        global_dir
            .join("state")
            .join("telemetry")
            .join("hooks")
            .join("events.ndjson"),
    )
}

fn read_hook_context(ship_dir: &Path) -> Option<String> {
    let path = ship_dir
        .join("agents")
        .join("runtime")
        .join("hook-context.md");
    let content = std::fs::read_to_string(path).ok()?;
    if content.trim().is_empty() {
        None
    } else {
        Some(content)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HookProvider {
    Claude,
    Gemini,
    Unknown,
}

impl HookProvider {
    fn as_str(self) -> Option<&'static str> {
        match self {
            Self::Claude => Some("claude"),
            Self::Gemini => Some("gemini"),
            Self::Unknown => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HookDecision {
    Allow,
    Ask,
    Deny,
    None,
}

impl Default for HookDecision {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Default)]
struct HookPolicyOutcome {
    decision: HookDecision,
    reason: Option<String>,
    updated_input: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
struct HookEnvelope {
    workspace_root: Option<PathBuf>,
    allowed_paths: Vec<String>,
    allow_network: bool,
    allow_installs: bool,
    auto_approve_patterns: Vec<String>,
    always_block_patterns: Vec<String>,
    require_confirmation_patterns: Vec<String>,
    tool_allow_patterns: Vec<String>,
    tool_deny_patterns: Vec<String>,
}

impl Default for HookEnvelope {
    fn default() -> Self {
        Self {
            workspace_root: None,
            allowed_paths: vec![".".to_string()],
            allow_network: false,
            allow_installs: false,
            auto_approve_patterns: vec![
                "find *".to_string(),
                "grep *".to_string(),
                "rg *".to_string(),
                "cat *".to_string(),
                "ls *".to_string(),
                "git status*".to_string(),
                "git log*".to_string(),
                "git diff*".to_string(),
            ],
            always_block_patterns: vec![
                "rm -rf *".to_string(),
                "git push --force*".to_string(),
                "npm publish*".to_string(),
                "cargo publish*".to_string(),
            ],
            require_confirmation_patterns: vec![],
            tool_allow_patterns: vec!["*".to_string()],
            tool_deny_patterns: vec![],
        }
    }
}

fn detect_hook_provider(
    provider_hint: Option<&str>,
    event: &str,
    payload: &serde_json::Value,
) -> HookProvider {
    if let Some(provider) = provider_hint {
        return match provider.trim().to_ascii_lowercase().as_str() {
            "claude" => HookProvider::Claude,
            "gemini" => HookProvider::Gemini,
            _ => HookProvider::Unknown,
        };
    }

    let normalized = normalize_hook_event(event);
    match normalized.as_str() {
        "userpromptsubmit" | "pretooluse" | "permissionrequest" | "posttoolusefailure"
        | "subagentstart" | "subagentstop" | "precompact" => return HookProvider::Claude,
        "beforetool"
        | "aftertool"
        | "beforeagent"
        | "afteragent"
        | "sessionend"
        | "beforemodel"
        | "aftermodel"
        | "precompress"
        | "beforetoolselection" => return HookProvider::Gemini,
        _ => {}
    }

    if payload.get("permission_mode").is_some() {
        return HookProvider::Claude;
    }

    if payload
        .get("tool_name")
        .and_then(|value| value.as_str())
        .is_some_and(|tool| tool.eq_ignore_ascii_case("run_shell_command"))
    {
        return HookProvider::Gemini;
    }

    HookProvider::Unknown
}

fn normalize_hook_event(event: &str) -> String {
    event
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase()
}

fn build_hook_response(
    provider: HookProvider,
    event: &str,
    payload: &serde_json::Value,
    ship_dir: Option<&Path>,
) -> Option<serde_json::Value> {
    let normalized_event = normalize_hook_event(event);

    if (normalized_event == "sessionstart" || normalized_event == "userpromptsubmit")
        && let Some(ship_dir) = ship_dir
        && let Some(context) = read_hook_context(ship_dir)
    {
        return Some(match provider {
            HookProvider::Gemini => {
                serde_json::json!({ "hookSpecificOutput": { "additionalContext": context } })
            }
            HookProvider::Claude | HookProvider::Unknown => serde_json::json!({
                "hookSpecificOutput": {
                    "hookEventName": if normalized_event == "userpromptsubmit" { "UserPromptSubmit" } else { "SessionStart" },
                    "additionalContext": context
                }
            }),
        });
    }

    let envelope = load_hook_envelope(ship_dir);
    match normalized_event.as_str() {
        "pretooluse" | "beforetool" => {
            let outcome = evaluate_pre_tool_policy(provider, payload, &envelope);
            map_pre_tool_outcome(provider, outcome)
        }
        "permissionrequest" => {
            let outcome = evaluate_permission_request_policy(payload, &envelope);
            map_permission_request_outcome(outcome)
        }
        _ => None,
    }
}

fn load_hook_envelope(ship_dir: Option<&Path>) -> HookEnvelope {
    let mut envelope = HookEnvelope::default();
    let Some(ship_dir) = ship_dir else {
        return envelope;
    };
    let path = ship_dir
        .join("agents")
        .join("runtime")
        .join("envelope.json");
    let Ok(raw) = std::fs::read_to_string(path) else {
        return envelope;
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return envelope;
    };
    let Some(obj) = value.as_object() else {
        return envelope;
    };

    envelope.workspace_root = obj
        .get("workspace_root")
        .and_then(|value| value.as_str())
        .map(PathBuf::from);
    if let Some(values) = obj.get("allowed_paths").and_then(as_string_array) {
        envelope.allowed_paths = values;
    }
    envelope.allow_network = obj
        .get("allow_network")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    envelope.allow_installs = obj
        .get("allow_installs")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    if let Some(values) = obj.get("auto_approve_patterns").and_then(as_string_array) {
        envelope.auto_approve_patterns = values;
    }
    if let Some(values) = obj.get("always_block_patterns").and_then(as_string_array) {
        envelope.always_block_patterns = values;
    }
    if let Some(values) = obj.get("require_confirmation").and_then(as_string_array) {
        envelope.require_confirmation_patterns = values;
    }
    if let Some(values) = obj.get("tools_allow").and_then(as_string_array) {
        envelope.tool_allow_patterns = values;
    }
    if let Some(values) = obj.get("tools_deny").and_then(as_string_array) {
        envelope.tool_deny_patterns = values;
    }
    envelope
}

fn as_string_array(value: &serde_json::Value) -> Option<Vec<String>> {
    value.as_array().map(|items| {
        items
            .iter()
            .filter_map(|item| item.as_str().map(|s| s.to_string()))
            .collect::<Vec<_>>()
    })
}

fn evaluate_pre_tool_policy(
    provider: HookProvider,
    payload: &serde_json::Value,
    envelope: &HookEnvelope,
) -> HookPolicyOutcome {
    let tool_name = payload
        .get("tool_name")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    let tool_input = payload
        .get("tool_input")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));

    if tool_name.is_empty() {
        return HookPolicyOutcome::default();
    }

    if tool_denied(tool_name, envelope) {
        return HookPolicyOutcome {
            decision: HookDecision::Deny,
            reason: Some(format!(
                "Tool '{}' is disabled by Ship permission policy.",
                tool_name
            )),
            updated_input: None,
        };
    }

    if is_shell_tool(provider, tool_name)
        && let Some(command) = extract_shell_command(&tool_input)
    {
        let outcome = evaluate_shell_command(provider, command, envelope);
        if outcome.decision != HookDecision::None {
            return outcome;
        }
        if tool_requires_confirmation(tool_name, envelope) && provider == HookProvider::Claude {
            return HookPolicyOutcome {
                decision: HookDecision::Ask,
                reason: Some(format!(
                    "Tool '{}' requires explicit confirmation in this workspace.",
                    tool_name
                )),
                updated_input: None,
            };
        }
        return HookPolicyOutcome::default();
    }

    if tool_requires_confirmation(tool_name, envelope) && provider == HookProvider::Claude {
        return HookPolicyOutcome {
            decision: HookDecision::Ask,
            reason: Some(format!(
                "Tool '{}' requires explicit confirmation in this workspace.",
                tool_name
            )),
            updated_input: None,
        };
    }

    if is_write_like_tool(tool_name) {
        let paths = extract_target_paths(&tool_input);
        if let Some(off_scope) = paths
            .iter()
            .find(|path| !is_path_in_allowed_scope(path, envelope))
        {
            return HookPolicyOutcome {
                decision: HookDecision::Deny,
                reason: Some(format!(
                    "Path '{}' is outside allowed workspace scope.",
                    off_scope
                )),
                updated_input: None,
            };
        }
    }

    HookPolicyOutcome::default()
}

fn evaluate_permission_request_policy(
    payload: &serde_json::Value,
    envelope: &HookEnvelope,
) -> HookPolicyOutcome {
    let tool_name = payload
        .get("tool_name")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    let tool_input = payload
        .get("tool_input")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));

    if tool_name.is_empty() {
        return HookPolicyOutcome::default();
    }

    if tool_denied(tool_name, envelope) {
        return HookPolicyOutcome {
            decision: HookDecision::Deny,
            reason: Some(format!("Ship policy blocks tool '{}'.", tool_name)),
            updated_input: None,
        };
    }

    if tool_name.eq_ignore_ascii_case("bash")
        && let Some(command) = extract_shell_command(&tool_input)
    {
        let mut outcome = evaluate_shell_command(HookProvider::Claude, command, envelope);
        if outcome.decision == HookDecision::Ask {
            outcome.decision = HookDecision::None;
            outcome.reason = None;
        }
        return outcome;
    }

    HookPolicyOutcome::default()
}

fn tool_denied(tool_name: &str, envelope: &HookEnvelope) -> bool {
    envelope
        .tool_deny_patterns
        .iter()
        .any(|pattern| wildcard_match_case_insensitive(pattern, tool_name))
}

fn tool_requires_confirmation(tool_name: &str, envelope: &HookEnvelope) -> bool {
    envelope
        .require_confirmation_patterns
        .iter()
        .any(|pattern| wildcard_match_case_insensitive(pattern, tool_name))
}

fn is_shell_tool(provider: HookProvider, tool_name: &str) -> bool {
    match provider {
        HookProvider::Gemini => tool_name.eq_ignore_ascii_case("run_shell_command"),
        HookProvider::Claude | HookProvider::Unknown => tool_name.eq_ignore_ascii_case("bash"),
    }
}

fn extract_shell_command(tool_input: &serde_json::Value) -> Option<&str> {
    tool_input
        .get("command")
        .and_then(|value| value.as_str())
        .or_else(|| tool_input.get("cmd").and_then(|value| value.as_str()))
}

fn evaluate_shell_command(
    provider: HookProvider,
    command: &str,
    envelope: &HookEnvelope,
) -> HookPolicyOutcome {
    let parts = split_command_chain(command);
    if parts.is_empty() {
        return HookPolicyOutcome::default();
    }

    let mut saw_unknown = false;
    let mut saw_confirmation = false;
    let mut saw_safe = false;

    for raw_part in parts {
        let part = strip_env_prefix(&raw_part);
        if part.is_empty() {
            continue;
        }

        if command_matches_patterns(&part, &envelope.always_block_patterns) {
            return HookPolicyOutcome {
                decision: HookDecision::Deny,
                reason: Some(format!("Command '{}' is blocked by Ship policy.", part)),
                updated_input: None,
            };
        }

        if !envelope.allow_network && looks_like_network_command(&part) {
            return HookPolicyOutcome {
                decision: HookDecision::Deny,
                reason: Some(format!(
                    "Network command '{}' is not allowed in this workspace.",
                    part
                )),
                updated_input: None,
            };
        }

        if !envelope.allow_installs && looks_like_install_command(&part) {
            return HookPolicyOutcome {
                decision: HookDecision::Deny,
                reason: Some(format!(
                    "Install command '{}' is blocked in this workspace.",
                    part
                )),
                updated_input: None,
            };
        }

        if command_matches_patterns(&part, &envelope.require_confirmation_patterns) {
            saw_confirmation = true;
            continue;
        }

        if command_matches_patterns(&part, &envelope.auto_approve_patterns)
            || looks_like_safe_read_command(&part)
        {
            saw_safe = true;
            continue;
        }

        saw_unknown = true;
    }

    if saw_confirmation && provider == HookProvider::Claude {
        return HookPolicyOutcome {
            decision: HookDecision::Ask,
            reason: Some("Command requires confirmation by Ship policy.".to_string()),
            updated_input: None,
        };
    }

    if saw_unknown {
        return HookPolicyOutcome::default();
    }

    if saw_safe {
        return HookPolicyOutcome {
            decision: HookDecision::Allow,
            reason: Some("Approved by Ship safe-command policy.".to_string()),
            updated_input: None,
        };
    }

    HookPolicyOutcome::default()
}

fn split_command_chain(command: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut chars = command.chars().peekable();
    let mut in_single = false;
    let mut in_double = false;
    let mut escaped = false;

    while let Some(ch) = chars.next() {
        if escaped {
            current.push(ch);
            escaped = false;
            continue;
        }

        if ch == '\\' {
            escaped = true;
            current.push(ch);
            continue;
        }

        if ch == '\'' && !in_double {
            in_single = !in_single;
            current.push(ch);
            continue;
        }
        if ch == '"' && !in_single {
            in_double = !in_double;
            current.push(ch);
            continue;
        }

        if !in_single && !in_double {
            if ch == ';' || ch == '\n' {
                push_part(&mut parts, &mut current);
                continue;
            }
            if ch == '&' && chars.peek() == Some(&'&') {
                let _ = chars.next();
                push_part(&mut parts, &mut current);
                continue;
            }
            if ch == '|' {
                if chars.peek() == Some(&'|') {
                    let _ = chars.next();
                }
                push_part(&mut parts, &mut current);
                continue;
            }
        }

        current.push(ch);
    }

    push_part(&mut parts, &mut current);
    parts
}

fn push_part(parts: &mut Vec<String>, current: &mut String) {
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        parts.push(trimmed.to_string());
    }
    current.clear();
}

fn strip_env_prefix(part: &str) -> String {
    let mut tokens = part.split_whitespace().peekable();
    let mut command_tokens = Vec::new();

    while let Some(token) = tokens.next() {
        if command_tokens.is_empty() && is_env_assignment(token) {
            continue;
        }

        command_tokens.push(token.to_string());
        command_tokens.extend(tokens.map(|tail| tail.to_string()));
        break;
    }

    command_tokens.join(" ")
}

fn is_env_assignment(token: &str) -> bool {
    let Some((name, value)) = token.split_once('=') else {
        return false;
    };
    if name.is_empty() || value.is_empty() {
        return false;
    }
    name.chars()
        .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
}

fn command_matches_patterns(command: &str, patterns: &[String]) -> bool {
    patterns
        .iter()
        .any(|pattern| command_pattern_matches(pattern, command))
}

fn command_pattern_matches(pattern: &str, command: &str) -> bool {
    let pattern = normalize_command(pattern);
    if pattern.is_empty() {
        return false;
    }
    let command = normalize_command(command);
    if pattern == "*" {
        return true;
    }
    if pattern.contains('*') {
        return wildcard_match_case_insensitive(&pattern, &command);
    }
    command == pattern || command.starts_with(&format!("{} ", pattern))
}

fn normalize_command(command: &str) -> String {
    command.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn wildcard_match_case_insensitive(pattern: &str, value: &str) -> bool {
    wildcard_match(&pattern.to_ascii_lowercase(), &value.to_ascii_lowercase())
}

fn wildcard_match(pattern: &str, value: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    let starts_with_wildcard = pattern.starts_with('*');
    let ends_with_wildcard = pattern.ends_with('*');
    let segments: Vec<&str> = pattern
        .split('*')
        .filter(|segment| !segment.is_empty())
        .collect();
    if segments.is_empty() {
        return true;
    }

    let mut cursor = 0usize;
    for (index, segment) in segments.iter().enumerate() {
        if index == 0 && !starts_with_wildcard {
            if !value[cursor..].starts_with(segment) {
                return false;
            }
            cursor += segment.len();
            continue;
        }

        if let Some(found) = value[cursor..].find(segment) {
            cursor += found + segment.len();
        } else {
            return false;
        }
    }

    if !ends_with_wildcard {
        if let Some(last) = segments.last() {
            return value.ends_with(last);
        }
    }
    true
}

fn looks_like_safe_read_command(command: &str) -> bool {
    let normalized = command.to_ascii_lowercase();
    normalized.starts_with("ls")
        || normalized.starts_with("cat ")
        || normalized.starts_with("pwd")
        || normalized.starts_with("rg ")
        || normalized.starts_with("grep ")
        || normalized.starts_with("find ")
        || normalized.starts_with("git status")
        || normalized.starts_with("git diff")
        || normalized.starts_with("git log")
}

fn looks_like_network_command(command: &str) -> bool {
    let normalized = command.to_ascii_lowercase();
    normalized.starts_with("curl ")
        || normalized.starts_with("wget ")
        || normalized.starts_with("nc ")
        || normalized.starts_with("ssh ")
        || normalized.starts_with("scp ")
}

fn looks_like_install_command(command: &str) -> bool {
    let normalized = command.to_ascii_lowercase();
    normalized.starts_with("npm install")
        || normalized.starts_with("pnpm add")
        || normalized.starts_with("yarn add")
        || normalized.starts_with("pip install")
        || normalized.starts_with("uv pip install")
        || normalized.starts_with("cargo add")
        || normalized.starts_with("go get")
        || normalized.starts_with("brew install")
}

fn is_write_like_tool(tool_name: &str) -> bool {
    let normalized = tool_name.to_ascii_lowercase();
    normalized.contains("write")
        || normalized.contains("edit")
        || normalized.contains("replace")
        || normalized.contains("delete")
}

fn extract_target_paths(tool_input: &serde_json::Value) -> Vec<String> {
    let Some(obj) = tool_input.as_object() else {
        return Vec::new();
    };

    let mut paths = Vec::new();
    for key in [
        "file_path",
        "path",
        "target_path",
        "destination_path",
        "absolute_path",
    ] {
        if let Some(path) = obj.get(key).and_then(|value| value.as_str()) {
            paths.push(path.to_string());
        }
    }

    if let Some(items) = obj.get("paths").and_then(|value| value.as_array()) {
        for item in items {
            if let Some(path) = item.as_str() {
                paths.push(path.to_string());
            }
        }
    }
    paths
}

fn is_path_in_allowed_scope(path: &str, envelope: &HookEnvelope) -> bool {
    if envelope.allowed_paths.is_empty() {
        return true;
    }

    let input_path = PathBuf::from(path);
    let absolute = if input_path.is_absolute() {
        input_path
    } else if let Some(root) = &envelope.workspace_root {
        root.join(input_path)
    } else {
        input_path
    };

    let relative = envelope
        .workspace_root
        .as_ref()
        .and_then(|root| absolute.strip_prefix(root).ok())
        .map(|path| path.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|| absolute.to_string_lossy().replace('\\', "/"));

    envelope.allowed_paths.iter().any(|pattern| {
        let normalized = pattern.trim();
        if normalized.is_empty() {
            return false;
        }
        if normalized == "." || normalized == "**/*" || normalized == "*" {
            return true;
        }
        if normalized.contains('*') {
            return wildcard_match_case_insensitive(normalized, &relative);
        }
        let normalized = normalized.trim_start_matches("./");
        relative == normalized || relative.starts_with(&format!("{}/", normalized))
    })
}

fn map_pre_tool_outcome(
    provider: HookProvider,
    outcome: HookPolicyOutcome,
) -> Option<serde_json::Value> {
    match provider {
        HookProvider::Claude | HookProvider::Unknown => match outcome.decision {
            HookDecision::Allow | HookDecision::Ask | HookDecision::Deny => {
                let decision = match outcome.decision {
                    HookDecision::Allow => "allow",
                    HookDecision::Ask => "ask",
                    HookDecision::Deny => "deny",
                    HookDecision::None => return None,
                };
                let mut payload = serde_json::json!({
                    "hookSpecificOutput": {
                        "hookEventName": "PreToolUse",
                        "permissionDecision": decision
                    }
                });
                if let Some(reason) = outcome.reason {
                    payload["hookSpecificOutput"]["permissionDecisionReason"] =
                        serde_json::json!(reason);
                }
                if let Some(updated) = outcome.updated_input {
                    payload["hookSpecificOutput"]["updatedInput"] = updated;
                }
                Some(payload)
            }
            HookDecision::None => None,
        },
        HookProvider::Gemini => match outcome.decision {
            HookDecision::Deny => Some(serde_json::json!({
                "decision": "deny",
                "reason": outcome.reason.unwrap_or_else(|| "Blocked by Ship policy.".to_string()),
            })),
            HookDecision::Allow => {
                let mut payload = serde_json::json!({ "decision": "allow" });
                if let Some(updated) = outcome.updated_input {
                    payload["hookSpecificOutput"] = serde_json::json!({ "tool_input": updated });
                }
                Some(payload)
            }
            HookDecision::Ask | HookDecision::None => None,
        },
    }
}

fn map_permission_request_outcome(outcome: HookPolicyOutcome) -> Option<serde_json::Value> {
    match outcome.decision {
        HookDecision::Allow => {
            let mut decision = serde_json::json!({
                "behavior": "allow",
            });
            if let Some(updated) = outcome.updated_input {
                decision["updatedInput"] = updated;
            }
            Some(serde_json::json!({
                "hookSpecificOutput": {
                    "hookEventName": "PermissionRequest",
                    "decision": decision
                }
            }))
        }
        HookDecision::Deny => Some(serde_json::json!({
            "hookSpecificOutput": {
                "hookEventName": "PermissionRequest",
                "decision": {
                    "behavior": "deny",
                    "message": outcome.reason.unwrap_or_else(|| "Blocked by Ship permission policy.".to_string()),
                    "interrupt": false
                }
            }
        })),
        HookDecision::Ask | HookDecision::None => None,
    }
}

fn extract_hook_decision(response: &serde_json::Value) -> Option<String> {
    response
        .get("decision")
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
        .or_else(|| {
            response
                .get("hookSpecificOutput")
                .and_then(|value| value.get("permissionDecision"))
                .and_then(|value| value.as_str())
                .map(|value| value.to_string())
        })
        .or_else(|| {
            response
                .get("hookSpecificOutput")
                .and_then(|value| value.get("decision"))
                .and_then(|value| value.get("behavior"))
                .and_then(|value| value.as_str())
                .map(|value| value.to_string())
        })
}

fn handle_migrate_command(force: bool) -> Result<()> {
    let project_dir = get_project_dir_cli()?;
    let global_dir = get_global_dir()?;
    let global = migrate_global_state(&global_dir)?;
    let project = migrate_project_state(&project_dir)?;
    let specs = import_specs_from_files(&project_dir)?;
    let config = migrate_json_config_file(&project_dir)?;
    let cleared_project_markers = runtime::clear_project_migration_meta(&project_dir)?;
    let cleared_global_markers = runtime::clear_global_migration_meta()?;
    ensure_user_notes_imported_once(true, true)?;
    ensure_project_imported_once(&project_dir, true, true)?;
    println!(
        "Migration complete{}:\n- file namespace copies: copied={} skipped={} conflicts={}\n- project DB: {} (applied {})\n- global DB: {} (applied {})\n- registry: {} -> {} entries (normalized {})\n- app_state paths normalized: {}\n- startup import markers reset: {} project marker{}, {} global marker{}\n- imported docs: {} spec{}{}.",
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

fn launch_ui_command(dev: bool, watch: bool, release: bool) -> Result<()> {
    if dev || watch || release {
        return launch_ui_dev_command(watch, release);
    }

    let cwd = env::current_dir()?;
    let repo_root = find_repo_root_with_ui(&cwd);
    launch_ui_executable(repo_root.as_deref())
}

fn launch_ui_dev_command(watch: bool, release: bool) -> Result<()> {
    let cwd = env::current_dir()?;
    let repo_root = find_repo_root_with_ui(&cwd).ok_or_else(|| {
        anyhow::anyhow!(
            "Could not find Ship source checkout from {}. Run this command from the Ship repo.",
            cwd.display()
        )
    })?;
    let ui_dir = repo_root.join("crates/ui");

    let mut command = ProcessCommand::new("pnpm");
    command.arg("--dir").arg(&ui_dir).arg("tauri").arg("dev");
    if !watch {
        command.arg("--no-watch");
    }
    if release {
        command.arg("--release");
    }

    let status = command.status()?;
    if !status.success() {
        anyhow::bail!("UI process exited with status {}", status);
    }
    Ok(())
}

fn launch_ui_executable(_repo_root: Option<&Path>) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        if let Some(root) = repo_root {
            let candidates = [
                root.join("target/release/bundle/macos/Shipwright.app"),
                root.join("target/debug/bundle/macos/Shipwright.app"),
                root.join("crates/ui/src-tauri/target/release/bundle/macos/Shipwright.app"),
                root.join("crates/ui/src-tauri/target/debug/bundle/macos/Shipwright.app"),
            ];
            for bundled_app in candidates {
                if bundled_app.exists() {
                    let status = ProcessCommand::new("open").arg(&bundled_app).status()?;
                    if status.success() {
                        return Ok(());
                    }
                }
            }
        }

        let status = ProcessCommand::new("open")
            .args(["-a", "Shipwright"])
            .status()?;
        if status.success() {
            return Ok(());
        }
    }

    #[cfg(target_os = "linux")]
    {
        let status = ProcessCommand::new("shipwright").status();
        if let Ok(status) = status {
            if status.success() {
                return Ok(());
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let status = ProcessCommand::new("cmd")
            .args(["/C", "start", "", "Shipwright.exe"])
            .status();
        if let Ok(status) = status {
            if status.success() {
                return Ok(());
            }
        }
    }

    anyhow::bail!(
        "Could not launch Shipwright executable. Build/install the desktop app, or run `ship ui --dev` from the Ship repository."
    )
}

fn find_repo_root_with_ui(start: &Path) -> Option<PathBuf> {
    let mut current = Some(start);
    while let Some(dir) = current {
        if dir.join("crates/ui/package.json").is_file()
            && dir.join("crates/ui/src-tauri/tauri.conf.json").is_file()
        {
            return Some(dir.to_path_buf());
        }
        current = dir.parent();
    }
    None
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

fn prompt_line(prompt: &str) -> Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn prompt_optional(prompt: &str) -> Result<Option<String>> {
    let input = prompt_line(prompt)?;
    if input.is_empty() {
        Ok(None)
    } else {
        Ok(Some(input))
    }
}

fn prompt_with_default(label: &str, default: &str) -> Result<String> {
    let prompt = format!("{} [{}]: ", label, default);
    let input = prompt_line(&prompt)?;
    if input.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(input)
    }
}

fn prompt_yes_no(prompt: &str, default_yes: bool) -> Result<bool> {
    loop {
        let input = prompt_line(prompt)?;
        if input.is_empty() {
            return Ok(default_yes);
        }
        match input.to_ascii_lowercase().as_str() {
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => println!("Please answer yes or no."),
        }
    }
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
    parts.push(format!("generation={}", workspace.config_generation));
    if let Some(compiled_at) = workspace.compiled_at.as_ref() {
        parts.push(format!("compiled_at={}", compiled_at));
    }
    if let Some(compile_error) = workspace.compile_error.as_deref() {
        parts.push(format!("compile_error=\"{}\"", compile_error));
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
    if let Some(generation) = session.config_generation_at_start {
        parts.push(format!("generation_at_start={}", generation));
    }
    parts.push(format!("stale_context={}", session.stale_context));
    if session.stale_context {
        parts.push("restart_required=true".to_string());
    }

    parts.join(" ")
}

#[derive(Default)]
struct WorkspaceReconcileReport {
    workspace_updates: usize,
    spec_updates: usize,
    ambiguous_feature_links: usize,
    ambiguous_spec_links: usize,
    ambiguous_feature_details: Vec<String>,
    ambiguous_spec_details: Vec<String>,
}

fn reconcile_workspace_links(project_dir: &Path, apply: bool) -> Result<WorkspaceReconcileReport> {
    let workspaces = list_workspaces(project_dir)?;
    let features = list_features(project_dir)?;
    let specs = list_specs(project_dir)?;

    let workspace_by_branch: HashMap<String, runtime::Workspace> = workspaces
        .iter()
        .map(|workspace| (workspace.branch.clone(), workspace.clone()))
        .collect();
    let workspace_by_id: HashMap<String, runtime::Workspace> = workspaces
        .iter()
        .map(|workspace| (workspace.id.clone(), workspace.clone()))
        .collect();

    let mut report = WorkspaceReconcileReport::default();

    for workspace in &workspaces {
        let feature_by_branch = features
            .iter()
            .filter(|entry| {
                entry.feature.metadata.branch.as_deref() == Some(workspace.branch.as_str())
            })
            .collect::<Vec<_>>();

        if workspace.feature_id.is_none() && feature_by_branch.len() > 1 {
            report.ambiguous_feature_links += 1;
            report.ambiguous_feature_details.push(format!(
                "{} -> [{}]",
                workspace.branch,
                feature_by_branch
                    .iter()
                    .map(|entry| entry.id.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        let selected_feature = if let Some(feature_id) = workspace.feature_id.as_deref() {
            features
                .iter()
                .find(|entry| entry.id == feature_id || entry.file_name == feature_id)
        } else if feature_by_branch.len() == 1 {
            feature_by_branch.first().copied()
        } else {
            None
        };

        let feature_candidate = if workspace.feature_id.is_none() {
            selected_feature.map(|entry| entry.id.clone())
        } else {
            None
        };

        let release_candidate = if workspace.release_id.is_none() {
            selected_feature.and_then(|entry| entry.feature.metadata.release_id.clone())
        } else {
            None
        };

        let branch_specs = specs
            .iter()
            .filter(|entry| {
                entry.spec.metadata.branch.as_deref() == Some(workspace.branch.as_str())
            })
            .collect::<Vec<_>>();

        if workspace.spec_id.is_none() && branch_specs.len() > 1 {
            report.ambiguous_spec_links += 1;
            report.ambiguous_spec_details.push(format!(
                "{} -> [{}]",
                workspace.branch,
                branch_specs
                    .iter()
                    .map(|entry| entry.id.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        let spec_candidate = if workspace.spec_id.is_none() {
            selected_feature
                .and_then(|entry| entry.feature.metadata.spec_id.clone())
                .or_else(|| {
                    if branch_specs.len() == 1 {
                        branch_specs.first().map(|entry| entry.id.clone())
                    } else {
                        None
                    }
                })
        } else {
            None
        };

        if feature_candidate.is_some() || spec_candidate.is_some() || release_candidate.is_some() {
            report.workspace_updates += 1;
            if apply {
                create_workspace(
                    project_dir,
                    CreateWorkspaceRequest {
                        branch: workspace.branch.clone(),
                        feature_id: feature_candidate,
                        spec_id: spec_candidate,
                        release_id: release_candidate,
                        ..CreateWorkspaceRequest::default()
                    },
                )?;
            }
        }
    }

    for spec_entry in specs {
        let mut spec = spec_entry.spec.clone();
        let mut changed = false;

        if spec.metadata.workspace_id.is_none()
            && let Some(branch) = spec.metadata.branch.as_deref()
            && let Some(workspace) = workspace_by_branch.get(branch)
        {
            spec.metadata.workspace_id = Some(workspace.id.clone());
            changed = true;
        }

        if spec.metadata.branch.is_none()
            && let Some(workspace_id) = spec.metadata.workspace_id.as_deref()
            && let Some(workspace) = workspace_by_id.get(workspace_id)
        {
            spec.metadata.branch = Some(workspace.branch.clone());
            changed = true;
        }

        if changed {
            report.spec_updates += 1;
            if apply {
                update_spec(project_dir, &spec_entry.id, spec)?;
            }
        }
    }

    Ok(report)
}

fn render_workspace_home(project_dir: &Path) -> Result<String> {
    let workspaces = list_workspaces(project_dir)?;
    let mut out = String::new();

    out.push_str("Workspace Home\n");
    out.push_str("--------------\n");

    if workspaces.is_empty() {
        out.push_str("No workspaces found.\n");
        out.push_str("Start here:\n");
        out.push_str("  ship workspace create feature/<name> --type feature\n");
        out.push_str(
            "  ship workspace create feature/<name> --type feature --feature-title \"<Feature Title>\" --checkout --activate --start-session --goal \"<goal>\" --provider <id> --no-input\n",
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
    out.push_str("  ship workspace reconcile [--apply]\n");
    out.push_str("  ship workspace repair --branch <branch> [--dry-run]\n");
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

            // Use the file name as the title directly (issue lookup removed)
            let issue_title = issue_file.clone();

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
        assert!(rendered.contains("Workspace Home"));
        assert!(
            rendered.contains("No workspaces found.") || rendered.contains("Workspaces:"),
            "unexpected workspace home output: {}",
            rendered
        );
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
                workspace_type: Some(ShipWorkspaceKind::Feature),
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
    fn cli_parses_feature_docs_update_with_status_and_verify() {
        let cli = Cli::try_parse_from([
            "ship",
            "feature",
            "docs",
            "update",
            "feat-auth",
            "--content",
            "Updated capability docs",
            "--status",
            "reviewed",
            "--verify",
        ])
        .expect("feature docs update should parse");

        match cli.command {
            Some(Commands::Feature {
                action:
                    FeatureCommands::Docs {
                        action:
                            FeatureDocCommands::Update {
                                id,
                                content,
                                status,
                                verify,
                            },
                    },
            }) => {
                assert_eq!(id, "feat-auth");
                assert_eq!(content, "Updated capability docs");
                assert_eq!(status.as_deref(), Some("reviewed"));
                assert!(verify);
            }
            other => panic!("unexpected parse result: {:?}", other),
        }
    }

    #[test]
    fn cli_parses_feature_docs_ensure_all() {
        let cli = Cli::try_parse_from(["ship", "feature", "docs", "ensure-all"])
            .expect("feature docs ensure-all should parse");

        match cli.command {
            Some(Commands::Feature {
                action:
                    FeatureCommands::Docs {
                        action: FeatureDocCommands::EnsureAll,
                    },
            }) => {}
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
                        start_session,
                        goal,
                        provider,
                        session_mode,
                        no_input,
                        ..
                    },
            }) => {
                assert_eq!(branch, "feature/workspace-first");
                assert_eq!(workspace_type.as_deref(), Some("feature"));
                assert_eq!(feature_title.as_deref(), Some("Workspace First"));
                assert!(checkout);
                assert!(activate);
                assert!(!start_session);
                assert_eq!(goal, None);
                assert_eq!(provider, None);
                assert_eq!(session_mode, None);
                assert!(!no_input);
            }
            other => panic!("unexpected parse result: {:?}", other),
        }
    }

    #[test]
    fn cli_parses_workspace_create_with_session_bootstrap_flags() {
        let cli = Cli::try_parse_from([
            "ship",
            "workspace",
            "create",
            "feature/session-bootstrap",
            "--type",
            "feature",
            "--start-session",
            "--goal",
            "Implement onboarding path",
            "--provider",
            "claude",
            "--session-mode",
            "code",
        ])
        .expect("workspace create with session bootstrap flags should parse");

        match cli.command {
            Some(Commands::Workspace {
                action:
                    WorkspaceCommands::Create {
                        branch,
                        workspace_type,
                        start_session,
                        goal,
                        provider,
                        session_mode,
                        no_input,
                        ..
                    },
            }) => {
                assert_eq!(branch, "feature/session-bootstrap");
                assert_eq!(workspace_type.as_deref(), Some("feature"));
                assert!(start_session);
                assert_eq!(goal.as_deref(), Some("Implement onboarding path"));
                assert_eq!(provider.as_deref(), Some("claude"));
                assert_eq!(session_mode.as_deref(), Some("code"));
                assert!(!no_input);
            }
            other => panic!("unexpected parse result: {:?}", other),
        }
    }

    #[test]
    fn cli_parses_workspace_create_with_no_input() {
        let cli = Cli::try_parse_from([
            "ship",
            "workspace",
            "create",
            "feature/automation-path",
            "--type",
            "feature",
            "--no-input",
        ])
        .expect("workspace create with no-input should parse");

        match cli.command {
            Some(Commands::Workspace {
                action:
                    WorkspaceCommands::Create {
                        branch,
                        workspace_type,
                        no_input,
                        ..
                    },
            }) => {
                assert_eq!(branch, "feature/automation-path");
                assert_eq!(workspace_type.as_deref(), Some("feature"));
                assert!(no_input);
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
    fn cli_parses_workspace_reconcile_apply() {
        let cli = Cli::try_parse_from(["ship", "workspace", "reconcile", "--apply"])
            .expect("workspace reconcile should parse");

        match cli.command {
            Some(Commands::Workspace {
                action: WorkspaceCommands::Reconcile { apply },
            }) => {
                assert!(apply);
            }
            other => panic!("unexpected parse result: {:?}", other),
        }
    }

    #[test]
    fn cli_parses_workspace_repair_dry_run() {
        let cli = Cli::try_parse_from([
            "ship",
            "workspace",
            "repair",
            "--branch",
            "feature/demo",
            "--dry-run",
        ])
        .expect("workspace repair should parse");

        match cli.command {
            Some(Commands::Workspace {
                action: WorkspaceCommands::Repair { branch, dry_run },
            }) => {
                assert_eq!(branch.as_deref(), Some("feature/demo"));
                assert!(dry_run);
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
    fn cli_parses_hooks_run_command() {
        let cli = Cli::try_parse_from(["ship", "hooks", "run", "--provider", "claude"])
            .expect("hooks run should parse");

        match cli.command {
            Some(Commands::Hooks {
                action: HooksCommands::Run { provider },
            }) => {
                assert_eq!(provider.as_deref(), Some("claude"));
            }
            other => panic!("unexpected parse result: {:?}", other),
        }
    }

    fn write_hook_runtime_artifacts_for_test(
        ship_dir: &Path,
        envelope: serde_json::Value,
        context: Option<&str>,
    ) -> Result<()> {
        let runtime_dir = ship_dir.join("agents").join("runtime");
        std::fs::create_dir_all(&runtime_dir)?;
        std::fs::write(
            runtime_dir.join("envelope.json"),
            serde_json::to_string_pretty(&envelope)?,
        )?;
        if let Some(context) = context {
            std::fs::write(runtime_dir.join("hook-context.md"), context)?;
        }
        Ok(())
    }

    #[test]
    fn claude_pre_tool_use_blocks_dangerous_shell_command() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = init_project(tmp.path().to_path_buf())?;
        write_hook_runtime_artifacts_for_test(
            &ship_dir,
            serde_json::json!({
                "workspace_root": tmp.path().to_string_lossy().to_string(),
                "allow_network": false,
                "allow_installs": false,
                "allowed_paths": ["."],
                "auto_approve_patterns": ["git status*", "git diff*"],
                "always_block_patterns": ["rm -rf *"],
                "require_confirmation": [],
                "tools_allow": ["*"],
                "tools_deny": []
            }),
            None,
        )?;

        let payload = serde_json::json!({
            "tool_name": "Bash",
            "tool_input": {
                "command": "git status && rm -rf /tmp/x"
            }
        });
        let response = build_hook_response(
            HookProvider::Claude,
            "PreToolUse",
            &payload,
            Some(ship_dir.as_path()),
        )
        .expect("expected deny response");
        assert_eq!(
            response["hookSpecificOutput"]["permissionDecision"].as_str(),
            Some("deny")
        );
        Ok(())
    }

    #[test]
    fn claude_pre_tool_use_auto_allows_safe_split_commands() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = init_project(tmp.path().to_path_buf())?;
        write_hook_runtime_artifacts_for_test(
            &ship_dir,
            serde_json::json!({
                "workspace_root": tmp.path().to_string_lossy().to_string(),
                "allow_network": false,
                "allow_installs": false,
                "allowed_paths": ["."],
                "auto_approve_patterns": ["git status*", "git diff*"],
                "always_block_patterns": ["rm -rf *"],
                "require_confirmation": [],
                "tools_allow": ["*"],
                "tools_deny": []
            }),
            None,
        )?;

        let payload = serde_json::json!({
            "tool_name": "Bash",
            "tool_input": {
                "command": "FOO=1 git status && BAR=2 git diff --stat"
            }
        });
        let response = build_hook_response(
            HookProvider::Claude,
            "PreToolUse",
            &payload,
            Some(ship_dir.as_path()),
        )
        .expect("expected allow response");
        assert_eq!(
            response["hookSpecificOutput"]["permissionDecision"].as_str(),
            Some("allow")
        );
        Ok(())
    }

    #[test]
    fn gemini_before_tool_denies_network_when_disabled() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = init_project(tmp.path().to_path_buf())?;
        write_hook_runtime_artifacts_for_test(
            &ship_dir,
            serde_json::json!({
                "workspace_root": tmp.path().to_string_lossy().to_string(),
                "allow_network": false,
                "allow_installs": false,
                "allowed_paths": ["."],
                "auto_approve_patterns": ["git status*"],
                "always_block_patterns": [],
                "require_confirmation": [],
                "tools_allow": ["*"],
                "tools_deny": []
            }),
            None,
        )?;

        let payload = serde_json::json!({
            "tool_name": "run_shell_command",
            "tool_input": {
                "command": "curl https://example.com"
            }
        });
        let response = build_hook_response(
            HookProvider::Gemini,
            "BeforeTool",
            &payload,
            Some(ship_dir.as_path()),
        )
        .expect("expected deny response");
        assert_eq!(response["decision"].as_str(), Some("deny"));
        Ok(())
    }

    #[test]
    fn session_start_emits_context_via_json_output() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = init_project(tmp.path().to_path_buf())?;
        write_hook_runtime_artifacts_for_test(
            &ship_dir,
            serde_json::json!({}),
            Some("# Hook Context\n\nShip-first instructions."),
        )?;

        let response = build_hook_response(
            HookProvider::Gemini,
            "SessionStart",
            &serde_json::json!({}),
            Some(ship_dir.as_path()),
        )
        .expect("expected context response");
        assert_eq!(
            response["hookSpecificOutput"]["additionalContext"].as_str(),
            Some("# Hook Context\n\nShip-first instructions.")
        );
        Ok(())
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
    fn cli_parses_ui_subcommand() {
        let cli = Cli::try_parse_from(["ship", "ui"]).expect("ui should parse");
        match cli.command {
            Some(Commands::Ui {
                dev,
                watch,
                release,
            }) => {
                assert!(!dev);
                assert!(!watch);
                assert!(!release);
            }
            other => panic!("unexpected parse result: {:?}", other),
        }
    }

    #[test]
    fn cli_parses_ui_subcommand_with_flags() {
        let cli = Cli::try_parse_from(["ship", "ui", "--dev", "--watch", "--release"])
            .expect("ui flags should parse");
        match cli.command {
            Some(Commands::Ui {
                dev,
                watch,
                release,
            }) => {
                assert!(dev);
                assert!(watch);
                assert!(release);
            }
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
