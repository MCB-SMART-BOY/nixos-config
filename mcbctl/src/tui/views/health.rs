use crate::tui::state::{ManagedGuardSnapshot, OverviewCheckState};

pub(super) fn render_compact_health_summary(
    active_label: &str,
    active_state: &OverviewCheckState,
    secondary_label: &str,
    secondary_state: &OverviewCheckState,
    guards: &[ManagedGuardSnapshot],
) -> String {
    let mut lines = render_compact_check_lines(active_label, active_state, true);
    lines.extend(render_compact_check_lines(
        secondary_label,
        secondary_state,
        false,
    ));
    lines.extend(render_guard_summary_lines(guards));
    lines.join("\n")
}

fn render_compact_check_lines(
    label: &str,
    state: &OverviewCheckState,
    include_detail: bool,
) -> Vec<String> {
    let mut lines = vec![format!("{label}: {}", state.summary_label())];
    if include_detail {
        if let Some((first, rest)) = state.detail_lines().split_first() {
            lines.push(format!("优先项：{first}"));
            if !rest.is_empty() {
                lines.push(format!("其余：另 {} 项", rest.len()));
            }
        }
    }
    lines
}

fn render_guard_summary_lines(guards: &[ManagedGuardSnapshot]) -> Vec<String> {
    let blocked = guards
        .iter()
        .filter(|guard| guard.available && !guard.errors.is_empty())
        .collect::<Vec<_>>();

    if blocked.is_empty() {
        return vec!["save-guards: ok".to_string()];
    }

    let first = blocked[0];
    let suffix = if blocked.len() == 1 {
        String::new()
    } else {
        format!("（另 {} 项）", blocked.len() - 1)
    };
    let reason = first
        .errors
        .first()
        .cloned()
        .unwrap_or_else(|| "受管分片校验失败".to_string());

    vec![
        format!(
            "save-guards: {}[{}] blocked{}",
            first.page, first.target, suffix
        ),
        format!("优先处理：{reason}"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_health_summary_expands_only_active_state() {
        let text = render_compact_health_summary(
            "doctor",
            &OverviewCheckState::Error {
                summary: "failed (2 check(s))".to_string(),
                details: vec!["缺少 nixos-rebuild".to_string(), "缺少 git".to_string()],
            },
            "repo-integrity",
            &OverviewCheckState::Healthy {
                summary: "ok".to_string(),
                details: vec!["这一行不应展开".to_string()],
            },
            &[],
        );

        assert!(text.contains("doctor: failed (2 check(s))"));
        assert!(text.contains("优先项：缺少 nixos-rebuild"));
        assert!(text.contains("其余：另 1 项"));
        assert!(text.contains("repo-integrity: ok"));
        assert!(!text.contains("这一行不应展开"));
        assert!(text.contains("save-guards: ok"));
    }

    #[test]
    fn compact_health_summary_prioritizes_first_blocked_guard() {
        let text = render_compact_health_summary(
            "repo-integrity",
            &OverviewCheckState::Healthy {
                summary: "ok".to_string(),
                details: Vec::new(),
            },
            "doctor",
            &OverviewCheckState::Healthy {
                summary: "ok".to_string(),
                details: Vec::new(),
            },
            &[
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
                    errors: vec!["another blocker".to_string()],
                },
            ],
        );

        assert!(text.contains("save-guards: Packages[alice] blocked（另 1 项）"));
        assert!(text.contains("优先处理：manual group blocks save"));
        assert!(!text.contains("another blocker"));
    }
}
