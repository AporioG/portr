use crate::app::{App, InputMode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
    Frame,
};

pub fn draw(frame: &mut Frame, app: &mut App) {
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(frame.area());

    render_header(frame, vertical_chunks[0], app);

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(64),
            Constraint::Percentage(36),
        ])
        .split(vertical_chunks[1]);

    render_table(frame, main_chunks[0], app);
    render_details(frame, main_chunks[1], app);
    render_footer(frame, vertical_chunks[2], app);
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let total = app.sockets.len();
    let tcp_count = app.sockets.iter().filter(|s| s.proto.as_str().starts_with("TCP")).count();
    let udp_count = app.sockets.iter().filter(|s| s.proto.as_str().starts_with("UDP")).count();
    let listeners = app.sockets.iter().filter(|s| s.state == "LISTEN" || s.state == "UNCONN").count();

    let text = format!(
        " portr | Total: {} | TCP: {} | UDP: {} | Listeners: {}",
        total, tcp_count, udp_count, listeners
    );

    let header_block = Block::default()
        .borders(Borders::ALL)
        .title(" Linux Kernel Network Auditor ")
        .style(Style::default().fg(Color::Cyan));

    frame.render_widget(Paragraph::new(text).block(header_block), area);
}

fn render_table(frame: &mut Frame, area: Rect, app: &mut App) {
    let rows: Vec<Row> = {
        let filtered = app.filtered_sockets();
        filtered
            .iter()
            .map(|s| {
                let pid_str = s.pid.map_or("-".to_string(), |p| p.to_string());
                let proc_str = s.process_name.clone().unwrap_or_else(|| "-".to_string());
                let srv_str = s.service.clone().unwrap_or_else(|| "-".to_string());

                let proto_color = match s.proto.as_str() {
                    "TCP" => Color::LightBlue,
                    "TCP6" => Color::Magenta,
                    "UDP" => Color::LightYellow,
                    "UDP6" => Color::Yellow,
                    _ => Color::White,
                };

                let state_color = match s.state.as_str() {
                    "LISTEN" | "UNCONN" => Color::Green,
                    "ESTABLISHED" | "ESTAB" => Color::Cyan,
                    "TIME_WAIT" => Color::DarkGray,
                    _ => Color::Yellow,
                };

                Row::new(vec![
                    Cell::from(s.proto.as_str().to_string()).style(Style::default().fg(proto_color)),
                    Cell::from(s.local.clone()),
                    Cell::from(srv_str).style(Style::default().fg(Color::LightCyan)),
                    Cell::from(s.state.clone()).style(Style::default().fg(state_color)),
                    Cell::from(pid_str),
                    Cell::from(proc_str),
                ])
            })
            .collect()
    };

    let header_cells = ["PROTO", "LOCAL ENDPOINT", "SERVICE", "STATE", "PID", "PROCESS"]
        .into_iter()
        .map(|h| Cell::from(h).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1);

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(9),
            Constraint::Percentage(36),
            Constraint::Percentage(13),
            Constraint::Percentage(16),
            Constraint::Percentage(8),
            Constraint::Percentage(18),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Sockets "))
    .row_highlight_style(
        Style::default()
            .bg(Color::Rgb(40, 44, 52))
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol(">> ");

    frame.render_stateful_widget(table, area, &mut app.table_state);
}

fn render_details(frame: &mut Frame, area: Rect, app: &App) {
    let filtered = app.filtered_sockets();
    let selected_entry = app
        .table_state
        .selected()
        .and_then(|idx| filtered.get(idx));

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Socket Details ")
        .style(Style::default().fg(Color::Magenta));

    let content = match selected_entry {
        Some(entry) => vec![
            Line::from(vec![
                Span::styled("Protocol: ", Style::default().fg(Color::Yellow)),
                Span::raw(entry.proto.as_str()),
                Span::raw(" | "),
                Span::styled("Service: ", Style::default().fg(Color::Yellow)),
                Span::raw(entry.service.as_deref().unwrap_or("unknown")),
            ]),
            Line::from(vec![
                Span::styled("Local: ", Style::default().fg(Color::Yellow)),
                Span::raw(&entry.local),
            ]),
            Line::from(vec![
                Span::styled("Remote: ", Style::default().fg(Color::Yellow)),
                Span::raw(&entry.remote),
            ]),
            Line::from(vec![
                Span::styled("State: ", Style::default().fg(Color::Yellow)),
                Span::raw(&entry.state),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("PID: ", Style::default().fg(Color::Green)),
                Span::raw(entry.pid.map_or("-".to_string(), |p| p.to_string())),
            ]),
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(Color::Green)),
                Span::raw(entry.process_name.as_deref().unwrap_or("-")),
            ]),
            Line::from(Span::styled("Binary Path: ", Style::default().fg(Color::Green))),
            Line::from(Span::styled(
                entry.exe.as_deref().unwrap_or("-"),
                Style::default().fg(Color::Gray),
            )),
            Line::from(""),
            Line::from(Span::styled("Command Line: ", Style::default().fg(Color::Cyan))),
            Line::from(Span::styled(
                entry.cmdline.as_deref().unwrap_or("-"),
                Style::default().fg(Color::White),
            )),
        ],
        None => vec![Line::from("No socket selected")],
    };

    let paragraph = Paragraph::new(content)
        .block(block)
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let (info_text, border_color) = match app.input_mode {
        InputMode::Normal => (
            format!(
                " [q] Quit | [j/k] Nav | [/] Filter | [t] SIGTERM | [K] SIGKILL | {}",
                app.status_message
            ),
            Color::White,
        ),
        InputMode::Filtering => (
            format!(" Search query: {}_ (Enter to apply, Esc to cancel)", app.filter),
            Color::Yellow,
        ),
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Controls ")
        .style(Style::default().fg(border_color));

    frame.render_widget(Paragraph::new(info_text).block(block), area);
}
