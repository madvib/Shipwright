//! Provider capability matrix.
//!
//! Declares what each provider supports and what Ship currently emits.
//! Used by the `ship matrix` CLI command and MCP tool to produce
//! a diffable gap analysis on demand.

use std::collections::BTreeMap;

use serde::Serialize;

// ─── Coverage level ──────────────────────────────────────────────────────────

/// How well Ship covers a specific provider capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Coverage {
    /// Ship does not emit this at all.
    None,
    /// Ship emits a subset of the provider's supported surface.
    Partial,
    /// Ship emits the full provider surface for this capability.
    Full,
    /// Provider does not support this capability.
    NotApplicable,
}

impl Coverage {
    pub fn symbol(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Partial => "partial",
            Self::None => "none",
            Self::NotApplicable => "n/a",
        }
    }
}

// ─── Capability entry ────────────────────────────────────────────────────────

/// A single capability row in the matrix.
#[derive(Debug, Clone, Serialize)]
pub struct Capability {
    /// Machine-readable identifier (e.g. `mcp_servers`, `hooks`, `env_vars`).
    pub id: &'static str,
    /// Human-readable label.
    pub label: &'static str,
    /// Does the provider support this capability?
    pub provider_supports: bool,
    /// How well does Ship cover it?
    pub ship_coverage: Coverage,
    /// What Ship emits today (brief).
    pub ship_emits: &'static str,
    /// What the provider supports that Ship doesn't cover yet.
    pub gap: &'static str,
}

// ─── Provider matrix ─────────────────────────────────────────────────────────

/// Full matrix for one provider.
#[derive(Debug, Clone, Serialize)]
pub struct ProviderMatrix {
    pub provider_id: &'static str,
    pub provider_name: &'static str,
    pub capabilities: Vec<Capability>,
    /// Overall coverage percentage (full=1.0, partial=0.5, none=0, n/a excluded).
    pub coverage_pct: f64,
}

/// Full matrix for all providers.
#[derive(Debug, Clone, Serialize)]
pub struct Matrix {
    pub generated: String,
    pub providers: Vec<ProviderMatrix>,
}

// ─── Provider capability declarations ────────────────────────────────────────

fn claude_capabilities() -> Vec<Capability> {
    vec![
        Capability {
            id: "context_file",
            label: "Context file (CLAUDE.md)",
            provider_supports: true,
            ship_coverage: Coverage::Full,
            ship_emits: "CLAUDE.md from rules + mode notice",
            gap: "",
        },
        Capability {
            id: "mcp_servers",
            label: "MCP servers (.mcp.json)",
            provider_supports: true,
            ship_coverage: Coverage::Full,
            ship_emits: "stdio, SSE, HTTP transports",
            gap: "",
        },
        Capability {
            id: "skills",
            label: "Skills (.claude/skills/)",
            provider_supports: true,
            ship_coverage: Coverage::Full,
            ship_emits: "SKILL.md with YAML frontmatter",
            gap: "",
        },
        Capability {
            id: "permissions",
            label: "Permissions (allow/deny/ask)",
            provider_supports: true,
            ship_coverage: Coverage::Full,
            ship_emits: "allow, deny, ask arrays + defaultMode + additionalDirectories",
            gap: "",
        },
        Capability {
            id: "hooks",
            label: "Hooks",
            provider_supports: true,
            ship_coverage: Coverage::Partial,
            ship_emits: "PreToolUse, PostToolUse, Notification, Stop, SubagentStop, PreCompact",
            gap: "Missing 18 events: SessionStart, SessionEnd, UserPromptSubmit, \
                  PostToolUseFailure, PermissionRequest, SubagentStart, TaskCompleted, \
                  TeammateIdle, InstructionsLoaded, ConfigChange, WorktreeCreate, \
                  WorktreeRemove, PreCompact (done), PostCompact, Elicitation, \
                  ElicitationResult, Setup. Also missing hook types: http, prompt, agent",
        },
        Capability {
            id: "model",
            label: "Model override",
            provider_supports: true,
            ship_coverage: Coverage::Full,
            ship_emits: "model field in settings patch",
            gap: "",
        },
        Capability {
            id: "env_vars",
            label: "Environment variables (env)",
            provider_supports: true,
            ship_coverage: Coverage::Full,
            ship_emits: "env field in settings patch from ProjectLibrary.env",
            gap: "",
        },
        Capability {
            id: "available_models",
            label: "Model allowlist (availableModels)",
            provider_supports: true,
            ship_coverage: Coverage::Full,
            ship_emits: "availableModels array in settings patch",
            gap: "",
        },
        Capability {
            id: "team_agents",
            label: "Subagent profiles (.claude/agents/, .gemini/agents/, etc.)",
            provider_supports: true,
            ship_coverage: Coverage::Full,
            ship_emits: "Provider-native agent files from .ship/agents/profiles/*.toml",
            gap: "",
        },
        Capability {
            id: "plugins",
            label: "Plugin enablement",
            provider_supports: true,
            ship_coverage: Coverage::Partial,
            ship_emits: "plugins_manifest with install intent",
            gap: "enabledPlugins settings field not emitted",
        },
        Capability {
            id: "agent_limits",
            label: "Agent limits (cost/turns)",
            provider_supports: true,
            ship_coverage: Coverage::Full,
            ship_emits: "maxCostPerSession, maxTurns in settings patch",
            gap: "",
        },
        Capability {
            id: "extra_settings",
            label: "Extra settings pass-through",
            provider_supports: true,
            ship_coverage: Coverage::Full,
            ship_emits: "provider_settings.claude merged verbatim",
            gap: "",
        },
        Capability {
            id: "status_line",
            label: "Status line command",
            provider_supports: true,
            ship_coverage: Coverage::None,
            ship_emits: "",
            gap: "statusLine: { type, command, padding } not emitted",
        },
        Capability {
            id: "auto_memory",
            label: "Auto memory settings",
            provider_supports: true,
            ship_coverage: Coverage::None,
            ship_emits: "",
            gap: "autoMemoryEnabled, autoMemoryDirectory not emitted",
        },
    ]
}

