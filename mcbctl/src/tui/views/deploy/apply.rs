use crate::tui::state::{ApplyExecutionGateModel, ApplyModel, ApplyPageModel, ApplySelectionModel};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use super::advanced::render_advanced_workspace_lines;
use super::chrome::{DeployLayoutAreas, render_deploy_controls_list, render_workspace_section};
use super::shared::{DeployPreviewFields, join_or_none, render_deploy_preview_lines};

pub(super) fn render_apply_page(frame: &mut Frame, area: Rect, model: &ApplyPageModel) {
    let layout = DeployLayoutAreas::new(area, model.shell.workspace_visible);
    render_apply_layout(frame, &layout, model);
}

fn render_apply_layout(frame: &mut Frame, layout: &DeployLayoutAreas, model: &ApplyPageModel) {
    frame.render_widget(
        Paragraph::new(render_execution_gate_lines(&model.gate))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(model.shell.summary_title),
            )
            .wrap(Wrap { trim: false }),
        layout.preview_summary,
    );
    frame.render_widget(
        Paragraph::new(render_plan_preview_lines(
            &model.apply,
            &model.preview_command_fallback,
            false,
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
        Paragraph::new(render_apply_current_selection_lines(&model.selection))
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
        model
            .workspace
            .as_ref()
            .map(|workspace| render_advanced_workspace_lines(None, None, Some(workspace))),
        model.shell.detail_title,
    );
}

pub(super) fn render_execution_gate_lines(model: &ApplyExecutionGateModel) -> String {
    [
        format!("状态：{}", model.status),
        format!("最近结果：{}", model.latest_result),
        model.primary_action.clone(),
        format!("阻塞项：{}", join_or_none(&model.blockers)),
        format!("警告项：{}", join_or_none(&model.warnings)),
        format!("交接项：{}", join_or_none(&model.handoffs)),
        format!("信息：{}", join_or_none(&model.infos)),
    ]
    .join("\n")
}

pub(super) fn render_plan_preview_lines(
    apply: &ApplyModel,
    command_fallback: &str,
    advanced_entry: bool,
) -> String {
    render_deploy_preview_lines(DeployPreviewFields {
        advanced_entry,
        target_host: apply.target_host.as_str(),
        task: apply.task,
        source: apply.source,
        source_detail: apply.source_detail.as_deref(),
        action: apply.action,
        flake_update: apply.flake_update,
        advanced: apply.advanced,
        sync_preview: apply.sync_preview.as_deref(),
        rebuild_preview: apply.rebuild_preview.as_deref(),
        command_fallback,
    })
}

pub(super) fn render_apply_current_selection_lines(model: &ApplySelectionModel) -> String {
    [
        format!(
            "当前聚焦：{} = {}",
            model.focused_row.label, model.focused_row.value
        ),
        model.default_target.clone(),
        model.recommendation.clone(),
        format!("直接执行：{}", model.execution_hint),
        model.operation_hint.clone(),
        model.advanced_action_hint.clone(),
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{render_view_text, test_app_state, test_apply_model};
    use super::*;
    use crate::domain::tui::DeploySource;
    use crate::tui::state::DeployPageModel;
    use ratatui::layout::Rect;

    #[test]
    fn execution_gate_lines_prioritize_direct_apply_status() {
        let gate = ApplyExecutionGateModel {
            status: "当前可直接 Apply".to_string(),
            latest_result:
                "Apply 已执行完成：switch nixos。 下一步：回到 Overview 检查健康和下一步"
                    .to_string(),
            primary_action: "主动作：按 x 立即执行当前 Apply".to_string(),
            blockers: Vec::new(),
            warnings: vec!["当前组合会使用 sudo -E 执行受权命令。".to_string()],
            handoffs: Vec::new(),
            infos: vec!["检测 hostname：nixos".to_string()],
        };

        let text = render_execution_gate_lines(&gate);

        assert!(text.contains("状态：当前可直接 Apply"));
        assert!(text.contains(
            "最近结果：Apply 已执行完成：switch nixos。 下一步：回到 Overview 检查健康和下一步"
        ));
        assert!(text.contains("主动作：按 x 立即执行当前 Apply"));
        assert!(text.contains("警告项：当前组合会使用 sudo -E 执行受权命令。"));
    }

    #[test]
    fn execution_gate_lines_surface_handoffs_with_recommended_advanced_action() {
        let mut state = test_app_state();
        state.deploy_source = DeploySource::RemotePinned;
        state.deploy_source_ref = "v5.0.0".to_string();
        let gate = state.apply_execution_gate_model();

        let text = render_execution_gate_lines(&gate);

        assert!(text.contains("状态：当前组合应转交给 Advanced"));
        assert!(text.contains("主动作：切到 Advanced 执行 launch deploy wizard"));
        assert!(text.contains(
            "交接项：当前来源是远端固定版本；默认 Apply 不会直接执行，必须交给完整高级路径。"
        ));
    }

    #[test]
    fn execution_gate_lines_switch_to_workspace_primary_action_when_workspace_is_open() {
        let mut state = test_app_state();
        state.show_advanced = true;
        let gate = state.apply_execution_gate_model();

        let text = render_execution_gate_lines(&gate);

        assert!(text.contains("状态：当前已打开高级工作区"));
        assert!(text.contains("主动作：在右侧高级工作区选择动作并按 X 执行"));
    }

    #[test]
    fn plan_preview_lines_keep_sync_and_rebuild_previews_visible() {
        let apply = test_apply_model();
        let text = render_plan_preview_lines(&apply, "当前组合可直接执行 Apply", false);

        assert!(text.contains("目标主机：nixos"));
        assert!(text.contains("来源：当前仓库"));
        assert!(text.contains("同步预览：sudo rsync /repo /etc/nixos"));
        assert!(
            text.contains("命令预览：sudo -E env nixos-rebuild switch --flake /etc/nixos#nixos")
        );
    }

    #[test]
    fn plan_preview_lines_show_remote_pinned_source_detail() {
        let mut apply = test_apply_model();
        apply.source = DeploySource::RemotePinned;
        apply.source_detail = Some("v5.0.0".to_string());
        apply.sync_preview = None;
        apply.rebuild_preview = None;

        let text = render_plan_preview_lines(
            &apply,
            "当前组合会转交给 Advanced 执行 launch deploy wizard",
            true,
        );

        assert!(text.contains("来源：远端固定版本"));
        assert!(text.contains("来源细节：v5.0.0"));
        assert!(text.contains("命令预览：当前组合会转交给 Advanced 执行 launch deploy wizard"));
    }

    #[test]
    fn current_selection_lines_highlight_focused_advanced_control() {
        let mut state = test_app_state();
        state.deploy_focus = 4;
        let selection = state.apply_selection_model();

        let text = render_apply_current_selection_lines(&selection);

        assert!(text.contains("当前聚焦：动作 = switch"));
        assert!(text.contains("默认目标：先看左侧预览，再决定是否调整右侧 Apply 项"));
        assert!(text.contains("建议：先看 blocker / warning，再决定是否直接 Apply。"));
    }

    #[test]
    fn current_selection_lines_switch_to_workspace_language_when_apply_workspace_is_open() {
        let mut state = test_app_state();
        state.show_advanced = true;
        let selection = state.apply_selection_model();

        let text = render_apply_current_selection_lines(&selection);

        assert!(text.contains("直接执行：当前已打开高级工作区"));
        assert!(text.contains("高级动作：J/K 选择  X 执行  x 仍按当前 Apply 路径处理"));
    }

    #[test]
    fn render_apply_page_builds_apply_screen_without_workspace_when_hidden() {
        let page = match test_app_state().deploy_page_model() {
            DeployPageModel::Apply(model) => model,
            other => panic!("expected apply page model, got {other:?}"),
        };

        let text = render_view_text(180, 60, |frame| {
            render_apply_page(frame, Rect::new(0, 0, 180, 60), page.as_ref())
        });

        assert!(text.contains("Execution Gate"));
        assert!(text.contains("Apply Preview"));
        assert!(text.contains("Current Selection"));
        assert!(text.contains("Apply Controls"));
        assert!(!text.contains("Advanced Actions"));
        assert!(!text.contains("Advanced Detail"));
    }

    #[test]
    fn render_apply_page_builds_workspace_section_when_apply_advanced_is_visible() {
        let mut state = test_app_state();
        state.show_advanced = true;
        let page = match state.deploy_page_model() {
            DeployPageModel::Apply(model) => model,
            other => panic!("expected apply page model, got {other:?}"),
        };

        let text = render_view_text(120, 40, |frame| {
            render_apply_page(frame, Rect::new(0, 0, 120, 40), page.as_ref())
        });

        assert!(text.contains("Advanced Actions"));
        assert!(text.contains("Advanced Detail"));
        assert!(text.contains("Apply /"));
    }

    #[test]
    fn apply_sections_keep_preview_gate_and_selection_aligned_for_handoff() {
        let mut state = test_app_state();
        state.deploy_source = DeploySource::RemotePinned;
        state.deploy_source_ref = "v5.0.0".to_string();
        let page = match state.deploy_page_model() {
            DeployPageModel::Apply(model) => model,
            other => panic!("expected apply page model, got {other:?}"),
        };

        let preview = render_plan_preview_lines(&page.apply, &page.preview_command_fallback, false);
        let gate = render_execution_gate_lines(&page.gate);
        let selection = render_apply_current_selection_lines(&page.selection);

        assert!(preview.contains("命令预览：当前组合会转交给 Advanced 执行 launch deploy wizard"));
        assert!(gate.contains("主动作：切到 Advanced 执行 launch deploy wizard"));
        assert!(selection.contains(
            "建议：当前默认应切到 Advanced 执行 launch deploy wizard；先看 handoff 预览。"
        ));
    }
}
