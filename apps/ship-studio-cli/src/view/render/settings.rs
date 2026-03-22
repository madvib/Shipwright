use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState},
};

use crate::view::App;
use super::{C_FG, C_MUT, C_PRI, C_SEL, panel};

pub fn draw_settings(f: &mut Frame, app: &App, area: Rect) {
    let mut items: Vec<ListItem> = Vec::new();

    // Workspace state (informational header rows, not editable)
    let agent_label = app.active_agent.as_deref().unwrap_or("(none)");
    items.push(ListItem::new(Line::from(vec![
        Span::styled(" active_agent          ", Style::default().fg(C_PRI).add_modifier(Modifier::BOLD)),
        Span::styled(agent_label.to_string(), Style::default().fg(C_FG)),
    ])));
    let compiled_label = app.compiled_at.as_deref().unwrap_or("(never)");
    items.push(ListItem::new(Line::from(vec![
        Span::styled(" compiled_at           ", Style::default().fg(C_PRI).add_modifier(Modifier::BOLD)),
        Span::styled(compiled_label.to_string(), Style::default().fg(C_FG)),
    ])));

    // Separator
    items.push(ListItem::new(Line::from(
        Span::styled(" ─────────────────────────────", Style::default().fg(C_MUT)),
    )));

    // Config entries
    if app.settings.is_empty() {
        items.push(ListItem::new(Line::from(
            Span::styled("  No config values set.  Enter to add", Style::default().fg(C_MUT)),
        )));
    } else {
        for (key, value) in &app.settings {
            items.push(ListItem::new(Line::from(vec![
                Span::styled(format!(" {:<24}", key), Style::default().fg(C_FG)),
                Span::styled(value.clone(), Style::default().fg(C_MUT)),
            ])));
        }
    }

    // The header rows (active_agent, compiled_at, separator) are not selectable config items.
    // Offset selection by 3 to skip them.
    let sel_visual = if app.settings.is_empty() { 3 } else { app.sel_setting + 3 };
    let mut state = ListState::default();
    state.select(Some(sel_visual));
    f.render_stateful_widget(
        List::new(items)
            .block(panel("Settings"))
            .highlight_style(Style::default().bg(C_SEL))
            .highlight_symbol("▶ "),
        area,
        &mut state,
    );
}
