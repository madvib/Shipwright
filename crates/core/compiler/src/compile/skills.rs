use std::collections::HashMap;

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
    skills: &[Skill],
) -> HashMap<String, String> {
    let Some(base) = desc.skills_dir.base_path() else {
        return HashMap::new();
    };
    skills
        .iter()
        .filter(|skill| !skill.content.trim().is_empty())
        .map(|skill| {
            let path = format!("{}/{}/SKILL.md", base, skill.id);
            let content = format_skill_file(skill);
            (path, content)
        })
        .collect()
}

pub(super) fn format_skill_file(skill: &Skill) -> String {
    let description = skill
        .description
        .as_deref()
        .unwrap_or("No description provided.");

    let mut fm = format!("---\nname: {}\ndescription: {}", skill.id, description);

    // Resolve template variables in content before emitting.
    // Warnings go to stderr so they surface during `ship use` / `ship compile`.
    let resolved_content;
    let content: &str = if skill.vars.is_empty() {
        &skill.content
    } else {
        resolved_content = crate::vars::resolve_template(&skill.content, &skill.vars);
        &resolved_content
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
        let out = format_skill_file(&skill);
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
        let out = format_skill_file(&skill);
        assert!(out.contains("\nlicense: MIT\n"), "got:\n{out}");
    }

    #[test]
    fn format_skill_file_with_compatibility() {
        let mut skill = base_skill();
        skill.compatibility = Some("claude >= 3".to_string());
        let out = format_skill_file(&skill);
        assert!(
            out.contains("\ncompatibility: claude >= 3\n"),
            "got:\n{out}"
        );
    }

    #[test]
    fn format_skill_file_with_allowed_tools() {
        let mut skill = base_skill();
        skill.allowed_tools = vec!["Read".to_string(), "Edit".to_string()];
        let out = format_skill_file(&skill);
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
        let out = format_skill_file(&skill);
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
        let out = format_skill_file(&skill);
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
        let out = format_skill_file(&skill);
        assert!(out.contains("Use gitmoji commits."), "got:\n{out}");
    }

    #[test]
    fn format_skill_file_no_vars_passthrough() {
        let skill = base_skill();
        let out = format_skill_file(&skill);
        assert!(out.contains("Instructions here."), "got:\n{out}");
    }

    #[test]
    fn format_skill_file_with_artifacts() {
        let mut skill = base_skill();
        skill.artifacts = vec!["html".to_string(), "adr".to_string()];
        let out = format_skill_file(&skill);
        assert!(out.contains("\nartifacts: [html, adr]\n"), "got:\n{out}");
    }

    #[test]
    fn format_skill_file_no_artifacts_omitted() {
        let skill = base_skill();
        let out = format_skill_file(&skill);
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
        // studio.* is already in the base subscription — not duplicated here
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
        // two distinct skills → two distinct custom namespaces
        assert!(subs.contains(&"my-skill.".to_string()));
        assert!(subs.contains(&"other-skill.".to_string()));
        assert_eq!(subs.len(), 2);
    }
}
