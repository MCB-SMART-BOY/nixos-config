use crate::tui::state::{
    AppState, InspectCommandDetailModel, InspectHealthFocus, InspectModel, ManagedGuardSnapshot,
    OverviewCheckState,
};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};

pub(super) fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let inspect = state.inspect_model();
    render_with_model(frame, area, &inspect);
}

fn render_with_model(frame: &mut Frame, area: Rect, inspect: &InspectModel) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(38), Constraint::Percentage(62)])
        .split(area);

    let rows = inspect
        .commands
        .iter()
        .map(|command| {
            let status = if command.available {
                "可执行"
            } else {
                "需切换场景"
            };
            ListItem::new(format!("{} / {}  {}", command.group, command.label, status))
        })
        .collect::<Vec<_>>();
    let mut list_state = ListState::default();
    list_state.select(Some(inspect.selected_index));
    let list = List::new(rows)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Inspect Commands"),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    frame.render_stateful_widget(list, chunks[0], &mut list_state);

    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(16), Constraint::Min(9)])
        .split(chunks[1]);
    frame.render_widget(
        Paragraph::new(render_health_detail_lines(inspect))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Health Details"),
            )
            .wrap(Wrap { trim: false }),
        right[0],
    );
    frame.render_widget(
        Paragraph::new(render_command_detail_lines(&inspect.detail))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Command Detail"),
            )
            .wrap(Wrap { trim: false }),
        right[1],
    );
}

fn render_health_detail_lines(inspect: &InspectModel) -> String {
    let mut lines = Vec::new();
    match inspect.health_focus {
        InspectHealthFocus::RepoIntegrity => {
            lines.extend(render_check_lines(
                "repo-integrity",
                &inspect.repo_integrity,
            ));
            lines.extend(render_check_lines("doctor", &inspect.doctor));
        }
        InspectHealthFocus::Doctor => {
            lines.extend(render_check_lines("doctor", &inspect.doctor));
            lines.extend(render_check_lines(
                "repo-integrity",
                &inspect.repo_integrity,
            ));
        }
    }
    lines.extend(render_managed_guard_lines(&inspect.managed_guards, true));
    lines.join("\n")
}

fn render_command_detail_lines(detail: &InspectCommandDetailModel) -> String {
    [
        format!("当前命令：{}", detail.label),
        format!("分组：{}", detail.group),
        format!(
            "状态：{}",
            if detail.available {
                "可执行"
            } else {
                "需切换场景"
            }
        ),
        format!(
            "命令预览：{}",
            detail.preview.as_deref().unwrap_or("无预览")
        ),
        format!("最近结果：{}", detail.latest_result),
        format!("当前页：{}", detail.page_title),
        "操作：j/k 选择命令  r/d/R 刷新健康项  x 直接执行当前 inspect 命令".to_string(),
    ]
    .join("\n")
}

fn render_check_lines(label: &str, state: &OverviewCheckState) -> Vec<String> {
    let mut lines = vec![format!("{label}: {}", state.summary_label())];
    for detail in state.detail_lines() {
        lines.push(format!("  - {detail}"));
    }
    lines
}

