//! Render the Config section: Agents, Skills, MCP Servers.
//! Agents read from .ship/agents/. Skills merge local + registry deps.

use std::path::PathBuf;

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, ListState, Paragraph};

use super::data::ViewData;
use super::nav::{NavState, Panel};
use super::theme::*;

/// A discovered skill entry (local or from a registry dependency).
pub struct SkillEntry {
    pub name: String,
    /// "local" or short package label like "garrytan/gstack".
    pub source: String,
    /// Path to the SKILL.md file.
    pub path: PathBuf,
}

pub fn draw(frame: &mut Frame, nav: &NavState, data: &ViewData, area: Rect) {
    match nav.panel() {
        Panel::AgentProfiles => draw_agents(frame, nav, area),
        Panel::Skills => draw_skills(frame, nav, area),
        Panel::McpServers => draw_mcp(frame, data, area),
        _ => {}
    }
}

fn draw_agents(frame: &mut Frame, nav: &NavState, area: Rect) {
    let agent_files = discover_agents();
    if agent_files.is_empty() {
        frame.render_widget(
            Paragraph::new("  No agent configs in .ship/agents/")
                .style(Style::default().fg(C_MUT).bg(C_BG))
                .block(panel("Agents")),
            area,
        );
        return;
    }

    let items: Vec<ListItem> = agent_files
        .iter()
        .map(|name| {
            let model = read_agent_model(name);
            let label = match model {
                Some(m) => format!("{name}  ({m})"),
                None => name.clone(),
            };
            ListItem::new(Line::from(vec![
                Span::styled("  \u{25c6} ", Style::default().fg(C_PRI)),
                Span::styled(label, Style::default().fg(C_FG)),
            ]))
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(nav.list_selected));
    frame.render_stateful_widget(
        List::new(items)
            .block(panel(format!("Agents  ({})", agent_files.len())))
            .highlight_style(selected_style())
            .highlight_symbol("\u{25b6} "),
        area,
        &mut state,
    );
}

fn draw_skills(frame: &mut Frame, nav: &NavState, area: Rect) {
    let skills = discover_all_skills();
    if skills.is_empty() {
        frame.render_widget(
            Paragraph::new("  No skills installed.")
                .style(Style::default().fg(C_MUT).bg(C_BG))
                .block(panel("Skills")),
            area,
        );
        return;
    }

    let items: Vec<ListItem> = skills
        .iter()
        .map(|entry| {
            let (icon_color, icon) = if entry.source == "local" {
                (C_BLUE, "\u{25cb}")
            } else {
                (C_GREEN, "\u{25cf}")
            };
            let suffix = if entry.source != "local" {
                format!("  ({})", entry.source)
            } else {
                String::new()
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("  {icon} "), Style::default().fg(icon_color)),
                Span::styled(entry.name.clone(), Style::default().fg(C_FG)),
                Span::styled(suffix, Style::default().fg(C_MUT)),
            ]))
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(nav.list_selected));
    frame.render_stateful_widget(
        List::new(items)
            .block(panel(format!("Skills  ({})", skills.len())))
            .highlight_style(selected_style())
            .highlight_symbol("\u{25b6} "),
        area,
        &mut state,
    );
}

fn draw_mcp(frame: &mut Frame, _data: &ViewData, area: Rect) {
    let servers = discover_mcp_servers();
    if servers.is_empty() {
        frame.render_widget(
            Paragraph::new("  No MCP servers configured.")
                .style(Style::default().fg(C_MUT).bg(C_BG))
                .block(panel("MCP Servers")),
            area,
        );
        return;
    }

    let items: Vec<ListItem> = servers
        .iter()
        .map(|s| {
            ListItem::new(Line::from(vec![
                Span::styled("  \u{25cb} ", Style::default().fg(C_GREEN)),
                Span::styled(s.clone(), Style::default().fg(C_FG)),
            ]))
        })
        .collect();

    let list = List::new(items).block(panel(format!("MCP Servers  ({})", servers.len())));
    frame.render_widget(list, area);
}

