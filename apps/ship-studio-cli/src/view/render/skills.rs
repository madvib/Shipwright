use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{List, ListItem, ListState, Paragraph},
};

use crate::view::App;
use super::{C_BG, C_FG, C_MUT, C_SEL, panel};

pub fn draw_skills(f: &mut Frame, app: &App, area: Rect) {
    if app.skills.is_empty() {
        f.render_widget(
            Paragraph::new("  No skills installed.  a add")
                .style(Style::default().fg(C_MUT).bg(C_BG))
                .block(panel("Skills")),
            area,
        );
        return;
    }
    let items: Vec<ListItem> = app
        .skills
        .iter()
        .map(|(id, scope)| {
            let line = Line::from(vec![
                Span::styled(format!(" {:<36}", id), Style::default().fg(C_FG)),
                Span::styled(format!("  {scope}"), Style::default().fg(C_MUT)),
            ]);
            ListItem::new(line)
        })
        .collect();
    let mut state = ListState::default();
    state.select(Some(app.sel_skill));
    f.render_stateful_widget(
        List::new(items)
            .block(panel(format!("Skills  ({})", app.skills.len())))
            .highlight_style(Style::default().bg(C_SEL))
            .highlight_symbol("▶ "),
        area,
        &mut state,
    );
}
