pub mod adr;
pub mod demo;
pub mod feature;
pub mod issue;
pub mod note;
pub mod ops;
pub mod plugin;
pub mod project;
pub mod release;
pub mod spec;

pub use adr::{
    ADR, AdrEntry, AdrMetadata, AdrStatus, create_adr, delete_adr, find_adr_path, get_adr_by_id,
    import_adrs_from_files, list_adrs, move_adr, update_adr,
};
pub use demo::init_demo_project;
pub use feature::{
    Feature, FeatureAgentConfig, FeatureCriterion, FeatureEntry, FeatureMetadata, FeatureStatus,
    FeatureTodo, create_feature, delete_feature, feature_done, feature_start, get_feature_by_id,
    import_features_from_files, list_features, move_feature, update_feature,
    update_feature_content,
};
pub use issue::{
    Issue, IssueEntry, IssueMetadata, IssuePriority, IssueStatus, create_issue, delete_issue,
    get_issue_by_id, import_issues_from_files, list_issues, move_issue, update_issue,
};
pub use note::{
    Note, NoteEntry, NoteScope, create_note, delete_note, get_note_by_id, import_notes_from_files,
    list_notes, update_note, update_note_content,
};
pub use ops::{OpsError, OpsResult, ShipModule};
pub use plugin::{IssuePlugin, IssuePluginRegistry};
pub use project::{
    ADR_STATUSES, DEFAULT_STATUSES, FEATURE_STATUSES, ISSUE_STATUSES, ProjectEntry,
    ProjectRegistry, SPEC_STATUSES, discover_projects, get_project_dir, get_project_name,
    init_project, list_registered_namespaces, list_registered_projects, read_template,
    register_project, register_ship_namespace, rename_project, sanitize_file_name,
    unregister_project, write_template,
};
pub use release::{
    Release, ReleaseBreakingChange, ReleaseEntry, ReleaseMetadata, ReleaseStatus, create_release,
    delete_release, get_release_by_id, import_releases_from_files, list_releases, update_release,
    update_release_content,
};
pub use spec::{
    Spec, SpecEntry, SpecMetadata, SpecStatus, create_spec, delete_spec, get_spec_by_id,
    import_specs_from_files, list_specs, move_spec, update_spec,
};
#[cfg(test)]
mod tests;
