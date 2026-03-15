use super::types::{
    Feature, FeatureDeclaration, FeatureDeclarationCriterion, FeatureDelta, FeatureModel,
    FeatureObservedStatus, FeatureStatusCheck,
};

impl Feature {
    pub fn declaration(&self) -> FeatureDeclaration {
        let mut normalized = self.clone();
        normalized.extract_structured_data();

        let narrative = extract_first_section(&normalized.body, &["Declaration", "Intent", "Why"])
            .unwrap_or_default();
        let acceptance_criteria = normalized
            .criteria
            .iter()
            .map(|criterion| FeatureDeclarationCriterion {
                text: criterion.text.trim().to_string(),
                has_pass_fail_condition: has_pass_fail_condition(&criterion.text),
            })
            .filter(|criterion| !criterion.text.is_empty())
            .collect();

        FeatureDeclaration {
            narrative,
            acceptance_criteria,
        }
    }

    pub fn observed_status(&self) -> FeatureObservedStatus {
        let narrative = extract_first_section(&self.body, &["Status"]).unwrap_or_default();
        let mut checks = parse_checklist(&self.body, "Status Checks");
        if checks.is_empty() {
            checks = parse_checklist(&self.body, "Status");
        }

        FeatureObservedStatus {
            narrative,
            checks: checks
                .into_iter()
                .map(|(text, passing)| FeatureStatusCheck { text, passing })
                .collect(),
        }
    }

    pub fn compute_delta(&self) -> FeatureDelta {
        let mut normalized = self.clone();
        normalized.extract_structured_data();

        let declaration = self.declaration();
        let status = self.observed_status();

        let declaration_missing =
            declaration.narrative.trim().is_empty() && declaration.acceptance_criteria.is_empty();
        let status_missing = status.narrative.trim().is_empty() && status.checks.is_empty();

        let unmet_acceptance_criteria = normalized
            .criteria
            .iter()
            .filter(|criterion| !criterion.met && !criterion.text.trim().is_empty())
            .map(|criterion| criterion.text.trim().to_string())
            .collect::<Vec<_>>();
        let failing_checks = status
            .checks
            .iter()
            .filter(|check| !check.passing && !check.text.trim().is_empty())
            .map(|check| check.text.trim().to_string())
            .collect::<Vec<_>>();
        let missing_pass_fail_criteria = declaration
            .acceptance_criteria
            .iter()
            .filter(|criterion| !criterion.has_pass_fail_condition)
            .map(|criterion| criterion.text.clone())
            .collect::<Vec<_>>();

        let mut actionable_items = Vec::new();
        if declaration_missing {
            actionable_items.push(
                "Add a Declaration section with desired-state prose and acceptance criteria."
                    .to_string(),
            );
        }
        if status_missing {
            actionable_items.push(
                "Add a Status section with observed checks and readiness signals.".to_string(),
            );
        }
        if !missing_pass_fail_criteria.is_empty() {
            actionable_items.push(format!(
                "Define explicit pass/fail conditions for {} acceptance criteria.",
                missing_pass_fail_criteria.len()
            ));
        }
        if !unmet_acceptance_criteria.is_empty() {
            actionable_items.push(format!(
                "Resolve or re-scope {} unmet acceptance criteria.",
                unmet_acceptance_criteria.len()
            ));
        }
        if !failing_checks.is_empty() {
            actionable_items.push(format!(
                "Investigate {} failing status checks.",
                failing_checks.len()
            ));
        }

        let drift_score = unmet_acceptance_criteria.len() as u32
            + failing_checks.len() as u32
            + missing_pass_fail_criteria.len() as u32
            + u32::from(declaration_missing)
            + u32::from(status_missing);

        FeatureDelta {
            declaration_missing,
            status_missing,
            unmet_acceptance_criteria,
            failing_checks,
            missing_pass_fail_criteria,
            drift_score,
            actionable_items,
        }
    }

    pub fn model(&self) -> FeatureModel {
        FeatureModel {
            declaration: self.declaration(),
            status: self.observed_status(),
            delta: self.compute_delta(),
        }
    }
}

