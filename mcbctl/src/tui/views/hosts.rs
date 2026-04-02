use crate::tui::state::AppState;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};

pub(super) fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(44), Constraint::Percentage(56)])
        .split(area);

    let rows = state
        .hosts_rows()
        .into_iter()
        .map(|(label, value)| ListItem::new(format!("{label:<16} {value}")))
        .collect::<Vec<_>>();

    let mut list_state = ListState::default();
    list_state.select(Some(state.hosts_focus));
    let list = List::new(rows)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Host Override"),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    frame.render_stateful_widget(list, chunks[0], &mut list_state);

    let summary = Paragraph::new(state.hosts_summary_lines().join("\n"))
        .block(Block::default().borders(Borders::ALL).title("Host Summary"))
        .wrap(Wrap { trim: false });
    frame.render_widget(summary, chunks[1]);
}