fn render_managed_guard_lines(
    guards: &[ManagedGuardSnapshot],
    include_details: bool,
) -> Vec<String> {
    let blocked = guards
        .iter()
        .filter(|guard| guard.available && !guard.errors.is_empty())
        .count();
    let mut lines = vec![if blocked == 0 {
        "save-guards: ok".to_string()
    } else {
        format!("save-guards: {blocked} blocked target(s)")
    }];

    for guard in guards {
        let status = if !guard.available {
            "无可用目标".to_string()
        } else if guard.errors.is_empty() {
            "ok".to_string()
        } else {
            format!("failed ({} issue(s))", guard.errors.len())
        };
        lines.push(format!("  - {}[{}]: {status}", guard.page, guard.target));
        if include_details {
            for error in &guard.errors {
                lines.push(format!("    * {error}"));
            }
        }
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::tui::ActionItem;
    use crate::tui::state::{
        ManagedGuardSnapshot, OverviewCheckState, UiFeedback, UiFeedbackLevel, UiFeedbackScope,
    };
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::buffer::Buffer;

    #[test]
    fn health_detail_lines_include_repo_and_doctor_sections() {
        let inspect = test_inspect_model();

        let text = render_health_detail_lines(&inspect);

        assert!(text.contains("repo-integrity: failed (1 finding(s))"));
        assert!(text.contains("doctor: ok with 1 warning(s)"));
        assert!(text.contains("缺少 cargo"));
        assert!(text.contains("save-guards: 1 blocked target(s)"));
        assert!(text.contains("Packages[alice]: failed (1 issue(s))"));
    }

    #[test]
    fn command_detail_lines_show_preview_and_latest_result() {
        let inspect = test_inspect_model();

        let text = render_command_detail_lines(&inspect.detail);

        assert!(text.contains("当前命令：flake check"));
        assert!(text.contains("命令预览：nix flake check path:/repo"));
        assert!(text.contains("最近结果：flake check 已完成。"));
        assert!(text.contains("当前页：Inspect"));
    }

    fn test_inspect_model() -> InspectModel {
        InspectModel {
            health_focus: InspectHealthFocus::RepoIntegrity,
            repo_integrity: OverviewCheckState::Error {
                summary: "failed (1 finding(s))".to_string(),
                details: vec!["- [rule] path: detail".to_string()],
            },
            doctor: OverviewCheckState::Healthy {
                summary: "ok with 1 warning(s)".to_string(),
                details: vec!["缺少 cargo".to_string()],
            },
            managed_guards: vec![
                ManagedGuardSnapshot {
                    page: "Packages",
                    target: "alice".to_string(),
                    available: true,
                    errors: vec!["manual group blocks save".to_string()],
                },
                ManagedGuardSnapshot {
                    page: "Home",
                    target: "alice".to_string(),
                    available: true,
                    errors: Vec::new(),
                },
            ],
            commands: vec![
                crate::tui::state::InspectCommandModel {
                    action: ActionItem::FlakeCheck,
                    group: "Repo Checks",
                    label: "flake check",
                    available: true,
                    preview: Some("nix flake check path:/repo".to_string()),
                },
                crate::tui::state::InspectCommandModel {
                    action: ActionItem::UpdateUpstreamCheck,
                    group: "Upstream Pins",
                    label: "check upstream pins",
                    available: true,
                    preview: Some("update-upstream-apps --check".to_string()),
                },
            ],
            selected_index: 0,
            detail: crate::tui::state::InspectCommandDetailModel {
                action: ActionItem::FlakeCheck,
                group: "Repo Checks",
                label: "flake check",
                available: true,
                preview: Some("nix flake check path:/repo".to_string()),
                latest_result: "flake check 已完成。".to_string(),
                page_title: "Inspect",
            },
            latest_result: Some(UiFeedback::new(
                UiFeedbackLevel::Success,
                UiFeedbackScope::Inspect,
                "flake check 已完成。",
                None,
            )),
        }
    }

    #[test]
    fn command_detail_lines_follow_selected_inspect_command() {
        let mut inspect = test_inspect_model();
        inspect.selected_index = 1;
        inspect.detail = crate::tui::state::InspectCommandDetailModel {
            action: ActionItem::UpdateUpstreamCheck,
            group: "Upstream Pins",
            label: "check upstream pins",
            available: true,
            preview: Some("update-upstream-apps --check".to_string()),
            latest_result: "check upstream pins 已完成。".to_string(),
            page_title: "Inspect",
        };

        let text = render_command_detail_lines(&inspect.detail);

        assert!(text.contains("当前命令：check upstream pins"));
        assert!(text.contains("分组：Upstream Pins"));
        assert!(text.contains("命令预览：update-upstream-apps --check"));
        assert!(text.contains("最近结果：check upstream pins 已完成。"));
    }

    #[test]
    fn render_inspect_page_surfaces_repo_integrity_failure_and_flake_check_detail() {
        let inspect = test_inspect_model();

        let text = render_view_text(120, 40, |frame| {
            render_with_model(frame, Rect::new(0, 0, 120, 40), &inspect)
        });

        assert!(text.contains("Inspect Commands"));
        assert!(text.contains("Health Details"));
        assert!(text.contains("Command Detail"));
        assert!(text.contains("repo-integrity: failed (1 finding(s))"));
        assert!(text.contains("doctor: ok with 1 warning(s)"));
        assert!(text.contains("save-guards: 1 blocked target(s)"));
        assert!(text.contains("flake check"));
        assert!(text.contains("nix flake check path:/repo"));
    }

    #[test]
    fn render_inspect_page_keeps_doctor_failure_and_flake_check_detail_aligned() {
        let mut inspect = test_inspect_model();
        inspect.health_focus = InspectHealthFocus::Doctor;
        inspect.repo_integrity = OverviewCheckState::Healthy {
            summary: "ok".to_string(),
            details: Vec::new(),
        };
        inspect.doctor = OverviewCheckState::Error {
            summary: "failed (1 check(s))".to_string(),
            details: vec!["缺少 nixos-rebuild".to_string()],
        };

        let text = render_view_text(120, 40, |frame| {
            render_with_model(frame, Rect::new(0, 0, 120, 40), &inspect)
        });

        assert!(text.contains("repo-integrity: ok"));
        assert!(text.contains("doctor: failed (1 check(s))"));
        assert!(text.contains("flake check"));
        assert!(text.contains("nix flake check path:/repo"));
        assert!(text.contains("check upstream pins"));
        let doctor_pos = text
            .find("doctor: failed (1 check(s))")
            .expect("doctor section should render");
        let repo_pos = text
            .find("repo-integrity: ok")
            .expect("repo section should render");
        assert!(doctor_pos < repo_pos);
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
