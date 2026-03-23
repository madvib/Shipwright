//! Content security scanner for skill and prompt files.
//!
//! Detects hidden Unicode characters that are invisible to humans but
//! tokenized by LLMs — a prompt injection vector. Scans run automatically
//! on `ship install` and are available via `ship audit`.
//!
//! Based on APM's ContentScanner (MIT licensed, Microsoft).

use std::path::Path;

/// Severity of a scan finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// No legitimate use in prompt files. Block by default.
    Critical,
    /// Suspicious — common copy-paste debris but can hide instructions.
    Warning,
    /// Unusual but mostly harmless (e.g. non-breaking space).
    Info,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Critical => write!(f, "critical"),
            Severity::Warning => write!(f, "warning"),
            Severity::Info => write!(f, "info"),
        }
    }
}

/// A single suspicious character found during scanning.
#[derive(Debug, Clone)]
pub struct Finding {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub codepoint: u32,
    pub severity: Severity,
    pub category: &'static str,
    pub description: &'static str,
}

impl std::fmt::Display for Finding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}: {} [{}] U+{:04X} — {}",
            self.file, self.line, self.column, self.severity, self.category,
            self.codepoint, self.description,
        )
    }
}

/// Suspicious Unicode character ranges.
///
/// Each entry: (start, end_inclusive, severity, category, description).
const SUSPICIOUS_RANGES: &[(u32, u32, Severity, &str, &str)] = &[
    // ── Critical: no legitimate use in prompt files ──
    // Unicode tag characters — invisible ASCII mapping
    (0xE0001, 0xE007F, Severity::Critical, "tag-character",
     "Unicode tag character (invisible ASCII mapping)"),
    // Bidirectional override characters
    (0x202A, 0x202A, Severity::Critical, "bidi-override", "Left-to-right embedding (LRE)"),
    (0x202B, 0x202B, Severity::Critical, "bidi-override", "Right-to-left embedding (RLE)"),
    (0x202C, 0x202C, Severity::Critical, "bidi-override", "Pop directional formatting (PDF)"),
    (0x202D, 0x202D, Severity::Critical, "bidi-override", "Left-to-right override (LRO)"),
    (0x202E, 0x202E, Severity::Critical, "bidi-override", "Right-to-left override (RLO)"),
    (0x2066, 0x2066, Severity::Critical, "bidi-override", "Left-to-right isolate (LRI)"),
    (0x2067, 0x2067, Severity::Critical, "bidi-override", "Right-to-left isolate (RLI)"),
    (0x2068, 0x2068, Severity::Critical, "bidi-override", "First strong isolate (FSI)"),
    (0x2069, 0x2069, Severity::Critical, "bidi-override", "Pop directional isolate (PDI)"),
    // Variation selectors (SMP) — Glassworm attack vector
    (0xE0100, 0xE01EF, Severity::Critical, "variation-selector",
     "Variation selector (SMP) — no legitimate use in prompt files"),

    // ── Warning: suspicious but sometimes benign ──
    (0x200B, 0x200B, Severity::Warning, "zero-width", "Zero-width space"),
    (0x200C, 0x200C, Severity::Warning, "zero-width", "Zero-width non-joiner (ZWNJ)"),
    (0x200D, 0x200D, Severity::Warning, "zero-width", "Zero-width joiner (ZWJ)"),
    (0x2060, 0x2060, Severity::Warning, "zero-width", "Word joiner"),
    (0xFE00, 0xFE0D, Severity::Warning, "variation-selector", "Variation selector (CJK)"),
    (0xFE0E, 0xFE0E, Severity::Warning, "variation-selector", "Text presentation selector"),
    (0x00AD, 0x00AD, Severity::Warning, "invisible-formatting", "Soft hyphen"),
    (0x200E, 0x200E, Severity::Warning, "bidi-mark", "Left-to-right mark (LRM)"),
    (0x200F, 0x200F, Severity::Warning, "bidi-mark", "Right-to-left mark (RLM)"),
    (0x061C, 0x061C, Severity::Warning, "bidi-mark", "Arabic letter mark (ALM)"),
    (0x2061, 0x2061, Severity::Warning, "invisible-formatting", "Function application"),
    (0x2062, 0x2062, Severity::Warning, "invisible-formatting", "Invisible times"),
    (0x2063, 0x2063, Severity::Warning, "invisible-formatting", "Invisible separator"),
    (0x2064, 0x2064, Severity::Warning, "invisible-formatting", "Invisible plus"),
    (0xFFF9, 0xFFF9, Severity::Warning, "annotation-marker", "Interlinear annotation anchor"),
    (0xFFFA, 0xFFFA, Severity::Warning, "annotation-marker", "Interlinear annotation separator"),
    (0xFFFB, 0xFFFB, Severity::Warning, "annotation-marker", "Interlinear annotation terminator"),
    (0x206A, 0x206F, Severity::Warning, "deprecated-formatting", "Deprecated formatting character"),

    // ── Info: unusual whitespace ──
    (0xFE0F, 0xFE0F, Severity::Info, "variation-selector", "Emoji presentation selector"),
    (0x00A0, 0x00A0, Severity::Info, "unusual-whitespace", "Non-breaking space"),
    (0x2000, 0x200A, Severity::Info, "unusual-whitespace", "Unicode whitespace character"),
    (0x205F, 0x205F, Severity::Info, "unusual-whitespace", "Medium mathematical space"),
    (0x3000, 0x3000, Severity::Info, "unusual-whitespace", "Ideographic space"),
    (0x180E, 0x180E, Severity::Info, "unusual-whitespace", "Mongolian vowel separator"),
];

