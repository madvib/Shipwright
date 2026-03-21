use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState, Paragraph},
};

use crate::view::App;
use super::{C_BG, C_FG, C_MUT, C_PRI, C_SEL, C_RED, panel};

pub fn draw_mcp(f: &mut Frame, app: &App, area: Rect) {
    if app.mcp_servers.is_empty() {
        f.render_widget(
            Paragraph::new("  No MCP servers configured.")
                .style(Style::default().fg(C_MUT).bg(C_BG))
                .block(panel("MCP Servers")),
            area,
        );
        return;
    }
    let items: Vec<ListItem> = app
        .mcp_servers
        .iter()
        .map(|s| {
            let transport = if s.url.is_some() { "http" } else { "stdio" };
            let endpoint = s.url.as_deref()
                .or(s.command.as_deref())
                .unwrap_or("—");
            let disabled_label = if s.disabled { " (disabled)" } else { "" };
            let name_color = if s.disabled { C_MUT } else { C_FG };
            let disabled_color = if s.disabled { C_RED } else { C_MUT };
            let line = Line::from(vec![
                Span::styled(format!(" {:<20}", s.id), Style::default().fg(name_color)),
                Span::styled(format!("  {:<6}", transport), Style::default().fg(C_MUT)),
                Span::styled(format!("  {endpoint}"), Style::default().fg(C_MUT)),
                Span::styled(disabled_label.to_string(), Style::default().fg(disabled_color)),
            ]);
            ListItem::new(line)
        })
        .collect();
    let mut state = ListState::default();
    state.select(Some(app.sel_mcp));
    f.render_stateful_widget(
        List::new(items)
            .block(panel(format!("MCP Servers  ({})", app.mcp_servers.len())))
            .highlight_style(Style::default().bg(C_SEL))
            .highlight_symbol("▶ "),
        area,
        &mut state,
    );
}

pub fn draw_mcp_detail(f: &mut Frame, app: &App, area: Rect) {
    let Some(s) = app.mcp_servers.get(app.sel_mcp) else { return };
    let transport = if s.url.is_some() { "http" } else { "stdio" };
    let name = s.name.as_deref().unwrap_or(&s.id);
    let mut lines = vec![
        Line::from(Span::styled(
            format!(" {name}"),
            Style::default().fg(C_FG).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled("", Style::default())),
        Line::from(vec![
            Span::styled("   id        ", Style::default().fg(C_MUT)),
            Span::styled(s.id.clone(), Style::default().fg(C_FG)),
        ]),
        Line::from(vec![
            Span::styled("   type      ", Style::default().fg(C_MUT)),
            Span::styled(transport.to_string(), Style::default().fg(C_FG)),
        ]),
        Line::from(vec![
            Span::styled("   scope     ", Style::default().fg(C_MUT)),
            Span::styled(s.scope.clone(), Style::default().fg(C_FG)),
        ]),
    ];
    if let Some(ref cmd) = s.command {
        let full = if s.args.is_empty() {
            cmd.clone()
        } else {
            format!("{} {}", cmd, s.args.join(" "))
        };
        lines.push(Line::from(vec![
            Span::styled("   command   ", Style::default().fg(C_MUT)),
            Span::styled(full, Style::default().fg(C_FG)),
        ]));
    }
    if let Some(ref url) = s.url {
        lines.push(Line::from(vec![
            Span::styled("   url       ", Style::default().fg(C_MUT)),
            Span::styled(url.clone(), Style::default().fg(C_FG)),
        ]));
    }
    if s.disabled {
        lines.push(Line::from(vec![
            Span::styled("   status    ", Style::default().fg(C_MUT)),
            Span::styled("disabled", Style::default().fg(C_RED)),
        ]));
    }
    if !s.env.is_empty() {
        lines.push(Line::from(Span::styled("", Style::default())));
        lines.push(Line::from(Span::styled(
            "   Environment",
            Style::default().fg(C_PRI).add_modifier(Modifier::BOLD),
        )));
        for (k, v) in &s.env {
            let display_v = if k.to_lowercase().contains("key")
                || k.to_lowercase().contains("secret")
                || k.to_lowercase().contains("token")
            {
                "********".to_string()
            } else {
                v.clone()
            };
            lines.push(Line::from(vec![
                Span::styled(format!("   {k}="), Style::default().fg(C_MUT)),
                Span::styled(display_v, Style::default().fg(C_FG)),
            ]));
        }
    }
    f.render_widget(
        Paragraph::new(lines).block(panel(format!("MCP · {}", s.id))),
        area,
    );
}
