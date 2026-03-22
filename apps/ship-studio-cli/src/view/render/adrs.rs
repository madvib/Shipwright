use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState, Paragraph, Wrap},
};

use super::{C_BG, C_FG, C_MUT, C_PRI, C_SEL, panel};
use crate::view::App;

fn adr_status_color(s: &str) -> Color {
    match s {
        "accepted" => super::C_GREEN,
        "proposed" => super::C_BLUE,
        "rejected" | "deprecated" => super::C_RED,
        "superseded" => C_MUT,
        _ => C_MUT,
    }
}

fn adr_status_sym(s: &str) -> &'static str {
    match s {
        "accepted" => "●",
        "proposed" => "◎",
        "rejected" | "deprecated" => "✖",
        "superseded" => "·",
        _ => "·",
    }
}

pub fn draw_adrs(f: &mut Frame, app: &App, area: Rect) {
    if app.adrs.is_empty() {
        f.render_widget(
            Paragraph::new("  No ADRs yet.")
                .style(Style::default().fg(C_MUT).bg(C_BG))
                .block(panel("ADRs")),
            area,
        );
        return;
    }
    let items: Vec<ListItem> = app
        .adrs
        .iter()
        .map(|a| {
            let sc = adr_status_color(&a.status);
            let date = a.date.get(..10).unwrap_or("");
            let line = Line::from(vec![
                Span::styled(
                    format!(" {} ", adr_status_sym(&a.status)),
                    Style::default().fg(sc),
                ),
                Span::styled(format!("{:<48}", a.title), Style::default().fg(C_FG)),
                Span::styled(format!("  {:<12}", a.status), Style::default().fg(sc)),
                Span::styled(date.to_string(), Style::default().fg(C_MUT)),
            ]);
            ListItem::new(line)
        })
        .collect();
    let mut state = ListState::default();
    state.select(Some(app.sel_adr));
    f.render_stateful_widget(
        List::new(items)
            .block(panel(format!("ADRs  ({})", app.adrs.len())))
            .highlight_style(Style::default().bg(C_SEL))
            .highlight_symbol("▶ "),
        area,
        &mut state,
    );
}

pub fn draw_adr_detail(f: &mut Frame, app: &App, area: Rect) {
    let Some(a) = app.adrs.get(app.sel_adr) else {
        return;
    };
    let sc = adr_status_color(&a.status);
    let date = a.date.get(..10).unwrap_or("");
    let mut lines = vec![
        Line::from(vec![
            Span::styled(
                format!(" {} ", adr_status_sym(&a.status)),
                Style::default().fg(sc),
            ),
            Span::styled(
                a.title.clone(),
                Style::default().fg(C_FG).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("   status  ", Style::default().fg(C_MUT)),
            Span::styled(a.status.clone(), Style::default().fg(sc)),
            Span::styled(format!("   {date}"), Style::default().fg(C_MUT)),
        ]),
        Line::from(Span::styled("", Style::default())),
        Line::from(Span::styled(
            "   Context",
            Style::default().fg(C_PRI).add_modifier(Modifier::BOLD),
        )),
    ];
    for ln in a.context.lines() {
        lines.push(Line::from(Span::styled(
            format!("   {ln}"),
            Style::default().fg(C_FG),
        )));
    }
    lines.push(Line::from(Span::styled("", Style::default())));
    lines.push(Line::from(Span::styled(
        "   Decision",
        Style::default().fg(C_PRI).add_modifier(Modifier::BOLD),
    )));
    for ln in a.decision.lines() {
        lines.push(Line::from(Span::styled(
            format!("   {ln}"),
            Style::default().fg(C_FG),
        )));
    }
    if let Some(ref sup) = a.supersedes_id {
        lines.push(Line::from(Span::styled("", Style::default())));
        lines.push(Line::from(vec![
            Span::styled("   supersedes  ", Style::default().fg(C_MUT)),
            Span::styled(sup.clone(), Style::default().fg(C_MUT)),
        ]));
    }
    f.render_widget(
        Paragraph::new(lines)
            .block(panel(a.title.clone()))
            .wrap(Wrap { trim: false }),
        area,
    );
}