fn cursor_capabilities() -> Vec<Capability> {
    vec![
        Capability {
            id: "rule_files",
            label: "Rule files (.cursor/rules/*.mdc)",
            provider_supports: true,
            ship_coverage: Coverage::Full,
            ship_emits: "per-file .mdc with description, globs, alwaysApply frontmatter",
            gap: "",
        },
        Capability {
            id: "mcp_servers",
            label: "MCP servers (.cursor/mcp.json)",
            provider_supports: true,
            ship_coverage: Coverage::Full,
            ship_emits: "stdio, SSE, HTTP transports",
            gap: "",
        },
        Capability {
            id: "skills",
            label: "Skills (.cursor/skills/)",
            provider_supports: true,
            ship_coverage: Coverage::Full,
            ship_emits: "SKILL.md with YAML frontmatter",
            gap: "",
        },
        Capability {
            id: "hooks",
            label: "Hooks (.cursor/hooks.json)",
            provider_supports: true,
            ship_coverage: Coverage::Partial,
            ship_emits: "beforeMCPExecution, beforeShellExecution, afterMCPExecution, \
                         afterShellExecution, sessionEnd",
            gap: "Missing: beforeFileEdit, afterFileEdit events (if they exist)",
        },
        Capability {
            id: "permissions",
            label: "Permissions (.cursor/cli.json)",
            provider_supports: true,
            ship_coverage: Coverage::Full,
            ship_emits: "Shell, Read, Write, WebFetch, Mcp typed patterns",
            gap: "",
        },
        Capability {
            id: "cursorignore",
            label: "Context exclusion (.cursorignore)",
            provider_supports: true,
            ship_coverage: Coverage::None,
            ship_emits: "",
            gap: "Gitignore-style file for excluding paths from AI context",
        },
        Capability {
            id: "model_selection",
            label: "Model selection",
            provider_supports: true,
            ship_coverage: Coverage::NotApplicable,
            ship_emits: "",
            gap: "Stored in SQLite, not file-configurable",
        },
        Capability {
            id: "environment_json",
            label: "Cloud agent config (environment.json)",
            provider_supports: true,
            ship_coverage: Coverage::None,
            ship_emits: "",
            gap: "Cloud agent provisioning config",
        },
        Capability {
            id: "dockerfile",
            label: "Custom Dockerfile (.cursor/Dockerfile)",
            provider_supports: true,
            ship_coverage: Coverage::None,
            ship_emits: "",
            gap: "Custom base image for remote environments",
        },
    ]
}

