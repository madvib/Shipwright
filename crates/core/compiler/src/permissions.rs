//! Compiler permissions enforcement.
//!
//! Validates that every tool permission required by a skill referenced by an
//! agent is explicitly granted by that agent. Hard error on violation.
//!
//! Key contract:
//! - Parse `allowed-tools` from SKILL.md YAML frontmatter (space-delimited string)
//! - For each skill in `agent.skills`, look up its `SkillPermissions`
//! - Every tool in `SkillPermissions.allowed_tools` must be in `agent.permissions.allow`
//!   OR the agent has `"*"` in its allow list (wildcard grants all)
//! - No transitive escalation: skills only declare their own tool requirements

use std::path::Path;

use crate::agent_parser::AgentDef;

// ── Types ──────────────────────────────────────────────────────────────────────

/// Tool permission requirements declared in a SKILL.md file.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SkillPermissions {
    /// Tools the skill requires, parsed from the `allowed-tools` frontmatter field.
    pub allowed_tools: Vec<String>,
}

/// A single permission violation: a skill requires a tool the agent has not granted.
#[derive(Debug, Clone, PartialEq)]
pub struct PermissionViolation {
    /// Name of the skill that requires the missing tool.
    pub skill_name: String,
    /// The tool permission string that is not granted.
    pub missing_tool: String,
}

impl PermissionViolation {
    /// Returns an actionable error message.
    pub fn message(&self) -> String {
        format!(
            "Skill '{}' requires tool '{}' but the agent does not grant it. \
             Add \"{}\" to [permissions].allow in the agent TOML.",
            self.skill_name, self.missing_tool, self.missing_tool
        )
    }
}

// ── Frontmatter parsing ────────────────────────────────────────────────────────

/// Parse `allowed-tools` from the YAML frontmatter of a SKILL.md file.
///
/// The frontmatter block is delimited by `---` on its own line.
/// `allowed-tools` is a space-delimited string, e.g.:
/// ```yaml
/// allowed-tools: Bash Read Write
/// ```
///
/// Absent frontmatter or absent `allowed-tools` field → empty `SkillPermissions`.
pub fn parse_skill_permissions(skill_md_path: &Path) -> anyhow::Result<SkillPermissions> {
    let content = std::fs::read_to_string(skill_md_path)
        .map_err(|e| anyhow::anyhow!("Cannot read {}: {e}", skill_md_path.display()))?;
    Ok(parse_skill_permissions_from_str(&content))
}

/// Parse `allowed-tools` from the content of a SKILL.md string (pure, no I/O).
pub fn parse_skill_permissions_from_str(content: &str) -> SkillPermissions {
    let tools = extract_allowed_tools(content);
    SkillPermissions {
        allowed_tools: tools,
    }
}

fn extract_allowed_tools(content: &str) -> Vec<String> {
    // Find the frontmatter block between the first two `---` lines.
    let mut lines = content.lines();

    // Skip leading blank lines then expect opening `---`
    let first = lines.find(|l| !l.trim().is_empty());
    if first.as_deref() != Some("---") {
        return vec![];
    }

    let mut frontmatter = String::new();
    for line in lines {
        if line.trim() == "---" {
            break;
        }
        frontmatter.push_str(line);
        frontmatter.push('\n');
    }

    // Look for `allowed-tools:` key.
    for line in frontmatter.lines() {
        if let Some(rest) = line.strip_prefix("allowed-tools:") {
            let value = rest.trim();
            if value.is_empty() {
                return vec![];
            }
            return value
                .split_whitespace()
                .map(str::to_string)
                .collect();
        }
    }
    vec![]
}

// ── Permission checking ────────────────────────────────────────────────────────

