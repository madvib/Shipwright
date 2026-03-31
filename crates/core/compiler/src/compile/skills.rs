use std::collections::HashMap;

use crate::resolve::ResolvedConfig;
use crate::types::Skill;

use super::provider::ProviderDescriptor;

// ── Artifact-to-event mapping ─────────────────────────────────────────────────

/// Maps an artifact type to the Studio-emitted event suffixes applicable to it.
///
/// Mirrors `runtime::events::artifact_events::events_for_artifact` — the compiler
/// cannot depend on the runtime, so this is an intentional local copy.
///
/// Returns Studio-action suffixes only (`annotation`, `feedback`, `selection`).
/// Agent-emitted lifecycle events (`artifact_created`, `artifact_deleted`) are NOT
/// listed here — they use `{skill.id}.` prefix at emit time.
fn events_for_artifact(artifact_type: &str) -> &'static [&'static str] {
    match artifact_type {
        "html" => &["annotation", "feedback", "selection"],
        "pdf" => &["selection", "feedback"],
        "markdown" => &["feedback", "selection"],
        "image" => &["annotation", "feedback"],
        "adr" => &["feedback"],
        "note" => &["feedback"],
        "url" => &["feedback"],
        "json" => &["feedback"],
        _ => &[],
    }
}

/// Compute the deduplicated set of event subscription namespaces for a list of skills.
///
/// Returns `{skill.id}.` for each skill. Studio-emitted events (`studio.*`) are
/// already covered by the base subscription actors register — no additional prefix
/// is emitted here.
pub(super) fn resolve_event_subscriptions(skills: &[Skill]) -> Vec<String> {
    let mut subs: Vec<String> = Vec::new();
    for skill in skills {
        if skill.id.is_empty() {
            continue;
        }
        let ns = format!("{}.", skill.id);
        if !subs.contains(&ns) {
            subs.push(ns);
        }
    }
    subs
}

pub(super) fn build_skill_files(
    desc: &ProviderDescriptor,
    resolved: &ResolvedConfig,
) -> HashMap<String, String> {
    let Some(base) = desc.skills_dir.base_path() else {
        return HashMap::new();
    };
    let runtime_vars = build_runtime_vars(resolved);
    resolved
        .skills
        .iter()
        .filter(|skill| !skill.content.trim().is_empty())
        .map(|skill| {
            let path = format!("{}/{}/SKILL.md", base, skill.id);
            let content = format_skill_file(skill, &runtime_vars);
            (path, content)
        })
        .collect()
}

/// Build the `runtime` context map injected into every skill template.
///
/// Skills access these as `{{ runtime.agents }}`, `{{ runtime.model }}`, etc.
/// This data is derived from `ResolvedConfig` at compile time — not user-configurable.
///
/// | Key | Type | Description |
/// |-----|------|-------------|
/// | `runtime.agents` | array of `{id, name, description}` | Agent profiles from `.ship/agents/*.toml` |
/// | `runtime.providers` | array of strings | Active provider IDs (e.g. `["claude", "cursor"]`) |
/// | `runtime.model` | string or `""` | Configured model override, empty if not set |
/// | `runtime.skills` | array of `{id, name, description}` | All skills active in this compile |
fn build_runtime_vars(resolved: &ResolvedConfig) -> serde_json::Value {
    let agents: Vec<serde_json::Value> = resolved
        .agent_profiles
        .iter()
        .map(|p| {
            serde_json::json!({
                "id": p.profile.id,
                "name": p.profile.name,
                "description": p.profile.description,
            })
        })
        .collect();

    let skills: Vec<serde_json::Value> = resolved
        .skills
        .iter()
        .map(|s| {
            serde_json::json!({
                "id": s.id,
                "name": s.name,
                "description": s.description,
            })
        })
        .collect();

    serde_json::json!({
        "agents": agents,
        "providers": resolved.providers,
        "model": resolved.model.as_deref().unwrap_or(""),
        "skills": skills,
    })
}

