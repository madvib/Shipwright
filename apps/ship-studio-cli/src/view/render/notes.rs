use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{List, ListItem, ListState, Paragraph, Wrap},
};

use super::{C_BG, C_FG, C_MUT, C_SEL, panel};
use crate::view::App;

pub fn draw_notes(f: &mut Frame, app: &App, area: Rect) {
    if app.notes.is_empty() {
        f.render_widget(
            Paragraph::new("  No notes yet.")
                .style(Style::default().fg(C_MUT).bg(C_BG))
                .block(panel("Notes")),
            area,
        );
        return;
    }
    let items: Vec<ListItem> = app
        .notes
        .iter()
        .map(|n| {
            let date = n.created_at.get(..10).unwrap_or("").to_string();
            let preview: String = n
                .content
                .lines()
                .next()
                .unwrap_or("")
                .chars()
                .take(56)
                .collect();
            let line = Line::from(vec![
                Span::styled(format!(" {:<36}", n.title), Style::default().fg(C_FG)),
                Span::styled(format!("  {date}  "), Style::default().fg(C_MUT)),
                Span::styled(preview, Style::default().fg(C_MUT)),
            ]);
            ListItem::new(line)
        })
        .collect();
    let mut state = ListState::default();
    state.select(Some(app.sel_note));
    f.render_stateful_widget(
        List::new(items)
            .block(panel(format!("Notes  ({})", app.notes.len())))
            .highlight_style(Style::default().bg(C_SEL))
            .highlight_symbol("▶ "),
        area,
        &mut state,
    );
}

pub fn draw_note_detail(f: &mut Frame, app: &App, area: Rect) {
    let Some(n) = app.notes.get(app.sel_note) else {
        return;
    };
    let date = n.created_at.get(..10).unwrap_or("");
    let title = format!("{}  ·  {date}", n.title);
    f.render_widget(
        Paragraph::new(n.content.clone())
            .block(panel(title))
            .wrap(Wrap { trim: false })
            .scroll((app.note_scroll, 0))
            .style(Style::default().fg(C_FG).bg(C_BG)),
        area,
    );
}