pub fn compute_feature_model(feature: &Feature) -> FeatureModel {
    feature.model()
}

fn has_pass_fail_condition(text: &str) -> bool {
    let normalized = text.to_ascii_lowercase();
    let has_explicit = normalized.contains("pass:") && normalized.contains("fail:");
    let has_gherkin = normalized.contains("given ")
        && normalized.contains("when ")
        && normalized.contains("then ");
    has_explicit || has_gherkin
}

fn extract_first_section(body: &str, section_names: &[&str]) -> Option<String> {
    section_names
        .iter()
        .find_map(|name| extract_section(body, name))
}

fn extract_section(body: &str, section_name: &str) -> Option<String> {
    let mut in_section = false;
    let mut lines = Vec::new();
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("## ") {
            let heading = trimmed.trim_start_matches("## ").trim();
            if in_section {
                break;
            }
            if heading.eq_ignore_ascii_case(section_name) {
                in_section = true;
                continue;
            }
        }
        if in_section {
            lines.push(line);
        }
    }

    let text = lines.join("\n").trim().to_string();
    if text.is_empty() { None } else { Some(text) }
}

fn parse_checklist(body: &str, section_name: &str) -> Vec<(String, bool)> {
    let mut in_section = false;
    let mut items = Vec::new();

    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("## ") {
            let heading = trimmed.trim_start_matches("## ").trim();
            if in_section && !heading.eq_ignore_ascii_case(section_name) {
                break;
            }
            if heading.eq_ignore_ascii_case(section_name) {
                in_section = true;
                continue;
            }
        }
        if !in_section {
            continue;
        }

        if trimmed.starts_with("- [ ]") || trimmed.starts_with("- [x]") {
            let passing = trimmed.starts_with("- [x]");
            let text = trimmed[5..].trim().to_string();
            if !text.is_empty() {
                items.push((text, passing));
            }
        }
    }

    items
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FeatureMetadata;

    fn test_feature(body: &str) -> Feature {
        Feature {
            metadata: FeatureMetadata {
                id: "feat-model".to_string(),
                title: "Feature model".to_string(),
                description: None,
                created: "2026-01-01T00:00:00Z".to_string(),
                updated: "2026-01-01T00:00:00Z".to_string(),
                release_id: None,
                active_target_id: None,
                branch: None,
                agent: None,
                tags: vec![],
            },
            body: body.to_string(),
            todos: vec![],
            criteria: vec![],
        }
    }

    #[test]
    fn model_detects_delta_from_unmet_criteria_and_failing_checks() {
        let feature = test_feature(
            r#"
## Declaration
Implement workspace provider diagnostics.

## Acceptance Criteria
- [ ] PASS: workspace matrix shows source FAIL: source missing
- [ ] diagnostics render in UI

## Status
Smoke test run against fixture project.

## Status Checks
- [x] CLI command parses
- [ ] E2E export matrix is green
"#,
        );

        let model = feature.model();
        assert!(!model.delta.declaration_missing);
        assert!(!model.delta.status_missing);
        assert_eq!(model.delta.unmet_acceptance_criteria.len(), 2);
        assert_eq!(model.delta.failing_checks.len(), 1);
        assert_eq!(model.delta.missing_pass_fail_criteria.len(), 1);
        assert!(model.delta.drift_score >= 4);
        assert!(!model.delta.actionable_items.is_empty());
    }

    #[test]
    fn model_reports_zero_drift_when_contract_and_status_align() {
        let feature = test_feature(
            r#"
## Declaration
Provide deterministic provider export.

## Acceptance Criteria
- [x] PASS: all providers export SKILL.md with frontmatter FAIL: any provider writes invalid format

## Status
All compiler matrix checks passed in CI.

## Status Checks
- [x] runtime export unit tests
- [x] e2e compiler matrix tests
"#,
        );

        let model = feature.model();
        assert_eq!(model.delta.drift_score, 0);
        assert!(model.delta.unmet_acceptance_criteria.is_empty());
        assert!(model.delta.failing_checks.is_empty());
        assert!(model.delta.missing_pass_fail_criteria.is_empty());
    }
}
