use crate::tui::state::{AppState, ApplyModel};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};

pub(super) fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let apply = state.apply_model();
    let root = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
        .split(area);
    let preview = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(10), Constraint::Min(12)])
        .split(root[0]);
    let controls = if apply.advanced {
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

    frame.render_widget(
        Paragraph::new(render_execution_gate_lines(&apply))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Execution Gate"),
            )
            .wrap(Wrap { trim: false }),
        preview[0],
    );
    frame.render_widget(
        Paragraph::new(render_plan_preview_lines(&apply))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Apply Preview"),
            )
            .wrap(Wrap { trim: false }),
        preview[1],
    );
    frame.render_widget(
        Paragraph::new(render_current_selection_lines(state, &apply))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Current Selection"),
            )
            .wrap(Wrap { trim: false }),
        controls[0],
    );

    let rows = state
        .deploy_rows()
        .into_iter()
        .map(|(label, value)| ListItem::new(format!("{label:<14} {value}")))
        .collect::<Vec<_>>();
    let mut list_state = ListState::default();
    list_state.select(Some(state.deploy_focus));
    let list = List::new(rows)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Advanced Controls"),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    frame.render_stateful_widget(list, controls[1], &mut list_state);

    if apply.advanced {
        let workspace = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(6), Constraint::Min(7)])
            .split(controls[2]);

        let rows = state
            .advanced_action_items()
            .iter()
            .map(|action| {
                let status = if state.action_available(*action) {
                    "可执行"
                } else {
                    "需切换场景"
                };
                ListItem::new(format!("{}  {}", action.label(), status))
            })
            .collect::<Vec<_>>();
        let mut list_state = ListState::default();
        list_state.select(Some(state.selected_advanced_row_index()));
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
        frame.render_stateful_widget(list, workspace[0], &mut list_state);

        frame.render_widget(
            Paragraph::new(render_advanced_workspace_lines(state))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Advanced Workspace"),
                )
                .wrap(Wrap { trim: false }),
            workspace[1],
        );
    }
}

fn render_execution_gate_lines(apply: &ApplyModel) -> String {
    let status = if apply.can_apply_current_host {
        "当前可直接 Apply"
    } else if apply.advanced {
        "当前处于 Advanced Workspace"
    } else if !apply.handoffs.is_empty() {
        "当前组合应转交给 Advanced"
    } else {
        "当前不能直接 Apply"
    };
    let primary_action = if apply.can_apply_current_host {
        "主动作：按 x 立即执行当前 Apply"
    } else if apply.advanced {
        "主动作：在右侧 Advanced Workspace 选择动作并按 X 执行"
    } else if !apply.handoffs.is_empty() {
        "主动作：打开 Advanced 继续复杂部署"
    } else {
        "主动作：先修复阻塞项，再回到 Apply"
    };

    [
        format!("状态：{status}"),
        primary_action.to_string(),
        format!("阻塞项：{}", join_or_none(&apply.blockers)),
        format!("警告项：{}", join_or_none(&apply.warnings)),
        format!("交接项：{}", join_or_none(&apply.handoffs)),
        format!("信息：{}", join_or_none(&apply.infos)),
    ]
    .join("\n")
}

fn render_plan_preview_lines(apply: &ApplyModel) -> String {
    [
        format!("目标主机：{}", apply.target_host),
        format!("任务：{}", apply.task.label()),
        format!("来源：{}", apply.source.label()),
        format!("动作：{}", apply.action.label()),
        format!(
            "flake update：{}",
            if apply.flake_update {
                "开启"
            } else {
                "关闭"
            }
        ),
        format!("高级模式：{}", if apply.advanced { "开启" } else { "关闭" }),
        format!(
            "同步预览：{}",
            apply
                .sync_preview
                .as_deref()
                .unwrap_or("当前组合不需要同步 /etc/nixos")
        ),
        format!(
            "命令预览：{}",
            apply
                .rebuild_preview
                .as_deref()
                .unwrap_or("当前组合会转交给 Advanced Deploy")
        ),
    ]
    .join("\n")
}

fn render_current_selection_lines(state: &AppState, apply: &ApplyModel) -> String {
    let focused = state
        .deploy_rows()
        .get(state.deploy_focus)
        .map(|(label, value)| format!("{label} = {value}"))
        .unwrap_or_else(|| "<无>".to_string());

    [
        format!("当前聚焦：{focused}"),
        "默认目标：先看左侧预览，再决定是否调整右侧高级项".to_string(),
        format!(
            "直接执行：{}",
            if apply.can_apply_current_host {
                "可执行"
            } else if apply.advanced {
                "当前已切到 Advanced Workspace"
            } else {
                "当前不可直接执行"
            }
        ),
        "操作：j/k 选项  h/l 或 Enter 调整".to_string(),
        if apply.advanced {
            "高级动作：J/K 选择  X 执行  x 仍按当前 Apply 路径处理".to_string()
        } else {
            "高级动作：打开高级模式后可在右下角执行 Advanced 动作".to_string()
        },
    ]
    .join("\n")
}

