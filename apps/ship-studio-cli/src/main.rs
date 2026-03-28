mod add;
mod add_from;
mod add_from_write;
mod agent;
mod agent_config;
mod audit;
mod auth;
mod cli;
mod commands;
mod compile;
mod config;
mod convert;
mod dep_skills;
#[cfg(feature = "unstable")]
mod diff;
#[cfg(feature = "unstable")]
mod events_cmd;
mod help_topics;
mod hook;
mod init;
mod install;
#[cfg(feature = "unstable")]
mod job;
mod loader;
mod logging;
mod mcp;
mod mcp_serve;
mod paths;
mod profile;
mod publish;
mod skill;
mod validate;
mod vars;
#[cfg(feature = "unstable")]
mod view;

use anyhow::Result;
use cli::{AgentCommands, Cli, Commands, ConfigCommands, HookCommands, McpCommands, SkillCommands, VarsCommands};
#[cfg(feature = "unstable")]
use cli::{EventsCommands, JobCommands};
use std::path::PathBuf;

pub fn build_version() -> &'static str {
    if cfg!(feature = "unstable") {
        concat!(env!("CARGO_PKG_VERSION"), "+unstable")
    } else {
        env!("CARGO_PKG_VERSION")
    }
}

fn main() -> Result<()> {
    use clap::Parser;
    let _log_guard = logging::init();
    let cli = Cli::parse();
    dispatch(cli.command)
}

fn dispatch(command: Option<Commands>) -> Result<()> {
    match command {
        None => run_status(None),
        Some(cmd) => match cmd {
            Commands::Init {
                global,
                provider,
                force: _,
            } => init::run(global, provider),
            Commands::Config { action } => dispatch_config(action),
            Commands::Login => auth::run_login(),
            Commands::Logout => auth::run_logout(),
            Commands::Whoami => auth::run_whoami(),
            Commands::Use {
                agent_id,
                path,
                compile: _,
            } => run_use(Some(&agent_id), path),
            Commands::Status { path } => run_status(path),
            Commands::Agents { action } => dispatch_agent(action),
            Commands::Compile {
                provider,
                dry_run,
                path,
            } => run_compile_cmd(provider.as_deref(), dry_run, path),
            Commands::Skill { action } => dispatch_skill(action),
            Commands::Vars { action } => dispatch_vars(action),
            Commands::Mcp { action } => dispatch_mcp(action),
            Commands::Convert { source } => convert::run_convert(&source),
            Commands::Docs { topic } => help_topics::run(topic.as_deref()),
            #[cfg(feature = "unstable")]
            Commands::Job { action } => dispatch_job(action),
            #[cfg(feature = "unstable")]
            Commands::Adrs => run_adrs(),
            #[cfg(feature = "unstable")]
            Commands::Notes => run_notes(),
            Commands::Publish { dry_run, tag } => {
                let root = std::env::current_dir()?;
                publish::run_publish(&root, dry_run, tag.as_deref())
            }
            Commands::Install { frozen, offline } => {
                let root = std::env::current_dir()?;
                install::run_install(&root, frozen, offline)
            }
            Commands::Add { package, from } => match (package, from) {
                (_, Some(url)) => add_from::run_add_from(&url),
                (Some(pkg), None) => {
                    let root = std::env::current_dir()?;
                    add::run_add(&root, &pkg)
                }
                (None, None) => anyhow::bail!("provide a package name or --from <url>"),
            },
            Commands::Audit { path, json } => audit::run_audit(path, json),
            Commands::Validate { agent, json, path } => {
                let root = path
                    .as_deref()
                    .map(std::fs::canonicalize)
                    .transpose()?
                    .unwrap_or_else(|| std::env::current_dir().unwrap());
                validate::run_validate(agent.as_deref(), json, &root)
            }
            #[cfg(feature = "unstable")]
            Commands::Diff { milestone } => diff::run(milestone.as_deref()),
            #[cfg(feature = "unstable")]
            Commands::Events { action } => dispatch_events(action),
            #[cfg(feature = "unstable")]
            Commands::View => view::run_view(),
            Commands::Hook { action } => dispatch_hook(action),
            Commands::Help => {
                use clap::CommandFactory;
                Cli::command().print_help()?;
                Ok(())
            }
        },
    }
}

// ── Config ────────────────────────────────────────────────────────────────────

fn dispatch_config(action: ConfigCommands) -> Result<()> {
    match action {
        ConfigCommands::Get { key } => {
            let cfg = config::ShipConfig::load();
            match cfg.get(&key) {
                Some(v) => println!("{}", v),
                None => {
                    eprintln!("{} is not set", key);
                    std::process::exit(1);
                }
            }
        }
        ConfigCommands::Set { key, value } => {
            let mut cfg = config::ShipConfig::load();
            cfg.set(&key, &value)?;
            cfg.save()?;
            println!("{} = {}", key, value);
        }
        ConfigCommands::List => {
            let cfg = config::ShipConfig::load();
            let entries = cfg.list();
            if entries.is_empty() {
                println!(
                    "No config set. File: {}",
                    config::ShipConfig::path().display()
                );
            } else {
                for (k, v) in &entries {
                    println!("{} = {}", k, v);
                }
            }
        }
        ConfigCommands::Path => {
            println!("{}", config::ShipConfig::path().display());
        }
    }
    Ok(())
}

