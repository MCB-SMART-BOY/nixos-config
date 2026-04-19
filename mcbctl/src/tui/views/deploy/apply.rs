use crate::tui::state::{ApplyExecutionGateModel, ApplyModel, ApplyPageModel, ApplySelectionModel};
use crate::tui::views::summary::render_mainline_summary;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use super::chrome::{DeployLayoutAreas, render_deploy_controls_list};
use super::shared::join_or_none;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ApplySelectionDensity {
    Standard,
    Compact,
    Tight,
}

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
        Paragraph::new(render_apply_current_selection_lines_with_density(
            &model.selection,
            selection_density_for_area(layout.context),
        ))
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
}

pub(super) fn render_execution_gate_lines(model: &ApplyExecutionGateModel) -> String {
    let mut lines = vec![render_mainline_summary(
        &compact_apply_status(&model.status),
        &compact_apply_latest_result(&model.latest_result),
        &compact_apply_next_step(&model.next_step),
        &compact_apply_primary_action(&model.primary_action),
        &[],
    )];
    lines.push(render_gate_category_line("blocker", &model.blockers));
    lines.push(render_gate_category_line("warning", &model.warnings));
    lines.push(render_gate_category_line("handoff", &model.handoffs));
    lines.push(render_gate_category_line("info", &model.infos));
    lines.join("\n")
}

pub(super) fn render_plan_preview_lines(
    apply: &ApplyModel,
    command_fallback: &str,
    _advanced_entry: bool,
) -> String {
    let mut lines = vec![
        format!("目标：{}", apply.target_host),
        format!("任务：{}", apply.task.label()),
        format!("来源：{}", apply.source.label()),
    ];
    if let Some(detail) = apply.source_detail.as_deref() {
        lines.push(format!("来源细节：{detail}"));
    } else if matches!(apply.source, crate::domain::tui::DeploySource::RemotePinned) {
        lines.push("来源细节：未设置固定 ref/pin".to_string());
    }
    lines.extend([
        format!("动作：{}", apply.action.label()),
        format!(
            "升级：{}",
            if apply.flake_update {
                "开启"
            } else {
                "关闭"
            }
        ),
        format!(
            "同步：{}",
            compact_sync_preview(apply.sync_preview.as_deref())
        ),
        format!(
            "执行：{}",
            compact_command_preview(apply.rebuild_preview.as_deref().unwrap_or(command_fallback))
        ),
    ]);
    lines.join("\n")
}

#[cfg(test)]
pub(super) fn render_apply_current_selection_lines(model: &ApplySelectionModel) -> String {
    render_apply_current_selection_lines_with_density(model, ApplySelectionDensity::Standard)
}

fn render_apply_current_selection_lines_with_density(
    model: &ApplySelectionModel,
    density: ApplySelectionDensity,
) -> String {
    match density {
        ApplySelectionDensity::Standard => [
            model.recommendation.clone(),
            format!("执行状态：{}", model.execution_hint),
            format!(
                "当前聚焦：{} = {}",
                model.focused_row.label, model.focused_row.value
            ),
            model.advanced_action_hint.clone(),
        ]
        .join("\n"),
        ApplySelectionDensity::Compact => [
            compact_selection_recommendation(&model.recommendation),
            format!(
                "状态/焦点：{} | {}",
                compact_execution_hint(&model.execution_hint),
                compact_focus(&model.focused_row)
            ),
            compact_advanced_hint(&model.advanced_action_hint),
        ]
        .join("\n"),
        ApplySelectionDensity::Tight => [
            format!(
                "建议/状态：{} | {}",
                compact_selection_recommendation(&model.recommendation),
                compact_execution_hint(&model.execution_hint)
            ),
            format!(
                "焦点/Advanced：{} | {}",
                compact_focus(&model.focused_row),
                compact_advanced_hint(&model.advanced_action_hint)
            ),
        ]
        .join("\n"),
    }
}

fn selection_density_for_area(area: Rect) -> ApplySelectionDensity {
    if area.height <= 4 {
        ApplySelectionDensity::Tight
    } else if area.height <= 5 {
        ApplySelectionDensity::Compact
    } else {
        ApplySelectionDensity::Standard
    }
}

