use crate::tui::state::{AppState, EditPageModel};
use ratatui::Frame;
use ratatui::layout::Rect;

use super::{EditPageConfig, render_edit_page_with_model};

pub(super) fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let page_model = state.hosts_page_model();
    render_with_model(frame, area, &page_model);
}

fn render_with_model(frame: &mut Frame, area: Rect, page_model: &EditPageModel) {
    render_edit_page_with_model(
        frame,
        area,
        EditPageConfig {
            left_percentage: 44,
            list_title: "Host Override".to_string(),
            summary_title: "Host Summary",
            label_width: 16,
        },
        page_model,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::state::{EditCheckModel, EditDetailModel, EditRow, EditSummaryModel};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::buffer::Buffer;

    #[test]
    fn render_with_model_shows_hosts_titles() {
        let page_model = EditPageModel {
            rows: vec![EditRow {
                label: "GPU 模式".to_string(),
                value: "igpu".to_string(),
            }],
            selected: 0,
            summary: EditSummaryModel {
                header_lines: vec!["当前主机：demo".to_string()],
                focused_row: None,
                field_lines: Vec::new(),
                detail: EditDetailModel {
                    status: "状态：当前主机的运行时分片没有未保存修改".to_string(),
                    validation: Some(EditCheckModel {
                        summary: "校验：通过".to_string(),
                        details: Vec::new(),
                    }),
                    managed_guard: EditCheckModel {
                        summary: "受管保护：通过".to_string(),
                        details: Vec::new(),
                    },
                    notes: Vec::new(),
                },
            },
        };

        let text = render_view_text(120, 24, |frame| {
            render_with_model(frame, Rect::new(0, 0, 120, 24), &page_model)
        });

        assert!(text.contains("Host Override"));
        assert!(text.contains("Host Summary"));
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
