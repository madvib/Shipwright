//! Render the Settings section: project identity + user preferences.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::data::{USER_PREF_KEYS, ViewData};
use super::theme::*;

pub fn draw(frame: &mut Frame, data: &ViewData, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10), // project identity
            Constraint::Min(6),     // user preferences
        ])
        .split(area);

    draw_identity(frame, data, chunks[0]);
    draw_user_prefs(frame, data, chunks[1]);
}

fn draw_identity(frame: &mut Frame, data: &ViewData, area: Rect) {
    let c = &data.config;
    let mut lines: Vec<Line> = Vec::new();

    let kv = |key: &str, val: &str| -> Line<'static> {
        Line::from(vec![
            Span::styled(
                format!("  {:<16}", key),
                Style::default().fg(C_MUT).add_modifier(Modifier::BOLD),
            ),
            Span::styled(val.to_string(), Style::default().fg(C_FG)),
        ])
    };

    lines.push(kv("Name", &c.name));
    lines.push(kv("Version", &c.version));
    if !c.description.is_empty() {
        lines.push(kv("Description", &c.description));
    }
    lines.push(kv("ID", &c.id));

    let p = Paragraph::new(lines).block(panel("Project"));
    frame.render_widget(p, area);
}

fn draw_user_prefs(frame: &mut Frame, data: &ViewData, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();

    let kv = |key: &str, val: &str| -> Line<'static> {
        let style = if val.is_empty() {
            Style::default().fg(C_MUT)
        } else {
            Style::default().fg(C_FG)
        };
        let display = if val.is_empty() {
            "(not set)".to_string()
        } else {
            val.to_string()
        };
        Line::from(vec![
            Span::styled(
                format!("  {:<20}", key),
                Style::default().fg(C_MUT).add_modifier(Modifier::BOLD),
            ),
            Span::styled(display, style),
        ])
    };

    for (i, (dot_key, _label)) in USER_PREF_KEYS.iter().enumerate() {
        let val = data
            .config
            .user_prefs
            .get(i)
            .map(|(_, v)| v.as_str())
            .unwrap_or("");
        lines.push(kv(dot_key, val));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  Press 'e' to edit  |  CLI: ship config set <key> <value>",
        Style::default().fg(C_MUT),
    )));

    let p = Paragraph::new(lines).block(panel("User Preferences"));
    frame.render_widget(p, area);
}