fn compact_selection_recommendation(recommendation: &str) -> String {
    if recommendation == "建议：左侧预览已可直接执行；确认无误后可按 x Apply。"
    {
        return "建议：可直接 Apply".to_string();
    }
    if recommendation.starts_with("建议：当前默认应切到 Advanced 执行 ") {
        return "建议：进 Advanced".to_string();
    }
    if recommendation == "建议：先看 blocker / warning，再决定是否直接 Apply。" {
        return "建议：先处理 blocker".to_string();
    }
    if recommendation == "建议：先看左侧预览和执行门槛，再决定是否直接 Apply。"
    {
        return "建议：先看预览".to_string();
    }
    recommendation.to_string()
}

fn compact_execution_hint(execution_hint: &str) -> String {
    match execution_hint {
        "可执行：当前组合可直接 Apply" => "可执行".to_string(),
        "不可执行：当前仍有 blocker" => "不可执行".to_string(),
        "待确认：先看预览和执行门槛" => "待确认".to_string(),
        hint if hint.starts_with("需交接：") => "需交接".to_string(),
        other => other.to_string(),
    }
}

fn compact_focus(row: &crate::tui::state::DeployControlRow) -> String {
    format!("{}={}", compact_focus_label(&row.label), row.value)
}

fn compact_focus_label(label: &str) -> &str {
    match label {
        "目标主机" => "主机",
        "固定 ref" => "ref",
        "flake update" => "升级",
        "区域切换" => "区域",
        other => other,
    }
}

fn compact_advanced_hint(advanced_hint: &str) -> String {
    if advanced_hint == "高级动作：如需完整向导或仓库维护，切到最后一行并按 Enter 进入 Advanced"
    {
        return "Advanced：Enter".to_string();
    }
    if advanced_hint.starts_with("高级动作：按 Enter 进入 Advanced，然后执行 ") {
        return "Advanced：Enter".to_string();
    }
    if advanced_hint == "高级动作：如需完整向导或仓库维护，修复 blocker 后再进入 Advanced"
    {
        return "Advanced：修 blocker 后进".to_string();
    }
    advanced_hint.to_string()
}

fn render_gate_category_line(label: &str, items: &[String]) -> String {
    format!("{label}：{}", summarize_gate_items(items))
}

fn summarize_gate_items(items: &[String]) -> String {
    if items.is_empty() {
        return join_or_none(items);
    }

    let first = compact_gate_item(&items[0]);
    if items.len() == 1 {
        first
    } else {
        format!("{first}（另 {} 项）", items.len() - 1)
    }
}

fn compact_gate_item(item: &str) -> String {
    if let Some(rest) = item.strip_prefix("仍有未保存修改；请先保存后再执行：") {
        return format!("先保存：{rest}");
    }
    if let Some(rest) = item.strip_prefix("repo-integrity 当前失败：") {
        return format!("repo-integrity 失败：{rest}");
    }
    if let Some(rest) = item.strip_prefix("doctor 当前失败：") {
        return format!("doctor 失败：{rest}");
    }
    if let Some(rest) = item.strip_prefix("主机 ") {
        if let Some((host, error)) = rest.split_once(" 的 TUI 配置未通过校验：") {
            return format!("{host} 校验失败：{error}");
        }
    }
    if item
        == "rootless 模式下当前页只能直接执行 build；如需 switch/test/boot，请使用 sudo/root 或退回 deploy wizard。"
    {
        return "rootless 仅允许 build".to_string();
    }
    if let Some(rest) = item.strip_prefix("当前组合会先把仓库同步到 /etc/nixos：") {
        return format!("同步 /etc/nixos：{}", compact_sync_preview(Some(rest)));
    }
    if item == "当前组合会以 --upgrade 执行重建。" {
        return "--upgrade 重建".to_string();
    }
    if item == "当前组合会使用 sudo -E 执行受权命令。" {
        return "sudo -E 执行".to_string();
    }
    if let Some(rest) = item.strip_prefix("当前组合要求 ") {
        if let Some(path) = rest.strip_suffix(" 存在真实 hardware-configuration.nix。") {
            return format!("hardware-config：{path}");
        }
    }
    if item == "当前来源是远端固定版本；默认 Apply 不会直接执行，必须交给完整高级路径。"
    {
        return "远端固定版本 -> Advanced".to_string();
    }
    if item == "当前来源是远端最新版本；默认 Apply 不会直接执行，必须交给完整高级路径。"
    {
        return "远端最新版本 -> Advanced".to_string();
    }
    if item == "当前组合不会直接执行，而是回退到完整 deploy wizard。" {
        return "执行路径：wizard".to_string();
    }
    if let Some(rest) = item.strip_prefix("检测 hostname：") {
        return format!("hostname={rest}");
    }
    item.to_string()
}

