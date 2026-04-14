use crate::tui::state::{AppState, OverviewCheckState, OverviewHostStatus};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

pub(super) fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let overview = state.overview_model();
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(46), Constraint::Percentage(54)])
        .split(area);

    let host_status = match &overview.host_status {
        OverviewHostStatus::Ready => "目标主机配置：可用".to_string(),
        OverviewHostStatus::Unavailable { message } => format!("目标主机配置：{message}"),
        OverviewHostStatus::Invalid { errors } => {
            format!("目标主机配置未通过校验：{}", errors.join("；"))
        }
    };
    let repo_integrity = render_check_lines("repo-integrity", &overview.repo_integrity).join("\n");
    let doctor = render_check_lines("doctor", &overview.doctor).join("\n");
    let apply_status = if overview.apply.can_apply_current_host {
        "当前可直接 Apply"
    } else if !overview.apply.handoffs.is_empty() {
        "当前组合需要交给 Advanced"
    } else {
        "当前不能直接 Apply"
    };
    let left = Paragraph::new(format!(
        "当前主机: {}\n目标主机: {}\n当前用户: {}\n权限模式: {}\n当前仓库: {}\n/etc/nixos: {}\n可用 hosts: {}\n可用用户: {}\n\n{}\n{}\n{}\n状态：{}\n提示：{}",
        overview.context.current_host,
        overview.context.target_host,
        overview.context.current_user,
        overview.context.privilege_mode,
        overview.context.repo_root.display(),
        overview.context.etc_root.display(),
        state.context.hosts.join(", "),
        state.context.users.join(", "),
        host_status,
        repo_integrity,
        doctor,
        apply_status,
        state.status
    ))
    .block(Block::default().borders(Borders::ALL).title("Overview Context"))
    .wrap(Wrap { trim: false });
    frame.render_widget(left, chunks[0]);

    let dirty = if overview.dirty_sections.is_empty() {
        "无未保存修改".to_string()
    } else {
        overview
            .dirty_sections
            .iter()
            .map(|section| format!("- {}: {}", section.name, section.items.join(", ")))
            .collect::<Vec<_>>()
            .join("\n")
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
    let infos = if overview.apply.infos.is_empty() {
        "无".to_string()
    } else {
        overview.apply.infos.join(" | ")
    };
    let right = Paragraph::new(format!(
        "未保存修改:\n{}\n\nApply 快照:\n- task: {}\n- source: {}\n- action: {}\n- flake update: {}\n- advanced: {}\n- sync: {}\n- rebuild: {}\n\n阻塞项:\n{}\n\n警告项:\n{}\n\n交接项:\n{}\n\n信息:\n{}",
        dirty,
        overview.apply.task.label(),
        overview.apply.source.label(),
        overview.apply.action.label(),
        if overview.apply.flake_update {
            "开启"
        } else {
            "关闭"
        },
        if overview.apply.advanced {
            "开启"
        } else {
            "关闭"
        },
        sync_preview,
        rebuild_preview,
        blockers,
        warnings,
        handoffs,
        infos
    ))
    .block(Block::default().borders(Borders::ALL).title("Apply Snapshot"))
    .wrap(Wrap { trim: false });
    frame.render_widget(right, chunks[1]);
}

fn render_check_lines(label: &str, state: &OverviewCheckState) -> Vec<String> {
    let mut lines = vec![format!("{label}: {}", state.summary_label())];
    for detail in state.detail_lines() {
        lines.push(format!("  - {detail}"));
    }
    lines
}
