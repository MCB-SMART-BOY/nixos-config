use crate::tui::state::{AdvancedActionsListModel, DeployControlRow, DeployControlsModel};
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
        let low_height_apply = !workspace_visible && area.height <= 20;
        let stacked_apply = !workspace_visible && area.width <= 90;
        if stacked_apply {
            let preview_summary_height = if low_height_apply {
                8
            } else if area.height <= 24 {
                9
            } else {
                10
            };
            let row_constraints = if low_height_apply {
                [Constraint::Percentage(80), Constraint::Min(4)]
            } else {
                [Constraint::Length(15), Constraint::Min(8)]
            };
            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints(row_constraints)
                .split(area);
            let preview = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(preview_summary_height),
                    Constraint::Min(5),
                ])
                .split(rows[0]);
            let bottom = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                .split(rows[1]);

            return Self {
                preview_summary: preview[0],
                preview_main: preview[1],
                context: bottom[0],
                controls: bottom[1],
                workspace_actions: None,
                workspace_detail: None,
            };
        }

        let root_constraints = if workspace_visible {
            [Constraint::Percentage(62), Constraint::Percentage(38)]
        } else if low_height_apply {
            [Constraint::Percentage(68), Constraint::Percentage(32)]
        } else if area.width < 110 {
            [Constraint::Percentage(64), Constraint::Percentage(36)]
        } else {
            [Constraint::Percentage(62), Constraint::Percentage(38)]
        };
        let root = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(root_constraints)
            .split(area);
        let preview_summary_height = if low_height_apply {
            8
        } else if !workspace_visible && area.height <= 24 {
            9
        } else {
            10
        };
        let preview = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(preview_summary_height),
                Constraint::Min(11),
            ])
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
            let context_height = if low_height_apply {
                5
            } else if area.height <= 24 {
                7
            } else {
                9
            };
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(context_height), Constraint::Min(10)])
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
    let compact = area.height <= 12 || area.width <= 36;
    let rows = model
        .rows
        .iter()
        .map(|row| ListItem::new(format_deploy_control_row(row, compact)))
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
        .highlight_symbol(if compact { "> " } else { ">> " });
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn format_deploy_control_row(row: &DeployControlRow, compact: bool) -> String {
    if !compact {
        return format!("{:<14} {}", row.label, row.value);
    }

    format!(
        "{}: {}",
        compact_control_label(&row.label),
        compact_control_value(&row.value)
    )
}

fn compact_control_label(label: &str) -> &str {
    match label {
        "目标主机" => "主机",
        "固定 ref" => "ref",
        "flake update" => "升级",
        "区域切换" => "区域",
        other => other,
    }
}

fn compact_control_value(value: &str) -> String {
    match value {
        "Enter 进入 Advanced" => "Enter->Advanced".to_string(),
        "Enter 返回 Apply" => "Enter->Apply".to_string(),
        other => other.to_string(),
    }
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

    #[test]
    fn hidden_apply_layout_compacts_context_and_gives_preview_more_width_on_small_terminals() {
        let compact = DeployLayoutAreas::new(Rect::new(0, 0, 100, 24), false);

        assert_eq!(compact.preview_summary.height, 9);
        assert_eq!(compact.context.height, 7);
        assert!(compact.preview_main.width > compact.controls.width);
    }

    #[test]
    fn hidden_apply_layout_stacks_preview_above_context_when_terminal_is_too_narrow() {
        let stacked = DeployLayoutAreas::new(Rect::new(0, 0, 84, 24), false);

        assert_eq!(stacked.preview_summary.width, 84);
        assert_eq!(stacked.preview_main.width, 84);
        assert!(stacked.preview_main.y > stacked.preview_summary.y);
        assert_eq!(stacked.context.y, stacked.controls.y);
        assert!(stacked.context.y > stacked.preview_main.y);
        assert!(stacked.controls.width > stacked.context.width);
    }

    #[test]
    fn hidden_apply_layout_prioritizes_left_column_when_terminal_is_short() {
        let compact = DeployLayoutAreas::new(Rect::new(0, 0, 120, 20), false);

        assert_eq!(compact.preview_summary.height, 8);
        assert_eq!(compact.context.height, 5);
        assert!(compact.preview_main.width > compact.controls.width);
        assert!(compact.preview_summary.width > compact.context.width);
    }

    #[test]
    fn stacked_apply_layout_preserves_taller_preview_region_when_terminal_is_short() {
        let stacked = DeployLayoutAreas::new(Rect::new(0, 0, 84, 18), false);

        assert_eq!(stacked.preview_summary.height, 8);
        assert!(stacked.preview_main.height > stacked.context.height);
        assert!(stacked.preview_main.y > stacked.preview_summary.y);
        assert!(stacked.context.y > stacked.preview_main.y);
    }

    #[test]
    fn compact_control_row_shortens_known_apply_labels_and_area_switch_value() {
        let row = DeployControlRow {
            label: "区域切换".to_string(),
            value: "Enter 进入 Advanced".to_string(),
        };

        assert_eq!(
            format_deploy_control_row(&row, true),
            "区域: Enter->Advanced"
        );
    }
}
