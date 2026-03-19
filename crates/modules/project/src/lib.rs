pub mod adr;
pub mod note;
pub mod ops;
pub mod project;

pub use adr::{
    ADR, AdrEntry, AdrMetadata, AdrStatus, create_adr, delete_adr, find_adr_path, get_adr_by_id,
    import_adrs_from_files, list_adrs, move_adr, update_adr,
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
