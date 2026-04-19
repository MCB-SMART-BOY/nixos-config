use super::health::render_compact_health_summary;
use super::summary::render_mainline_summary;
use crate::tui::state::{
    AppState, InspectCommandDetailModel, InspectHealthFocus, InspectModel, InspectSummaryModel,
};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};

struct InspectLayoutAreas {
    commands: Rect,
    summary: Rect,
    health: Rect,
    detail: Rect,
}

impl InspectLayoutAreas {
    fn new(area: Rect) -> Self {
        let low_height = area.height <= 20;
        let compact_height = area.height <= 24;
        let narrow = area.width <= 90;
        let root = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(if low_height {
                [Constraint::Percentage(34), Constraint::Percentage(66)]
            } else if narrow {
                [Constraint::Percentage(36), Constraint::Percentage(64)]
            } else {
                [Constraint::Percentage(38), Constraint::Percentage(62)]
            })
            .split(area);
        let right = Layout::default()
            .direction(Direction::Vertical)
            .constraints(if low_height {
                [
                    Constraint::Length(7),
                    Constraint::Length(9),
                    Constraint::Min(4),
                ]
            } else if compact_height {
                [
                    Constraint::Length(7),
                    Constraint::Length(11),
                    Constraint::Min(6),
                ]
            } else {
                [
                    Constraint::Length(8),
                    Constraint::Length(13),
                    Constraint::Min(8),
                ]
            })
            .split(root[1]);

        Self {
            commands: root[0],
            summary: right[0],
            health: right[1],
            detail: right[2],
        }
    }
}

pub(super) fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let inspect = state.inspect_model();
    render_with_model(frame, area, &inspect);
}

fn render_with_model(frame: &mut Frame, area: Rect, inspect: &InspectModel) {
    let layout = InspectLayoutAreas::new(area);
    let compact_rows = layout.commands.width <= 34;

    let rows = inspect
        .commands
        .iter()
        .map(|command| ListItem::new(format_inspect_command_row(command, compact_rows)))
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
        .highlight_symbol(if compact_rows { "> " } else { ">> " });
    frame.render_stateful_widget(list, layout.commands, &mut list_state);

    frame.render_widget(
        Paragraph::new(render_summary_lines(&inspect.summary))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Inspect Summary"),
            )
            .wrap(Wrap { trim: false }),
        layout.summary,
    );
    frame.render_widget(
        Paragraph::new(render_health_detail_lines(inspect))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Health Details"),
            )
            .wrap(Wrap { trim: false }),
        layout.health,
    );
    frame.render_widget(
        Paragraph::new(render_command_detail_lines(&inspect.detail))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Command Detail"),
            )
            .wrap(Wrap { trim: false }),
        layout.detail,
    );
}

fn render_summary_lines(summary: &InspectSummaryModel) -> String {
    render_mainline_summary(
        &compact_inspect_status(&summary.status),
        &compact_inspect_latest_result(&summary.latest_result),
        &compact_inspect_next_step(&summary.next_step),
        &compact_inspect_primary_action(&summary.primary_action),
        &[],
    )
}

fn render_health_detail_lines(inspect: &InspectModel) -> String {
    match inspect.health_focus {
        InspectHealthFocus::RepoIntegrity => render_compact_health_summary(
            "repo-integrity",
            &inspect.repo_integrity,
            "doctor",
            &inspect.doctor,
            &inspect.managed_guards,
        ),
        InspectHealthFocus::Doctor => render_compact_health_summary(
            "doctor",
            &inspect.doctor,
            "repo-integrity",
            &inspect.repo_integrity,
            &inspect.managed_guards,
        ),
    }
}

fn render_command_detail_lines(detail: &InspectCommandDetailModel) -> String {
    [
        format!("命令：{}", detail.label),
        format!(
            "状态：{}",
            if detail.available {
                "可执行"
            } else {
                "切场景"
            }
        ),
        format!(
            "预览：{}",
            compact_inspect_command_preview(detail.preview.as_deref().unwrap_or("无预览"))
        ),
        format!("分组：{}", detail.group),
        if detail.available {
            "动作：x 执行  r/d/R 刷新健康".to_string()
        } else {
            "动作：先切场景  r/d/R 刷新健康".to_string()
        },
    ]
    .join("\n")
}

fn format_inspect_command_row(
    command: &crate::tui::state::InspectCommandModel,
    compact: bool,
) -> String {
    let status = if command.available {
        "可执行"
    } else if compact {
        "切场景"
    } else {
        "需切换场景"
    };

    if compact {
        return format!(
            "{}/{} {}",
            compact_inspect_group(command.group),
            command.label,
            status
        );
    }

    format!("{} / {}  {}", command.group, command.label, status)
}

