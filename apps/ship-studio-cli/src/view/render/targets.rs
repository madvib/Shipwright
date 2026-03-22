use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState, Paragraph},
};

use super::{C_BG, C_FG, C_GREEN, C_MUT, C_PRI, C_SEL, panel, status_color, status_sym};
use crate::view::{App, data};

pub fn draw_targets(f: &mut Frame, app: &App, area: Rect) {
    if app.targets.is_empty() {
        f.render_widget(
            Paragraph::new("  No targets yet. Use commander to create one.")
                .style(Style::default().fg(C_MUT).bg(C_BG))
                .block(panel("Targets")),
            area,
        );
        return;
    }
    let items: Vec<ListItem> = app
        .targets
        .iter()
        .map(|t| {
            let sc = status_color(&t.status);
            let (actual, total) = data::load_cap_progress(&app.ship_dir, &t.id);
            let progress = if total > 0 {
                let pct = (actual * 100) / total;
                let bar_w = 10;
                let filled = (actual * bar_w) / total;
                let bar: String = "█".repeat(filled) + &"░".repeat(bar_w - filled);
                format!(" {bar} {actual}/{total} ({pct}%)")
            } else {
                String::new()
            };
            let line = Line::from(vec![
                Span::styled(
                    format!(" {} ", status_sym(&t.status)),
                    Style::default().fg(sc),
                ),
                Span::styled(format!("{:<32}", t.title), Style::default().fg(C_FG)),
                Span::styled(format!(" {:<10}", t.kind), Style::default().fg(C_MUT)),
                Span::styled(progress, Style::default().fg(C_GREEN)),
            ]);
            ListItem::new(line)
        })
        .collect();
    let mut state = ListState::default();
    state.select(Some(app.sel_target));
    f.render_stateful_widget(
        List::new(items)
            .block(panel(format!("Targets  ({})", app.targets.len())))
            .highlight_style(Style::default().bg(C_SEL))
            .highlight_symbol("▶ "),
        area,
        &mut state,
    );
}

pub fn draw_target_detail(f: &mut Frame, app: &App, area: Rect) {
    let Some(t) = app.targets.get(app.sel_target) else {
        return;
    };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(7), Constraint::Min(0)])
        .split(area);

    let sc = status_color(&t.status);
    let mut lines = vec![
        Line::from(vec![
            Span::styled(
                format!(" {} ", status_sym(&t.status)),
                Style::default().fg(sc),
            ),
            Span::styled(
                t.title.clone(),
                Style::default().fg(C_FG).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("  [{}]", t.kind), Style::default().fg(C_MUT)),
        ]),
        Line::from(vec![
            Span::styled("   status   ", Style::default().fg(C_MUT)),
            Span::styled(t.status.clone(), Style::default().fg(sc)),
        ]),
    ];
    if let Some(ref g) = t.goal {
        lines.push(Line::from(vec![
            Span::styled("   goal     ", Style::default().fg(C_MUT)),
            Span::styled(g.clone(), Style::default().fg(C_FG)),
        ]));
    }
    if let Some(ref d) = t.description {
        lines.push(Line::from(Span::styled("", Style::default())));
        lines.push(Line::from(Span::styled(
            format!("   {d}"),
            Style::default().fg(C_MUT),
        )));
    }
    f.render_widget(
        Paragraph::new(lines).block(panel(t.title.clone())),
        chunks[0],
    );

    if app.caps.is_empty() {
        f.render_widget(
            Paragraph::new("  No capabilities.")
                .style(Style::default().fg(C_MUT).bg(C_BG))
                .block(panel("Capabilities")),
            chunks[1],
        );
        return;
    }
    let items: Vec<ListItem> = app
        .caps
        .iter()
        .map(|c| {
            let sc = status_color(&c.status);
            let line = Line::from(vec![
                Span::styled(
                    format!(" {} ", status_sym(&c.status)),
                    Style::default().fg(sc),
                ),
                Span::styled(format!("{:<48}", c.title), Style::default().fg(C_FG)),
                Span::styled(c.status.clone(), Style::default().fg(sc)),
            ]);
            ListItem::new(line)
        })
        .collect();
    let mut state = ListState::default();
    state.select(Some(app.sel_cap));
    f.render_stateful_widget(
        List::new(items)
            .block(panel(format!("Capabilities  ({})", app.caps.len())))
            .highlight_style(Style::default().bg(C_SEL))
            .highlight_symbol("▶ "),
        chunks[1],
        &mut state,
    );
}

pub fn draw_cap_detail(f: &mut Frame, app: &App, area: Rect) {
    let (Some(t), Some(c)) = (app.targets.get(app.sel_target), app.caps.get(app.sel_cap)) else {
        return;
    };
    let sc = status_color(&c.status);
    let mut lines = vec![
        Line::from(vec![
            Span::styled(
                format!(" {} ", status_sym(&c.status)),
                Style::default().fg(sc),
            ),
            Span::styled(
                c.title.clone(),
                Style::default().fg(C_FG).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(Span::styled("", Style::default())),
        Line::from(vec![
            Span::styled("   status     ", Style::default().fg(C_MUT)),
            Span::styled(c.status.clone(), Style::default().fg(sc)),
        ]),
        Line::from(vec![
            Span::styled("   target     ", Style::default().fg(C_MUT)),
            Span::styled(t.title.clone(), Style::default().fg(C_FG)),
        ]),
    ];
    if let Some(ref m) = c.milestone_id {
        lines.push(Line::from(vec![
            Span::styled("   milestone  ", Style::default().fg(C_MUT)),
            Span::styled(m.clone(), Style::default().fg(C_FG)),
        ]));
    }
    if let Some(ref e) = c.evidence {
        lines.push(Line::from(Span::styled("", Style::default())));
        lines.push(Line::from(Span::styled(
            "   Evidence",
            Style::default().fg(C_PRI).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            format!("   {e}"),
            Style::default().fg(C_FG),
        )));
    }
    lines.push(Line::from(Span::styled("", Style::default())));
    lines.push(Line::from(vec![
        Span::styled("   id         ", Style::default().fg(C_MUT)),
        Span::styled(c.id.clone(), Style::default().fg(C_MUT)),
    ]));
    f.render_widget(Paragraph::new(lines).block(panel("Capability")), area);
}