/// Classify a codepoint against the suspicious ranges.
fn classify_char(cp: u32) -> Option<(Severity, &'static str, &'static str)> {
    for &(start, end, sev, cat, desc) in SUSPICIOUS_RANGES {
        if cp >= start && cp <= end {
            return Some((sev, cat, desc));
        }
    }
    None
}

/// Returns true if the character is an emoji base (Unicode general category "So").
fn is_emoji(ch: char) -> bool {
    // Symbol, Other — covers most emoji base characters.
    // This is a fast heuristic matching APM's approach.
    matches!(ch as u32,
        0x2600..=0x27BF |   // Misc symbols, dingbats
        0x1F300..=0x1F9FF | // Main emoji blocks
        0x1FA00..=0x1FAFF   // Extended-A
    )
}

/// Check if a ZWJ at `idx` is part of a legitimate emoji sequence.
fn zwj_in_emoji_context(text: &str, byte_idx: usize) -> bool {
    let chars: Vec<char> = text.chars().collect();
    let char_idx = text[..byte_idx].chars().count();

    // Look backward past VS16 (U+FE0F) and skin-tone modifiers (U+1F3FB-1F3FF).
    let mut prev = char_idx.wrapping_sub(1);
    while prev < chars.len() {
        let cp = chars[prev] as u32;
        if cp == 0xFE0F || (0x1F3FB..=0x1F3FF).contains(&cp) {
            prev = prev.wrapping_sub(1);
            continue;
        }
        break;
    }
    let prev_ok = prev < chars.len() && is_emoji(chars[prev]);

    // Look forward — next char must be an emoji base.
    let nxt = char_idx + 1;
    let next_ok = nxt < chars.len() && is_emoji(chars[nxt]);

    prev_ok && next_ok
}

/// Scan text content for suspicious Unicode characters.
///
/// Returns findings sorted by line/column. Pure ASCII content returns
/// immediately (fast path).
pub fn scan_text(content: &str, filename: &str) -> Vec<Finding> {
    // Fast path: pure ASCII has no suspicious codepoints.
    if content.is_ascii() {
        return vec![];
    }

    let mut findings = Vec::new();

    for (line_idx, line) in content.split('\n').enumerate() {
        let mut col = 0usize;
        for ch in line.chars() {
            let cp = ch as u32;
            col += 1;

            // BOM: start-of-file is info, mid-file is warning.
            if cp == 0xFEFF {
                let (sev, desc) = if line_idx == 0 && col == 1 {
                    (Severity::Info, "Byte order mark at start of file")
                } else {
                    (Severity::Warning, "Byte order mark mid-file (possible hidden content)")
                };
                findings.push(Finding {
                    file: filename.to_string(),
                    line: line_idx + 1,
                    column: col,
                    codepoint: cp,
                    severity: sev,
                    category: "bom",
                    description: desc,
                });
                continue;
            }

            if let Some((mut sev, cat, mut desc)) = classify_char(cp) {
                // ZWJ between emoji is legitimate.
                if cp == 0x200D {
                    let byte_offset = line.char_indices()
                        .nth(col - 1)
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    if zwj_in_emoji_context(line, byte_offset) {
                        sev = Severity::Info;
                        desc = "Zero-width joiner (emoji sequence)";
                    }
                }
                findings.push(Finding {
                    file: filename.to_string(),
                    line: line_idx + 1,
                    column: col,
                    codepoint: cp,
                    severity: sev,
                    category: cat,
                    description: desc,
                });
            }
        }
    }

    findings
}