fn compact_inspect_group(group: &str) -> &str {
    match group {
        "Repo Checks" => "Repo",
        "Upstream Pins" => "Pins",
        other => other,
    }
}

fn compact_inspect_command_preview(preview: &str) -> String {
    preview.replace(
        "nix --extra-experimental-features 'nix-command flakes' flake check ",
        "nix flake check ",
    )
}

fn compact_inspect_status(status: &str) -> String {
    match status {
        "当前应先复查 repo-integrity" => "当前应复查 repo-integrity".to_string(),
        "当前应先复查 doctor" => "当前应复查 doctor".to_string(),
        "当前可直接执行当前 Inspect 命令" => "当前可直接执行检查".to_string(),
        "当前命令需切换场景" => "当前需切换场景".to_string(),
        other => other.to_string(),
    }
}

fn compact_inspect_latest_result(latest_result: &str) -> String {
    if latest_result == "无" {
        return latest_result.to_string();
    }
    latest_result.trim_end_matches('。').to_string()
}

fn compact_inspect_next_step(next_step: &str) -> String {
    if next_step.starts_with("先看 repo-integrity 详情；如需复查，再按 x 执行 ") {
        return "先看 repo-integrity，需要时按 x".to_string();
    }
    if next_step.starts_with("先看 doctor 详情；如需复查，再按 x 执行 ") {
        return "先看 doctor，需要时按 x".to_string();
    }
    if next_step.starts_with("先看健康摘要；如需继续，按 x 执行 ") {
        return "先看健康摘要，需要时按 x".to_string();
    }
    if next_step.starts_with("先切换到适合 ") {
        return "先切场景，再回 Inspect".to_string();
    }
    if next_step == "切到 Inspect 查看检查结果" {
        return "留在 Inspect 复查".to_string();
    }
    if next_step == "切到 Inspect 查看 pin 状态" {
        return "留在 Inspect 看 pin".to_string();
    }
    next_step.to_string()
}

