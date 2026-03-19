use std::path::Path;

pub fn create_adr(project_dir: &Path, title: &str, decision: &str) -> String {
    let ship_dir = project_dir.join(".ship");
    match runtime::db::adrs::create_adr(&ship_dir, title, "", decision, "proposed") {
        Ok(entry) => format!("Created ADR '{}' (id: {})", entry.title, entry.id),
        Err(e) => format!("Error: {}", e),
    }
}