fn render_advanced_workspace_lines(state: &AppState) -> String {
    let action = state.current_advanced_action();
    let status = if state.action_available(action) {
        "当前环境可直接执行"
    } else {
        "当前环境需切换场景或权限"
    };
    let latest = if matches!(
        state.feedback.scope,
        crate::tui::state::UiFeedbackScope::Advanced
    ) {
        state.status.clone()
    } else {
        "暂无".to_string()
    };

    [
        format!("当前动作：{}", action.label()),
        format!("分组：{}", action.group_label()),
        format!("说明：{}", action.description()),
        format!("状态：{status}"),
        format!(
            "命令预览：{}",
            state
                .action_command_preview(action)
                .unwrap_or_else(|| "无".to_string())
        ),
        format!("最近结果：{latest}"),
        "操作：J/K 选择高级动作  X 执行当前高级动作".to_string(),
    ]
    .join("\n")
}

fn join_or_none(items: &[String]) -> String {
    if items.is_empty() {
        "无".to_string()
    } else {
        items.join(" | ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::tui::{DeployAction, DeploySource, DeployTask};
    use crate::tui::state::{AppContext, AppState};
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::PathBuf;

    #[test]
    fn execution_gate_lines_prioritize_direct_apply_status() {
        let apply = test_apply_model();

        let text = render_execution_gate_lines(&apply);

        assert!(text.contains("状态：当前可直接 Apply"));
        assert!(text.contains("主动作：按 x 立即执行当前 Apply"));
        assert!(text.contains("警告项：当前组合会使用 sudo -E 执行受权命令。"));
    }

    #[test]
    fn execution_gate_lines_surface_handoffs_and_blockers() {
        let mut apply = test_apply_model();
        apply.can_apply_current_host = false;
        apply.blockers = vec!["仍有未保存修改".to_string()];
        apply.handoffs = vec!["远端固定版本必须交给 Advanced Deploy 处理。".to_string()];

        let text = render_execution_gate_lines(&apply);

        assert!(text.contains("状态：当前组合应转交给 Advanced"));
        assert!(text.contains("主动作：打开 Advanced 继续复杂部署"));
        assert!(text.contains("阻塞项：仍有未保存修改"));
        assert!(text.contains("交接项：远端固定版本必须交给 Advanced Deploy 处理。"));
    }

    #[test]
    fn plan_preview_lines_keep_sync_and_rebuild_previews_visible() {
        let apply = test_apply_model();

        let text = render_plan_preview_lines(&apply);

        assert!(text.contains("目标主机：nixos"));
        assert!(text.contains("来源：当前仓库"));
        assert!(text.contains("同步预览：sudo rsync /repo /etc/nixos"));
        assert!(
            text.contains("命令预览：sudo -E env nixos-rebuild switch --flake /etc/nixos#nixos")
        );
    }

    #[test]
    fn current_selection_lines_highlight_focused_advanced_control() {
        let mut state = test_app_state();
        state.deploy_focus = 3;
        let apply = state.apply_model();

        let text = render_current_selection_lines(&state, &apply);

        assert!(text.contains("当前聚焦：动作 = switch"));
        assert!(text.contains("默认目标：先看左侧预览，再决定是否调整右侧高级项"));
    }

    #[test]
    fn execution_gate_lines_show_advanced_workspace_when_enabled() {
        let mut apply = test_apply_model();
        apply.can_apply_current_host = false;
        apply.advanced = true;
        apply.handoffs = vec!["当前已打开高级选项，应交给 Advanced Deploy 处理。".to_string()];

        let text = render_execution_gate_lines(&apply);

        assert!(text.contains("状态：当前处于 Advanced Workspace"));
        assert!(text.contains("Advanced Workspace 选择动作并按 X 执行"));
    }

    #[test]
    fn advanced_workspace_lines_surface_selected_action_and_preview() {
        let mut state = test_app_state();
        state.show_advanced = true;
        state.ensure_advanced_action_focus();
        state.next_advanced_action();

        let text = render_advanced_workspace_lines(&state);

        assert!(text.contains("当前动作：update upstream pins"));
        assert!(text.contains("分组：Repository Maintenance"));
        assert!(text.contains("命令预览：update-upstream-apps"));
        assert!(text.contains("J/K 选择高级动作"));
    }

    fn test_apply_model() -> ApplyModel {
        ApplyModel {
            target_host: "nixos".to_string(),
            task: DeployTask::DirectDeploy,
            source: DeploySource::CurrentRepo,
            action: DeployAction::Switch,
            flake_update: false,
            advanced: false,
            sync_preview: Some("sudo rsync /repo /etc/nixos".to_string()),
            rebuild_preview: Some(
                "sudo -E env nixos-rebuild switch --flake /etc/nixos#nixos".to_string(),
            ),
            can_execute_directly: true,
            can_apply_current_host: true,
            blockers: Vec::new(),
            warnings: vec!["当前组合会使用 sudo -E 执行受权命令。".to_string()],
            handoffs: Vec::new(),
            infos: vec!["检测 hostname：nixos".to_string()],
        }
    }

    fn test_app_state() -> AppState {
        AppState {
            context: AppContext {
                repo_root: PathBuf::from("/repo"),
                etc_root: PathBuf::from("/etc/nixos"),
                current_host: "nixos".to_string(),
                current_system: "x86_64-linux".to_string(),
                current_user: "alice".to_string(),
                privilege_mode: "sudo-available".to_string(),
                hosts: vec!["nixos".to_string()],
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
            active_page: 1,
            deploy_focus: 0,
            target_host: "nixos".to_string(),
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
            overview_repo_integrity: crate::tui::state::OverviewCheckState::NotRun,
            overview_doctor: crate::tui::state::OverviewCheckState::NotRun,
            feedback: crate::tui::state::UiFeedback::default(),
            status: String::new(),
        }
    }
}