fn compact_inspect_primary_action(primary_action: &str) -> String {
    let Some(value) = primary_action.strip_prefix("主动作：") else {
        return primary_action.to_string();
    };

    let compacted = if value == "先看健康详情，再决定是否执行当前检查" {
        "先看健康详情".to_string()
    } else if value == "先切换到适合当前命令的场景" {
        "先切换场景".to_string()
    } else if value.starts_with("按 x 执行 ") {
        "按 x 执行".to_string()
    } else {
        value.to_string()
    };

    format!("主动作：{compacted}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::tui::ActionItem;
    use crate::tui::state::{
        InspectSummaryModel, ManagedGuardSnapshot, OverviewCheckState, UiFeedback, UiFeedbackLevel,
        UiFeedbackScope,
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
        assert!(text.contains("优先项：- [rule] path: detail"));
        assert!(!text.contains("缺少 cargo"));
        assert!(text.contains("save-guards: Packages[alice] blocked"));
        assert!(text.contains("优先处理：manual group blocks save"));
    }

    #[test]
    fn health_detail_lines_expand_only_active_health_focus() {
        let mut inspect = test_inspect_model();
        inspect.health_focus = InspectHealthFocus::Doctor;
        inspect.repo_integrity = OverviewCheckState::Healthy {
            summary: "ok".to_string(),
            details: Vec::new(),
        };
        inspect.doctor = OverviewCheckState::Error {
            summary: "failed (2 check(s))".to_string(),
            details: vec!["缺少 nixos-rebuild".to_string(), "缺少 git".to_string()],
        };

        let text = render_health_detail_lines(&inspect);

        assert!(text.contains("doctor: failed (2 check(s))"));
        assert!(text.contains("优先项：缺少 nixos-rebuild"));
        assert!(text.contains("其余：另 1 项"));
        assert!(text.contains("repo-integrity: ok"));
    }

    #[test]
    fn summary_lines_show_judgement_result_and_next_step() {
        let inspect = test_inspect_model();

        let text = render_summary_lines(&inspect.summary);

        let status_pos = text
            .find("当前判断：当前应复查 repo-integrity")
            .expect("status should render");
        let result_pos = text
            .find("最近结果：flake check 已完成")
            .expect("latest result should render");
        let next_step_pos = text
            .find("下一步：先看 repo-integrity，需要时按 x")
            .expect("next step should render");
        let primary_pos = text
            .find("主动作：先看健康详情")
            .expect("primary action should render");

        assert!(status_pos < result_pos);
        assert!(result_pos < next_step_pos);
        assert!(next_step_pos < primary_pos);
        assert!(text.contains("当前判断：当前应复查 repo-integrity"));
        assert!(text.contains("最近结果：flake check 已完成"));
        assert!(text.contains("下一步：先看 repo-integrity，需要时按 x"));
        assert!(text.contains("主动作：先看健康详情"));
    }

    #[test]
    fn command_detail_lines_show_preview_after_summary_shell() {
        let inspect = test_inspect_model();

        let text = render_command_detail_lines(&inspect.detail);

        assert!(text.contains("命令：flake check"));
        assert!(text.contains("状态：可执行"));
        let preview_pos = text
            .find("预览：nix flake check path:/repo")
            .expect("preview should render");
        let status_pos = text.find("状态：可执行").expect("status should render");
        assert!(status_pos < preview_pos);
        assert!(text.contains("预览：nix flake check path:/repo"));
        assert!(text.contains("动作：x 执行  r/d/R 刷新健康"));
        assert!(!text.contains("最近结果："));
        assert!(!text.contains("操作：j/k 选择命令"));
    }

    #[test]
    fn inspect_command_rows_compact_group_and_status_on_narrow_lists() {
        let command = &test_inspect_model().commands[1];

        let text = format_inspect_command_row(command, true);

        assert_eq!(text, "Pins/check upstream pins 可执行");
    }

    #[test]
    fn inspect_layout_prioritizes_right_column_and_health_on_short_terminals() {
        let layout = InspectLayoutAreas::new(Rect::new(0, 0, 120, 20));

        assert_eq!(layout.summary.height, 7);
        assert_eq!(layout.health.height, 9);
        assert!(layout.detail.height >= 4);
        assert!(layout.summary.width > layout.commands.width);
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
            summary: InspectSummaryModel {
                status: "当前应先复查 repo-integrity".to_string(),
                latest_result: "flake check 已完成。".to_string(),
                next_step: "先看 repo-integrity 详情；如需复查，再按 x 执行 flake check。"
                    .to_string(),
                primary_action: "主动作：先看健康详情，再决定是否执行当前检查".to_string(),
            },
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
        inspect.summary = InspectSummaryModel {
            status: "当前应先复查 repo-integrity".to_string(),
            latest_result: "check upstream pins 已完成。".to_string(),
            next_step: "先看 repo-integrity 详情；如需复查，再按 x 执行 check upstream pins。"
                .to_string(),
            primary_action: "主动作：先看健康详情，再决定是否执行当前检查".to_string(),
        };
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

        assert!(text.contains("命令：check upstream pins"));
        assert!(text.contains("分组：Upstream Pins"));
        assert!(text.contains("预览：update-upstream-apps --check"));
        assert!(text.contains("状态：可执行"));
    }

    #[test]
    fn render_inspect_page_surfaces_repo_integrity_failure_and_flake_check_detail() {
        let inspect = test_inspect_model();

        let text = render_view_text(120, 40, |frame| {
            render_with_model(frame, Rect::new(0, 0, 120, 40), &inspect)
        });

        assert!(text.contains("Inspect Commands"));
        assert!(text.contains("Inspect Summary"));
        assert!(text.contains("Health Details"));
        assert!(text.contains("Command Detail"));
        assert!(text.contains("repo-integrity: failed (1 finding(s))"));
        assert!(text.contains("doctor: ok with 1 warning(s)"));
        assert!(text.contains("save-guards: Packages[alice] blocked"));
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
        inspect.summary = InspectSummaryModel {
            status: "当前应先复查 doctor".to_string(),
            latest_result: "flake check 已完成。".to_string(),
            next_step: "先看 doctor 详情；如需复查，再按 x 执行 flake check。".to_string(),
            primary_action: "主动作：先看健康详情，再决定是否执行当前检查".to_string(),
        };

        let text = render_view_text(120, 40, |frame| {
            render_with_model(frame, Rect::new(0, 0, 120, 40), &inspect)
        });

        assert!(text.contains("Inspect Summary"));
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

    #[test]
    fn render_inspect_page_keeps_summary_and_health_visible_in_short_body_area() {
        let inspect = test_inspect_model();

        let text = render_view_text(120, 20, |frame| {
            render_with_model(frame, Rect::new(0, 0, 120, 20), &inspect)
        });

        assert!(text.contains("Inspect Commands"));
        assert!(text.contains("Inspect Summary"));
        assert!(text.contains("Health Details"));
        assert!(text.contains("Command Detail"));
        assert!(text.contains("repo-integrity: failed (1 finding(s))"));
        assert!(text.contains("doctor: ok with 1 warning(s)"));
        assert!(text.contains("save-guards: Packages[alice] blocked"));
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
