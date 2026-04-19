use crate::tui::state::{
    AdvancedContextModel, AdvancedMaintenanceModel, AdvancedMaintenancePageModel,
    AdvancedSummaryModel, AdvancedWizardDetailModel, AdvancedWizardModel, AdvancedWizardPageModel,
};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use super::chrome::{DeployLayoutAreas, render_deploy_controls_list, render_workspace_section};
use super::shared::{DeployPreviewFields, join_or_none, render_deploy_preview_lines};

pub(super) fn render_advanced_maintenance_page(
    frame: &mut Frame,
    area: Rect,
    model: &AdvancedMaintenancePageModel,
) {
    let layout = DeployLayoutAreas::new(area, model.shell.workspace_visible);
    render_advanced_maintenance_layout(frame, &layout, model);
}

fn render_advanced_maintenance_layout(
    frame: &mut Frame,
    layout: &DeployLayoutAreas,
    model: &AdvancedMaintenancePageModel,
) {
    frame.render_widget(
        Paragraph::new(render_advanced_maintenance_summary_lines(
            &model.maintenance,
        ))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(model.shell.summary_title),
        )
        .wrap(Wrap { trim: false }),
        layout.preview_summary,
    );
    frame.render_widget(
        Paragraph::new(render_advanced_maintenance_preview_lines(
            &model.maintenance,
        ))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(model.shell.preview_title),
        )
        .wrap(Wrap { trim: false }),
        layout.preview_main,
    );
    frame.render_widget(
        Paragraph::new(render_advanced_context_lines(&model.context))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(model.shell.context_title),
            )
            .wrap(Wrap { trim: false }),
        layout.context,
    );
    frame.render_widget(
        Paragraph::new(render_advanced_repository_context_lines(&model.maintenance))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(model.shell.controls_title),
            )
            .wrap(Wrap { trim: false }),
        layout.controls,
    );
    render_workspace_section(
        frame,
        layout,
        model.advanced_actions.as_ref(),
        Some(render_advanced_workspace_lines(
            Some(&model.maintenance),
            None,
        )),
        model.shell.detail_title,
    );
}

pub(super) fn render_advanced_wizard_page(
    frame: &mut Frame,
    area: Rect,
    model: &AdvancedWizardPageModel,
) {
    let layout = DeployLayoutAreas::new(area, model.shell.workspace_visible);
    render_advanced_wizard_layout(frame, &layout, model);
}

fn render_advanced_wizard_layout(
    frame: &mut Frame,
    layout: &DeployLayoutAreas,
    model: &AdvancedWizardPageModel,
) {
    frame.render_widget(
        Paragraph::new(render_advanced_summary_lines(
            &model.summary,
            &model.wizard,
            &model.detail.latest_result,
        ))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(model.shell.summary_title),
        )
        .wrap(Wrap { trim: false }),
        layout.preview_summary,
    );
    frame.render_widget(
        Paragraph::new(render_advanced_wizard_preview_lines(&model.wizard))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(model.shell.preview_title),
            )
            .wrap(Wrap { trim: false }),
        layout.preview_main,
    );
    frame.render_widget(
        Paragraph::new(render_advanced_context_lines(&model.context))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(model.shell.context_title),
            )
            .wrap(Wrap { trim: false }),
        layout.context,
    );
    render_deploy_controls_list(
        frame,
        layout.controls,
        &model.controls,
        model.shell.controls_title,
    );
    render_workspace_section(
        frame,
        layout,
        model.advanced_actions.as_ref(),
        Some(render_advanced_workspace_lines(None, Some(&model.detail))),
        model.shell.detail_title,
    );
}

