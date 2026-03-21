use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState, Paragraph, Wrap},
};

use crate::view::{App, JobFilter};
use super::{C_BG, C_FG, C_MUT, C_PRI, C_SEL, panel, status_color, status_sym};

fn jobs_panel_title(app: &App) -> String {
    let count = app.jobs.len();
    match app.job_filter {
        JobFilter::All => format!("Jobs  ({count})"),
        f => format!("Jobs  ({count}) [{filter}]", filter = f.label()),
    }
}

pub fn draw_jobs(f: &mut Frame, app: &App, area: Rect) {
    if app.jobs.is_empty() {
        let title = jobs_panel_title(app);
        f.render_widget(
            Paragraph::new("  No jobs.")
                .style(Style::default().fg(C_MUT).bg(C_BG))
                .block(panel(title)),
            area,
        );
        return;
    }
    let items: Vec<ListItem> = app
        .jobs
        .iter()
        .map(|j| {
            let sc = status_color(&j.status);
            let id_short = j.id.get(..8).unwrap_or(&j.id);
            let branch = j.branch.as_deref().unwrap_or("—");
            let claimed = j.claimed_by.as_deref().unwrap_or("—");
            let line = Line::from(vec![
                Span::styled(format!(" {} ", status_sym(&j.status)), Style::default().fg(sc)),
                Span::styled(format!("{id_short:<10}"), Style::default().fg(C_MUT)),
                Span::styled(format!("  {:<14}", j.kind), Style::default().fg(C_FG)),
                Span::styled(format!("  {:<10}", j.status), Style::default().fg(sc)),
                Span::styled(format!("  {:<28}", branch), Style::default().fg(C_MUT)),
                Span::styled(claimed.to_string(), Style::default().fg(C_MUT)),
            ]);
            ListItem::new(line)
        })
        .collect();
    let mut state = ListState::default();
    state.select(Some(app.sel_job));
    f.render_stateful_widget(
        List::new(items)
            .block(panel(jobs_panel_title(app)))
            .highlight_style(Style::default().bg(C_SEL))
            .highlight_symbol("▶ "),
        area,
        &mut state,
    );
}

pub fn draw_job_detail(f: &mut Frame, app: &App, area: Rect) {
    let Some(j) = app.jobs.get(app.sel_job) else { return };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(10), Constraint::Min(0)])
        .split(area);

    let sc = status_color(&j.status);
    let title = j.payload.get("title").and_then(|v| v.as_str()).unwrap_or("—");
    let desc = j.payload.get("description").and_then(|v| v.as_str()).unwrap_or("");
    let branch = j.branch.as_deref().unwrap_or("—");
    let claimed = j.claimed_by.as_deref().unwrap_or("—");

    let mut lines = vec![
        Line::from(vec![
            Span::styled(format!(" {} ", status_sym(&j.status)), Style::default().fg(sc)),
            Span::styled(title.to_string(), Style::default().fg(C_FG).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(Span::styled("", Style::default())),
        Line::from(vec![
            Span::styled("   status   ", Style::default().fg(C_MUT)),
            Span::styled(j.status.clone(), Style::default().fg(sc)),
        ]),
        Line::from(vec![
            Span::styled("   kind     ", Style::default().fg(C_MUT)),
            Span::styled(j.kind.clone(), Style::default().fg(C_FG)),
        ]),
        Line::from(vec![
            Span::styled("   branch   ", Style::default().fg(C_MUT)),
            Span::styled(branch.to_string(), Style::default().fg(C_FG)),
        ]),
        Line::from(vec![
            Span::styled("   claimed  ", Style::default().fg(C_MUT)),
            Span::styled(claimed.to_string(), Style::default().fg(C_FG)),
        ]),
        Line::from(vec![
            Span::styled("   id       ", Style::default().fg(C_MUT)),
            Span::styled(j.id.clone(), Style::default().fg(C_MUT)),
        ]),
    ];
    if !j.file_scope.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("   scope    ", Style::default().fg(C_MUT)),
            Span::styled(j.file_scope.join(", "), Style::default().fg(C_FG)),
        ]));
    }
    if !desc.is_empty() {
        lines.push(Line::from(Span::styled(format!("   {desc}"), Style::default().fg(C_MUT))));
    }
    let id_short = j.id.get(..12).unwrap_or(&j.id);
    f.render_widget(Paragraph::new(lines).block(panel(format!("Job · {id_short}"))), chunks[0]);

    if app.logs.is_empty() {
        f.render_widget(
            Paragraph::new("  No log entries.")
                .style(Style::default().fg(C_MUT).bg(C_BG))
                .block(panel("Log")),
            chunks[1],
        );
        return;
    }
    let log_lines: Vec<Line<'static>> = app
        .logs
        .iter()
        .map(|l| {
            let ts = l.created_at.get(11..16).unwrap_or("").to_string();
            let actor = l.actor.as_deref().unwrap_or("·").to_string();
            let msg = l.message.clone();
            Line::from(vec![
                Span::styled(format!(" {ts}  "), Style::default().fg(C_MUT)),
                Span::styled(format!("[{actor}]  "), Style::default().fg(C_PRI)),
                Span::styled(msg, Style::default().fg(C_FG)),
            ])
        })
        .collect();
    f.render_widget(
        Paragraph::new(log_lines)
            .block(panel("Log  (recent 20)"))
            .wrap(Wrap { trim: false })
            .scroll((app.log_scroll, 0)),
        chunks[1],
    );
}
