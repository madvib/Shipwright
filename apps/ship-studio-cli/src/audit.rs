use std::path::PathBuf;

use anyhow::Result;
use runtime::security::{self, Severity};

/// Run `ship audit` — scan for hidden Unicode characters.
pub fn run_audit(path: Option<PathBuf>, json: bool) -> Result<()> {
    let scan_dir = match path {
        Some(p) => p,
        None => {
            let cwd = std::env::current_dir()?;
            let ship = cwd.join(".ship");
            if ship.is_dir() {
                ship
            } else {
                cwd
            }
        }
    };

    if !scan_dir.exists() {
        anyhow::bail!("path does not exist: {}", scan_dir.display());
    }

    let findings = security::scan_dir(&scan_dir);

    if json {
        let items: Vec<serde_json::Value> = findings
            .iter()
            .map(|f| {
                serde_json::json!({
                    "file": f.file,
                    "line": f.line,
                    "column": f.column,
                    "codepoint": format!("U+{:04X}", f.codepoint),
                    "severity": f.severity.to_string(),
                    "category": f.category,
                    "description": f.description,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&items)?);
        if security::has_critical(&findings) {
            std::process::exit(1);
        }
        return Ok(());
    }

    if findings.is_empty() {
        println!("No suspicious Unicode characters found in {}", scan_dir.display());
        return Ok(());
    }

    let (critical, warning, info) = security::summarize(&findings);

    for f in &findings {
        let color = match f.severity {
            Severity::Critical => "\x1b[31m",
            Severity::Warning => "\x1b[33m",
            Severity::Info => "\x1b[36m",
        };
        println!("{color}{f}\x1b[0m");
    }

    println!(
        "\n{} finding(s): {} critical, {} warning, {} info",
        findings.len(),
        critical,
        warning,
        info
    );

    if critical > 0 {
        eprintln!("\nCritical findings detected — these characters are invisible but tokenized by LLMs.");
        std::process::exit(1);
    }

    Ok(())
}