fn gemini_capabilities() -> Vec<Capability> {
    vec![
        Capability {
            id: "context_file",
            label: "Context file (GEMINI.md)",
            provider_supports: true,
            ship_coverage: Coverage::Full,
            ship_emits: "GEMINI.md from rules + mode notice",
            gap: "",
        },
        Capability {
            id: "mcp_servers",
            label: "MCP servers (.gemini/settings.json)",
            provider_supports: true,
            ship_coverage: Coverage::Full,
            ship_emits: "mcpServers with stdio/SSE/HTTP, httpUrl for streamable HTTP",
            gap: "",
        },
        Capability {
            id: "skills",
            label: "Skills (.agents/skills/)",
            provider_supports: true,
            ship_coverage: Coverage::Full,
            ship_emits: "SKILL.md with YAML frontmatter",
            gap: "",
        },
        Capability {
            id: "hooks",
            label: "Hooks (.gemini/settings.json)",
            provider_supports: true,
            ship_coverage: Coverage::Partial,
            ship_emits: "BeforeTool, AfterTool, Notification, SessionEnd, PreCompress",
            gap: "Missing: SessionStart, UserInput events",
        },
        Capability {
            id: "permissions",
            label: "Policies (.gemini/policies/ship.toml)",
            provider_supports: true,
            ship_coverage: Coverage::Full,
            ship_emits: "shell, file_read, file_write, web_fetch, mcp tool policies",
            gap: "",
        },
        Capability {
            id: "sandbox",
            label: "Sandbox configuration",
            provider_supports: true,
            ship_coverage: Coverage::None,
            ship_emits: "",
            gap: "Sandbox mode settings for Gemini CLI",
        },
        Capability {
            id: "model_selection",
            label: "Model selection",
            provider_supports: true,
            ship_coverage: Coverage::Full,
            ship_emits: "model field in .gemini/settings.json patch",
            gap: "",
        },
    ]
}

fn codex_capabilities() -> Vec<Capability> {
    vec![
        Capability {
            id: "context_file",
            label: "Context file (AGENTS.md)",
            provider_supports: true,
            ship_coverage: Coverage::Full,
            ship_emits: "AGENTS.md from rules + mode notice",
            gap: "",
        },
        Capability {
            id: "mcp_servers",
            label: "MCP servers (.codex/config.toml)",
            provider_supports: true,
            ship_coverage: Coverage::Full,
            ship_emits: "[mcp_servers.*] TOML tables with command/args/env",
            gap: "",
        },
        Capability {
            id: "skills",
            label: "Skills (.agents/skills/)",
            provider_supports: true,
            ship_coverage: Coverage::Full,
            ship_emits: "SKILL.md with YAML frontmatter",
            gap: "",
        },
        Capability {
            id: "hooks",
            label: "Hooks",
            provider_supports: false,
            ship_coverage: Coverage::NotApplicable,
            ship_emits: "",
            gap: "",
        },
        Capability {
            id: "permissions",
            label: "Permissions",
            provider_supports: false,
            ship_coverage: Coverage::NotApplicable,
            ship_emits: "",
            gap: "",
        },
        Capability {
            id: "model_selection",
            label: "Model selection",
            provider_supports: true,
            ship_coverage: Coverage::Full,
            ship_emits: "model field in .codex/config.toml",
            gap: "",
        },
        Capability {
            id: "sandbox",
            label: "Sandbox mode",
            provider_supports: true,
            ship_coverage: Coverage::None,
            ship_emits: "",
            gap: "sandbox field in .codex/config.toml (full, network-only, off)",
        },
    ]
}

// ─── Matrix builder ──────────────────────────────────────────────────────────

fn build_provider_matrix(
    provider_id: &'static str,
    provider_name: &'static str,
    caps: Vec<Capability>,
) -> ProviderMatrix {
    let (total, score) = caps.iter().fold((0u32, 0.0f64), |(t, s), c| {
        if c.ship_coverage == Coverage::NotApplicable {
            (t, s)
        } else {
            let v = match c.ship_coverage {
                Coverage::Full => 1.0,
                Coverage::Partial => 0.5,
                Coverage::None => 0.0,
                Coverage::NotApplicable => unreachable!(),
            };
            (t + 1, s + v)
        }
    });
    let coverage_pct = if total == 0 {
        0.0
    } else {
        (score / total as f64 * 100.0).round()
    };

    ProviderMatrix {
        provider_id,
        provider_name,
        capabilities: caps,
        coverage_pct,
    }
}

/// Build the full provider capability matrix.
pub fn build_matrix() -> Matrix {
    let providers = vec![
        build_provider_matrix("claude", "Claude Code", claude_capabilities()),
        build_provider_matrix("cursor", "Cursor", cursor_capabilities()),
        build_provider_matrix("gemini", "Gemini CLI", gemini_capabilities()),
        build_provider_matrix("codex", "OpenAI Codex", codex_capabilities()),
    ];

    Matrix {
        generated: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        providers,
    }
}

