use crate::tui::state::{
    AppState, ManagedGuardSnapshot, OverviewCheckState, OverviewHostStatus, OverviewModel,
    OverviewPrimaryActionKind,
};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

pub(super) fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let overview = state.overview_model();
    let root = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(area);
    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(9),
            Constraint::Length(10),
            Constraint::Min(8),
        ])
        .split(root[0]);
    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(9), Constraint::Min(12)])
        .split(root[1]);

    let feedback_message = if !state.feedback.message.is_empty() {
        state.feedback.message.clone()
    } else if !state.status.is_empty() {
        state.status.clone()
    } else {
        "无".to_string()
    };

    frame.render_widget(
        Paragraph::new(render_context_lines(
            &overview,
            state.context.hosts.len(),
            state.context.users.len(),
        ))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Current Context"),
        )
        .wrap(Wrap { trim: false }),
        left[0],
    );
    frame.render_widget(
        Paragraph::new(render_health_lines(&overview))
            .block(Block::default().borders(Borders::ALL).title("Health"))
            .wrap(Wrap { trim: false }),
        left[1],
    );
    frame.render_widget(
        Paragraph::new(render_primary_action_lines(
            &overview,
            &feedback_message,
            state.feedback.next_step.as_deref(),
        ))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Primary Action"),
        )
        .wrap(Wrap { trim: false }),
        left[2],
    );
    frame.render_widget(
        Paragraph::new(render_dirty_lines(&overview))
            .block(Block::default().borders(Borders::ALL).title("Dirty State"))
            .wrap(Wrap { trim: false }),
        right[0],
    );
    frame.render_widget(
        Paragraph::new(render_apply_lines(&overview))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Apply Readiness"),
            )
            .wrap(Wrap { trim: false }),
        right[1],
    );
}

fn render_context_lines(overview: &OverviewModel, host_count: usize, user_count: usize) -> String {
    [
        format!("当前主机：{}", overview.context.current_host),
        format!("目标主机：{}", overview.context.target_host),
        format!("当前用户：{}", overview.context.current_user),
        format!("权限模式：{}", overview.context.privilege_mode),
        format!("当前仓库：{}", overview.context.repo_root.display()),
        format!("/etc/nixos：{}", overview.context.etc_root.display()),
        format!("已知 hosts：{host_count}"),
        format!("已知用户：{user_count}"),
    ]
    .join("\n")
}

fn render_health_lines(overview: &OverviewModel) -> String {
    let host_status = match &overview.host_status {
        OverviewHostStatus::Ready => "目标主机配置：可用".to_string(),
        OverviewHostStatus::Unavailable { message } => format!("目标主机配置：{message}"),
        OverviewHostStatus::Invalid { errors } => {
            format!("目标主机配置未通过校验：{}", errors.join("；"))
        }
    };
    let mut lines = vec![host_status];
    lines.extend(render_check_lines(
        "repo-integrity",
        &overview.repo_integrity,
    ));
    lines.extend(render_check_lines("doctor", &overview.doctor));
    lines.extend(render_managed_guard_lines(&overview.managed_guards, false));
    lines.join("\n")
}

fn render_primary_action_lines(
    overview: &OverviewModel,
    feedback_message: &str,
    feedback_next_step: Option<&str>,
) -> String {
    let mut lines = vec![
        format!(
            "主动作：{}",
            overview_primary_action_label(overview.primary_action.kind)
        ),
        format!("原因：{}", overview.primary_action.reason),
        format!("最近提示：{feedback_message}"),
    ];
    if let Some(next_step) = feedback_next_step
        && !next_step.is_empty()
    {
        lines.push(format!("下一步：{next_step}"));
    }
    lines.join("\n")
}

