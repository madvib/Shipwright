//! Ship design tokens -> terminal RGB palette.
//! Mirrors the web app's oklch dark-mode palette.

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders};

// -- Core palette -------------------------------------------------------
pub const C_BG: Color = Color::Rgb(28, 30, 38);
pub const C_FG: Color = Color::Rgb(200, 196, 190);
pub const C_PRI: Color = Color::Rgb(200, 156, 74); // amber primary
pub const C_MUT: Color = Color::Rgb(110, 100, 88); // muted text
pub const C_BOR: Color = Color::Rgb(55, 58, 72); // borders
pub const C_SEL: Color = Color::Rgb(44, 47, 60); // selection bg

// -- Status palette -----------------------------------------------------
pub const C_GREEN: Color = Color::Rgb(82, 168, 112);
pub const C_BLUE: Color = Color::Rgb(75, 135, 195);
pub const C_RED: Color = Color::Rgb(190, 80, 60);
pub const C_AMBER: Color = Color::Rgb(192, 162, 52);
pub const C_PURPLE: Color = Color::Rgb(140, 96, 180);

pub fn status_color(s: &str) -> Color {
    match s {
        "actual" | "complete" | "done" | "accepted" => C_GREEN,
        "pending" | "proposed" => C_BLUE,
        "aspirational" => C_PURPLE,
        "active" | "running" | "in_progress" => C_AMBER,
        "failed" | "blocked" | "rejected" | "superseded" => C_RED,
        _ => C_MUT,
    }
}

pub fn status_sym(s: &str) -> &'static str {
    match s {
        "actual" | "complete" | "done" | "accepted" => "\u{25cf}", // filled circle
        "pending" | "proposed" => "\u{25cb}",                      // open circle
        "aspirational" => "\u{25ce}",                              // bullseye
        "active" | "running" | "in_progress" => "\u{25c6}",        // diamond
        "failed" | "blocked" | "rejected" => "\u{2716}",           // heavy X
        _ => "\u{00b7}",                                           // middle dot
    }
}

/// Standard panel block with Ship styling.
pub fn panel(title: impl Into<String>) -> Block<'static> {
    Block::default()
        .title(format!(" {} ", title.into()))
        .title_style(Style::default().fg(C_PRI).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(C_BOR))
        .style(Style::default().bg(C_BG))
}

/// Row style for selected items.
pub fn selected_style() -> Style {
    Style::default().bg(C_SEL).fg(C_FG)
}

/// Row style for normal items.
pub fn normal_style() -> Style {
    Style::default().fg(C_FG)
}

/// Header row style.
pub fn header_style() -> Style {
    Style::default().fg(C_PRI).add_modifier(Modifier::BOLD)
}

/// Truncate a string with ellipsis (char-safe).
pub fn truncate(s: &str, max: usize) -> String {
    let chars: usize = s.chars().count();
    if chars <= max {
        s.to_string()
    } else {
        let end: String = s.chars().take(max.saturating_sub(3)).collect();
        format!("{end}...")
    }
}

/// Basic markdown line styling: headers, bold, lists, rules, code fences.
pub fn style_md_line(line: &str) -> Line<'static> {
    let trimmed = line.trim_start();
    if let Some(rest) = trimmed.strip_prefix("### ") {
        return Line::from(Span::styled(
            format!("  {rest}"),
            Style::default().fg(C_PRI),
        ));
    }
    if let Some(rest) = trimmed.strip_prefix("## ") {
        return Line::from(Span::styled(
            rest.to_string(),
            Style::default().fg(C_PRI).add_modifier(Modifier::BOLD),
        ));
    }
    if let Some(rest) = trimmed.strip_prefix("# ") {
        return Line::from(Span::styled(
            rest.to_string(),
            Style::default()
                .fg(C_PRI)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        ));
    }
    if trimmed.starts_with("---") {
        return Line::from(Span::styled(
            "\u{2500}".repeat(40),
            Style::default().fg(C_BOR),
        ));
    }
    if let Some(rest) = trimmed
        .strip_prefix("- ")
        .or_else(|| trimmed.strip_prefix("* "))
    {
        return Line::from(vec![
            Span::styled("  \u{2022} ", Style::default().fg(C_PRI)),
            Span::styled(rest.to_string(), Style::default().fg(C_FG)),
        ]);
    }
    if trimmed.starts_with("```") {
        return Line::from(Span::styled(line.to_string(), Style::default().fg(C_MUT)));
    }
    if line.contains("**") {
        return bold_spans(line);
    }
    Line::from(Span::styled(line.to_string(), Style::default().fg(C_FG)))
}

fn bold_spans(line: &str) -> Line<'static> {
    let mut spans = Vec::new();
    let mut rest = line;
    while let Some(start) = rest.find("**") {
        if start > 0 {
            spans.push(Span::styled(
                rest[..start].to_string(),
                Style::default().fg(C_FG),
            ));
        }
        rest = &rest[start + 2..];
        if let Some(end) = rest.find("**") {
            spans.push(Span::styled(
                rest[..end].to_string(),
                Style::default().fg(C_FG).add_modifier(Modifier::BOLD),
            ));
            rest = &rest[end + 2..];
        } else {
            spans.push(Span::styled(format!("**{rest}"), Style::default().fg(C_FG)));
            rest = "";
        }
    }
    if !rest.is_empty() {
        spans.push(Span::styled(rest.to_string(), Style::default().fg(C_FG)));
    }
    Line::from(spans)
}

/// Truncate an ISO timestamp to just the date.
pub fn truncate_ts(s: &str) -> String {
    s.get(..10).unwrap_or(s).to_string()
}
