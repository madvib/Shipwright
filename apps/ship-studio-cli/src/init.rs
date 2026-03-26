//! `ship init` — scaffold .ship/ in a project or configure ~/.ship/ globally.

use anyhow::Result;

use crate::paths;

pub fn run(global: bool, provider: Option<String>) -> Result<()> {
    if global {
        run_global()
    } else {
        run_project(provider)
    }
}

/// Quote a string for TOML output.
pub(crate) fn quote_toml(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

/// Replace characters that are not safe in filenames.
pub(crate) fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

// ── Normal init paths ────────────────────────────────────────────────────────

fn run_global() -> Result<()> {
    paths::ensure_global_dirs()?;
    let gdir = paths::global_dir();
    let cfg_path = gdir.join("config.toml");
    if !cfg_path.exists() {
        std::fs::write(
            &cfg_path,
            "# Ship global configuration\n\n[identity]\nname = \"\"\n",
        )?;
    }
    if !gdir.join("README.md").exists() {
        std::fs::write(
            gdir.join("README.md"),
            "# Ship\n\nSee https://getship.dev\n",
        )?;
    }
    println!("initialized global config at ~/.ship/");
    println!("  Edit ~/.ship/config.toml to set your identity");
    Ok(())
}

fn run_project(provider: Option<String>) -> Result<()> {
    let project_root = std::env::current_dir()?;

    // ── Auto-detect existing provider configs when no --provider flag ─────────
    let is_fresh = !paths::project_ship_jsonc().exists() && !paths::project_ship_toml().exists();
    let detected = if is_fresh && provider.is_none() {
        let d = compiler::detect_providers(&project_root);
        if d.any() {
            Some(d)
        } else {
            None
        }
    } else {
        None
    };

    paths::ensure_project_dirs()?;
    let ship_jsonc = paths::project_ship_jsonc();

    if let Some(ref detected) = detected {
        // Import detected provider configs into .ship/
        let providers = detected.as_list();
        println!(
            "Detected existing configs: {}",
            providers.join(", ")
        );

        let library = compiler::decompile_all(&project_root);

        // Write ship.jsonc with detected providers and provider_defaults
        if !ship_jsonc.exists() {
            let mut project = serde_json::json!({
                "providers": providers.iter()
                    .map(|p| serde_json::Value::String(p.to_string()))
                    .collect::<Vec<_>>()
            });
            if !library.provider_defaults.is_empty() {
                let defaults: serde_json::Map<String, serde_json::Value> = library
                    .provider_defaults
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                project["provider_defaults"] = serde_json::Value::Object(defaults);
            }
            let manifest = serde_json::json!({
                "$schema": "../schemas/ship.schema.json",
                "project": project
            });
            std::fs::write(&ship_jsonc, serde_json::to_string_pretty(&manifest)?)?;
        }

        // Write MCP servers
        if !library.mcp_servers.is_empty() {
            let mcp_jsonc = paths::mcp_path();
            if !mcp_jsonc.exists() {
                let mut servers = serde_json::Map::new();
                for s in &library.mcp_servers {
                    let mut entry = serde_json::json!({});
                    if !s.command.is_empty() {
                        entry["command"] = serde_json::json!(s.command);
                    }
                    if !s.args.is_empty() {
                        entry["args"] = serde_json::json!(s.args);
                    }
                    if !s.env.is_empty() {
                        entry["env"] = serde_json::json!(s.env);
                    }
                    if let Some(url) = &s.url {
                        entry["url"] = serde_json::json!(url);
                    }
                    servers.insert(s.id.clone(), entry);
                }
                let mcp_obj = serde_json::json!({
                    "$schema": "../schemas/mcp.schema.json",
                    "mcp": { "servers": servers }
                });
                std::fs::write(&mcp_jsonc, serde_json::to_string_pretty(&mcp_obj)?)?;
                println!("  imported {} MCP servers", library.mcp_servers.len());
            }
        }

        // Write rules
        if !library.rules.is_empty() {
            let rules_dir = paths::rules_dir();
            std::fs::create_dir_all(&rules_dir)?;
            for rule in &library.rules {
                let file_name = if rule.file_name.ends_with(".md") {
                    rule.file_name.clone()
                } else {
                    format!("{}.md", rule.file_name)
                };
                let rule_path = rules_dir.join(&file_name);
                if !rule_path.exists() {
                    std::fs::write(&rule_path, &rule.content)?;
                }
            }
            println!("  imported {} rules", library.rules.len());
        }
    } else if !ship_jsonc.exists() && !paths::project_ship_toml().exists() {
        // No detection — scaffold default
        let prov = provider.as_deref().unwrap_or("claude");
        std::fs::write(
            &ship_jsonc,
            format!(
                "{{\n  \"$schema\": \"../schemas/ship.schema.json\",\n  \
                 \"project\": {{\n    \"providers\": [\"{prov}\"],\n  }},\n}}"
            ),
        )?;
    }

    let pdir = paths::project_dir();
    if !pdir.join(".gitignore").exists() {
        std::fs::write(
            pdir.join(".gitignore"),
            "# Ship compiled artifacts\n/secrets/\nCLAUDE.md\nGEMINI.md\n\
             AGENTS.md\n.mcp.json\n.codex/\n.gemini/\n.cursor/\n",
        )?;
    }
    if !pdir.join("README.md").exists() {
        std::fs::write(
            pdir.join("README.md"),
            "# Ship Configuration\n\nManaged by [Ship](https://getship.dev). \
             Run `ship use <agent-id>` to activate.\n",
        )?;
    }

    // Seed default @ship/* dependencies and attempt registry install.
    let (deps_seeded, registry_ok) = seed_and_install_deps(&pdir);

    let effective_providers = if let Some(ref d) = detected {
        d.as_list().join(", ")
    } else {
        provider.as_deref().unwrap_or("claude").to_string()
    };

    println!("initialized .ship/ in current directory");
    println!("  providers: {}", effective_providers);
    if detected.is_some() {
        println!("  imported existing provider configs into .ship/");
    }
    if deps_seeded > 0 {
        if registry_ok {
            println!(
                "  dependencies: {} packages installed from registry",
                deps_seeded,
            );
        } else {
            println!(
                "  dependencies: {} declared (install with `ship install` when online)",
                deps_seeded,
            );
        }
    }
    println!("\nNext steps:");
    println!("  ship use <agent-id>     activate an agent");
    println!("  ship compile            re-compile current agent");
    Ok(())
}

/// Seed default deps and attempt install. Never fails -- returns counts for display.
fn seed_and_install_deps(ship_dir: &std::path::Path) -> (usize, bool) {
    use runtime::registry::init_deps;

    let seeded = match init_deps::seed_default_dependencies(ship_dir) {
        Ok(r) => r.added,
        Err(_) => 0,
    };

    if seeded == 0 {
        return (0, false);
    }

    let installed = init_deps::try_install_init_deps(ship_dir).unwrap_or_default();

    (seeded, installed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_and_quote() {
        assert_eq!(sanitize_filename("hello world"), "hello-world");
        assert_eq!(sanitize_filename("path/name"), "path-name");
        assert_eq!(quote_toml("simple"), "\"simple\"");
        assert_eq!(quote_toml("a\"b"), "\"a\\\"b\"");
    }
}