fn render_dirty_lines(overview: &OverviewModel) -> String {
    if overview.dirty_sections.is_empty() {
        return "无未保存修改".to_string();
    }

    overview
        .dirty_sections
        .iter()
        .map(|section| format!("- {}: {}", section.name, section.items.join(", ")))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_apply_lines(overview: &OverviewModel) -> String {
    let apply_status = if overview.apply.can_apply_current_host {
        "当前可直接 Apply"
    } else if !overview.apply.handoffs.is_empty() {
        "当前组合需要交给 Advanced"
    } else {
        "当前不能直接 Apply"
    };
    let sync_preview = overview
        .apply
        .sync_preview
        .clone()
        .unwrap_or_else(|| "当前组合不需要同步 /etc/nixos".to_string());
    let rebuild_preview = overview
        .apply
        .rebuild_preview
        .clone()
        .unwrap_or_else(|| "当前组合会转交给 Advanced Deploy".to_string());
    let blockers = if overview.apply.blockers.is_empty() {
        "无".to_string()
    } else {
        overview.apply.blockers.join(" | ")
    };
    let warnings = if overview.apply.warnings.is_empty() {
        "无".to_string()
    } else {
        overview.apply.warnings.join(" | ")
    };
    let handoffs = if overview.apply.handoffs.is_empty() {
        "无".to_string()
    } else {
        overview.apply.handoffs.join(" | ")
    };

    [
        format!("状态：{apply_status}"),
        format!("来源：{}", overview.apply.source.label()),
        format!("动作：{}", overview.apply.action.label()),
        format!(
            "flake update：{}",
            if overview.apply.flake_update {
                "开启"
            } else {
                "关闭"
            }
        ),
        format!(
            "高级模式：{}",
            if overview.apply.advanced {
                "开启"
            } else {
                "关闭"
            }
        ),
        format!("同步预览：{sync_preview}"),
        format!("命令预览：{rebuild_preview}"),
        format!("阻塞项：{blockers}"),
        format!("警告项：{warnings}"),
        format!("交接项：{handoffs}"),
    ]
    .join("\n")
}

fn overview_primary_action_label(kind: OverviewPrimaryActionKind) -> &'static str {
    match kind {
        OverviewPrimaryActionKind::SaveDirtyPages => "Save Dirty Pages",
        OverviewPrimaryActionKind::ReviewInspect => "Open Inspect",
        OverviewPrimaryActionKind::ReviewManagedGuards => "Review Save Guards",
        OverviewPrimaryActionKind::OpenAdvancedApply => "Open Advanced",
        OverviewPrimaryActionKind::ReviewApply => "Open Apply",
        OverviewPrimaryActionKind::ApplyCurrentHost => "Apply Current Host",
    }
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
    use crate::domain::tui::{DeployAction, DeploySource, DeployTask};
    use crate::tui::state::{
        ApplyModel, ManagedGuardSnapshot, OverviewContext, OverviewDirtySection,
        OverviewPrimaryAction,
    };
    use std::path::PathBuf;

    #[test]
    fn overview_primary_action_labels_match_task_language() {
        assert_eq!(
            overview_primary_action_label(OverviewPrimaryActionKind::SaveDirtyPages),
            "Save Dirty Pages"
        );
        assert_eq!(
            overview_primary_action_label(OverviewPrimaryActionKind::ReviewInspect),
            "Open Inspect"
        );
        assert_eq!(
            overview_primary_action_label(OverviewPrimaryActionKind::ReviewManagedGuards),
            "Review Save Guards"
        );
        assert_eq!(
            overview_primary_action_label(OverviewPrimaryActionKind::ApplyCurrentHost),
            "Apply Current Host"
        );
    }

    #[test]
    fn primary_action_lines_show_reason_and_next_step() {
        let overview = test_overview_model(OverviewPrimaryActionKind::OpenAdvancedApply);

        let text = render_primary_action_lines(
            &overview,
            "当前组合需要交给 Advanced。",
            Some("打开 Advanced 完成复杂部署"),
        );

        assert!(text.contains("主动作：Open Advanced"));
        assert!(text.contains("原因：远端最新版本必须交给 Advanced Deploy 处理。"));
        assert!(text.contains("最近提示：当前组合需要交给 Advanced。"));
        assert!(text.contains("下一步：打开 Advanced 完成复杂部署"));
    }

    #[test]
    fn apply_lines_surface_readiness_blockers_and_handoffs() {
        let mut overview = test_overview_model(OverviewPrimaryActionKind::ReviewApply);
        overview.apply.can_apply_current_host = false;
        overview.apply.blockers = vec!["仍有未保存修改".to_string()];
        overview.apply.handoffs = vec!["远端最新版本必须交给 Advanced Deploy 处理。".to_string()];

        let text = render_apply_lines(&overview);

        assert!(text.contains("状态：当前组合需要交给 Advanced"));
        assert!(text.contains("阻塞项：仍有未保存修改"));
        assert!(text.contains("交接项：远端最新版本必须交给 Advanced Deploy 处理。"));
        assert!(text.contains("同步预览：sudo rsync"));
    }

    #[test]
    fn dirty_lines_collapse_to_clean_when_no_pending_changes() {
        let mut overview = test_overview_model(OverviewPrimaryActionKind::ApplyCurrentHost);
        overview.dirty_sections.clear();

        assert_eq!(render_dirty_lines(&overview), "无未保存修改");
    }

    #[test]
    fn health_lines_include_managed_guard_summary() {
        let mut overview = test_overview_model(OverviewPrimaryActionKind::ReviewInspect);
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

        assert!(text.contains("save-guards: 1 blocked target(s)"));
        assert!(text.contains("Packages[alice]: failed (1 issue(s))"));
        assert!(text.contains("Home[alice]: ok"));
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
                handoffs: vec!["远端最新版本必须交给 Advanced Deploy 处理。".to_string()],
                infos: vec!["检测 hostname：nixos".to_string()],
            },
            primary_action: OverviewPrimaryAction {
                kind: primary_action,
                reason: "远端最新版本必须交给 Advanced Deploy 处理。".to_string(),
            },
        }
    }
}