pub(super) fn format_skill_file(skill: &Skill, runtime: &serde_json::Value) -> String {
    let description = skill
        .description
        .as_deref()
        .unwrap_or("No description provided.");

    let mut fm = format!("---\nname: {}\ndescription: {}", skill.id, description);

    // Merge user vars with runtime-injected context before template resolution.
    // `runtime.*` is always available; user vars take the flat namespace.
    // `stable_id` is the skill's canonical stable-id from frontmatter (reserved, read-only).
    let resolved_content;
    let needs_resolution = !skill.vars.is_empty()
        || skill.content.contains("runtime.")
        || skill.content.contains("stable_id");
    let content: &str = if needs_resolution {
        let mut vars = skill.vars.clone();
        vars.insert("runtime".to_string(), runtime.clone());
        if let Some(ref sid) = skill.stable_id {
            vars.insert("stable_id".to_string(), serde_json::json!(sid));
        }
        resolved_content = crate::vars::resolve_template(&skill.content, &vars);
        &resolved_content
    } else {
        &skill.content
    };

    if let Some(license) = &skill.license {
        fm.push_str(&format!("\nlicense: {}", license));
    }

    if let Some(compatibility) = &skill.compatibility {
        fm.push_str(&format!("\ncompatibility: {}", compatibility));
    }

    if !skill.allowed_tools.is_empty() {
        fm.push_str(&format!(
            "\nallowed-tools: {}",
            skill.allowed_tools.join(" ")
        ));
    }

    if !skill.artifacts.is_empty() {
        fm.push_str(&format!("\nartifacts: [{}]", skill.artifacts.join(", ")));
    }

    if !skill.metadata.is_empty() {
        // Sort keys for deterministic output.
        let mut keys: Vec<&String> = skill.metadata.keys().collect();
        keys.sort();
        fm.push_str("\nmetadata:");
        for key in keys {
            fm.push_str(&format!("\n  {}: {}", key, skill.metadata[key]));
        }
    }

    fm.push_str("\n---\n\n");
    fm.push_str(content);
    fm
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn no_runtime() -> serde_json::Value {
        serde_json::json!({ "agents": [] })
    }

    fn base_skill() -> Skill {
        Skill {
            id: "my-skill".to_string(),
            name: "My Skill".to_string(),
            stable_id: None,
            description: Some("Does things.".to_string()),
            license: None,
            compatibility: None,
            allowed_tools: vec![],
            artifacts: vec![],
            metadata: HashMap::new(),
            content: "Instructions here.".to_string(),
            source: Default::default(),
            vars: HashMap::new(),
        }
    }

    #[test]
    fn format_skill_file_minimal() {
        let skill = base_skill();
        let out = format_skill_file(&skill, &no_runtime());
        assert!(out.starts_with("---\nname: my-skill\ndescription: Does things."));
        assert!(out.contains("\n---\n\nInstructions here."));
        // No optional fields present
        assert!(!out.contains("license:"));
        assert!(!out.contains("compatibility:"));
        assert!(!out.contains("allowed-tools:"));
        assert!(!out.contains("metadata:"));
    }

    #[test]
    fn format_skill_file_with_license() {
        let mut skill = base_skill();
        skill.license = Some("MIT".to_string());
        let out = format_skill_file(&skill, &no_runtime());
        assert!(out.contains("\nlicense: MIT\n"), "got:\n{out}");
    }

    #[test]
    fn format_skill_file_with_compatibility() {
        let mut skill = base_skill();
        skill.compatibility = Some("claude >= 3".to_string());
        let out = format_skill_file(&skill, &no_runtime());
        assert!(
            out.contains("\ncompatibility: claude >= 3\n"),
            "got:\n{out}"
        );
    }

    #[test]
    fn format_skill_file_with_allowed_tools() {
        let mut skill = base_skill();
        skill.allowed_tools = vec!["Read".to_string(), "Edit".to_string()];
        let out = format_skill_file(&skill, &no_runtime());
        assert!(out.contains("\nallowed-tools: Read Edit\n"), "got:\n{out}");
    }

    #[test]
    fn format_skill_file_with_metadata_sorted() {
        let mut skill = base_skill();
        skill
            .metadata
            .insert("version".to_string(), "1.0.0".to_string());
        skill
            .metadata
            .insert("author".to_string(), "alice".to_string());
        let out = format_skill_file(&skill, &no_runtime());
        assert!(out.contains("\nmetadata:\n"), "got:\n{out}");
        // author comes before version alphabetically
        let author_pos = out.find("author").unwrap();
        let version_pos = out.find("version").unwrap();
        assert!(author_pos < version_pos, "metadata keys must be sorted");
    }

    #[test]
    fn format_skill_file_no_description_uses_fallback() {
        let mut skill = base_skill();
        skill.description = None;
        let out = format_skill_file(&skill, &no_runtime());
        assert!(
            out.contains("description: No description provided."),
            "got:\n{out}"
        );
    }

    #[test]
    fn format_skill_file_resolves_template_vars() {
        let mut skill = base_skill();
        skill.content = "Use {{ style }} commits.".to_string();
        skill
            .vars
            .insert("style".to_string(), serde_json::json!("gitmoji"));
        let out = format_skill_file(&skill, &no_runtime());
        assert!(out.contains("Use gitmoji commits."), "got:\n{out}");
    }

    #[test]
    fn format_skill_file_no_vars_passthrough() {
        let skill = base_skill();
        let out = format_skill_file(&skill, &no_runtime());
        assert!(out.contains("Instructions here."), "got:\n{out}");
    }

    #[test]
    fn format_skill_file_with_artifacts() {
        let mut skill = base_skill();
        skill.artifacts = vec!["html".to_string(), "adr".to_string()];
        let out = format_skill_file(&skill, &no_runtime());
        assert!(out.contains("\nartifacts: [html, adr]\n"), "got:\n{out}");
    }

    #[test]
    fn format_skill_file_no_artifacts_omitted() {
        let skill = base_skill();
        let out = format_skill_file(&skill, &no_runtime());
        assert!(!out.contains("artifacts:"), "got:\n{out}");
    }

    #[test]
    fn resolve_event_subscriptions_empty() {
        assert!(resolve_event_subscriptions(&[]).is_empty());
    }

    #[test]
    fn resolve_event_subscriptions_returns_custom_namespace_only() {
        let mut skill = base_skill();
        skill.artifacts = vec!["html".to_string()];
        let subs = resolve_event_subscriptions(&[skill]);
        assert!(!subs.iter().any(|s| s.starts_with("studio.")));
        assert!(!subs.iter().any(|s| s.starts_with("ship.")));
        assert!(subs.contains(&"my-skill.".to_string()));
    }

    #[test]
    fn resolve_event_subscriptions_deduplicates() {
        let mut s1 = base_skill();
        s1.artifacts = vec!["adr".to_string()];
        let mut s2 = base_skill();
        s2.id = "other-skill".to_string();
        s2.artifacts = vec!["adr".to_string()];
        let subs = resolve_event_subscriptions(&[s1, s2]);
        assert!(subs.contains(&"my-skill.".to_string()));
        assert!(subs.contains(&"other-skill.".to_string()));
        assert_eq!(subs.len(), 2);
    }

    #[test]
    fn format_skill_file_stable_id_injected() {
        let mut skill = base_skill();
        skill.stable_id = Some("web-qa".to_string());
        skill.content = "Write to .ship-session/{{ stable_id }}/report.md".to_string();
        let out = format_skill_file(&skill, &no_runtime());
        assert!(
            out.contains("Write to .ship-session/web-qa/report.md"),
            "got:\n{out}"
        );
    }

    #[test]
    fn format_skill_file_stable_id_none_leaves_placeholder() {
        let mut skill = base_skill();
        skill.stable_id = None;
        skill.content = "Write to .ship-session/{{ stable_id }}/report.md".to_string();
        let out = format_skill_file(&skill, &no_runtime());
        assert!(
            !out.contains("{{ stable_id }}"),
            "unresolved placeholder in output:\n{out}"
        );
    }

    #[test]
    fn format_skill_file_runtime_agents_injected() {
        let mut skill = base_skill();
        skill.content =
            "{% for a in runtime.agents %}- {{ a.id }}\n{% endfor %}".to_string();
        let runtime = serde_json::json!({
            "agents": [
                {"id": "rust-compiler", "name": "Rust Compiler", "description": null},
                {"id": "web-lane", "name": "Web Lane", "description": null},
            ]
        });
        let out = format_skill_file(&skill, &runtime);
        assert!(out.contains("- rust-compiler\n"), "got:\n{out}");
        assert!(out.contains("- web-lane\n"), "got:\n{out}");
    }
}
