use crate::tui::state::{AppState, PackagePageModel};
use ratatui::Frame;
use ratatui::layout::Rect;

use super::{PackagePageConfig, render_package_page};

pub(super) fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let page_model = state.package_page_model();
    render_with_model(frame, area, &page_model);
}

fn render_with_model(frame: &mut Frame, area: Rect, page_model: &PackagePageModel) {
    render_package_page(
        frame,
        area,
        PackagePageConfig {
            summary_percentage: 28,
            list_percentage: 39,
            summary_title: "Packages Summary",
            selection_title: "Selection",
        },
        page_model,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::state::{
        EditCheckModel, EditDetailModel, EditSummaryModel, PackageListModel, PackageSelectionModel,
    };
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::buffer::Buffer;

    #[test]
    fn package_group_row_display_marks_filter_and_current_entry() {
        let line = crate::tui::state::PackageGroupOverviewRow {
            group_label: "misc".to_string(),
            count: 3,
            filter_selected: true,
            current_selected: true,
        }
        .display_line();

        assert_eq!(line, ">* misc (3)");
    }

    #[test]
    fn package_list_item_display_marks_selected_entries() {
        let line = crate::tui::state::PackageListItemModel {
            selected: true,
            name: "Hello".to_string(),
            category: "cli".to_string(),
            group_label: "misc".to_string(),
        }
        .display_line();

        assert_eq!(line, "[x] Hello (cli, -> misc)");
    }

    #[test]
    fn render_with_model_shows_selection_title() {
        let page_model = PackagePageModel {
            summary: EditSummaryModel {
                header_lines: vec!["数据源：本地覆盖/已声明".to_string()],
                focused_row: None,
                field_lines: vec!["当前用户：alice".to_string()],
                detail: EditDetailModel {
                    status: "状态：当前用户没有未保存修改".to_string(),
                    action_summary: None,
                    validation: None,
                    managed_guard: EditCheckModel {
                        summary: "受管保护：通过".to_string(),
                        details: Vec::new(),
                    },
                    notes: Vec::new(),
                },
            },
            list: PackageListModel {
                title: "Packages (local)".to_string(),
                empty_text: None,
                items: vec![crate::tui::state::PackageListItemModel {
                    selected: true,
                    name: "Hello".to_string(),
                    category: "cli".to_string(),
                    group_label: "misc".to_string(),
                }],
                selected_index: Some(0),
            },
            selection: PackageSelectionModel {
                current_entry_fields: vec![crate::tui::state::EditRow {
                    label: "当前条目".to_string(),
                    value: "Hello".to_string(),
                }],
                group_rows: vec![crate::tui::state::PackageGroupOverviewRow {
                    group_label: "misc".to_string(),
                    count: 1,
                    filter_selected: true,
                    current_selected: true,
                }],
                workflow_rows: vec![crate::tui::state::PackageWorkflowOverviewRow {
                    workflow_label: "容器与集群 [containers]".to_string(),
                    total_count: 1,
                    selected_count: 1,
                    filter_selected: true,
                    current_selected: true,
                }],
                active_workflow: None,
                action_summary: None,
                selected_rows: vec![crate::tui::state::PackageSelectedEntryRow {
                    name: "Hello".to_string(),
                    category: "cli".to_string(),
                    group_label: "misc".to_string(),
                }],
                status: "ready".to_string(),
            },
        };

        let text = render_view_text(120, 24, |frame| {
            render_with_model(frame, Rect::new(0, 0, 120, 24), &page_model)
        });

        assert!(text.contains("Packages Summary"));
        assert!(text.contains("Selection"));
    }

    #[test]
    fn render_with_model_keeps_titles_visible_on_narrow_width() {
        let page_model = PackagePageModel {
            summary: EditSummaryModel {
                header_lines: vec!["数据源：本地覆盖/已声明".to_string()],
                focused_row: None,
                field_lines: vec!["当前用户：alice".to_string()],
                detail: EditDetailModel {
                    status: "状态：当前用户没有未保存修改".to_string(),
                    action_summary: None,
                    validation: None,
                    managed_guard: EditCheckModel {
                        summary: "受管保护：通过".to_string(),
                        details: Vec::new(),
                    },
                    notes: Vec::new(),
                },
            },
            list: PackageListModel {
                title: "Packages (本地覆盖/已声明)".to_string(),
                empty_text: None,
                items: vec![crate::tui::state::PackageListItemModel {
                    selected: true,
                    name: "Hello".to_string(),
                    category: "cli".to_string(),
                    group_label: "misc".to_string(),
                }],
                selected_index: Some(0),
            },
            selection: PackageSelectionModel {
                current_entry_fields: vec![crate::tui::state::EditRow {
                    label: "当前条目".to_string(),
                    value: "Hello".to_string(),
                }],
                group_rows: vec![crate::tui::state::PackageGroupOverviewRow {
                    group_label: "misc".to_string(),
                    count: 1,
                    filter_selected: true,
                    current_selected: true,
                }],
                workflow_rows: vec![crate::tui::state::PackageWorkflowOverviewRow {
                    workflow_label: "容器与集群 [containers]".to_string(),
                    total_count: 1,
                    selected_count: 1,
                    filter_selected: true,
                    current_selected: true,
                }],
                active_workflow: None,
                action_summary: None,
                selected_rows: vec![crate::tui::state::PackageSelectedEntryRow {
                    name: "Hello".to_string(),
                    category: "cli".to_string(),
                    group_label: "misc".to_string(),
                }],
                status: "ready".to_string(),
            },
        };

        let text = render_view_text(104, 24, |frame| {
            render_with_model(frame, Rect::new(0, 0, 104, 24), &page_model)
        });

        assert!(text.contains("Packages Summary"));
        assert!(text.contains("Selection"));
        assert!(text.contains("Hello"));
    }

    fn render_view_text(
        width: u16,
        height: u16,
        render: impl FnOnce(&mut ratatui::Frame<'_>),
    ) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
        terminal
            .draw(|frame| render(frame))
            .expect("test terminal draw should succeed");
        buffer_to_string(terminal.backend().buffer())
    }

    fn buffer_to_string(buffer: &Buffer) -> String {
        (0..buffer.area.height)
            .map(|y| {
                (0..buffer.area.width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}