/// Check that an agent's grants cover all tool requirements of its skills.
///
/// `skill_permissions` maps skill name (path or id) → `SkillPermissions`.
/// Only skills listed in `agent.skills` are checked.
///
/// Returns a list of violations. Empty means all permissions are satisfied.
pub fn check_agent_permissions(
    agent: &AgentDef,
    skill_permissions: &[(String, SkillPermissions)],
) -> Vec<PermissionViolation> {
    let wildcard = agent.permissions.allow.iter().any(|p| p == "*");
    if wildcard {
        return vec![];
    }

    let agent_grants: std::collections::HashSet<&str> = agent
        .permissions
        .allow
        .iter()
        .map(|s| s.as_str())
        .collect();

    let mut violations = Vec::new();
    for skill_ref in &agent.skills {
        // Find the matching skill permissions entry.
        let perms = skill_permissions
            .iter()
            .find(|(name, _)| name == skill_ref)
            .map(|(_, p)| p);

        let perms = match perms {
            Some(p) => p,
            None => continue, // No permissions record = no requirements
        };

        for tool in &perms.allowed_tools {
            if !agent_grants.contains(tool.as_str()) {
                violations.push(PermissionViolation {
                    skill_name: skill_ref.clone(),
                    missing_tool: tool.clone(),
                });
            }
        }
    }
    violations
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_parser::{AgentDef, AgentPermissions};

    // ── Frontmatter parsing ────────────────────────────────────────────────────

    #[test]
    fn parse_allowed_tools_from_frontmatter() {
        let content = "---\nallowed-tools: Bash Read Write\n---\n\n# My Skill\n";
        let perms = parse_skill_permissions_from_str(content);
        assert_eq!(perms.allowed_tools, vec!["Bash", "Read", "Write"]);
    }

    #[test]
    fn absent_frontmatter_returns_empty() {
        let content = "# My Skill\n\nNo frontmatter here.\n";
        let perms = parse_skill_permissions_from_str(content);
        assert!(perms.allowed_tools.is_empty());
    }

    #[test]
    fn absent_allowed_tools_returns_empty() {
        let content = "---\nname: my-skill\ndescription: A skill.\n---\n\n# Body\n";
        let perms = parse_skill_permissions_from_str(content);
        assert!(perms.allowed_tools.is_empty());
    }

    #[test]
    fn empty_allowed_tools_value_returns_empty() {
        let content = "---\nallowed-tools: \n---\n\n# Body\n";
        let perms = parse_skill_permissions_from_str(content);
        assert!(perms.allowed_tools.is_empty());
    }

    #[test]
    fn single_tool_parsed() {
        let content = "---\nallowed-tools: Bash\n---\n";
        let perms = parse_skill_permissions_from_str(content);
        assert_eq!(perms.allowed_tools, vec!["Bash"]);
    }

    #[test]
    fn unknown_tool_strings_preserved() {
        let content = "---\nallowed-tools: Bash CustomTool(arg:value) UnknownFutureTool\n---\n";
        let perms = parse_skill_permissions_from_str(content);
        assert_eq!(
            perms.allowed_tools,
            vec!["Bash", "CustomTool(arg:value)", "UnknownFutureTool"]
        );
    }

    // ── check_agent_permissions ────────────────────────────────────────────────

    fn make_agent(skills: Vec<&str>, allow: Vec<&str>) -> AgentDef {
        AgentDef {
            name: "test-agent".into(),
            description: None,
            rules: vec![],
            skills: skills.into_iter().map(str::to_string).collect(),
            permissions: AgentPermissions {
                allow: allow.into_iter().map(str::to_string).collect(),
            },
            mcp: vec![],
            providers: Default::default(),
        }
    }

    #[test]
    fn no_skills_no_violations() {
        let agent = make_agent(vec![], vec![]);
        let violations = check_agent_permissions(&agent, &[]);
        assert!(violations.is_empty());
    }

    #[test]
    fn skill_with_no_requirements_no_violations() {
        let agent = make_agent(vec!["my-skill"], vec![]);
        let skill_perms = vec![(
            "my-skill".to_string(),
            SkillPermissions {
                allowed_tools: vec![],
            },
        )];
        let violations = check_agent_permissions(&agent, &skill_perms);
        assert!(violations.is_empty());
    }

    #[test]
    fn all_tools_granted_no_violations() {
        let agent = make_agent(vec!["my-skill"], vec!["Bash", "Read"]);
        let skill_perms = vec![(
            "my-skill".to_string(),
            SkillPermissions {
                allowed_tools: vec!["Bash".into(), "Read".into()],
            },
        )];
        let violations = check_agent_permissions(&agent, &skill_perms);
        assert!(violations.is_empty());
    }

    #[test]
    fn missing_tool_produces_violation() {
        let agent = make_agent(vec!["my-skill"], vec!["Read"]);
        let skill_perms = vec![(
            "my-skill".to_string(),
            SkillPermissions {
                allowed_tools: vec!["Bash".into(), "Read".into()],
            },
        )];
        let violations = check_agent_permissions(&agent, &skill_perms);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].skill_name, "my-skill");
        assert_eq!(violations[0].missing_tool, "Bash");
    }

    #[test]
    fn wildcard_grants_all() {
        let agent = make_agent(vec!["my-skill"], vec!["*"]);
        let skill_perms = vec![(
            "my-skill".to_string(),
            SkillPermissions {
                allowed_tools: vec!["Bash".into(), "Write".into(), "SomeOtherTool".into()],
            },
        )];
        let violations = check_agent_permissions(&agent, &skill_perms);
        assert!(violations.is_empty());
    }

    #[test]
    fn skill_not_in_permissions_map_skipped() {
        // Agent references a skill that has no permissions entry — no violation.
        let agent = make_agent(vec!["unlisted-skill"], vec![]);
        let violations = check_agent_permissions(&agent, &[]);
        assert!(violations.is_empty());
    }

    #[test]
    fn violation_message_is_actionable() {
        let v = PermissionViolation {
            skill_name: "backend-rust".into(),
            missing_tool: "Bash".into(),
        };
        let msg = v.message();
        assert!(msg.contains("backend-rust"), "{msg}");
        assert!(msg.contains("Bash"), "{msg}");
        assert!(msg.contains("[permissions].allow"), "{msg}");
    }

    #[test]
    fn multiple_skills_multiple_violations() {
        let agent = make_agent(
            vec!["skill-a", "skill-b"],
            vec!["Read"], // Only Read granted
        );
        let skill_perms = vec![
            (
                "skill-a".to_string(),
                SkillPermissions {
                    allowed_tools: vec!["Bash".into()],
                },
            ),
            (
                "skill-b".to_string(),
                SkillPermissions {
                    allowed_tools: vec!["Write".into()],
                },
            ),
        ];
        let violations = check_agent_permissions(&agent, &skill_perms);
        assert_eq!(violations.len(), 2);
        let missing: std::collections::HashSet<&str> =
            violations.iter().map(|v| v.missing_tool.as_str()).collect();
        assert!(missing.contains("Bash"));
        assert!(missing.contains("Write"));
    }

    #[test]
    fn parse_skill_permissions_from_file() {
        let dir = tempfile::tempdir().unwrap();
        let skill_path = dir.path().join("SKILL.md");
        std::fs::write(
            &skill_path,
            "---\nallowed-tools: Bash Read\n---\n\n# My Skill\n",
        )
        .unwrap();
        let perms = parse_skill_permissions(&skill_path).unwrap();
        assert_eq!(perms.allowed_tools, vec!["Bash", "Read"]);
    }

    #[test]
    fn parse_skill_permissions_missing_file_error() {
        let r = parse_skill_permissions(Path::new("/nonexistent/SKILL.md"));
        assert!(r.is_err());
    }
}
