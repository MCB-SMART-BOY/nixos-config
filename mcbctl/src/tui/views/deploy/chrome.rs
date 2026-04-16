use crate::tui::state::{AdvancedActionsListModel, DeployControlsModel};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};

pub(super) struct DeployLayoutAreas {
    pub(super) preview_summary: Rect,
    pub(super) preview_main: Rect,
    pub(super) context: Rect,
    pub(super) controls: Rect,
    pub(super) workspace_actions: Option<Rect>,
    pub(super) workspace_detail: Option<Rect>,
}

impl DeployLayoutAreas {
    pub(super) fn new(area: Rect, workspace_visible: bool) -> Self {
        let root = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
            .split(area);
        let preview = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(10), Constraint::Min(12)])
            .split(root[0]);
        let controls = if workspace_visible {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(7),
                    Constraint::Length(9),
                    Constraint::Min(12),
                ])
                .split(root[1])
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(9), Constraint::Min(10)])
                .split(root[1])
        };
        let (workspace_actions, workspace_detail) = if workspace_visible {
            let workspace = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(6), Constraint::Min(7)])
                .split(controls[2]);
            (Some(workspace[0]), Some(workspace[1]))
        } else {
            (None, None)
        };

        Self {
            preview_summary: preview[0],
            preview_main: preview[1],
            context: controls[0],
            controls: controls[1],
            workspace_actions,
            workspace_detail,
        }
    }
}

pub(super) fn render_workspace_section(
    frame: &mut Frame,
    layout: &DeployLayoutAreas,
    advanced_actions: Option<&AdvancedActionsListModel>,
    detail_lines: Option<String>,
    detail_title: &str,
) {
    let Some(actions_area) = layout.workspace_actions else {
        return;
    };
    let Some(detail_area) = layout.workspace_detail else {
        return;
    };

    render_advanced_actions_list(
        frame,
        actions_area,
        advanced_actions.expect("advanced actions should exist when workspace is visible"),
    );

    frame.render_widget(
        Paragraph::new(
            detail_lines.expect("advanced workspace detail should exist when workspace is visible"),
        )
        .block(Block::default().borders(Borders::ALL).title(detail_title))
        .wrap(Wrap { trim: false }),
        detail_area,
    );
}

pub(super) fn render_deploy_controls_list(
    frame: &mut Frame,
    area: Rect,
    model: &DeployControlsModel,
    title: &str,
) {
    let rows = model
        .rows
        .iter()
        .map(|row| ListItem::new(format!("{:<14} {}", row.label, row.value)))
        .collect::<Vec<_>>();
    let mut list_state = ListState::default();
    list_state.select(Some(model.selected_focus));
    let list = List::new(rows)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_advanced_actions_list(frame: &mut Frame, area: Rect, model: &AdvancedActionsListModel) {
    let rows = model
        .rows
        .iter()
        .map(|row| {
            if row.selectable {
                ListItem::new(format!("{:<24} {}", row.label, row.value))
            } else {
                ListItem::new(row.label.clone())
            }
        })
        .collect::<Vec<_>>();
    let mut list_state = ListState::default();
    list_state.select(Some(model.selected_index));
    let list = List::new(rows)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Advanced Actions"),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    frame.render_stateful_widget(list, area, &mut list_state);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deploy_layout_areas_only_allocate_workspace_when_visible() {
        let hidden = DeployLayoutAreas::new(Rect::new(0, 0, 120, 40), false);
        let visible = DeployLayoutAreas::new(Rect::new(0, 0, 120, 40), true);

        assert!(hidden.workspace_actions.is_none());
        assert!(hidden.workspace_detail.is_none());
        assert!(visible.workspace_actions.is_some());
        assert!(visible.workspace_detail.is_some());
    }
}