// ─── Diffable text output ────────────────────────────────────────────────────

/// Render the matrix as a diffable plain-text table.
/// Suitable for piping to `diff` or committing as a snapshot.
pub fn render_text(matrix: &Matrix) -> String {
    let mut out = String::new();
    out.push_str("# Ship Provider Matrix\n");
    out.push_str(&format!("# Generated: {}\n\n", matrix.generated));

    for pm in &matrix.providers {
        out.push_str(&format!(
            "## {} ({}) — {}% coverage\n\n",
            pm.provider_name, pm.provider_id, pm.coverage_pct as u32
        ));
        out.push_str(&format!(
            "{:<25} {:<10} {}\n",
            "CAPABILITY", "COVERAGE", "GAP"
        ));
        out.push_str(&format!("{}\n", "-".repeat(76)));

        for cap in &pm.capabilities {
            let gap_display = if cap.gap.is_empty() {
                "-"
            } else {
                cap.gap
            };
            out.push_str(&format!(
                "{:<25} {:<10} {}\n",
                cap.label.chars().take(25).collect::<String>(),
                cap.ship_coverage.symbol(),
                gap_display,
            ));
        }
        out.push('\n');
    }

    out
}

/// Render the matrix as a compact diffable table (one line per provider×capability).
/// Format: `provider/capability_id  coverage  gap_summary`
/// Designed for `diff` — sort is deterministic (provider alpha, then cap order).
pub fn render_diffable(matrix: &Matrix) -> String {
    let mut lines: Vec<String> = Vec::new();
    lines.push("# provider/capability  coverage  gap".to_string());

    for pm in &matrix.providers {
        for cap in &pm.capabilities {
            let gap = if cap.gap.is_empty() {
                "-".to_string()
            } else {
                // First 80 chars of gap, single line
                cap.gap.replace('\n', " ").chars().take(80).collect()
            };
            lines.push(format!(
                "{}/{:<25} {:<10} {}",
                pm.provider_id,
                cap.id,
                cap.ship_coverage.symbol(),
                gap,
            ));
        }
    }

    lines.join("\n") + "\n"
}

/// Summary: provider name → coverage percentage.
pub fn render_summary(matrix: &Matrix) -> BTreeMap<String, f64> {
    matrix
        .providers
        .iter()
        .map(|pm| (pm.provider_id.to_string(), pm.coverage_pct))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_builds_all_providers() {
        let m = build_matrix();
        assert_eq!(m.providers.len(), 4);
        let ids: Vec<&str> = m.providers.iter().map(|p| p.provider_id).collect();
        assert!(ids.contains(&"claude"));
        assert!(ids.contains(&"cursor"));
        assert!(ids.contains(&"gemini"));
        assert!(ids.contains(&"codex"));
    }

    #[test]
    fn coverage_pct_is_bounded() {
        let m = build_matrix();
        for pm in &m.providers {
            assert!(pm.coverage_pct >= 0.0);
            assert!(pm.coverage_pct <= 100.0);
        }
    }

    #[test]
    fn full_coverage_has_empty_gap() {
        let m = build_matrix();
        for pm in &m.providers {
            for cap in &pm.capabilities {
                if cap.ship_coverage == Coverage::Full {
                    assert!(
                        cap.gap.is_empty(),
                        "{}/{}: full coverage but non-empty gap: {}",
                        pm.provider_id,
                        cap.id,
                        cap.gap
                    );
                }
            }
        }
    }

    #[test]
    fn none_coverage_has_gap() {
        let m = build_matrix();
        for pm in &m.providers {
            for cap in &pm.capabilities {
                if cap.ship_coverage == Coverage::None {
                    assert!(
                        !cap.gap.is_empty(),
                        "{}/{}: none coverage but empty gap description",
                        pm.provider_id,
                        cap.id,
                    );
                }
            }
        }
    }

    #[test]
    fn diffable_output_is_deterministic() {
        let m1 = build_matrix();
        let m2 = build_matrix();
        // Zero out timestamps for comparison
        let r1 = render_diffable(&m1);
        let r2 = render_diffable(&m2);
        assert_eq!(r1, r2);
    }

    #[test]
    fn text_output_contains_all_providers() {
        let m = build_matrix();
        let text = render_text(&m);
        assert!(text.contains("Claude Code"));
        assert!(text.contains("Cursor"));
        assert!(text.contains("Gemini CLI"));
        assert!(text.contains("OpenAI Codex"));
    }
}
