#![allow(dead_code)]

use crate::tui::state::AppState;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};

pub(super) fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    let rows = state
        .action_display_rows()
        .into_iter()
        .map(|row| {
            let content = if row.selectable {
                format!("{}  {}", row.label, row.value)
            } else {
                row.label
            };
            ListItem::new(content)
        })
        .collect::<Vec<_>>();

    let mut list_state = ListState::default();
    list_state.select(Some(state.selected_action_row_index()));
    let list = List::new(rows)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Transition Actions"),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    frame.render_stateful_widget(list, chunks[0], &mut list_state);

    let mut lines = state.actions_summary_lines();
    lines.push(String::new());
    lines.push(format!("状态：{}", state.status));
    let summary = Paragraph::new(lines.join("\n"))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Transition Summary"),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(summary, chunks[1]);
}
