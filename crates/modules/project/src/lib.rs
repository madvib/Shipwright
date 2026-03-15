pub mod adr;
pub mod demo;
pub mod feature;
pub mod note;
pub mod ops;
pub mod project;
pub mod release;

pub use adr::{
    ADR, AdrEntry, AdrMetadata, AdrStatus, create_adr, delete_adr, find_adr_path, get_adr_by_id,
    import_adrs_from_files, list_adrs, move_adr, update_adr,
};
pub use demo::init_demo_project;
pub use feature::{
    Feature, FeatureAgentConfig, FeatureCriterion, FeatureDeclaration, FeatureDeclarationCriterion,
    FeatureDelta, FeatureDocStatus, FeatureDocumentation, FeatureEntry, FeatureMetadata,
    FeatureModel, FeatureObservedStatus, FeatureStatus, FeatureStatusCheck, FeatureTodo,
    compute_feature_model, create_feature, delete_feature, ensure_feature_documentation,
    feature_done, feature_start, get_feature_by_id, get_feature_documentation, get_feature_model,
    import_features_from_files, list_features, move_feature, record_feature_session_update,
    update_feature, update_feature_content, update_feature_documentation,
};
pub use note::{
    Note, NoteEntry, NoteScope, create_note, delete_note, get_note_by_id, import_notes_from_files,
    list_notes, update_note, update_note_content,
};
pub use ops::{OpsError, OpsResult, ShipModule};
pub use project::{
    discover_projects, get_project_dir, get_project_name, init_project, list_registered_namespaces,
    list_registered_projects, read_template, register_project, register_ship_namespace,
    rename_project, sanitize_file_name, unregister_project, write_template,
};
pub use release::{
    Release, ReleaseBreakingChange, ReleaseEntry, ReleaseMetadata, ReleaseStatus, create_release,
    create_release_with_metadata, delete_release, get_release_by_id, import_releases_from_files,
    list_releases, update_release, update_release_content,
};
#[cfg(test)]
mod tests;
