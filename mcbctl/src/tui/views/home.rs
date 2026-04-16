use crate::tui::state::{AppState, EditPageModel};
use ratatui::Frame;
use ratatui::layout::Rect;

use super::{EditPageConfig, render_edit_page_with_model};

pub(super) fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let page_model = state.home_page_model();
    render_with_model(
        frame,
        area,
        &format!(
            "Home Settings ({})",
            state.current_home_user().unwrap_or("无可用用户")
        ),
        &page_model,
    );
}

fn render_with_model(frame: &mut Frame, area: Rect, list_title: &str, page_model: &EditPageModel) {
    render_edit_page_with_model(
        frame,
        area,
        EditPageConfig {
            left_percentage: 42,
            list_title: list_title.to_string(),
            summary_title: "Settings Preview",
            label_width: 20,
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
    fn render_with_model_shows_home_titles() {
        let page_model = EditPageModel {
            rows: vec![EditRow {
                label: "Noctalia".to_string(),
                value: "default".to_string(),
            }],
            selected: 0,
            summary: EditSummaryModel {
                header_lines: vec!["当前用户：alice".to_string()],
                focused_row: None,
                field_lines: vec!["Noctalia：default".to_string()],
                detail: EditDetailModel {
                    status: "状态：当前用户没有未保存修改".to_string(),
                    validation: None,
                    managed_guard: EditCheckModel {
                        summary: "受管保护：通过".to_string(),
                        details: Vec::new(),
                    },
                    notes: Vec::new(),
                },
            },
        };

        let text = render_view_text(120, 24, |frame| {
            render_with_model(
                frame,
                Rect::new(0, 0, 120, 24),
                "Home Settings (alice)",
                &page_model,
            )
        });

        assert!(text.contains("Home Settings (alice)"));
        assert!(text.contains("Settings Preview"));
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
