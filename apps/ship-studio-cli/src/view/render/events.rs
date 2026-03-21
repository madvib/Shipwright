use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState, Paragraph},
};

use crate::view::App;
use super::{C_BG, C_FG, C_MUT, C_PRI, C_SEL, panel, status_color};

pub fn draw_events(f: &mut Frame, app: &App, area: Rect) {
    if app.events.is_empty() {
        let msg = Paragraph::new("  No events yet.")
            .style(Style::default().fg(C_MUT).bg(C_BG))
            .block(panel("Events"));
        f.render_widget(msg, area);
        return;
    }
    let items: Vec<ListItem> = app
        .events
        .iter()
        .map(|e| {
            let action_str = e.action.as_str();
            let action_color = status_color(action_str);
            let time = e.timestamp.format("%H:%M:%S").to_string();
            let detail = e.details.as_deref().unwrap_or("");
            let detail_short = if detail.len() > 60 { &detail[..57] } else { detail };
            let line = Line::from(vec![
                Span::styled(format!(" {time} "), Style::default().fg(C_MUT)),
                Span::styled(
                    format!("{:<12}", e.entity.as_str()),
                    Style::default().fg(C_PRI).add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!("{:<10}", action_str), Style::default().fg(action_color)),
                Span::styled(format!("{:<12}", e.subject), Style::default().fg(C_FG)),
                Span::styled(
                    if detail_short.is_empty() { String::new() } else { format!(" {detail_short}") },
                    Style::default().fg(C_MUT),
                ),
            ]);
            ListItem::new(line)
        })
        .collect();
    let mut state = ListState::default();
    state.select(Some(app.sel_event));
    f.render_stateful_widget(
        List::new(items)
            .block(panel(format!("Events  (latest {})", app.events.len())))
            .highlight_style(Style::default().bg(C_SEL))
            .highlight_symbol("▶ "),
        area,
        &mut state,
    );
}

pub fn draw_event_detail(f: &mut Frame, app: &App, area: Rect) {
    let Some(e) = app.events.get(app.sel_event) else { return };
    let action_str = e.action.as_str();
    let sc = status_color(action_str);
    let ts = e.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
    let mut lines = vec![
        Line::from(vec![
            Span::styled(
                format!(" {}", e.entity.as_str()),
                Style::default().fg(C_PRI).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("  {action_str}"), Style::default().fg(sc)),
        ]),
        Line::from(Span::styled("", Style::default())),
        Line::from(vec![
            Span::styled("   subject   ", Style::default().fg(C_MUT)),
            Span::styled(e.subject.clone(), Style::default().fg(C_FG)),
        ]),
        Line::from(vec![
            Span::styled("   actor     ", Style::default().fg(C_MUT)),
            Span::styled(e.actor.clone(), Style::default().fg(C_FG)),
        ]),
        Line::from(vec![
            Span::styled("   time      ", Style::default().fg(C_MUT)),
            Span::styled(ts, Style::default().fg(C_FG)),
        ]),
    ];
    if let Some(ref ws) = e.workspace_id {
        lines.push(Line::from(vec![
            Span::styled("   workspace ", Style::default().fg(C_MUT)),
            Span::styled(ws.clone(), Style::default().fg(C_FG)),
        ]));
    }
    if let Some(ref s) = e.session_id {
        lines.push(Line::from(vec![
            Span::styled("   session   ", Style::default().fg(C_MUT)),
            Span::styled(s.clone(), Style::default().fg(C_FG)),
        ]));
    }
    if let Some(ref j) = e.job_id {
        lines.push(Line::from(vec![
            Span::styled("   job       ", Style::default().fg(C_MUT)),
            Span::styled(j.clone(), Style::default().fg(C_FG)),
        ]));
    }
    if let Some(ref d) = e.details {
        lines.push(Line::from(Span::styled("", Style::default())));
        lines.push(Line::from(Span::styled(
            "   Details",
            Style::default().fg(C_PRI).add_modifier(Modifier::BOLD),
        )));
        for dl in d.lines() {
            lines.push(Line::from(Span::styled(
                format!("   {dl}"),
                Style::default().fg(C_FG),
            )));
        }
    }
    lines.push(Line::from(Span::styled("", Style::default())));
    lines.push(Line::from(vec![
        Span::styled("   id        ", Style::default().fg(C_MUT)),
        Span::styled(e.id.clone(), Style::default().fg(C_MUT)),
    ]));
    f.render_widget(Paragraph::new(lines).block(panel("Event")), area);
}