pub(super) fn render_advanced_summary_lines(
    summary: &AdvancedSummaryModel,
    wizard: &AdvancedWizardModel,
    latest_result: &str,
) -> String {
    let alignment = if summary.current_action == summary.recommended_action {
        "当前动作已经对准默认推荐，可直接执行。".to_string()
    } else {
        format!(
            "当前动作不是默认推荐；可先切到 {}。",
            summary.recommended_action.label()
        )
    };

    [
        format!("当前动作：{}", summary.current_action.label()),
        format!("推荐动作：{}", summary.recommended_action.label()),
        format!("状态：{alignment}"),
        format!("最近结果：{}", latest_result),
        format!("下一步：{}", summary.completion_hint),
        format!("原因：{}", summary.reason),
        format!("交接项：{}", join_or_none(&wizard.handoffs)),
        format!("警告项：{}", join_or_none(&wizard.warnings)),
    ]
    .join("\n")
}

pub(super) fn render_advanced_maintenance_summary_lines(
    model: &AdvancedMaintenanceModel,
) -> String {
    let alignment = if model.summary.current_action == model.summary.recommended_action {
        "当前动作已经对准默认推荐，可直接执行。".to_string()
    } else {
        format!(
            "当前动作不是默认推荐；可先切到 {}。",
            model.summary.recommended_action.label()
        )
    };

    [
        format!("当前动作：{}", model.summary.current_action.label()),
        format!("推荐动作：{}", model.summary.recommended_action.label()),
        format!("状态：{alignment}"),
        format!("最近结果：{}", model.latest_result),
        format!("下一步：{}", model.return_hint),
        format!("原因：{}", model.summary.reason),
        format!("仓库根：{}", model.repo_root.display()),
    ]
    .join("\n")
}

pub(super) fn render_advanced_wizard_preview_lines(model: &AdvancedWizardModel) -> String {
    render_deploy_preview_lines(DeployPreviewFields {
        advanced_entry: true,
        target_host: model.target_host.as_str(),
        task: model.task,
        source: model.source,
        source_detail: model.source_detail.as_deref(),
        action: model.action,
        flake_update: model.flake_update,
        sync_preview: model.sync_preview.as_deref(),
        rebuild_preview: Some(model.command_preview.as_str()),
        command_fallback: model.command_preview.as_str(),
    })
}

pub(super) fn render_advanced_maintenance_preview_lines(
    model: &AdvancedMaintenanceModel,
) -> String {
    [
        "入口：当前动作属于仓库维护路径，不使用 deploy 参数。".to_string(),
        format!("当前动作：{}", model.summary.current_action.label()),
        format!("分组：{}", model.summary.current_action.group_label()),
        format!("主要写回：{}", model.write_target),
        format!("影响：{}", model.impact),
        format!("回主线：{}", model.return_hint),
        "返回路径：如果只想回默认应用路径，按 b 返回 Apply。".to_string(),
    ]
    .join("\n")
}

pub(super) fn render_advanced_repository_context_lines(model: &AdvancedMaintenanceModel) -> String {
    [
        format!(
            "命令预览：{}",
            model
                .command_preview
                .clone()
                .unwrap_or_else(|| "无".to_string())
        ),
        format!("主要写回：{}", model.write_target),
        format!("仓库根：{}", model.repo_root.display()),
        format!("下一步：{}", model.return_hint),
    ]
    .join("\n")
}

pub(super) fn render_advanced_context_lines(model: &AdvancedContextModel) -> String {
    [
        format!(
            "当前聚焦：{} = {}",
            model.focused_row.label, model.focused_row.value
        ),
        model.default_target.clone(),
        model.recommendation.clone(),
        format!("默认执行：{}", model.execution_hint),
        model.operation_hint.clone(),
        model.advanced_action_hint.clone(),
    ]
    .join("\n")
}

pub(super) fn render_advanced_workspace_lines(
    maintenance: Option<&AdvancedMaintenanceModel>,
    wizard_detail: Option<&AdvancedWizardDetailModel>,
) -> String {
    if let Some(maintenance) = maintenance {
        return render_advanced_maintenance_detail_lines(maintenance);
    }
    render_advanced_wizard_detail_lines(wizard_detail.expect("advanced wizard detail should exist"))
}