// ── Use ───────────────────────────────────────────────────────────────────────

/// Activate an agent: load, compile, install plugins, write workspace state.
fn run_use(agent_id: Option<&str>, path: Option<PathBuf>) -> Result<()> {
    let ship_dir = match path {
        Some(ref p) => {
            let sd = std::fs::canonicalize(p)?.join(".ship");
            anyhow::ensure!(
                sd.exists(),
                ".ship/ not found in {}. Run: ship init",
                p.display()
            );
            sd
        }
        None => paths::project_ship_dir_required()?,
    };
    let project_root = ship_dir.parent().unwrap().to_path_buf();

    // When --path is not given and project_root was resolved via worktree pointer,
    // the cwd (worktree) differs from project_root (main repo). Compiled output
    // (CLAUDE.md, .claude/settings.json) must go to the cwd, not the main repo.
    let output_root = if path.is_none() {
        let cwd = std::env::current_dir()?;
        if cwd != project_root { Some(cwd) } else { None }
    } else {
        None
    };

    profile::activate_agent(agent_id, &project_root, output_root.as_deref())
}

// ── Status ────────────────────────────────────────────────────────────────────

fn run_status(path: Option<PathBuf>) -> Result<()> {
    let target = path
        .as_deref()
        .map(std::fs::canonicalize)
        .transpose()?
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    let ship_dir = target.join(".ship");
    let state = profile::WorkspaceState::load(&ship_dir);
    match state.active_agent {
        Some(ref p) => {
            println!("active agent: {}", p);
            if let Some(ref at) = state.compiled_at {
                println!("compiled: {}", at);
            }
            if !state.plugins_installed.is_empty() {
                println!("plugins: {}", state.plugins_installed.join(", "));
            }
        }
        None => {
            println!("No active agent for {}", target.display());
            println!("Run: ship use <agent-id>");
        }
    }
    Ok(())
}

// ── Agent ─────────────────────────────────────────────────────────────────────

fn dispatch_agent(action: AgentCommands) -> Result<()> {
    match action {
        AgentCommands::List { local, project } => {
            let agents = paths::list_agent_ids(local, project);
            if agents.is_empty() {
                println!("No agents found.");
                println!("Create one with: ship agents create <name>");
            } else {
                for (id, scope) in &agents {
                    println!("  {} [{}]", id, scope);
                }
            }
        }
        AgentCommands::Create { name, global } => {
            let dir = if global {
                paths::global_agents_dir()
            } else {
                paths::agents_dir()
            };
            std::fs::create_dir_all(&dir)?;
            let path = dir.join(format!("{}.jsonc", name));
            if path.exists() {
                anyhow::bail!("Agent '{}' already exists at {}", name, path.display());
            }
            std::fs::write(&path, agent_config::AgentConfig::scaffold_jsonc(&name))?;
            println!("created agent '{}' at {}", name, path.display());
        }
        AgentCommands::Edit { name, editor } => {
            let path = profile::find_agent_file(&name, &std::env::current_dir()?)
                .ok_or_else(|| anyhow::anyhow!("Agent '{}' not found", name))?;
            let editor = editor
                .or_else(|| std::env::var("EDITOR").ok())
                .unwrap_or_else(|| "vi".to_string());
            std::process::Command::new(&editor).arg(&path).status()?;
        }
        AgentCommands::Delete { name } => {
            let path = profile::find_agent_file(&name, &std::env::current_dir()?)
                .ok_or_else(|| anyhow::anyhow!("Agent '{}' not found", name))?;
            std::fs::remove_file(&path)?;
            println!("deleted agent '{}'", name);
        }
        AgentCommands::Clone { source, target } => {
            let cwd = std::env::current_dir()?;
            let src_path = profile::find_agent_file(&source, &cwd)
                .ok_or_else(|| anyhow::anyhow!("Source agent '{}' not found", source))?;
            let dst_path = src_path.parent().unwrap().join(format!("{}.jsonc", target));
            if dst_path.exists() {
                anyhow::bail!("Target agent '{}' already exists", target);
            }
            let content = std::fs::read_to_string(&src_path)?
                .replace(
                    &format!("\"id\": \"{}\"", source),
                    &format!("\"id\": \"{}\"", target),
                )
                .replace(
                    &format!("\"name\": \"{}\"", source),
                    &format!("\"name\": \"{}\"", target),
                );
            std::fs::write(&dst_path, content)?;
            println!("cloned '{}' -> '{}'", source, target);
        }
        AgentCommands::Log { message } => agent::agent_log(&message)?,
    }
    Ok(())
}