/// Scan a file for suspicious Unicode characters.
///
/// Returns an empty vec if the file can't be read as UTF-8 (binary files).
pub fn scan_file(path: &Path) -> Vec<Finding> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    scan_text(&content, &path.to_string_lossy())
}

/// Scan all files in a directory tree, returning findings for each file.
pub fn scan_dir(dir: &Path) -> Vec<Finding> {
    let mut all = Vec::new();
    let walker = walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_entry(|e| e.file_name() != ".git");
    for entry in walker.flatten() {
        if entry.file_type().is_file() {
            all.extend(scan_file(entry.path()));
        }
    }
    all
}

/// Returns true if any finding has critical severity.
pub fn has_critical(findings: &[Finding]) -> bool {
    findings.iter().any(|f| f.severity == Severity::Critical)
}

/// Count findings by severity.
pub fn summarize(findings: &[Finding]) -> (usize, usize, usize) {
    let mut critical = 0;
    let mut warning = 0;
    let mut info = 0;
    for f in findings {
        match f.severity {
            Severity::Critical => critical += 1,
            Severity::Warning => warning += 1,
            Severity::Info => info += 1,
        }
    }
    (critical, warning, info)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_content_returns_empty() {
        let findings = scan_text("Hello, world!\nPlain ASCII.", "test.md");
        assert!(findings.is_empty());
    }

    #[test]
    fn detects_bidi_override() {
        // U+202E = Right-to-left override
        let content = "normal \u{202E}hidden";
        let findings = scan_text(content, "test.md");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Critical);
        assert_eq!(findings[0].category, "bidi-override");
        assert_eq!(findings[0].codepoint, 0x202E);
    }

    #[test]
    fn detects_tag_characters() {
        // U+E0001 = Language tag
        let content = format!("start{}end", '\u{E0001}');
        let findings = scan_text(&content, "test.md");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Critical);
        assert_eq!(findings[0].category, "tag-character");
    }

    #[test]
    fn detects_zero_width_space() {
        let content = "before\u{200B}after";
        let findings = scan_text(content, "test.md");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Warning);
        assert_eq!(findings[0].category, "zero-width");
    }

    #[test]
    fn bom_at_start_is_info() {
        let content = "\u{FEFF}content";
        let findings = scan_text(content, "test.md");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Info);
        assert_eq!(findings[0].category, "bom");
    }

    #[test]
    fn bom_mid_file_is_warning() {
        let content = "line1\n\u{FEFF}line2";
        let findings = scan_text(content, "test.md");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Warning);
    }

    #[test]
    fn non_breaking_space_is_info() {
        let content = "hello\u{00A0}world";
        let findings = scan_text(content, "test.md");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Info);
    }

    #[test]
    fn line_and_column_correct() {
        let content = "line1\nab\u{200B}cd";
        let findings = scan_text(content, "test.md");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].line, 2);
        assert_eq!(findings[0].column, 3);
    }

    #[test]
    fn multiple_findings_across_lines() {
        let content = "\u{202E}first\nsecond\u{200B}third";
        let findings = scan_text(content, "test.md");
        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0].line, 1);
        assert_eq!(findings[1].line, 2);
    }

    #[test]
    fn has_critical_detects_severity() {
        let content = "\u{202E}bad";
        let findings = scan_text(content, "test.md");
        assert!(has_critical(&findings));

        let content2 = "\u{200B}mild";
        let findings2 = scan_text(content2, "test.md");
        assert!(!has_critical(&findings2));
    }

    #[test]
    fn summarize_counts_correctly() {
        let content = "\u{202E}\u{200B}\u{00A0}";
        let findings = scan_text(content, "test.md");
        let (c, w, i) = summarize(&findings);
        assert_eq!(c, 1); // bidi override
        assert_eq!(w, 1); // zero-width space
        assert_eq!(i, 1); // non-breaking space
    }

    #[test]
    fn variation_selector_smp_is_critical() {
        let content = format!("x{}y", '\u{E0100}');
        let findings = scan_text(&content, "test.md");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Critical);
        assert_eq!(findings[0].category, "variation-selector");
    }

    #[test]
    fn empty_content_returns_empty() {
        assert!(scan_text("", "test.md").is_empty());
    }
}
