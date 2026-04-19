use crate::domain::tui::{DeployAction, DeploySource, DeployTask};

pub(super) struct DeployPreviewFields<'a> {
    pub(super) advanced_entry: bool,
    pub(super) target_host: &'a str,
    pub(super) task: DeployTask,
    pub(super) source: DeploySource,
    pub(super) source_detail: Option<&'a str>,
    pub(super) action: DeployAction,
    pub(super) flake_update: bool,
    pub(super) sync_preview: Option<&'a str>,
    pub(super) rebuild_preview: Option<&'a str>,
    pub(super) command_fallback: &'a str,
}

pub(super) fn render_deploy_preview_lines(preview: DeployPreviewFields<'_>) -> String {
    let mut lines = Vec::new();
    if preview.advanced_entry {
        lines.push("入口：Advanced 区负责复杂部署、仓库维护和专家动作。".to_string());
        lines.push("返回路径：如果只想回默认应用路径，按 b 返回 Apply。".to_string());
    }
    lines.extend([
        format!("目标主机：{}", preview.target_host),
        format!("任务：{}", preview.task.label()),
        format!("来源：{}", preview.source.label()),
    ]);
    if let Some(detail) = preview.source_detail {
        lines.push(format!("来源细节：{detail}"));
    } else if preview.source == DeploySource::RemotePinned {
        lines.push("来源细节：未设置固定 ref/pin".to_string());
    }
    lines.extend([
        format!("动作：{}", preview.action.label()),
        format!(
            "flake update：{}",
            if preview.flake_update {
                "开启"
            } else {
                "关闭"
            }
        ),
        format!(
            "同步预览：{}",
            preview
                .sync_preview
                .unwrap_or("当前组合不需要同步 /etc/nixos")
        ),
        format!(
            "命令预览：{}",
            preview.rebuild_preview.unwrap_or(preview.command_fallback)
        ),
    ]);
    lines.join("\n")
}

pub(super) fn join_or_none(items: &[String]) -> String {
    if items.is_empty() {
        "无".to_string()
    } else {
        items.join(" | ")
    }
}
