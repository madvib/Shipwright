//! Artifact-to-event mapping for platform event subscriptions.
//!
//! Skills declare what they produce via the `artifacts` frontmatter field.
//! This module maps artifact types to the Studio-emitted events an agent may
//! receive for that artifact, and derives custom skill namespace subscriptions
//! from skill IDs.
//!
//! ## Event ownership
//!
//! - **Studio-emitted** (`studio.annotation`, `studio.feedback`, `studio.selection`):
//!   Studio UI actions directed at a rendered artifact. Agents receive these
//!   because they already subscribe to `studio.*` — no additional subscription
//!   prefix is needed.
//!
//! - **Agent-emitted** (e.g. `canvas.artifact_created`): The agent prefixes these
//!   with its skill ID at emit time. Studio subscribes to `{skill.id}.` to receive
//!   them. `artifact_created` / `artifact_deleted` are therefore NOT subscription
//!   topics — they are emit topics.

use crate::agents::skill::Skill;

/// Maps an artifact type to the Studio-emitted event suffixes applicable to it.
///
/// These are `studio.*` events (e.g. `studio.annotation`). Agents already
/// subscribe to `studio.*`, so no new subscription prefix is generated from
/// these suffixes — the mapping exists to document which Studio actions are
/// meaningful for each artifact type.
///
/// Returns an empty slice for unrecognised artifact types.
pub fn events_for_artifact(artifact_type: &str) -> &'static [&'static str] {
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

/// Compute the custom skill namespace prefixes an actor should subscribe to.
///
/// Returns a deduplicated list of `{skill.id}.` prefixes — one per skill.
/// Agents already subscribe to `studio.*` in their base subscription, so no
/// `studio.*` entries are added here.
///
/// Used by both the agent actor (to receive agent-emitted skill events routed
/// back) and by the Studio actor.
pub fn skill_event_subscriptions(skills: &[Skill]) -> Vec<String> {
    skill_custom_namespaces(skills)
}

/// Collect the custom skill namespace prefixes (e.g. `"canvas."`) from a slice
/// of skills. Used by the Studio actor to subscribe to agent-emitted skill events.
pub fn skill_custom_namespaces(skills: &[Skill]) -> Vec<String> {
    let mut namespaces: Vec<String> = Vec::new();
    for skill in skills {
        if skill.id.is_empty() {
            continue;
        }
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
    fn html_includes_studio_events_not_artifact_lifecycle() {
        let evts = events_for_artifact("html");
        assert!(evts.contains(&"annotation"));
        assert!(evts.contains(&"selection"));
        assert!(evts.contains(&"feedback"));
        // agent-emitted lifecycle events are NOT subscription topics
        assert!(!evts.contains(&"artifact_created"));
        assert!(!evts.contains(&"artifact_deleted"));
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

    #[test]
    fn no_ship_prefix_anywhere() {
        for t in &["html", "pdf", "markdown", "image", "adr", "note", "url", "json"] {
            for suffix in events_for_artifact(t) {
                assert!(
                    !suffix.starts_with("ship."),
                    "suffix {suffix} must not use ship.* namespace"
                );
            }
        }
    }

    // ── skill_event_subscriptions ─────────────────────────────────────────────

    #[test]
    fn no_skills_empty_subscriptions() {
        assert!(skill_event_subscriptions(&[]).is_empty());
    }

    #[test]
    fn returns_only_custom_namespace_not_studio_prefix() {
        let skills = vec![make_skill("canvas", &["html"])];
        let subs = skill_event_subscriptions(&skills);
        // studio.* already covered by base subscription — not duplicated here
        assert!(!subs.iter().any(|s| s.starts_with("studio.")));
        assert!(!subs.iter().any(|s| s.starts_with("ship.")));
        assert!(subs.contains(&"canvas.".to_string()));
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
        assert!(subs.contains(&"skill-a.".to_string()));
        assert!(subs.contains(&"skill-b.".to_string()));
        // no studio.* or ship.* entries
        assert!(!subs.iter().any(|s| s.starts_with("studio.")));
        assert!(!subs.iter().any(|s| s.starts_with("ship.")));
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

    #[test]
    fn empty_id_skipped() {
        let skills = vec![make_skill("", &[])];
        assert!(skill_custom_namespaces(&skills).is_empty());
    }
}
