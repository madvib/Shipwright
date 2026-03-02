pub mod adr;
pub mod demo;
pub mod note;

pub use adr::{
    ADR, AdrEntry, AdrMetadata, AdrStatus, create_adr, delete_adr, find_adr_path, get_adr_by_id,
    import_adrs_from_files, list_adrs, move_adr, update_adr,
};
pub use demo::init_demo_project;
pub use note::{
    Note, NoteEntry, NoteScope, create_note, delete_note, get_note_by_id, import_notes_from_files,
    list_notes, update_note, update_note_content,
};