// -- Discovery helpers --------------------------------------------------

pub fn discover_agents() -> Vec<String> {
    let dir = std::env::current_dir()
        .ok()
        .map(|p| p.join(".ship").join("agents"));
    let Some(dir) = dir else { return vec![] };
    if !dir.is_dir() {
        return vec![];
    }
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let p = e.path();
            p.is_file()
                && p.extension()
                    .is_some_and(|x| x == "jsonc" || x == "toml" || x == "json")
        })
        .filter_map(|e| {
            e.path()
                .file_stem()
                .and_then(|s| s.to_str().map(String::from))
        })
        .collect();
    names.sort();
    names
}

/// Discover all skills: local (.ship/skills/) + registry dependencies.
pub fn discover_all_skills() -> Vec<SkillEntry> {
    let mut entries = discover_local_skills();
    entries.extend(discover_dep_skills());
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    entries
}

/// Local skills live in `.ship/skills/` — each is a directory with SKILL.md.
fn discover_local_skills() -> Vec<SkillEntry> {
    let dir = match std::env::current_dir().ok() {
        Some(p) => p.join(".ship").join("skills"),
        None => return vec![],
    };
    if !dir.is_dir() {
        return vec![];
    }
    std::fs::read_dir(&dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir() && e.path().join("SKILL.md").is_file())
        .map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let path = e.path().join("SKILL.md");
            SkillEntry {
                name,
                source: "local".to_string(),
                path,
            }
        })
        .collect()
}

/// Discover skills from registry dependencies (ship.jsonc deps → ship.lock → cache).
fn discover_dep_skills() -> Vec<SkillEntry> {
    let cwd = match std::env::current_dir().ok() {
        Some(p) => p,
        None => return vec![],
    };
    let ship_dir = cwd.join(".ship");

    // Parse manifest for dependency keys.
    let manifest_str = match std::fs::read_to_string(ship_dir.join("ship.jsonc")).ok() {
        Some(s) => s,
        None => return vec![],
    };
    let manifest = match compiler::manifest::ShipManifest::from_jsonc_str(&manifest_str).ok() {
        Some(m) => m,
        None => return vec![],
    };
    if manifest.dependencies.is_empty() {
        return vec![];
    }

    // Parse lockfile for cache hashes.
    let lock = match compiler::lockfile::ShipLock::from_file(&ship_dir.join("ship.lock")).ok() {
        Some(l) => l,
        None => return vec![],
    };

    let cache_root = match dirs::home_dir() {
        Some(h) => h.join(".ship").join("cache"),
        None => return vec![],
    };

    let mut entries = Vec::new();
    for (dep_path, _) in &manifest.dependencies {
        let hex = match crate::dep_skills::hash_from_lock(&lock, dep_path).ok() {
            Some(h) => h,
            None => continue,
        };
        let pkg_dir = cache_root.join("objects").join(&hex);
        if !pkg_dir.is_dir() {
            continue;
        }
        // Short label: "owner/pkg" from "github.com/owner/pkg"
        let label: String = dep_path.split('/').skip(1).collect::<Vec<_>>().join("/");
        // Check for root-level SKILL.md (the package itself is a skill).
        let root_skill = pkg_dir.join("SKILL.md");
        if root_skill.is_file() {
            let pkg_name = dep_path.rsplit('/').next().unwrap_or(&label);
            entries.push(SkillEntry {
                name: pkg_name.to_string(),
                source: label.clone(),
                path: root_skill,
            });
        }
        scan_skills_in_dir(&pkg_dir, &label, &mut entries);
    }
    entries
}

