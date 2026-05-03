use super::health::render_compact_health_summary;
use super::summary::render_mainline_summary;
use crate::tui::state::{
    AppState, OverviewHealthFocus, OverviewHostStatus, OverviewModel, OverviewPrimaryActionKind,
};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

pub(super) fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let overview = state.overview_model();
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(7), Constraint::Min(11)])
        .split(area);
    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(rows[0]);
    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(54), Constraint::Percentage(46)])
        .split(rows[1]);
    let bottom_right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(5)])
        .split(bottom[1]);

    frame.render_widget(
        Paragraph::new(render_overview_summary_lines(&overview))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Overview Summary"),
            )
            .wrap(Wrap { trim: false }),
        top[0],
    );
    frame.render_widget(
        Paragraph::new(render_context_lines(&overview))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Current Context"),
            )
            .wrap(Wrap { trim: false }),
        top[1],
    );
    frame.render_widget(
        Paragraph::new(render_health_lines(&overview))
            .block(Block::default().borders(Borders::ALL).title("Health"))
            .wrap(Wrap { trim: false }),
        bottom[0],
    );
    frame.render_widget(
        Paragraph::new(render_dirty_lines(&overview))
            .block(Block::default().borders(Borders::ALL).title("Dirty State"))
            .wrap(Wrap { trim: false }),
        bottom_right[0],
    );
    frame.render_widget(
        Paragraph::new(render_apply_lines(&overview))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Apply Snapshot"),
            )
            .wrap(Wrap { trim: false }),
        bottom_right[1],
    );
}

fn render_context_lines(overview: &OverviewModel) -> String {
    [
        format!(
            "目标/当前：{} / {}",
            overview.context.target_host, overview.context.current_host
        ),
        format!(
            "用户/权限：{} / {}",
            overview.context.current_user, overview.context.privilege_mode
        ),
        format!("当前仓库：{}", overview.context.repo_root.display()),
        format!("/etc/nixos：{}", overview.context.etc_root.display()),
    ]
    .join("\n")
}

fn render_health_lines(overview: &OverviewModel) -> String {
    let mut lines = vec![match &overview.host_status {
        OverviewHostStatus::Ready => "host-config: ok".to_string(),
        OverviewHostStatus::Unavailable { message } => {
            format!("host-config: unavailable ({message})")
        }
        OverviewHostStatus::Invalid { errors } => match errors.split_first() {
            Some((first, [])) => format!("host-config: invalid ({first})"),
            Some((first, rest)) => format!("host-config: invalid ({first}，另 {} 项)", rest.len()),
            None => "host-config: invalid".to_string(),
        },
    }];
    lines.push(match overview.health_focus {
        OverviewHealthFocus::RepoIntegrity => render_compact_health_summary(
            "repo-integrity",
            &overview.repo_integrity,
            "doctor",
            &overview.doctor,
            &overview.managed_guards,
        ),
        OverviewHealthFocus::Doctor => render_compact_health_summary(
            "doctor",
            &overview.doctor,
            "repo-integrity",
            &overview.repo_integrity,
            &overview.managed_guards,
        ),
    });
    lines.join("\n")
}

fn render_overview_summary_lines(overview: &OverviewModel) -> String {
    let primary_action = format!(
        "主动作：{}",
        overview_primary_action_label(overview.primary_action.kind)
    );
    render_mainline_summary(
        &overview.apply_summary.status,
        &overview.apply_summary.latest_result,
        &overview.primary_action.next_step,
        &primary_action,
        &[("原因", overview.primary_action.reason.as_str())],
    )
}

fn render_dirty_lines(overview: &OverviewModel) -> String {
    if overview.dirty_sections.is_empty() {
        return [
            "状态：clean".to_string(),
            "待保存：无".to_string(),
            "优先保存：无".to_string(),
        ]
        .join("\n");
    }

    let pending = overview
        .dirty_sections
        .iter()
        .map(|section| format_dirty_section(section.name, &section.items))
        .collect::<Vec<_>>()
        .join(" | ");
    let first = &overview.dirty_sections[0];
    let first_target = first
        .items
        .first()
        .cloned()
        .unwrap_or_else(|| "当前目标".to_string());

    [
        format!("状态：{} 页待保存", overview.dirty_sections.len()),
        format!("待保存：{pending}"),
        format!("优先保存：{}[{}]", first.name, first_target),
    ]
    .join("\n")
}

fn render_apply_lines(overview: &OverviewModel) -> String {
    let sync_plan = if overview.apply.sync_preview.is_some() {
        "需要同步 /etc/nixos".to_string()
    } else {
        "不需要同步 /etc/nixos".to_string()
    };

    [
        format!("默认来源：{}", overview.apply.source.label()),
        format!("默认动作：{}", overview.apply.action.label()),
        format!("同步：{sync_plan}"),
    ]
    .join("\n")
}

