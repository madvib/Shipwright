//! Artifact-to-event mapping for platform event subscriptions.
//!
//! Skills declare what they produce via the `artifacts` frontmatter field.
//! This module maps those artifact types to the `ship.*` platform event suffixes
//! an actor should subscribe to, and derives custom skill namespace subscriptions
//! from skill IDs.

use crate::agents::skill::Skill;

/// Maps an artifact type to the platform event suffixes applicable to it.
///
/// Suffixes correspond to `ship.{suffix}` event types (e.g. `ship.annotation`).
/// Returns an empty slice for unrecognised artifact types.
pub fn events_for_artifact(artifact_type: &str) -> &'static [&'static str] {
    match artifact_type {
        "html" => &[
            "annotation",
            "feedback",
            "selection",
            "artifact_created",
            "artifact_deleted",
        ],
        "pdf" => &["selection", "feedback", "artifact_created", "artifact_deleted"],
        "markdown" => &["feedback", "selection", "artifact_created", "artifact_deleted"],
        "image" => &["annotation", "feedback", "artifact_created", "artifact_deleted"],
        "adr" => &["feedback", "artifact_created", "artifact_deleted"],
        "note" => &["feedback", "artifact_created", "artifact_deleted"],
        "url" => &["feedback"],
        "json" => &["feedback", "artifact_created", "artifact_deleted"],
        _ => &[],
    }
}

/// Compute the full set of event subscription namespaces for a slice of skills.
///
/// Returns a deduplicated list of:
/// - `ship.{suffix}` for each platform event inferred from artifact declarations.
/// - `{skill.id}.` for each skill's custom event namespace.
pub fn skill_event_subscriptions(skills: &[Skill]) -> Vec<String> {
    let mut subs: Vec<String> = Vec::new();

    for skill in skills {
        // Platform event subscriptions derived from artifact declarations.
        for artifact in &skill.artifacts {
            for suffix in events_for_artifact(artifact) {
                let ns = format!("ship.{suffix}");
                if !subs.contains(&ns) {
                    subs.push(ns);
                }
            }
        }
        // Custom skill namespace — the skill emits events in its own id.* namespace.
        if !skill.artifacts.is_empty() || !skill.id.is_empty() {
            let ns = format!("{}.", skill.id);
            if !subs.contains(&ns) {
                subs.push(ns);
            }
        }
    }

    subs
}

/// Collect only the custom skill namespace prefixes (e.g. `"canvas."`) from a slice
/// of skills. Used by the studio actor to subscribe to agent-emitted skill events.
pub fn skill_custom_namespaces(skills: &[Skill]) -> Vec<String> {
    let mut namespaces: Vec<String> = Vec::new();
    for skill in skills {
        let ns = format!("{}.", skill.id);
        if !namespaces.contains(&ns) {
            namespaces.push(ns);
        }
    }
    namespaces
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::skill::{Skill, SkillSource};

    fn make_skill(id: &str, artifacts: &[&str]) -> Skill {
        Skill {
            id: id.to_string(),
            name: id.to_string(),
            description: None,
            version: None,
            author: None,
            content: String::new(),
            source: SkillSource::Custom,
            artifacts: artifacts.iter().map(|s| s.to_string()).collect(),
        }
    }

    // ── events_for_artifact ───────────────────────────────────────────────────

    #[test]
    fn html_includes_annotation_and_selection() {
        let evts = events_for_artifact("html");
        assert!(evts.contains(&"annotation"));
        assert!(evts.contains(&"selection"));
        assert!(evts.contains(&"feedback"));
        assert!(evts.contains(&"artifact_created"));
        assert!(evts.contains(&"artifact_deleted"));
    }

    #[test]
    fn pdf_no_annotation() {
        let evts = events_for_artifact("pdf");
        assert!(!evts.contains(&"annotation"));
        assert!(evts.contains(&"selection"));
        assert!(evts.contains(&"feedback"));
    }

    #[test]
    fn url_only_feedback() {
        let evts = events_for_artifact("url");
        assert_eq!(evts, &["feedback"]);
    }

    #[test]
    fn unknown_artifact_empty() {
        assert!(events_for_artifact("unknown").is_empty());
        assert!(events_for_artifact("").is_empty());
    }

    #[test]
    fn all_declared_types_return_nonempty() {
        for t in &["html", "pdf", "markdown", "image", "adr", "note", "url", "json"] {
            assert!(
                !events_for_artifact(t).is_empty(),
                "expected non-empty for {t}"
            );
        }
    }

    // ── skill_event_subscriptions ─────────────────────────────────────────────

    #[test]
    fn no_skills_empty_subscriptions() {
        assert!(skill_event_subscriptions(&[]).is_empty());
    }

    #[test]
    fn html_skill_produces_ship_annotation() {
        let skills = vec![make_skill("my-skill", &["html"])];
        let subs = skill_event_subscriptions(&skills);
        assert!(subs.contains(&"ship.annotation".to_string()));
        assert!(subs.contains(&"ship.feedback".to_string()));
        assert!(subs.contains(&"ship.artifact_created".to_string()));
    }

    #[test]
    fn skill_custom_namespace_included() {
        let skills = vec![make_skill("canvas", &["html"])];
        let subs = skill_event_subscriptions(&skills);
        assert!(subs.contains(&"canvas.".to_string()));
    }

    #[test]
    fn deduplicates_across_skills() {
        let skills = vec![
            make_skill("skill-a", &["adr"]),
            make_skill("skill-b", &["adr"]),
        ];
        let subs = skill_event_subscriptions(&skills);
        // ship.feedback should appear exactly once
        let count = subs.iter().filter(|s| *s == "ship.feedback").count();
        assert_eq!(count, 1, "ship.feedback should be deduplicated");
        // Both custom namespaces present
        assert!(subs.contains(&"skill-a.".to_string()));
        assert!(subs.contains(&"skill-b.".to_string()));
    }

    #[test]
    fn skill_with_no_artifacts_still_gets_custom_namespace() {
        let skills = vec![make_skill("my-tool", &[])];
        let subs = skill_event_subscriptions(&skills);
        assert!(subs.contains(&"my-tool.".to_string()));
    }

    // ── skill_custom_namespaces ───────────────────────────────────────────────

    #[test]
    fn custom_namespaces_deduped() {
        let skills = vec![make_skill("canvas", &[]), make_skill("canvas", &[])];
        let ns = skill_custom_namespaces(&skills);
        assert_eq!(ns, vec!["canvas.".to_string()]);
    }

    #[test]
    fn custom_namespaces_all_skills() {
        let skills = vec![make_skill("canvas", &[]), make_skill("pdf-viewer", &[])];
        let ns = skill_custom_namespaces(&skills);
        assert!(ns.contains(&"canvas.".to_string()));
        assert!(ns.contains(&"pdf-viewer.".to_string()));
    }
}