fn compact_sync_preview(sync_preview: Option<&str>) -> String {
    match sync_preview {
        Some(preview) => compact_sync_command(preview).unwrap_or_else(|| preview.to_string()),
        None => "不需要".to_string(),
    }
}

fn compact_sync_command(command: &str) -> Option<String> {
    let parts = command.split_whitespace().collect::<Vec<_>>();
    match parts.as_slice() {
        ["sudo", "rsync", source, target, ..] | ["rsync", source, target, ..] => {
            Some(format!("{source} -> {target}"))
        }
        _ => None,
    }
}

fn compact_command_preview(command: &str) -> String {
    if command == "当前组合可直接执行 Apply" {
        return "可直接 Apply".to_string();
    }
    if let Some(rest) = command.strip_prefix("当前组合会转交给 Advanced 执行 ") {
        return format!("交给 Advanced：{rest}");
    }
    if command == "当前组合暂不生成直接命令预览；请先处理 blocker / warning" {
        return "先处理 blocker / warning".to_string();
    }
    if command == "当前组合暂不生成直接命令预览；请先确认预览和执行门槛" {
        return "先确认预览和门槛".to_string();
    }
    command.replacen("sudo -E env ", "sudo -E ", 1)
}

fn compact_apply_status(status: &str) -> String {
    match status {
        "当前组合应转交给 Advanced" => "当前应转交 Advanced".to_string(),
        "当前待确认 Apply 门槛" => "当前待确认门槛".to_string(),
        other => other.to_string(),
    }
}

fn compact_apply_latest_result(latest_result: &str) -> String {
    if latest_result == "暂无" {
        return latest_result.to_string();
    }
    if latest_result.starts_with("Apply 已执行完成：") {
        let first_sentence = latest_result.split("。").next().unwrap_or(latest_result);
        return first_sentence.replacen("Apply 已执行完成：", "Apply 完成：", 1);
    }
    if latest_result.starts_with("Overview 已进入 Apply 预览；") {
        return "已进入 Apply 预览".to_string();
    }
    if latest_result.starts_with("已从 Advanced 返回 Apply；") {
        return "已从 Advanced 返回".to_string();
    }
    latest_result.to_string()
}

fn compact_apply_next_step(next_step: &str) -> String {
    if next_step == "在 Apply 查看预览；确认后按 x 直接运行" {
        return "看预览，确认后按 x".to_string();
    }
    if next_step.starts_with("在 Apply 先看 handoff 预览；如需继续，切到 Advanced 执行 ")
    {
        return "先看 handoff，需要时进 Advanced".to_string();
    }
    if next_step == "在 Apply 先看 blocker / warning，再决定是否调整 Apply 项" {
        return "先看 blocker / warning".to_string();
    }
    if next_step == "在 Apply 查看预览并决定下一步" {
        return "先看预览".to_string();
    }
    if next_step == "回到 Overview 检查健康和下一步" {
        return "回 Overview 复查".to_string();
    }
    next_step.to_string()
}