/// Recursively scan a directory for skills (dirs containing SKILL.md).
/// Handles both flat skills and one-level namespace directories.
fn scan_skills_in_dir(dir: &std::path::Path, source: &str, out: &mut Vec<SkillEntry>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        // Skip hidden dirs and common non-skill dirs.
        if name.starts_with('.') || name == "node_modules" || name == "scripts" || name == "bin" {
            continue;
        }
        let skill_md = path.join("SKILL.md");
        let skill_md_upper = path.join("SKILL.MD");
        if skill_md.is_file() {
            out.push(SkillEntry {
                name: name.clone(),
                source: source.to_string(),
                path: skill_md,
            });
        } else if skill_md_upper.is_file() {
            out.push(SkillEntry {
                name: name.clone(),
                source: source.to_string(),
                path: skill_md_upper,
            });
        } else {
            // Namespace: check sub-directories for SKILL.md.
            if let Ok(sub_entries) = std::fs::read_dir(&path) {
                for sub in sub_entries.flatten() {
                    let sub_path = sub.path();
                    if !sub_path.is_dir() {
                        continue;
                    }
                    let sub_name = sub.file_name().to_string_lossy().to_string();
                    let sub_skill = sub_path.join("SKILL.md");
                    let sub_skill_upper = sub_path.join("SKILL.MD");
                    if sub_skill.is_file() {
                        out.push(SkillEntry {
                            name: format!("{name}/{sub_name}"),
                            source: source.to_string(),
                            path: sub_skill,
                        });
                    } else if sub_skill_upper.is_file() {
                        out.push(SkillEntry {
                            name: format!("{name}/{sub_name}"),
                            source: source.to_string(),
                            path: sub_skill_upper,
                        });
                    }
                }
            }
        }
    }
}

fn discover_mcp_servers() -> Vec<String> {
    let ship_dir = std::env::current_dir().ok().map(|p| p.join(".ship"));
    let Some(ship_dir) = ship_dir else {
        return vec![];
    };
    runtime::list_mcp_servers(Some(ship_dir))
        .ok()
        .unwrap_or_default()
        .iter()
        .map(|s| format!("{} ({})", s.id, mcp_label(&s.server_type)))
        .collect()
}

fn mcp_label(t: &runtime::McpServerType) -> &'static str {
    match t {
        runtime::McpServerType::Http => "http",
        runtime::McpServerType::Sse => "sse",
        runtime::McpServerType::Stdio => "stdio",
    }
}

// -- Detail helpers (called from input.rs) ------------------------------

/// Read the model field from an agent .jsonc file, if available.
fn read_agent_model(name: &str) -> Option<String> {
    let path = agent_config_path(name)?;
    let text = std::fs::read_to_string(&path).ok()?;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") {
            continue;
        }
        if (trimmed.starts_with("\"model\"") || trimmed.starts_with("model"))
            && let Some(val) = trimmed.split(':').nth(1)
        {
            let v = val
                .trim()
                .trim_matches(|c: char| c == '"' || c == ',' || c == ' ');
            if !v.is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}

/// Open agent .jsonc file as a full-page detail view.
pub fn open_agent_detail(nav: &mut NavState, name: &str) {
    let Some(path) = agent_config_path(name) else {
        return;
    };
    let body = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| format!("Could not read {}", path.display()));
    nav.enter_detail(format!("Agent: {name}"), body);
}

/// Open a skill's SKILL.md as a full-page detail view.
pub fn open_skill_detail_by_entry(nav: &mut NavState, entry: &SkillEntry) {
    let body = std::fs::read_to_string(&entry.path)
        .unwrap_or_else(|_| format!("Could not read {}", entry.path.display()));
    let title = if entry.source == "local" {
        format!("Skill: {}", entry.name)
    } else {
        format!("Skill: {} ({})", entry.name, entry.source)
    };
    nav.enter_detail(title, body);
}

fn agent_config_path(name: &str) -> Option<std::path::PathBuf> {
    let dir = std::env::current_dir().ok()?.join(".ship").join("agents");
    for ext in &["jsonc", "toml", "json"] {
        let p = dir.join(format!("{name}.{ext}"));
        if p.is_file() {
            return Some(p);
        }
    }
    None
}
