//! `ship compile` — load the project library, resolve, compile, write.

mod agent;
mod mcp_allowlist;
pub(crate) mod output;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_agent;

use anyhow::{Context, Result};
use compiler::{compile, resolve_library};
use std::path::Path;

use crate::dep_skills::resolve_dep_skills;
use crate::loader::load_library;

use agent::{apply_agent_to_library, collect_all_skill_refs};
use mcp_allowlist::{
    ensure_ship_mcp_allowed_claude, ensure_ship_mcp_allowed_cursor, ensure_ship_mcp_allowed_gemini,
};
use output::{ensure_session_gitignored, print_dry_run};

// Re-export the public API so `use crate::compile::{CompileOptions, run_compile}` works.
pub use output::write_output;

// ── Entry point ──────────────────────────────────────────────────────────────

pub struct CompileOptions<'a> {
    /// Project root (where CLAUDE.md etc. get written).
    pub project_root: &'a Path,
    /// Optionally restrict to a single provider.
    pub provider: Option<&'a str>,
    /// Print what would be written without touching the filesystem.
    pub dry_run: bool,
    /// Active mode id (already resolved from PathContext / ship.toml).
    pub active_agent: Option<&'a str>,
}

pub fn run_compile(opts: CompileOptions<'_>) -> Result<()> {
    let ship_dir = opts.project_root.join(".ship");

    // 1. Load raw library from .ship/ (flat layout)
    let mut library = load_library(&ship_dir).context("failed to load .ship/")?;

    // 2. Apply mode overrides (permissions, inline rules, provider list)
    if let Some(mode_id) = opts.active_agent {
        apply_agent_to_library(&mut library, mode_id, opts.project_root)?;
    }
    library.active_agent = opts.active_agent.map(str::to_string);

    // 2b. Resolve dep skill refs from cached packages.
    //     Collect all skill refs declared across agent profiles and mode configs,
    //     resolve any github.com/ refs from ship.lock + cache, and merge into
    //     library.skills before passing to the compiler. Local refs are skipped.
    {
        let all_skill_refs = collect_all_skill_refs(&library);
        if !all_skill_refs.is_empty() {
            let lock_path = opts.project_root.join(".ship").join("ship.lock");
            let dep_skills = resolve_dep_skills(
                &all_skill_refs,
                &library.skills,
                &lock_path,
                None, // use default ~/.ship/cache/
            )
            .context("resolving dep skills from cache")?;
            library.skills.extend(dep_skills);
        }
    }

    // 3. Resolve (mode filtering, provider selection)
    let resolved = resolve_library(&library, None, opts.active_agent);

    // 4. Determine providers to compile for
    let providers: Vec<String> = match opts.provider {
        Some(p) => vec![p.to_string()],
        None => resolved.providers.clone(),
    };

    if providers.is_empty() {
        println!("No providers configured. Add providers to .ship/ship.toml or mode file.");
        return Ok(());
    }

    // 5. Compile + write
    for provider_id in &providers {
        let Some(output) = compile(&resolved, provider_id) else {
            eprintln!("warning: unknown provider '{}', skipping", provider_id);
            continue;
        };
        if opts.dry_run {
            print_dry_run(provider_id, &output);
        } else {
            write_output(opts.project_root, provider_id, &output)
                .with_context(|| format!("failed to write {} output", provider_id))?;
        }
    }

    // 6. Pre-approve ship MCP tools globally for each provider.
    //    The ship-mcp server is always injected by the compiler into every compile.
    //    This avoids per-session approval prompts for ship MCP tools.
    if !opts.dry_run {
        for provider in &providers {
            let result = match provider.as_str() {
                "claude" => ensure_ship_mcp_allowed_claude(),
                "gemini" => ensure_ship_mcp_allowed_gemini(),
                "cursor" => ensure_ship_mcp_allowed_cursor(opts.project_root),
                _ => Ok(()),
            };
            if let Err(e) = result {
                eprintln!("warning: could not update {provider} MCP permissions: {e}");
            }
        }
    }

    if !opts.dry_run {
        ensure_session_gitignored(opts.project_root)?;
        println!("✓ compiled for: {}", providers.join(", "));
    }
    Ok(())
}