pub(super) fn render_advanced_maintenance_detail_lines(model: &AdvancedMaintenanceModel) -> String {
    let status = if model.available {
        "当前环境可直接执行"
    } else {
        "当前环境需切换场景或权限"
    };

    [
        "工作流：Repository Maintenance".to_string(),
        format!("当前动作：{}", model.summary.current_action.label()),
        format!("最近结果：{}", model.latest_result),
        format!("状态：{status}"),
        format!(
            "命令预览：{}",
            model
                .command_preview
                .clone()
                .unwrap_or_else(|| "无".to_string())
        ),
        format!("主要写回：{}", model.write_target),
        format!("说明：{}", model.summary.current_action.description()),
        format!("下一步：{}", model.return_hint),
        "操作：J/K 选择高级动作  x/X 执行当前高级动作  b 返回 Apply".to_string(),
    ]
    .join("\n")
}

pub(super) fn render_advanced_wizard_detail_lines(model: &AdvancedWizardDetailModel) -> String {
    [
        "工作流：Deploy Wizard".to_string(),
        format!("当前动作：{}", model.action.label()),
        format!("最近结果：{}", model.latest_result),
        format!("状态：{}", model.status),
        format!("命令预览：{}", model.command_preview),
        "用途：处理远端来源、初始化和复杂交互。".to_string(),
        format!("下一步：{}", model.completion_hint),
        "操作：j/k 选 deploy 参数  J/K 选高级动作  x/X 执行  b 返回 Apply".to_string(),
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{
        render_view_text, test_advanced_wizard_model, test_app_state,
    };
    use super::*;
    use crate::domain::tui::{ActionItem, DeploySource, DeployTask};
    use crate::tui::state::DeployPageModel;
    use ratatui::layout::Rect;

    fn focus_launch_deploy_wizard(state: &mut crate::tui::state::AppState) {
        state.advanced_action = ActionItem::LaunchDeployWizard;
    }

    fn compact_rendered_text(text: &str) -> String {
        text.chars().filter(|c| !c.is_whitespace()).collect()
    }

    #[test]
    fn current_selection_lines_use_advanced_wizard_focus_in_advanced_area() {
        let mut state = test_app_state();
        state.open_advanced();
        state.advanced_action = ActionItem::LaunchDeployWizard;
        state.advanced_deploy_focus = 1;
        let wizard = state.advanced_wizard_model();
        let context = state.advanced_context_model();

        let text = render_advanced_context_lines(&context);

        assert!(text.contains("当前聚焦：任务 = 直接部署这台机器"));
        assert_eq!(wizard.task, DeployTask::DirectDeploy);
    }

    #[test]
    fn advanced_summary_lines_surface_current_task_reason_and_return_path() {
        let mut state = test_app_state();
        state.open_advanced();
        focus_launch_deploy_wizard(&mut state);
        state.advanced_deploy_source = DeploySource::RemoteHead;
        let wizard = state.advanced_wizard_model();
        let summary = state.advanced_summary_model();
        let detail = state.advanced_wizard_detail_model();

        let text = render_advanced_summary_lines(&summary, &wizard, &detail.latest_result);

        assert!(text.contains("当前动作：launch deploy wizard"));
        assert!(text.contains("推荐动作：launch deploy wizard"));
        assert!(text.contains("原因：当前来源是远端最新版本"));
        assert!(text.contains("下一步：做完后回 Apply 或 Overview"));
    }

    #[test]
    fn advanced_repository_context_lines_switch_to_repo_context_for_maintenance_actions() {
        let mut state = test_app_state();
        state.open_advanced();
        state.ensure_advanced_action_focus();
        let maintenance = state.advanced_maintenance_model();

        let text = render_advanced_repository_context_lines(&maintenance);

        assert!(text.contains("命令预览：nix --extra-experimental-features 'nix-command flakes' flake update --flake /repo"));
        assert!(text.contains("主要写回：flake.lock"));
        assert!(text.contains("仓库根：/repo"));
    }

    #[test]
    fn advanced_workspace_lines_surface_selected_action_and_preview() {
        let mut state = test_app_state();
        state.open_advanced();
        state.ensure_advanced_action_focus();
        state.next_advanced_action();
        let maintenance = state.advanced_maintenance_model();

        let text = render_advanced_maintenance_detail_lines(&maintenance);

        assert!(text.contains("当前动作：update upstream pins"));
        assert!(text.contains("命令预览：update-upstream-apps"));
        assert!(text.contains("工作流：Repository Maintenance"));
        assert!(text.contains("x/X 执行当前高级动作  b 返回 Apply"));
    }

    #[test]
    fn advanced_workspace_lines_surface_maintenance_specific_detail_language() {
        let mut state = test_app_state();
        state.open_advanced();
        state.ensure_advanced_action_focus();
        let maintenance = state.advanced_maintenance_model();

        let text = render_advanced_maintenance_detail_lines(&maintenance);

        assert!(text.contains("工作流：Repository Maintenance"));
        assert!(text.contains("主要写回：flake.lock"));
        assert!(text.contains("说明：更新当前仓库的 flake.lock。"));
        assert!(text.contains("下一步：完成后回 Inspect 或 Overview 复查"));
    }

    #[test]
    fn advanced_workspace_lines_surface_maintenance_latest_result_after_completion() {
        let mut state = test_app_state();
        state.open_advanced();
        state.ensure_advanced_action_focus();
        state.set_advanced_maintenance_completion_feedback(ActionItem::FlakeUpdate);
        let maintenance = state.advanced_maintenance_model();

        let text = render_advanced_maintenance_detail_lines(&maintenance);

        assert!(
            text.contains("下一步：做完后回 Inspect 或 Overview 复查仓库状态，必要时再回 Apply。")
        );
        assert!(text.contains("最近结果：flake update 已完成。 下一步：做完后回 Inspect 或 Overview 复查仓库状态，必要时再回 Apply。"));
    }

    #[test]
    fn advanced_workspace_lines_surface_wizard_specific_detail_language() {
        let mut state = test_app_state();
        state.open_advanced();
        focus_launch_deploy_wizard(&mut state);
        state.advanced_deploy_source = DeploySource::RemoteHead;
        let detail = state.advanced_wizard_detail_model();

        let text = render_advanced_wizard_detail_lines(&detail);

        assert!(text.contains("工作流：Deploy Wizard"));
        assert!(text.contains("当前动作：launch deploy wizard"));
        assert!(text.contains("用途：处理远端来源、初始化和复杂交互。"));
        assert!(text.contains("下一步：做完后回 Apply 或 Overview 检查默认路径、健康和下一步。"));
        assert!(text.contains("操作：j/k 选 deploy 参数  J/K 选高级动作  x/X 执行  b 返回 Apply"));
    }

    #[test]
    fn advanced_workspace_lines_surface_wizard_completion_feedback_when_active() {
        let mut state = test_app_state();
        state.open_advanced();
        focus_launch_deploy_wizard(&mut state);
        state.set_deploy_wizard_return_feedback();

        let detail = state.advanced_wizard_detail_model();
        let text = render_advanced_wizard_detail_lines(&detail);

        assert!(text.contains("下一步：继续在 Advanced 完成复杂部署"));
        assert!(
            text.contains("最近结果：已返回完整部署向导。 下一步：继续在 Advanced 完成复杂部署")
        );
    }

    #[test]
    fn advanced_preview_lines_surface_deploy_preview_for_wizard_action() {
        let mut state = test_app_state();
        state.open_advanced();
        focus_launch_deploy_wizard(&mut state);
        let wizard = state.advanced_wizard_model();

        let text = render_advanced_wizard_preview_lines(&wizard);

        assert!(text.contains("入口：Advanced 区负责复杂部署、仓库维护和专家动作。"));
        assert!(text.contains("目标主机：nixos"));
        assert!(text.contains("返回路径：如果只想回默认应用路径，按 b 返回 Apply。"));
    }

    #[test]
    fn render_advanced_maintenance_page_builds_maintenance_screen_entry() {
        let mut state = test_app_state();
        state.open_advanced();
        state.ensure_advanced_action_focus();
        let page = match state.deploy_page_model() {
            DeployPageModel::AdvancedMaintenance(model) => model,
            other => panic!("expected advanced maintenance page model, got {other:?}"),
        };

        let text = render_view_text(120, 40, |frame| {
            render_advanced_maintenance_page(frame, Rect::new(0, 0, 120, 40), page.as_ref())
        });

        assert!(text.contains("Advanced Summary"));
        assert!(text.contains("Maintenance Preview"));
        assert!(text.contains("Advanced Context"));
        assert!(text.contains("Repository Context"));
        assert!(text.contains("Maintenance Detail"));
        assert!(text.contains("Advanced Actions"));
    }

    #[test]
    fn advanced_maintenance_page_model_keeps_sections_aligned_with_selected_repo_action() {
        let mut state = test_app_state();
        state.open_advanced();
        state.ensure_advanced_action_focus();
        state.next_advanced_action();
        let page = match state.deploy_page_model() {
            DeployPageModel::AdvancedMaintenance(model) => model,
            other => panic!("expected advanced maintenance page model, got {other:?}"),
        };

        let summary = render_advanced_maintenance_summary_lines(&page.maintenance);
        let preview = render_advanced_maintenance_preview_lines(&page.maintenance);
        let context = render_advanced_context_lines(&page.context);
        let repository_context = render_advanced_repository_context_lines(&page.maintenance);
        let detail = render_advanced_maintenance_detail_lines(&page.maintenance);

        assert!(summary.contains("当前动作：update upstream pins"));
        assert!(summary.contains("推荐动作：update upstream pins"));
        assert!(context.contains("当前聚焦：动作 = update upstream pins"));
        assert!(preview.contains("当前动作：update upstream pins"));
        assert!(detail.contains("当前动作：update upstream pins"));
        assert!(preview.contains("主要写回：source.nix / upstream pins"));
        assert!(repository_context.contains("主要写回：source.nix / upstream pins"));
        assert!(detail.contains("主要写回：source.nix / upstream pins"));
    }

    #[test]
    fn advanced_maintenance_page_model_keeps_handoff_recommendation_aligned() {
        let mut state = test_app_state();
        state.open_advanced();
        state.ensure_advanced_action_focus();
        state.advanced_deploy_source = DeploySource::RemoteHead;
        let page = match state.deploy_page_model() {
            DeployPageModel::AdvancedMaintenance(model) => model,
            other => panic!("expected advanced maintenance page model, got {other:?}"),
        };

        let summary = render_advanced_maintenance_summary_lines(&page.maintenance);
        let context = render_advanced_context_lines(&page.context);
        let repository_context = render_advanced_repository_context_lines(&page.maintenance);
        let detail = render_advanced_maintenance_detail_lines(&page.maintenance);

        assert!(summary.contains("当前动作：flake update"));
        assert!(summary.contains("推荐动作：launch deploy wizard"));
        assert!(context.contains("建议：先切到 launch deploy wizard"));
        assert!(
            repository_context
                .contains("下一步：完成后回 Inspect / Overview 复查；如果要继续复杂部署，切到 launch deploy wizard。")
        );
        assert!(
            detail.contains(
                "下一步：完成后回 Inspect / Overview 复查；如果要继续复杂部署，切到 launch deploy wizard。"
            )
        );
    }

    #[test]
    fn render_advanced_maintenance_page_surfaces_completion_hint_and_latest_result_together() {
        let mut state = test_app_state();
        state.open_advanced();
        state.ensure_advanced_action_focus();
        state.set_advanced_maintenance_completion_feedback(ActionItem::FlakeUpdate);
        let page = match state.deploy_page_model() {
            DeployPageModel::AdvancedMaintenance(model) => model,
            other => panic!("expected advanced maintenance page model, got {other:?}"),
        };

        let text = render_view_text(120, 40, |frame| {
            render_advanced_maintenance_page(frame, Rect::new(0, 0, 120, 40), page.as_ref())
        });
        let compact = compact_rendered_text(&text);

        assert!(
            compact.contains("下一步：做完后回Inspect或Overview复查仓库状态，必要时再回Apply。")
        );
        assert!(compact.contains("最近结果：flakeupdate已完成。"));
    }

    #[test]
    fn render_advanced_wizard_page_builds_wizard_screen_entry() {
        let mut state = test_app_state();
        state.open_advanced();
        focus_launch_deploy_wizard(&mut state);
        let page = match state.deploy_page_model() {
            DeployPageModel::AdvancedWizard(model) => model,
            other => panic!("expected advanced wizard page model, got {other:?}"),
        };

        let text = render_view_text(120, 40, |frame| {
            render_advanced_wizard_page(frame, Rect::new(0, 0, 120, 40), page.as_ref())
        });

        assert!(text.contains("Advanced Summary"));
        assert!(text.contains("Deploy Preview"));
        assert!(text.contains("Advanced Context"));
        assert!(text.contains("Deploy Parameters"));
        assert!(text.contains("Deploy Wizard Detail"));
        assert!(text.contains("Advanced Actions"));
    }

    #[test]
    fn render_advanced_wizard_page_surfaces_remote_pin_focus_across_screen_sections() {
        let mut state = test_app_state();
        state.open_advanced();
        focus_launch_deploy_wizard(&mut state);
        state.advanced_deploy_source = DeploySource::RemotePinned;
        state.advanced_deploy_source_ref = "v5.0.0".to_string();
        state.advanced_deploy_focus = 3;
        let page = match state.deploy_page_model() {
            DeployPageModel::AdvancedWizard(model) => model,
            other => panic!("expected advanced wizard page model, got {other:?}"),
        };

        let text = render_view_text(120, 40, |frame| {
            render_advanced_wizard_page(frame, Rect::new(0, 0, 120, 40), page.as_ref())
        });
        let compact = compact_rendered_text(&text);

        assert!(text.contains("Deploy Preview"));
        assert!(compact.contains("远端固定版本"));
        assert!(compact.contains("固定ref=v5.0.0"));
        assert!(compact.contains("当前聚焦：固定ref=v5.0.0"));
        assert!(compact.contains("当前动作：launchdeploywizard"));
        assert!(compact.contains("mcb-deploy"));
    }

    #[test]
    fn render_advanced_wizard_page_surfaces_remote_head_recommendation_across_screen_sections() {
        let mut state = test_app_state();
        state.open_advanced();
        focus_launch_deploy_wizard(&mut state);
        state.advanced_deploy_source = DeploySource::RemoteHead;
        state.advanced_deploy_focus = 2;
        let page = match state.deploy_page_model() {
            DeployPageModel::AdvancedWizard(model) => model,
            other => panic!("expected advanced wizard page model, got {other:?}"),
        };

        let text = render_view_text(120, 40, |frame| {
            render_advanced_wizard_page(frame, Rect::new(0, 0, 120, 40), page.as_ref())
        });
        let compact = compact_rendered_text(&text);

        assert!(text.contains("Advanced Summary"));
        assert!(compact.contains("推荐动作：launchdeploywizard"));
        assert!(compact.contains("当前来源是远端最新版本"));
        assert!(compact.contains("当前动作就是默认推荐，可直接按x/X。"));
        assert!(text.contains("Deploy Wizard Detail"));
    }

    #[test]
    fn advanced_wizard_page_model_keeps_context_and_controls_aligned_with_parameter_focus() {
        let mut state = test_app_state();
        state.open_advanced();
        focus_launch_deploy_wizard(&mut state);
        state.advanced_deploy_source = DeploySource::RemotePinned;
        state.advanced_deploy_source_ref = "v5.0.0".to_string();
        state.advanced_deploy_focus = 3;
        let page = match state.deploy_page_model() {
            DeployPageModel::AdvancedWizard(model) => model,
            other => panic!("expected advanced wizard page model, got {other:?}"),
        };

        let context = render_advanced_context_lines(&page.context);
        let detail = render_advanced_wizard_detail_lines(&page.detail);

        assert_eq!(page.controls.focused_row.label, "固定 ref");
        assert_eq!(page.controls.focused_row.value, "v5.0.0");
        assert!(context.contains("当前聚焦：固定 ref = v5.0.0"));
        assert!(context.contains("默认目标：先选右侧高级动作，再决定是否调整左侧向导参数"));
        assert!(context.contains("默认执行：高级动作优先"));
        assert!(detail.contains("当前动作：launch deploy wizard"));
        assert!(detail.contains("命令预览：mcb-deploy"));
    }

    #[test]
    fn advanced_wizard_page_model_keeps_handoff_recommendation_aligned_across_sections() {
        let mut state = test_app_state();
        state.open_advanced();
        focus_launch_deploy_wizard(&mut state);
        state.advanced_deploy_source = DeploySource::RemoteHead;
        state.advanced_deploy_focus = 2;
        let page = match state.deploy_page_model() {
            DeployPageModel::AdvancedWizard(model) => model,
            other => panic!("expected advanced wizard page model, got {other:?}"),
        };

        let summary =
            render_advanced_summary_lines(&page.summary, &page.wizard, &page.detail.latest_result);
        let context = render_advanced_context_lines(&page.context);
        let detail = render_advanced_wizard_detail_lines(&page.detail);

        assert!(summary.contains("当前动作：launch deploy wizard"));
        assert!(summary.contains("推荐动作：launch deploy wizard"));
        assert!(summary.contains("原因：当前来源是远端最新版本"));
        assert!(context.contains("当前聚焦：来源 = 远端最新版本"));
        assert!(context.contains("建议：当前动作就是默认推荐，可直接按 x/X。"));
        assert!(detail.contains("当前动作：launch deploy wizard"));
        assert!(detail.contains("下一步：做完后回 Apply 或 Overview 检查默认路径、健康和下一步。"));
    }

    #[test]
    fn advanced_wizard_preview_lines_use_wizard_model_without_apply_only_fields() {
        let mut wizard = test_advanced_wizard_model();
        wizard.source = DeploySource::RemotePinned;
        wizard.source_detail = Some("v5.0.0".to_string());
        wizard.rebuild_preview = None;

        let text = render_advanced_wizard_preview_lines(&wizard);

        assert!(text.contains("来源：远端固定版本"));
        assert!(text.contains("来源细节：v5.0.0"));
        assert!(text.contains("命令预览：mcb-deploy"));
    }

    #[test]
    fn advanced_preview_lines_surface_maintenance_preview_for_repo_actions() {
        let mut state = test_app_state();
        state.open_advanced();
        state.ensure_advanced_action_focus();
        let maintenance = state.advanced_maintenance_model();

        let text = render_advanced_maintenance_preview_lines(&maintenance);

        assert!(text.contains("入口：当前动作属于仓库维护路径"));
        assert!(text.contains("当前动作：flake update"));
        assert!(text.contains("主要写回：flake.lock"));
        assert!(text.contains("影响：会刷新 flake.lock"));
    }

    #[test]
    fn advanced_context_lines_recommend_switching_to_default_task_when_needed() {
        let mut state = test_app_state();
        state.open_advanced();
        state.advanced_deploy_source = DeploySource::RemoteHead;
        state.advanced_action = ActionItem::FlakeUpdate;
        let context = state.advanced_context_model();

        let text = render_advanced_context_lines(&context);

        assert!(text.contains("建议：先切到 launch deploy wizard"));
    }

    #[test]
    fn advanced_context_lines_hide_deploy_parameter_language_for_repo_maintenance() {
        let mut state = test_app_state();
        state.open_advanced();
        state.ensure_advanced_action_focus();
        let context = state.advanced_context_model();

        let text = render_advanced_context_lines(&context);

        assert!(text.contains("当前动作是仓库维护"));
        assert!(text.contains("当前动作不使用 deploy 参数"));
        assert!(text.contains("j/k 或 J/K 切高级动作"));
    }
}
