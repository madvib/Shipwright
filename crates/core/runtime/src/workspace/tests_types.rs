#[cfg(test)]
mod tests {
    use super::helpers::*;
    use super::types::*;
    use std::str::FromStr;

    #[test]
    fn lifecycle_transition_matrix_covers_expected_paths() {
        assert!(
            validate_workspace_transition(
                ShipWorkspaceKind::Feature,
                WorkspaceStatus::Archived,
                WorkspaceStatus::Active
            )
            .is_ok()
        );
        assert!(
            validate_workspace_transition(
                ShipWorkspaceKind::Feature,
                WorkspaceStatus::Active,
                WorkspaceStatus::Archived
            )
            .is_ok()
        );
        assert!(
            validate_workspace_transition(
                ShipWorkspaceKind::Feature,
                WorkspaceStatus::Archived,
                WorkspaceStatus::Archived
            )
            .is_ok()
        );
        assert!(
            validate_workspace_transition(
                ShipWorkspaceKind::Feature,
                WorkspaceStatus::Archived,
                WorkspaceStatus::Archived
            )
            .is_ok()
        );
    }

    #[test]
    fn runtime_status_transitions_cover_active_archived() {
        assert!(
            validate_workspace_transition(
                ShipWorkspaceKind::Feature,
                WorkspaceStatus::Archived,
                WorkspaceStatus::Archived,
            )
            .is_ok()
        );
    }

    #[test]
    fn workspace_kind_does_not_restrict_runtime_status() {
        assert!(
            validate_workspace_transition(
                ShipWorkspaceKind::Service,
                WorkspaceStatus::Active,
                WorkspaceStatus::Archived,
            )
            .is_ok()
        );
    }

    #[test]
    fn workspace_kind_from_str_accepts_only_canonical_values() {
        assert_eq!(
            ShipWorkspaceKind::from_str("feature").unwrap(),
            ShipWorkspaceKind::Feature
        );
        assert_eq!(
            ShipWorkspaceKind::from_str("patch").unwrap(),
            ShipWorkspaceKind::Patch
        );
        assert_eq!(
            ShipWorkspaceKind::from_str("service").unwrap(),
            ShipWorkspaceKind::Service
        );
        assert_eq!(
            ShipWorkspaceKind::from_str("hotfix")
                .expect_err("hotfix should not parse")
                .to_string(),
            "Invalid workspace type: hotfix"
        );
        assert_eq!(
            ShipWorkspaceKind::from_str("refactor")
                .expect_err("refactor should not parse")
                .to_string(),
            "Invalid workspace type: refactor"
        );
        assert_eq!(
            ShipWorkspaceKind::from_str("experiment")
                .expect_err("experiment should not parse")
                .to_string(),
            "Invalid workspace type: experiment"
        );
    }

    #[test]
    fn workspace_read_model_parsers_reject_unknown_values() {
        assert_eq!(
            parse_workspace_type_required("weird-type")
                .expect_err("invalid workspace type should fail")
                .to_string(),
            "Invalid workspace type 'weird-type'; expected one of: feature, patch, service"
        );
        assert_eq!(
            parse_workspace_status_required("in-progress")
                .expect_err("invalid status should fail")
                .to_string(),
            "Invalid workspace status 'in-progress'; expected one of: active, archived"
        );
    }

    #[test]
    fn workspace_branch_key_validation_rejects_empty_values() {
        let err = ensure_branch_key("   ").unwrap_err();
        assert!(
            err.to_string()
                .contains("Workspace branch/key cannot be empty")
        );
    }

    #[test]
    fn inferred_workspace_type_prefers_feature_links_then_prefixes() {
        assert_eq!(
            infer_workspace_type("sandbox/personal", Some("auth-redesign")),
            ShipWorkspaceKind::Feature
        );
        assert_eq!(
            infer_workspace_type("service/agent-lab", None),
            ShipWorkspaceKind::Feature
        );
        assert_eq!(
            infer_workspace_type("patch/token", None),
            ShipWorkspaceKind::Patch
        );
    }
}
