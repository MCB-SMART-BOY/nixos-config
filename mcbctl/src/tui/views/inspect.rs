use crate::tui::state::{AppState, InspectModel, ManagedGuardSnapshot, OverviewCheckState};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};

pub(super) fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let inspect = state.inspect_model();
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
    list_state.select(Some(state.selected_inspect_row_index()));
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
        Paragraph::new(render_health_detail_lines(&inspect))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Health Details"),
            )
            .wrap(Wrap { trim: false }),
        right[0],
    );
    frame.render_widget(
        Paragraph::new(render_command_detail_lines(state, &inspect))
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
    lines.extend(render_check_lines(
        "repo-integrity",
        &inspect.repo_integrity,
    ));
    lines.extend(render_check_lines("doctor", &inspect.doctor));
    lines.extend(render_managed_guard_lines(&inspect.managed_guards, true));
    lines.join("\n")
}

fn render_command_detail_lines(state: &AppState, inspect: &InspectModel) -> String {
    let command = &inspect.commands[state.selected_inspect_row_index()];
    let latest_result = inspect
        .latest_result
        .as_ref()
        .map(|feedback| feedback.legacy_status_text())
        .unwrap_or_else(|| "无".to_string());

    [
        format!("当前命令：{}", command.label),
        format!("分组：{}", command.group),
        format!(
            "状态：{}",
            if command.available {
                "可执行"
            } else {
                "需切换场景"
            }
        ),
        format!(
            "命令预览：{}",
            command.preview.as_deref().unwrap_or("无预览")
        ),
        format!("最近结果：{latest_result}"),
        format!("当前页：{}", state.page().title()),
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
    use crate::domain::tui::{ActionItem, DeployAction, DeploySource, DeployTask, Page};
    use crate::tui::state::{
        AppContext, AppState, ManagedGuardSnapshot, OverviewCheckState, UiFeedback,
        UiFeedbackLevel, UiFeedbackScope,
    };
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::PathBuf;

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
        let state = test_state();
        let inspect = test_inspect_model();

        let text = render_command_detail_lines(&state, &inspect);

        assert!(text.contains("当前命令：flake check"));
        assert!(text.contains("命令预览：nix flake check path:/repo"));
        assert!(text.contains("最近结果：flake check 已完成。"));
        assert!(text.contains("当前页：Inspect"));
    }

    fn test_inspect_model() -> InspectModel {
        InspectModel {
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
            latest_result: Some(UiFeedback::new(
                UiFeedbackLevel::Success,
                UiFeedbackScope::Inspect,
                "flake check 已完成。",
                None,
            )),
        }
    }

    fn test_state() -> AppState {
        AppState {
            context: AppContext {
                repo_root: PathBuf::from("/repo"),
                etc_root: PathBuf::from("/etc/nixos"),
                current_host: "demo".to_string(),
                current_system: "x86_64-linux".to_string(),
                current_user: "alice".to_string(),
                privilege_mode: "sudo-available".to_string(),
                hosts: vec!["demo".to_string()],
                users: vec!["alice".to_string()],
                catalog_path: PathBuf::from("catalog/packages"),
                catalog_groups_path: PathBuf::from("catalog/groups.toml"),
                catalog_home_options_path: PathBuf::from("catalog/home-options.toml"),
                catalog_entries: Vec::new(),
                catalog_groups: BTreeMap::new(),
                catalog_home_options: Vec::new(),
                catalog_categories: Vec::new(),
                catalog_sources: Vec::new(),
            },
            active_page: Page::ALL
                .iter()
                .position(|page| *page == Page::Inspect)
                .expect("inspect page index"),
            deploy_focus: 0,
            target_host: "demo".to_string(),
            deploy_task: DeployTask::DirectDeploy,
            deploy_source: DeploySource::CurrentRepo,
            deploy_action: DeployAction::Switch,
            flake_update: false,
            show_advanced: false,
            users_focus: 0,
            hosts_focus: 0,
            users_text_mode: None,
            hosts_text_mode: None,
            host_text_input: String::new(),
            host_settings_by_name: BTreeMap::new(),
            host_settings_errors_by_name: BTreeMap::new(),
            host_dirty_user_hosts: BTreeSet::new(),
            host_dirty_runtime_hosts: BTreeSet::new(),
            package_user_index: 0,
            package_mode: crate::domain::tui::PackageDataMode::Search,
            package_cursor: 0,
            package_category_index: 0,
            package_group_filter: None,
            package_source_filter: None,
            package_search: String::new(),
            package_search_result_indices: Vec::new(),
            package_local_entry_ids: BTreeSet::new(),
            package_search_mode: false,
            package_group_create_mode: false,
            package_group_rename_mode: false,
            package_group_rename_source: String::new(),
            package_group_input: String::new(),
            package_user_selections: BTreeMap::new(),
            package_dirty_users: BTreeSet::new(),
            home_user_index: 0,
            home_focus: 0,
            home_settings_by_user: BTreeMap::new(),
            home_dirty_users: BTreeSet::new(),
            actions_focus: 0,
            overview_repo_integrity: OverviewCheckState::NotRun,
            overview_doctor: OverviewCheckState::NotRun,
            feedback: UiFeedback::default(),
            status: String::new(),
        }
    }
}