fn overview_primary_action_label(kind: OverviewPrimaryActionKind) -> &'static str {
    match kind {
        OverviewPrimaryActionKind::PreviewApply => "Preview Apply",
    }
}

fn format_dirty_section(name: &str, items: &[String]) -> String {
    if items.is_empty() {
        name.to_string()
    } else if items.len() == 1 {
        format!("{name}[{}]", items[0])
    } else {
        format!("{name}[{} +{}]", items[0], items.len() - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::tui::{DeployAction, DeploySource, DeployTask};
    use crate::tui::state::{
        AppContext, AppState, ApplyModel, ManagedGuardSnapshot, OverviewApplySummaryModel,
        OverviewCheckState, OverviewContext, OverviewDirtySection, OverviewPrimaryAction,
        UiFeedback,
    };
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::buffer::Buffer;
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::PathBuf;

    #[test]
    fn overview_primary_action_labels_match_task_language() {
        assert_eq!(
            overview_primary_action_label(OverviewPrimaryActionKind::PreviewApply),
            "Preview Apply"
        );
    }

    #[test]
    fn primary_action_lines_show_reason_and_next_step() {
        let overview = test_overview_model(OverviewPrimaryActionKind::PreviewApply);

        let text = render_overview_summary_lines(&overview);

        let status_pos = text.find("当前判断：当前可直接 Apply").unwrap();
        let reason_pos = text
            .find("原因：默认主路径：先进入 Apply 预览，确认 handoff 和当前主机执行门槛。")
            .unwrap();
        let result_pos = text.find("最近结果：暂无").unwrap();
        let next_step_pos = text
            .find("下一步：在 Apply 先看 handoff 预览；如需继续，切到 Advanced 执行 launch deploy wizard")
            .unwrap();
        let primary_pos = text.find("主动作：Preview Apply").unwrap();

        assert!(status_pos < reason_pos);
        assert!(reason_pos < result_pos);
        assert!(result_pos < next_step_pos);
        assert!(next_step_pos < primary_pos);
        assert!(text.contains("主动作：Preview Apply"));
        assert!(text.contains("当前判断：当前可直接 Apply"));
        assert!(
            text.contains("原因：默认主路径：先进入 Apply 预览，确认 handoff 和当前主机执行门槛。")
        );
        assert!(text.contains("最近结果：暂无"));
        assert!(text.contains(
            "下一步：在 Apply 先看 handoff 预览；如需继续，切到 Advanced 执行 launch deploy wizard"
        ));
    }

    #[test]
    fn apply_lines_only_surface_apply_route_inputs() {
        let mut overview = test_overview_model(OverviewPrimaryActionKind::PreviewApply);
        overview.apply.can_apply_current_host = false;
        overview.apply.rebuild_preview = None;
        overview.apply.blockers = vec!["仍有未保存修改".to_string()];
        overview.apply.handoffs = vec![
            "当前来源是远端最新版本；默认 Apply 不会直接执行，必须交给完整高级路径。".to_string(),
        ];
        overview.apply_summary = OverviewApplySummaryModel {
            status: "当前组合应转交给 Advanced".to_string(),
            preview_command_fallback: "当前组合会转交给 Advanced 执行 launch deploy wizard"
                .to_string(),
            next_step:
                "在 Apply 先看 handoff 预览；如需继续，切到 Advanced 执行 launch deploy wizard"
                    .to_string(),
            latest_result:
                "Apply 已执行完成：switch nixos。 下一步：回到 Overview 检查健康和下一步"
                    .to_string(),
        };

        let text = render_apply_lines(&overview);

        assert!(text.contains("默认来源：当前仓库"));
        assert!(text.contains("默认动作：switch"));
        assert!(text.contains("同步：需要同步 /etc/nixos"));
        assert!(!text.contains("状态："));
        assert!(!text.contains("blocker："));
        assert!(!text.contains("warning："));
        assert!(!text.contains("handoff："));
        assert!(!text.contains("命令预览："));
        assert!(!text.contains("最近结果："));
    }

    #[test]
    fn dirty_lines_collapse_to_clean_summary_when_no_pending_changes() {
        let mut overview = test_overview_model(OverviewPrimaryActionKind::PreviewApply);
        overview.dirty_sections.clear();

        let text = render_dirty_lines(&overview);

        assert!(text.contains("状态：clean"));
        assert!(text.contains("待保存：无"));
        assert!(text.contains("优先保存：无"));
    }

    #[test]
    fn dirty_lines_prioritize_first_dirty_section_and_save_target() {
        let overview = test_overview_model(OverviewPrimaryActionKind::PreviewApply);

        let text = render_dirty_lines(&overview);

        assert!(text.contains("状态：1 页待保存"));
        assert!(text.contains("待保存：Home[alice]"));
        assert!(text.contains("优先保存：Home[alice]"));
        assert!(!text.contains("Apply 影响："));
        assert!(!text.contains("下一步："));
    }

    #[test]
    fn health_lines_use_compact_host_config_label() {
        let overview = test_overview_model(OverviewPrimaryActionKind::PreviewApply);

        let text = render_health_lines(&overview);

        assert!(text.contains("host-config: ok"));
        assert!(!text.contains("目标主机配置：可用"));
    }

    #[test]
    fn health_lines_compact_managed_guard_summary() {
        let mut overview = test_overview_model(OverviewPrimaryActionKind::PreviewApply);
        overview.managed_guards = vec![
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
        ];

        let text = render_health_lines(&overview);

        assert!(text.contains("save-guards: Packages[alice] blocked"));
        assert!(text.contains("优先处理：manual group blocks save"));
        assert!(!text.contains("Home[alice]: ok"));
    }

    #[test]
    fn health_lines_prioritize_doctor_when_doctor_is_active_failure() {
        let mut overview = test_overview_model(OverviewPrimaryActionKind::PreviewApply);
        overview.health_focus = OverviewHealthFocus::Doctor;
        overview.repo_integrity = OverviewCheckState::Healthy {
            summary: "ok".to_string(),
            details: Vec::new(),
        };
        overview.doctor = OverviewCheckState::Error {
            summary: "failed (1 check(s))".to_string(),
            details: vec!["缺少 nixos-rebuild".to_string()],
        };

        let text = render_health_lines(&overview);
        let doctor_pos = text
            .find("doctor: failed (1 check(s))")
            .expect("doctor section should render");
        let repo_pos = text
            .find("repo-integrity: ok")
            .expect("repo-integrity section should render");

        assert!(doctor_pos < repo_pos);
        assert!(text.contains("优先项：缺少 nixos-rebuild"));
        assert!(!text.contains("缺少 cargo"));
    }

    #[test]
    fn context_lines_focus_target_without_inventory_counts() {
        let overview = test_overview_model(OverviewPrimaryActionKind::PreviewApply);

        let text = render_context_lines(&overview);

        assert!(text.starts_with("目标/当前：nixos / nixos"));
        assert!(text.contains("用户/权限：alice / sudo-available"));
        assert!(text.contains("当前仓库：/repo"));
        assert!(!text.contains("已知 hosts"));
        assert!(!text.contains("已知用户"));
    }

    #[test]
    fn render_overview_uses_apply_snapshot_title_and_compact_first_screen() {
        let state = test_render_state();

        let text = render_view_text(140, 36, |frame| {
            render(frame, Rect::new(0, 0, 140, 36), &state)
        });

        assert!(text.contains("Overview Summary"));
        assert!(text.contains("Current Context"));
        assert!(text.contains("Health"));
        assert!(text.contains("Dirty State"));
        assert!(text.contains("Apply Snapshot"));
        assert!(!text.contains("Apply Readiness"));
    }

    #[test]
    fn render_overview_keeps_all_panels_visible_in_short_body_area() {
        let state = test_render_state();

        let text = render_view_text(84, 18, |frame| {
            render(frame, Rect::new(0, 0, 84, 18), &state)
        });

        assert!(text.contains("Overview Summary"));
        assert!(text.contains("Current Context"));
        assert!(text.contains("Health"));
        assert!(text.contains("Dirty State"));
        assert!(text.contains("Apply Snapshot"));
        assert!(text.contains("host-config"));
        assert!(text.contains("clean"));
        assert!(text.contains("switch"));
        assert!(text.contains("/repo"));
    }

    fn test_overview_model(primary_action: OverviewPrimaryActionKind) -> OverviewModel {
        OverviewModel {
            context: OverviewContext {
                current_host: "nixos".to_string(),
                target_host: "nixos".to_string(),
                current_user: "alice".to_string(),
                privilege_mode: "sudo-available".to_string(),
                repo_root: PathBuf::from("/repo"),
                etc_root: PathBuf::from("/etc/nixos"),
            },
            dirty_sections: vec![OverviewDirtySection {
                name: "Home",
                items: vec!["alice".to_string()],
            }],
            host_status: OverviewHostStatus::Ready,
            health_focus: OverviewHealthFocus::RepoIntegrity,
            repo_integrity: OverviewCheckState::Healthy {
                summary: "ok".to_string(),
                details: Vec::new(),
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
                    errors: Vec::new(),
                },
                ManagedGuardSnapshot {
                    page: "Home",
                    target: "alice".to_string(),
                    available: true,
                    errors: Vec::new(),
                },
                ManagedGuardSnapshot {
                    page: "Users",
                    target: "nixos".to_string(),
                    available: true,
                    errors: Vec::new(),
                },
                ManagedGuardSnapshot {
                    page: "Hosts",
                    target: "nixos".to_string(),
                    available: true,
                    errors: Vec::new(),
                },
            ],
            apply: ApplyModel {
                target_host: "nixos".to_string(),
                task: DeployTask::DirectDeploy,
                source: DeploySource::CurrentRepo,
                source_detail: None,
                action: DeployAction::Switch,
                flake_update: false,
                sync_preview: Some("sudo rsync /repo /etc/nixos".to_string()),
                rebuild_preview: Some(
                    "sudo -E env nixos-rebuild switch --flake /etc/nixos#nixos".to_string(),
                ),
                can_execute_directly: true,
                can_apply_current_host: true,
                blockers: Vec::new(),
                warnings: vec!["当前组合会使用 sudo -E 执行受权命令。".to_string()],
                handoffs: vec![
                    "当前来源是远端最新版本；默认 Apply 不会直接执行，必须交给完整高级路径。"
                        .to_string(),
                ],
                infos: vec!["检测 hostname：nixos".to_string()],
            },
            apply_summary: OverviewApplySummaryModel {
                status: "当前可直接 Apply".to_string(),
                preview_command_fallback: "当前组合可直接执行 Apply".to_string(),
                next_step: "在 Apply 查看预览；确认后按 x 直接运行".to_string(),
                latest_result: "暂无".to_string(),
            },
            primary_action: OverviewPrimaryAction {
                kind: primary_action,
                reason: "默认主路径：先进入 Apply 预览，确认 handoff 和当前主机执行门槛。"
                    .to_string(),
                recent_feedback:
                    "当前来源是远端最新版本；默认 Apply 不会直接执行，必须交给完整高级路径。"
                        .to_string(),
                next_step:
                    "在 Apply 先看 handoff 预览；如需继续，切到 Advanced 执行 launch deploy wizard"
                        .to_string(),
            },
        }
    }

    fn test_render_state() -> AppState {
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
                catalog_workflows_path: PathBuf::from("catalog/workflows.toml"),
                catalog_entries: Vec::new(),
                catalog_groups: BTreeMap::new(),
                catalog_home_options: Vec::new(),
                catalog_workflows: BTreeMap::new(),
                catalog_categories: Vec::new(),
                catalog_sources: Vec::new(),
            },
            active_page: 0,
            active_edit_page: 0,
            deploy_focus: 0,
            advanced_deploy_focus: 0,
            target_host: "demo".to_string(),
            deploy_task: DeployTask::DirectDeploy,
            deploy_source: DeploySource::CurrentRepo,
            deploy_source_ref: String::new(),
            deploy_action: DeployAction::Switch,
            flake_update: false,
            advanced_target_host: "demo".to_string(),
            advanced_deploy_task: DeployTask::DirectDeploy,
            advanced_deploy_source: DeploySource::CurrentRepo,
            advanced_deploy_source_ref: String::new(),
            advanced_deploy_action: DeployAction::Switch,
            advanced_flake_update: false,
            inspect_action: crate::domain::tui::ActionItem::FlakeCheck,
            advanced_action: crate::domain::tui::ActionItem::FlakeUpdate,
            help_overlay_visible: false,
            users_focus: 0,
            hosts_focus: 0,
            deploy_text_mode: None,
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
            package_workflow_filter: None,
            package_search: String::new(),
            package_search_result_indices: Vec::new(),
            package_local_entry_ids: BTreeSet::new(),
            package_search_mode: false,
            package_group_create_mode: false,
            package_group_rename_mode: false,
            package_workflow_add_confirm_mode: false,
            package_group_rename_source: String::new(),
            package_group_input: String::new(),
            package_user_selections: BTreeMap::new(),
            package_dirty_users: BTreeSet::new(),
            home_user_index: 0,
            home_focus: 0,
            home_settings_by_user: BTreeMap::new(),
            home_dirty_users: BTreeSet::new(),
            overview_repo_integrity: OverviewCheckState::Healthy {
                summary: "ok".to_string(),
                details: Vec::new(),
            },
            overview_doctor: OverviewCheckState::NotRun,
            feedback: UiFeedback::default(),
            status: String::new(),
        }
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
