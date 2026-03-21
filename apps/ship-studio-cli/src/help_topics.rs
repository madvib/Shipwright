//! `ship help <topic>` — extended help for common workflows.

use anyhow::Result;

pub fn run(topic: Option<&str>) -> Result<()> {
    match topic {
        None | Some("topics") => print_topic_list(),
        Some(t) => match lookup(t) {
            Some(text) => print!("{}", text),
            None => {
                eprintln!("Unknown help topic: {}", t);
                eprintln!();
                print_topic_list();
                anyhow::bail!("run `ship help topics` to see available topics");
            }
        },
    }
    Ok(())
}

fn print_topic_list() {
    println!("Available help topics:");
    println!();
    for (name, summary) in TOPICS {
        println!("  {:<12} {}", name, summary);
    }
    println!();
    println!("Run `ship help <topic>` for details.");
}

const TOPICS: &[(&str, &str)] = &[
    ("agents", "Creating and managing agent definitions"),
    ("compile", "How compilation works and provider output"),
    ("config", "User preferences and environment variables"),
    ("mcp", "MCP server configuration and the Ship MCP server"),
    ("providers", "Supported providers and their output formats"),
    ("skills", "Adding and authoring agent skills"),
    ("workflow", "Typical day-to-day workflow with Ship"),
];

fn lookup(topic: &str) -> Option<&'static str> {
    match topic {
        "agents" => Some(TOPIC_AGENTS),
        "compile" => Some(TOPIC_COMPILE),
        "config" => Some(TOPIC_CONFIG),
        "mcp" => Some(TOPIC_MCP),
        "providers" => Some(TOPIC_PROVIDERS),
        "skills" => Some(TOPIC_SKILLS),
        "workflow" => Some(TOPIC_WORKFLOW),
        _ => None,
    }
}

const TOPIC_AGENTS: &str = "\
Agents

An agent is a TOML file in .ship/agents/ that declares an AI assistant's
identity, skills, permissions, and provider targets.

  Create:   ship agent create rust-expert
  List:     ship agent list
  Edit:     ship agent edit rust-expert
  Activate: ship use rust-expert
  Delete:   ship agent delete rust-expert
  Clone:    ship agent clone rust-expert go-expert

Agent files live in .ship/agents/<id>.toml. Global agents live in
~/.ship/agents/<id>.toml (create with --global).

After editing an agent, run `ship compile` or `ship use <id>` to regenerate
provider-native config files.
";

const TOPIC_COMPILE: &str = "\
Compilation

`ship compile` reads the active agent from .ship/ and writes provider-native
config files to the project root (e.g. CLAUDE.md, .cursor/, .mcp.json).

  Compile all providers:    ship compile
  Single provider:          ship compile --provider claude
  Preview without writing:  ship compile --dry-run

The compiler resolves skills, permissions, MCP servers, and rules into a
single output per provider. Output files are build artifacts and should be
gitignored.

Run `ship validate` before compile to catch config errors early.
";

const TOPIC_CONFIG: &str = "\
Configuration

User preferences live in ~/.ship/config.toml. Read and write them with:

  ship config get <key>       Read a value
  ship config set <key> <val> Write a value
  ship config list            Show all set values
  ship config path            Show config file location

Available keys:

  terminal.program    Terminal for dispatch: wt, iterm, tmux, gnome, vscode, manual
  dispatch.confirm    Show spec and ask y/n before launching agent (true/false)
  worktrees.dir       Base directory for worktrees (default: ~/dev/ship-worktrees)
  defaults.provider   Default compilation provider (claude, gemini, codex, cursor)
  defaults.mode       Default agent permission mode
  identity.name       Your display name
  identity.email      Your email
  cloud.base_url      Ship API base URL

Environment variables override config values when set:

  SHIP_DEFAULT_TERMINAL   Overrides terminal.program
  SHIP_DISPATCH_CONFIRM   Set to 1 to override dispatch.confirm
  SHIP_WORKTREE_DIR       Overrides worktrees.dir

Resolution order: command flag > env var > ship config > default.
";

const TOPIC_MCP: &str = "\
MCP Servers

Ship can both consume and serve MCP (Model Context Protocol) servers.

Consuming (declare servers your agent connects to):
  ship mcp add my-server --url https://example.com/mcp
  ship mcp add-stdio my-tool my-binary --name \"My Tool\"
  ship mcp list
  ship mcp remove my-server

Serving (run the Ship MCP server for project intelligence):
  ship mcp serve              stdio mode (default, for Claude Code)
  ship mcp serve --http       HTTP daemon mode for CI/remote agents

Server definitions live in .ship/agents/mcp.toml.
";

const TOPIC_PROVIDERS: &str = "\
Providers

Ship compiles agent config to provider-native formats:

  claude    CLAUDE.md + .mcp.json
  cursor    .cursor/ directory
  codex     .codex/ directory + AGENTS.md
  gemini    .gemini/ directory + GEMINI.md

Set providers in your agent TOML:
  [agent]
  providers = [\"claude\", \"cursor\"]

Or compile for one provider at a time:
  ship compile --provider claude
";

const TOPIC_SKILLS: &str = "\
Skills

Skills are markdown files that add domain knowledge or workflow instructions
to an agent. They live in .ship/agents/skills/.

  Install from registry: ship skill add ship-coordination
  Install from path:     ship skill add ./my-skills/review
  Create a new skill:    ship skill create my-skill
  List installed:        ship skill list
  Remove:                ship skill remove my-skill

Skills are referenced in agent TOML files:
  [skills]
  refs = [\"ship-coordination\", \"my-skill\"]

A skill is either a single .md file or a directory with SKILL.md.
";

const TOPIC_WORKFLOW: &str = "\
Typical Workflow

1. Initialize a project:
   ship init

2. Create an agent:
   ship agent create my-agent

3. Edit the agent to add skills, permissions, providers:
   ship agent edit my-agent

4. Activate and compile:
   ship use my-agent

5. After editing agent config, recompile:
   ship compile

6. Validate config before committing:
   ship validate

7. Install shared dependencies:
   ship install

The compiled output (CLAUDE.md, .cursor/, etc.) is gitignored.
Only .ship/ is committed to version control.
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_topics_resolve() {
        for (name, _) in TOPICS {
            assert!(lookup(name).is_some(), "topic '{}' listed but not found", name);
        }
    }

    #[test]
    fn unknown_topic_returns_none() {
        assert!(lookup("nonexistent").is_none());
    }

    #[test]
    fn run_topics_list_succeeds() {
        assert!(run(Some("topics")).is_ok());
    }

    #[test]
    fn run_known_topic_succeeds() {
        assert!(run(Some("agents")).is_ok());
    }

    #[test]
    fn run_unknown_topic_fails() {
        assert!(run(Some("nonexistent")).is_err());
    }

    #[test]
    fn run_none_shows_list() {
        assert!(run(None).is_ok());
    }
}