fn compact_apply_primary_action(primary_action: &str) -> String {
    let Some(value) = primary_action.strip_prefix("主动作：") else {
        return primary_action.to_string();
    };

    let compacted = if value == "按 x 立即执行当前 Apply" {
        "按 x 执行".to_string()
    } else if value.starts_with("切到 Advanced 执行 ") {
        "进 Advanced".to_string()
    } else if value == "先修复阻塞项，再回到 Apply" {
        "先修复 blocker".to_string()
    } else if value == "先确认预览和执行门槛" {
        "先确认预览".to_string()
    } else {
        value.to_string()
    };

    format!("主动作：{compacted}")
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
            next_step: "回到 Overview 检查健康和下一步".to_string(),
            primary_action: "主动作：按 x 立即执行当前 Apply".to_string(),
            blockers: Vec::new(),
            warnings: vec!["当前组合会使用 sudo -E 执行受权命令。".to_string()],
            handoffs: Vec::new(),
            infos: vec!["检测 hostname：nixos".to_string()],
        };

        let text = render_execution_gate_lines(&gate);

        let primary_pos = text
            .find("主动作：按 x 执行")
            .expect("primary action should render");
        let status_pos = text
            .find("当前判断：当前可直接 Apply")
            .expect("status should render");
        let result_pos = text
            .find("最近结果：Apply 完成：switch nixos")
            .expect("latest result should render");
        let next_step_pos = text
            .find("下一步：回 Overview 复查")
            .expect("next step should render");

        assert!(status_pos < result_pos);
        assert!(result_pos < next_step_pos);
        assert!(next_step_pos < primary_pos);
        assert!(text.contains("当前判断：当前可直接 Apply"));
        assert!(text.contains("最近结果：Apply 完成：switch nixos"));
        assert!(text.contains("下一步：回 Overview 复查"));
        assert!(text.contains("主动作：按 x 执行"));
        assert!(text.contains("warning：sudo -E 执行"));
    }

    #[test]
    fn execution_gate_lines_surface_handoffs_with_recommended_advanced_action() {
        let mut state = test_app_state();
        state.deploy_source = DeploySource::RemotePinned;
        state.deploy_source_ref = "v5.0.0".to_string();
        let gate = state.apply_execution_gate_model();

        let text = render_execution_gate_lines(&gate);

        assert!(text.contains("当前判断：当前应转交 Advanced"));
        assert!(text.contains("下一步：先看 handoff，需要时进 Advanced"));
        assert!(text.contains("主动作：进 Advanced"));
        assert!(text.contains("handoff：远端固定版本 -> Advanced"));
    }

    #[test]
    fn execution_gate_lines_keep_blocker_next_step_ahead_of_handoff_copy() {
        let mut state = test_app_state();
        state.deploy_source = DeploySource::RemotePinned;
        state.deploy_source_ref = "v5.0.0".to_string();
        state.overview_repo_integrity = crate::tui::state::OverviewCheckState::Error {
            summary: "failed (1 finding(s))".to_string(),
            details: vec!["- [rule] path: detail".to_string()],
        };
        let gate = state.apply_execution_gate_model();

        let text = render_execution_gate_lines(&gate);

        assert!(text.contains("当前判断：当前不能直接 Apply"));
        assert!(text.contains("下一步：先看 blocker / warning"));
        assert!(text.contains("主动作：先修复 blocker"));
        assert!(text.contains("handoff：远端固定版本 -> Advanced"));
    }

    #[test]
    fn plan_preview_lines_keep_sync_and_rebuild_previews_visible() {
        let apply = test_apply_model();
        let text = render_plan_preview_lines(&apply, "当前组合可直接执行 Apply", false);

        assert!(text.contains("目标：nixos"));
        assert!(text.contains("来源：当前仓库"));
        assert!(text.contains("同步：/repo -> /etc/nixos"));
        assert!(text.contains("执行：sudo -E nixos-rebuild switch --flake /etc/nixos#nixos"));
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
        assert!(text.contains("执行：交给 Advanced：launch deploy wizard"));
    }

    #[test]
    fn current_selection_lines_highlight_focused_advanced_control() {
        let mut state = test_app_state();
        state.deploy_focus = 4;
        let selection = state.apply_selection_model();

        let text = render_apply_current_selection_lines(&selection);

        let recommendation_pos = text
            .find("建议：左侧预览已可直接执行；确认无误后可按 x Apply。")
            .expect("recommendation should render");
        let execution_pos = text
            .find("执行状态：可执行：当前组合可直接 Apply")
            .expect("execution status should render");
        let focus_pos = text
            .find("当前聚焦：动作 = switch")
            .expect("focus should render");
        let advanced_pos = text
            .find("高级动作：如需完整向导或仓库维护，切到最后一行并按 Enter 进入 Advanced")
            .expect("advanced action hint should render");

        assert!(recommendation_pos < execution_pos);
        assert!(execution_pos < focus_pos);
        assert!(focus_pos < advanced_pos);
        assert!(recommendation_pos < focus_pos);
        assert!(text.contains("当前聚焦：动作 = switch"));
        assert!(text.contains("建议：左侧预览已可直接执行；确认无误后可按 x Apply。"));
        assert!(!text.contains("默认目标：先看左侧预览，再决定是否调整右侧 Apply 项"));
        assert!(!text.contains("操作：j/k 选 Apply 项  h/l 或 Enter 调整"));
    }

    #[test]
    fn current_selection_lines_compact_to_three_lines_in_low_height() {
        let mut state = test_app_state();
        state.deploy_focus = 4;
        let selection = state.apply_selection_model();
        let focus = compact_focus(&selection.focused_row);

        let text = render_apply_current_selection_lines_with_density(
            &selection,
            ApplySelectionDensity::Compact,
        );

        assert!(text.contains("建议：可直接 Apply"));
        assert!(text.contains(&format!("状态/焦点：可执行 | {focus}")));
        assert!(text.contains("Advanced：Enter"));
        assert!(!text.contains("执行状态："));
        assert!(!text.contains("当前聚焦："));
        assert_eq!(text.lines().count(), 3);
    }

    #[test]
    fn current_selection_lines_collapse_to_two_lines_in_tight_height() {
        let mut state = test_app_state();
        state.deploy_focus = 4;
        let selection = state.apply_selection_model();
        let focus = compact_focus(&selection.focused_row);

        let text = render_apply_current_selection_lines_with_density(
            &selection,
            ApplySelectionDensity::Tight,
        );

        assert!(text.contains("建议/状态：建议：可直接 Apply | 可执行"));
        assert!(text.contains(&format!("焦点/Advanced：{focus} | Advanced：Enter")));
        assert!(!text.contains("执行状态："));
        assert!(!text.contains("当前聚焦："));
        assert_eq!(text.lines().count(), 2);
    }

    #[test]
    fn selection_density_uses_height_thresholds_for_compact_and_tight_modes() {
        assert_eq!(
            selection_density_for_area(Rect::new(0, 0, 32, 6)),
            ApplySelectionDensity::Standard
        );
        assert_eq!(
            selection_density_for_area(Rect::new(0, 0, 32, 5)),
            ApplySelectionDensity::Compact
        );
        assert_eq!(
            selection_density_for_area(Rect::new(0, 0, 32, 4)),
            ApplySelectionDensity::Tight
        );
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

        assert!(text.contains("Apply Summary"));
        assert!(text.contains("Apply Preview"));
        assert!(text.contains("Current Selection"));
        assert!(text.contains("Apply Controls"));
        assert!(!text.contains("Advanced Actions"));
        assert!(!text.contains("Advanced Detail"));
    }

    #[test]
    fn render_apply_page_stays_compact_after_returning_from_advanced() {
        let mut state = test_app_state();
        state.open_advanced();
        state.return_from_advanced_to_apply();
        let page = match state.deploy_page_model() {
            DeployPageModel::Apply(model) => model,
            other => panic!("expected apply page model, got {other:?}"),
        };

        let text = render_view_text(120, 40, |frame| {
            render_apply_page(frame, Rect::new(0, 0, 120, 40), page.as_ref())
        });

        assert!(text.contains("Apply Summary"));
        assert!(text.contains("Apply Controls"));
        assert!(!text.contains("Advanced Actions"));
        assert!(!text.contains("Advanced Detail"));
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

        assert!(preview.contains("执行：交给 Advanced：launch deploy wizard"));
        assert!(gate.contains("主动作：进 Advanced"));
        assert!(selection.contains(
            "建议：当前默认应切到 Advanced 执行 launch deploy wizard；先看 handoff 预览。"
        ));
    }

    #[test]
    fn render_apply_page_keeps_main_panels_scannable_in_short_body_area() {
        let page = match test_app_state().deploy_page_model() {
            DeployPageModel::Apply(model) => model,
            other => panic!("expected apply page model, got {other:?}"),
        };

        let text = render_view_text(100, 24, |frame| {
            render_apply_page(frame, Rect::new(0, 0, 100, 24), page.as_ref())
        });

        assert!(text.contains("Apply Summary"));
        assert!(text.contains("Apply Preview"));
        assert!(text.contains("Current Selection"));
        assert!(text.contains("Apply Controls"));
        assert!(text.contains("switch"));
        assert!(text.contains("/etc/nixos"));
        assert!(text.contains("sudo"));
    }

    #[test]
    fn render_apply_page_stacks_mainline_panels_cleanly_in_narrow_width() {
        let page = match test_app_state().deploy_page_model() {
            DeployPageModel::Apply(model) => model,
            other => panic!("expected apply page model, got {other:?}"),
        };

        let text = render_view_text(84, 24, |frame| {
            render_apply_page(frame, Rect::new(0, 0, 84, 24), page.as_ref())
        });

        assert!(text.contains("Apply Summary"));
        assert!(text.contains("Apply Preview"));
        assert!(text.contains("Current Selection"));
        assert!(text.contains("Apply Controls"));
        assert!(text.contains("switch"));
        assert!(text.contains("/etc/nixos"));
        assert!(text.contains("Advanced"));
    }

    #[test]
    fn render_apply_page_keeps_mainline_priority_in_low_height() {
        let page = match test_app_state().deploy_page_model() {
            DeployPageModel::Apply(model) => model,
            other => panic!("expected apply page model, got {other:?}"),
        };

        let text = render_view_text(120, 20, |frame| {
            render_apply_page(frame, Rect::new(0, 0, 120, 20), page.as_ref())
        });

        assert!(text.contains("Apply Summary"));
        assert!(text.contains("Apply Preview"));
        assert!(text.contains("Current Selection"));
        assert!(text.contains("Apply Controls"));
        assert!(text.contains("/etc/nixos"));
        assert!(text.contains("switch"));
        assert!(text.contains("sudo"));
    }

    #[test]
    fn gate_category_lines_compact_common_messages_and_merge_extra_items() {
        let text = render_gate_category_line(
            "warning",
            &[
                "当前组合会先把仓库同步到 /etc/nixos：sudo rsync /repo /etc/nixos".to_string(),
                "当前组合会使用 sudo -E 执行受权命令。".to_string(),
            ],
        );

        assert_eq!(
            text,
            "warning：同步 /etc/nixos：/repo -> /etc/nixos（另 1 项）"
        );
    }

    #[test]
    fn compact_command_preview_shortens_direct_apply_and_handoff_copy() {
        assert_eq!(
            compact_command_preview("当前组合可直接执行 Apply"),
            "可直接 Apply"
        );
        assert_eq!(
            compact_command_preview("当前组合会转交给 Advanced 执行 launch deploy wizard"),
            "交给 Advanced：launch deploy wizard"
        );
        assert_eq!(
            compact_command_preview("sudo -E env nixos-rebuild switch --flake /etc/nixos#nixos"),
            "sudo -E nixos-rebuild switch --flake /etc/nixos#nixos"
        );
    }

    #[test]
    fn compact_apply_summary_top_lines_shorten_route_and_review_copy() {
        assert_eq!(
            compact_apply_latest_result(
                "Overview 已进入 Apply 预览；当前组合仍有 blocker：repo-integrity 当前失败：failed (1 finding(s))。"
            ),
            "已进入 Apply 预览"
        );
        assert_eq!(
            compact_apply_latest_result(
                "已从 Advanced 返回 Apply；当前来源是远端固定版本；默认 Apply 不会直接执行，必须交给完整高级路径。"
            ),
            "已从 Advanced 返回"
        );
        assert_eq!(
            compact_apply_next_step("在 Apply 查看预览并决定下一步"),
            "先看预览"
        );
        assert_eq!(
            compact_apply_primary_action("主动作：先确认预览和执行门槛"),
            "主动作：先确认预览"
        );
        assert_eq!(
            compact_apply_status("当前组合应转交给 Advanced"),
            "当前应转交 Advanced"
        );
    }
}