// ── Compile ───────────────────────────────────────────────────────────────────

fn run_compile_cmd(
    provider: Option<&str>,
    dry_run: bool,
    path: Option<PathBuf>,
) -> Result<()> {
    let project_root = path
        .as_deref()
        .map(std::fs::canonicalize)
        .transpose()?
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    if !project_root.join(".ship").exists() {
        anyhow::bail!(".ship/ not found. Run: ship init");
    }

    let state = profile::WorkspaceState::load(&project_root.join(".ship"));
    compile::run_compile(compile::CompileOptions {
        project_root: &project_root,
        output_root: None,
        provider,
        dry_run,
        active_agent: state.active_agent.as_deref(),
    })
}

// ── Subcommand dispatchers ────────────────────────────────────────────────────

#[cfg(feature = "unstable")]
fn dispatch_job(action: JobCommands) -> Result<()> {
    match action {
        JobCommands::Create {
            kind,
            title,
            milestone,
            description,
            branch,
        } => job::create(
            &kind,
            &title,
            milestone.as_deref(),
            description.as_deref(),
            branch.as_deref(),
        ),
        JobCommands::List {
            status,
            branch,
            milestone,
        } => job::list(status.as_deref(), branch.as_deref(), milestone.as_deref()),
        JobCommands::Update { id, status } => job::update(&id, &status),
        JobCommands::Done { id } => job::update(&id, "complete"),
    }
}

fn dispatch_skill(action: SkillCommands) -> Result<()> {
    match action {
        SkillCommands::List => skill::list(),
        SkillCommands::Create {
            id,
            name,
            description,
        } => skill::create(&id, name.as_deref(), description.as_deref()),
        SkillCommands::Remove { id, global } => skill::remove(&id, global),
        SkillCommands::Add {
            source,
            skill,
            global,
        } => skill::add(&source, skill.as_deref(), global),
    }
}

fn dispatch_vars(action: VarsCommands) -> Result<()> {
    let ship_dir = paths::project_ship_dir_required()?;
    match action {
        VarsCommands::Set { skill_id, key, value } => {
            vars::run_vars_set(&ship_dir, &skill_id, &key, &value)
        }
        VarsCommands::Get { skill_id, key } => {
            vars::run_vars_get(&ship_dir, &skill_id, key.as_deref())
        }
        VarsCommands::Append { skill_id, key, json } => {
            vars::run_vars_append(&ship_dir, &skill_id, &key, &json)
        }
        VarsCommands::Reset { skill_id } => {
            vars::run_vars_reset(&ship_dir, &skill_id)
        }
    }
}

fn dispatch_mcp(action: McpCommands) -> Result<()> {
    match action {
        McpCommands::Serve { http, port } => mcp_serve::run(http, port),
        McpCommands::List => mcp::list(),
        McpCommands::Add { id, name, url, .. } => {
            let url =
                url.ok_or_else(|| anyhow::anyhow!("--url is required for HTTP/SSE servers"))?;
            mcp::add_http(&id, name, &url)
        }
        McpCommands::AddStdio {
            id,
            command,
            args,
            name,
            ..
        } => mcp::add_stdio(&id, name, &command, args),
        McpCommands::Remove { id } => mcp::remove(&id),
    }
}

// ── Hidden / legacy ──────────────────────────────────────────────────────────

#[cfg(feature = "unstable")]
fn run_adrs() -> Result<()> {
    let _ship_dir = paths::project_ship_dir_required()?;
    let adrs = runtime::db::adrs::list_adrs()?;
    if adrs.is_empty() {
        println!("No ADRs found.");
    } else {
        for entry in &adrs {
            println!("{}\t[{}]\t{}", entry.id, entry.status, entry.title);
        }
    }
    Ok(())
}

#[cfg(feature = "unstable")]
fn run_notes() -> Result<()> {
    let _ship_dir = paths::project_ship_dir_required()?;
    let notes = runtime::db::notes::list_notes(None)?;
    if notes.is_empty() {
        println!("No notes found.");
    } else {
        for entry in &notes {
            println!("{}\t{}", entry.id, entry.title);
        }
    }
    Ok(())
}

fn dispatch_hook(action: HookCommands) -> Result<()> {
    match action {
        HookCommands::BeforeTool => hook::run_before_tool(),
        HookCommands::AfterTool => hook::run_after_tool(),
        HookCommands::SessionEnd => hook::run_session_end(),
    }
}

#[cfg(feature = "unstable")]
fn dispatch_events(action: EventsCommands) -> Result<()> {
    let ship_dir = paths::project_ship_dir_required()?;
    match action {
        EventsCommands::List {
            since,
            actor,
            entity,
            action,
            limit,
            json,
        } => events_cmd::run_events(&ship_dir, since, actor, entity, action, limit, json),
    }
}
