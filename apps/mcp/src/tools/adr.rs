use std::path::Path;

pub fn create_adr(_project_dir: &Path, title: &str, decision: &str) -> String {
    match runtime::db::adrs::create_adr(title, "", decision, "proposed") {
        Ok(entry) => format!("Created ADR '{}' (id: {})", entry.title, entry.id),
        Err(e) => format!("Error: {}", e),
    }
}
