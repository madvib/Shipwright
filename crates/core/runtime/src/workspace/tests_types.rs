#[cfg(test)]
mod tests {
    use crate::workspace::helpers::*;
    use crate::workspace::types::*;

    #[test]
    fn lifecycle_transition_matrix_covers_expected_paths() {
        assert!(validate_workspace_transition(
            WorkspaceStatus::Archived,
            WorkspaceStatus::Active
        )
        .is_ok());
        assert!(validate_workspace_transition(
            WorkspaceStatus::Active,
            WorkspaceStatus::Archived
        )
        .is_ok());
        assert!(validate_workspace_transition(
            WorkspaceStatus::Archived,
            WorkspaceStatus::Archived
        )
        .is_ok());
    }

    #[test]
    fn workspace_read_model_parsers_reject_unknown_status() {
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
}
