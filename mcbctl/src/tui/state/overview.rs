use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OverviewModel {
    pub(crate) context: OverviewContext,
    pub(crate) dirty_sections: Vec<OverviewDirtySection>,
    pub(crate) host_status: OverviewHostStatus,
    pub(crate) health_focus: OverviewHealthFocus,
    pub(crate) repo_integrity: OverviewCheckState,
    pub(crate) doctor: OverviewCheckState,
    pub(crate) managed_guards: Vec<ManagedGuardSnapshot>,
    pub(crate) apply: ApplyModel,
    pub(crate) apply_summary: OverviewApplySummaryModel,
    pub(crate) primary_action: OverviewPrimaryAction,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OverviewApplySummaryModel {
    pub(crate) status: String,
    pub(crate) preview_command_fallback: String,
    pub(crate) next_step: String,
    pub(crate) latest_result: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OverviewContext {
    pub(crate) current_host: String,
    pub(crate) target_host: String,
    pub(crate) current_user: String,
    pub(crate) privilege_mode: String,
    pub(crate) repo_root: PathBuf,
    pub(crate) etc_root: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OverviewDirtySection {
    pub(crate) name: &'static str,
    pub(crate) items: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum OverviewHostStatus {
    Ready,
    Unavailable { message: String },
    Invalid { errors: Vec<String> },
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) enum OverviewCheckState {
    #[default]
    NotRun,
    Running,
    Healthy {
        summary: String,
        details: Vec<String>,
    },
    Error {
        summary: String,
        details: Vec<String>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OverviewHealthFocus {
    RepoIntegrity,
    Doctor,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OverviewPrimaryAction {
    pub(crate) kind: OverviewPrimaryActionKind,
    pub(crate) reason: String,
    pub(crate) recent_feedback: String,
    pub(crate) next_step: String,
}

enum InspectRouteOrigin {
    Overview,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct EditRouteFeedback {
    primary_reason: String,
    feedback_message: String,
    next_step: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RouteFeedback {
    pub(crate) message: String,
    pub(crate) next_step: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DirtyRoute {
    page: Page,
    scope: UiFeedbackScope,
    label: &'static str,
    items: Vec<String>,
    feedback: EditRouteFeedback,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ManagedGuardSnapshot {
    pub(crate) page: &'static str,
    pub(crate) target: String,
    pub(crate) available: bool,
    pub(crate) errors: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OverviewPrimaryActionKind {
    PreviewApply,
}

impl OverviewCheckState {
    pub(crate) fn summary_label(&self) -> String {
        match self {
            OverviewCheckState::NotRun => "未刷新".to_string(),
            OverviewCheckState::Running => "刷新中".to_string(),
            OverviewCheckState::Healthy { summary, .. } => summary.clone(),
            OverviewCheckState::Error { summary, .. } => summary.clone(),
        }
    }

    pub(crate) fn detail_lines(&self) -> &[String] {
        match self {
            OverviewCheckState::Healthy { details, .. }
            | OverviewCheckState::Error { details, .. } => details,
            OverviewCheckState::NotRun | OverviewCheckState::Running => &[],
        }
    }

    fn short_outcome(&self) -> &'static str {
        match self {
            OverviewCheckState::NotRun => "not-run",
            OverviewCheckState::Running => "running",
            OverviewCheckState::Healthy { .. } => "ok",
            OverviewCheckState::Error { .. } => "failed",
        }
    }
}

impl AppState {
    pub(crate) fn overview_model(&self) -> OverviewModel {
        let dirty_sections = self.overview_dirty_sections();
        let repo_integrity = self.overview_repo_integrity.clone();
        let doctor = self.overview_doctor.clone();
        let managed_guards = self.managed_guard_snapshots();
        let apply = self.apply_model();
        let apply_feedback = self.current_apply_feedback_summary();
        let apply_summary = OverviewApplySummaryModel {
            status: self.apply_execution_gate_model().status,
            preview_command_fallback: self.apply_preview_command_fallback(),
            next_step: apply_feedback.next_step,
            latest_result: self.current_apply_latest_result(),
        };
        let primary_action = self.overview_primary_action(&apply);

        OverviewModel {
            context: OverviewContext {
                current_host: self.context.current_host.clone(),
                target_host: self.target_host.clone(),
                current_user: self.context.current_user.clone(),
                privilege_mode: self.context.privilege_mode.clone(),
                repo_root: self.context.repo_root.clone(),
                etc_root: self.context.etc_root.clone(),
            },
            dirty_sections: dirty_sections.clone(),
            host_status: self.overview_host_status(),
            health_focus: preferred_overview_health_focus(&repo_integrity, &doctor),
            repo_integrity: repo_integrity.clone(),
            doctor: doctor.clone(),
            managed_guards,
            apply: apply.clone(),
            apply_summary,
            primary_action,
        }
    }

    pub(crate) fn refresh_overview_repo_integrity(&mut self) {
        self.overview_repo_integrity = OverviewCheckState::Running;
        let snapshot = repo_integrity_check_state(&self.context.repo_root);
        self.set_feedback_message(
            UiFeedbackLevel::Info,
            UiFeedbackScope::Overview,
            format!(
                "Overview: repo-integrity 已刷新（{}）。",
                snapshot.short_outcome()
            ),
        );
        self.overview_repo_integrity = snapshot;
    }

    pub(crate) fn refresh_overview_doctor(&mut self) {
        self.overview_doctor = OverviewCheckState::Running;
        let snapshot = doctor_check_state(&self.context.repo_root);
        self.set_feedback_message(
            UiFeedbackLevel::Info,
            UiFeedbackScope::Overview,
            format!("Overview: doctor 已刷新（{}）。", snapshot.short_outcome()),
        );
        self.overview_doctor = snapshot;
    }

    pub(crate) fn refresh_overview_health(&mut self) {
        self.overview_repo_integrity = OverviewCheckState::Running;
        self.overview_doctor = OverviewCheckState::Running;
        let repo_snapshot = repo_integrity_check_state(&self.context.repo_root);
        let doctor_snapshot = doctor_check_state(&self.context.repo_root);
        self.set_feedback_message(
            UiFeedbackLevel::Info,
            UiFeedbackScope::Overview,
            format!(
                "Overview 健康项已刷新：repo-integrity={}，doctor={}。",
                repo_snapshot.short_outcome(),
                doctor_snapshot.short_outcome()
            ),
        );
        self.overview_repo_integrity = repo_snapshot;
        self.overview_doctor = doctor_snapshot;
    }

    pub(crate) fn open_overview_primary_action(&mut self) {
        self.open_overview_apply();
    }

    pub(crate) fn open_overview_inspect(&mut self) {
        let feedback = self.inspect_route_feedback(InspectRouteOrigin::Overview);
        self.open_inspect();
        self.set_feedback_with_next_step(
            UiFeedbackLevel::Info,
            UiFeedbackScope::Inspect,
            feedback.message,
            feedback.next_step,
        );
    }

    pub(crate) fn open_overview_apply(&mut self) {
        self.deploy_focus = 0;
        self.open_apply();
        let feedback = self.overview_apply_route_feedback();
        self.set_feedback_with_next_step(
            UiFeedbackLevel::Info,
            UiFeedbackScope::Apply,
            feedback.message,
            feedback.next_step,
        );
    }

    pub(crate) fn preferred_edit_dirty_section(&self) -> Option<(&'static str, String)> {
        let route = preferred_dirty_route(&self.overview_dirty_sections())?;
        Some((route.label, route.items.first()?.clone()))
    }

    pub(crate) fn preferred_edit_managed_guard(&self) -> Option<(&'static str, String, String)> {
        let route = preferred_managed_guard_route(&self.managed_guard_snapshots())?;
        Some((route.label, route.target, route.reason))
    }

    fn overview_primary_action(&self, apply: &ApplyModel) -> OverviewPrimaryAction {
        let feedback = self.current_apply_feedback_summary();

        OverviewPrimaryAction {
            kind: OverviewPrimaryActionKind::PreviewApply,
            reason: overview_apply_entry_reason(apply),
            recent_feedback: feedback.message,
            next_step: feedback.next_step,
        }
    }

    fn inspect_route_feedback(&self, origin: InspectRouteOrigin) -> RouteFeedback {
        match preferred_inspect_review(&self.overview_repo_integrity, &self.overview_doctor) {
            Some(review) => inspect_review_feedback(origin, review),
            None => generic_inspect_route_feedback(origin),
        }
    }

    fn overview_dirty_sections(&self) -> Vec<OverviewDirtySection> {
        let mut sections = Vec::new();
        push_dirty_section(&mut sections, "Users", &self.host_dirty_user_hosts);
        push_dirty_section(&mut sections, "Hosts", &self.host_dirty_runtime_hosts);
        push_dirty_section(&mut sections, "Packages", &self.package_dirty_users);
        push_dirty_section(&mut sections, "Home", &self.home_dirty_users);
        sections
    }

    fn overview_host_status(&self) -> OverviewHostStatus {
        if let Some(message) = self.current_host_settings_unavailable_message() {
            return OverviewHostStatus::Unavailable { message };
        }

        let errors = self.host_configuration_validation_errors(&self.target_host);
        if errors.is_empty() {
            OverviewHostStatus::Ready
        } else {
            OverviewHostStatus::Invalid { errors }
        }
    }

    pub(crate) fn managed_guard_snapshots(&self) -> Vec<ManagedGuardSnapshot> {
        let package_target = self
            .current_package_user()
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| "无可用用户".to_string());
        let home_target = self
            .current_home_user()
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| "无可用用户".to_string());
        let host_target = if self.target_host.trim().is_empty() {
            "无可用主机".to_string()
        } else {
            self.target_host.clone()
        };

        vec![
            ManagedGuardSnapshot {
                page: "Packages",
                target: package_target,
                available: self.current_package_user().is_some(),
                errors: self.current_package_managed_guard_errors(),
            },
            ManagedGuardSnapshot {
                page: "Home",
                target: home_target,
                available: self.current_home_user().is_some(),
                errors: self.current_home_managed_guard_errors(),
            },
            ManagedGuardSnapshot {
                page: "Users",
                target: host_target.clone(),
                available: !self.target_host.trim().is_empty(),
                errors: self.current_host_managed_guard_errors(),
            },
            ManagedGuardSnapshot {
                page: "Hosts",
                target: host_target,
                available: !self.target_host.trim().is_empty(),
                errors: self.current_host_managed_guard_errors(),
            },
        ]
    }
}

fn preferred_dirty_route(dirty_sections: &[OverviewDirtySection]) -> Option<DirtyRoute> {
    let first = dirty_sections.first()?;
    let (page, scope) = match first.name {
        "Users" => (Page::Users, UiFeedbackScope::Users),
        "Hosts" => (Page::Hosts, UiFeedbackScope::Hosts),
        "Packages" => (Page::Packages, UiFeedbackScope::Packages),
        "Home" => (Page::Home, UiFeedbackScope::Home),
        _ => (Page::Dashboard, UiFeedbackScope::Overview),
    };
    Some(DirtyRoute {
        page,
        scope,
        label: first.name,
        items: first.items.clone(),
        feedback: dirty_route_feedback(dirty_sections, first),
    })
}

fn dirty_route_feedback(
    dirty_sections: &[OverviewDirtySection],
    first: &OverviewDirtySection,
) -> EditRouteFeedback {
    EditRouteFeedback {
        primary_reason: format!(
            "存在未保存修改：{}。",
            dirty_sections
                .iter()
                .map(|section| format!("{}: {}", section.name, section.items.join(", ")))
                .collect::<Vec<_>>()
                .join(" | ")
        ),
        feedback_message: format!(
            "Overview 检测到 {} 页仍有未保存修改：{}。",
            first.name,
            first.items.join(", ")
        ),
        next_step: format!("先在 {} 页保存，再回到 Overview / Apply", first.name),
    }
}

fn overview_apply_entry_reason(apply: &ApplyModel) -> String {
    if !apply.blockers.is_empty() {
        return "默认主路径：先进入 Apply 预览，确认 blocker / warning 和当前主机执行门槛。"
            .to_string();
    }

    if !apply.handoffs.is_empty() {
        return "默认主路径：先进入 Apply 预览，确认 handoff 和当前主机执行门槛。".to_string();
    }

    if apply.can_apply_current_host {
        return "默认主路径：先进入 Apply 预览，确认无误后再执行当前主机。".to_string();
    }

    "默认主路径：先进入 Apply 预览，确认当前主机预览和执行门槛。".to_string()
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct InspectReviewRoute {
    label: &'static str,
    summary: String,
    reason: String,
    feedback: RouteFeedback,
}

fn preferred_inspect_review(
    repo_integrity: &OverviewCheckState,
    doctor: &OverviewCheckState,
) -> Option<InspectReviewRoute> {
    match preferred_overview_health_focus(repo_integrity, doctor) {
        OverviewHealthFocus::RepoIntegrity => {
            inspect_review_route("repo-integrity", repo_integrity)
                .or_else(|| inspect_review_route("doctor", doctor))
        }
        OverviewHealthFocus::Doctor => inspect_review_route("doctor", doctor)
            .or_else(|| inspect_review_route("repo-integrity", repo_integrity)),
    }
}

fn preferred_overview_health_focus(
    repo_integrity: &OverviewCheckState,
    doctor: &OverviewCheckState,
) -> OverviewHealthFocus {
    if matches!(repo_integrity, OverviewCheckState::Error { .. }) {
        OverviewHealthFocus::RepoIntegrity
    } else if matches!(doctor, OverviewCheckState::Error { .. }) {
        OverviewHealthFocus::Doctor
    } else {
        OverviewHealthFocus::RepoIntegrity
    }
}

fn inspect_review_feedback(
    origin: InspectRouteOrigin,
    review: InspectReviewRoute,
) -> RouteFeedback {
    match origin {
        InspectRouteOrigin::Overview => review.feedback,
    }
}

fn generic_inspect_route_feedback(origin: InspectRouteOrigin) -> RouteFeedback {
    match origin {
        InspectRouteOrigin::Overview => RouteFeedback {
            message: "Overview 已跳到 Inspect。".to_string(),
            next_step: "在 Inspect 查看健康详情和检查命令".to_string(),
        },
    }
}

fn inspect_review_route(
    label: &'static str,
    state: &OverviewCheckState,
) -> Option<InspectReviewRoute> {
    let OverviewCheckState::Error { summary, .. } = state else {
        return None;
    };

    let (reason_suffix, next_step) = match label {
        "repo-integrity" => (
            "查看 flake check 和健康详情。",
            "在 Inspect 先看 repo-integrity，再决定是否执行 flake check",
        ),
        "doctor" => (
            "查看 doctor 和健康详情。",
            "在 Inspect 先看 doctor 详情；如需仓库校验，再执行 flake check",
        ),
        _ => ("查看详情。", "在 Inspect 查看详情"),
    };

    Some(InspectReviewRoute {
        label,
        summary: summary.clone(),
        reason: format!("{label} 当前失败（{summary}）；应先进入 Inspect {reason_suffix}"),
        feedback: RouteFeedback {
            message: format!("Overview 推荐先进入 Inspect 处理 {label}（{summary}）。"),
            next_step: next_step.to_string(),
        },
    })
}

fn managed_guard_route_feedback(
    label: &'static str,
    target: &str,
    reason: &str,
    focus_label: &'static str,
) -> EditRouteFeedback {
    EditRouteFeedback {
        primary_reason: format!("{label}[{target}] 的受管保护存在阻塞：{reason}"),
        feedback_message: format!("Overview 发现 {label}[{target}] 的受管保护阻塞：{reason}"),
        next_step: format!("先在 {label} 页查看 {focus_label} 附近的摘要并处理受管分片阻塞"),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ManagedGuardRoute {
    page: Page,
    scope: UiFeedbackScope,
    label: &'static str,
    target: String,
    reason: String,
    feedback: EditRouteFeedback,
    focus: ManagedGuardFocus,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ManagedGuardFocus {
    Packages {
        group: Option<String>,
        label: &'static str,
    },
    Home {
        index: usize,
        label: &'static str,
    },
    Users {
        index: usize,
        label: &'static str,
    },
    Hosts {
        index: usize,
        label: &'static str,
    },
}

fn preferred_managed_guard_route(guards: &[ManagedGuardSnapshot]) -> Option<ManagedGuardRoute> {
    if let Some(guard) = guards
        .iter()
        .find(|guard| guard.available && guard.page == "Packages" && !guard.errors.is_empty())
    {
        let reason = guard.errors[0].clone();
        let focus = ManagedGuardFocus::Packages {
            group: package_focus_group_from_error(&reason),
            label: "软件组过滤",
        };
        return Some(ManagedGuardRoute {
            page: Page::Packages,
            scope: UiFeedbackScope::Packages,
            label: "Packages",
            target: guard.target.clone(),
            feedback: managed_guard_route_feedback(
                "Packages",
                &guard.target,
                &reason,
                managed_guard_focus_label(&focus),
            ),
            focus,
            reason,
        });
    }

    if let Some(guard) = guards
        .iter()
        .find(|guard| guard.available && guard.page == "Home" && !guard.errors.is_empty())
    {
        let reason = guard.errors[0].clone();
        let focus = ManagedGuardFocus::Home {
            index: home_focus_index_for_guard_error(&reason),
            label: "设置列表",
        };
        return Some(ManagedGuardRoute {
            page: Page::Home,
            scope: UiFeedbackScope::Home,
            label: "Home",
            target: guard.target.clone(),
            feedback: managed_guard_route_feedback(
                "Home",
                &guard.target,
                &reason,
                managed_guard_focus_label(&focus),
            ),
            focus,
            reason,
        });
    }

    let users_guard = guards
        .iter()
        .find(|guard| guard.available && guard.page == "Users" && !guard.errors.is_empty());
    let hosts_guard = guards
        .iter()
        .find(|guard| guard.available && guard.page == "Hosts" && !guard.errors.is_empty());

    if let Some(guard) = hosts_guard.filter(|guard| {
        guard
            .errors
            .iter()
            .any(|error| is_hosts_runtime_guard_error(error))
    }) {
        let reason = first_matching_guard_error(&guard.errors, is_hosts_runtime_guard_error);
        let focus = ManagedGuardFocus::Hosts {
            index: hosts_focus_index_for_guard_error(&reason),
            label: hosts_focus_label_for_guard_error(&reason),
        };
        return Some(ManagedGuardRoute {
            page: Page::Hosts,
            scope: UiFeedbackScope::Hosts,
            label: "Hosts",
            target: guard.target.clone(),
            feedback: managed_guard_route_feedback(
                "Hosts",
                &guard.target,
                &reason,
                managed_guard_focus_label(&focus),
            ),
            focus,
            reason,
        });
    }

    if let Some(guard) =
        users_guard.filter(|guard| guard.errors.iter().any(|error| is_users_guard_error(error)))
    {
        let reason = first_matching_guard_error(&guard.errors, is_users_guard_error);
        let focus = ManagedGuardFocus::Users {
            index: users_focus_index_for_guard_error(&reason),
            label: users_focus_label_for_guard_error(&reason),
        };
        return Some(ManagedGuardRoute {
            page: Page::Users,
            scope: UiFeedbackScope::Users,
            label: "Users",
            target: guard.target.clone(),
            feedback: managed_guard_route_feedback(
                "Users",
                &guard.target,
                &reason,
                managed_guard_focus_label(&focus),
            ),
            focus,
            reason,
        });
    }

    users_guard
        .map(|guard| {
            let reason = guard.errors[0].clone();
            let focus = ManagedGuardFocus::Users {
                index: users_focus_index_for_guard_error(&reason),
                label: users_focus_label_for_guard_error(&reason),
            };
            ManagedGuardRoute {
                page: Page::Users,
                scope: UiFeedbackScope::Users,
                label: "Users",
                target: guard.target.clone(),
                feedback: managed_guard_route_feedback(
                    "Users",
                    &guard.target,
                    &reason,
                    managed_guard_focus_label(&focus),
                ),
                focus,
                reason,
            }
        })
        .or_else(|| {
            hosts_guard.map(|guard| {
                let reason = guard.errors[0].clone();
                let focus = ManagedGuardFocus::Hosts {
                    index: hosts_focus_index_for_guard_error(&reason),
                    label: hosts_focus_label_for_guard_error(&reason),
                };
                ManagedGuardRoute {
                    page: Page::Hosts,
                    scope: UiFeedbackScope::Hosts,
                    label: "Hosts",
                    target: guard.target.clone(),
                    feedback: managed_guard_route_feedback(
                        "Hosts",
                        &guard.target,
                        &reason,
                        managed_guard_focus_label(&focus),
                    ),
                    focus,
                    reason,
                }
            })
        })
}

fn managed_guard_focus_label(focus: &ManagedGuardFocus) -> &'static str {
    match focus {
        ManagedGuardFocus::Packages { label, .. }
        | ManagedGuardFocus::Home { label, .. }
        | ManagedGuardFocus::Users { label, .. }
        | ManagedGuardFocus::Hosts { label, .. } => label,
    }
}

fn first_matching_guard_error(errors: &[String], predicate: impl Fn(&str) -> bool) -> String {
    errors
        .iter()
        .find(|error| predicate(error))
        .cloned()
        .unwrap_or_else(|| {
            errors
                .first()
                .cloned()
                .unwrap_or_else(|| "unknown".to_string())
        })
}

fn is_users_guard_error(error: &str) -> bool {
    error.contains("host-users") || error.contains("host-managed-default")
}

fn is_hosts_runtime_guard_error(error: &str) -> bool {
    error.contains("host-network")
        || error.contains("host-gpu")
        || error.contains("host-virtualization")
}

fn users_focus_index_for_guard_error(error: &str) -> usize {
    if error.contains("host-users") { 2 } else { 1 }
}

fn users_focus_label_for_guard_error(error: &str) -> &'static str {
    if error.contains("host-users") {
        "托管用户字段"
    } else {
        "主用户字段"
    }
}

fn hosts_focus_index_for_guard_error(error: &str) -> usize {
    if error.contains("host-network") {
        4
    } else if error.contains("host-gpu") {
        18
    } else if error.contains("host-virtualization") {
        27
    } else {
        1
    }
}

fn hosts_focus_label_for_guard_error(error: &str) -> &'static str {
    if error.contains("host-network") {
        "代理模式字段"
    } else if error.contains("host-gpu") {
        "GPU 模式字段"
    } else if error.contains("host-virtualization") {
        "Docker 字段"
    } else {
        "缓存策略字段"
    }
}

fn home_focus_index_for_guard_error(_error: &str) -> usize {
    0
}

fn package_focus_group_from_error(error: &str) -> Option<String> {
    if let Some(rest) = error.split("package-group:").nth(1) {
        let group = rest
            .chars()
            .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '-' || *ch == '_')
            .collect::<String>();
        if !group.is_empty() {
            return Some(group);
        }
    }

    error
        .split("/managed/packages/")
        .nth(1)
        .and_then(|path| path.split(".nix").next())
        .filter(|group| !group.is_empty())
        .map(ToOwned::to_owned)
}

pub(super) fn repo_integrity_check_state(root: &std::path::Path) -> OverviewCheckState {
    match crate::repo::audit_repository(root) {
        Ok(report) if report.is_clean() => OverviewCheckState::Healthy {
            summary: "ok".to_string(),
            details: Vec::new(),
        },
        Ok(report) => OverviewCheckState::Error {
            summary: format!("failed ({} finding(s))", report.findings.len()),
            details: report.render_lines().into_iter().skip(1).collect(),
        },
        Err(err) => OverviewCheckState::Error {
            summary: "failed to run repo-integrity".to_string(),
            details: vec![err.to_string()],
        },
    }
}

fn doctor_check_state(root: &std::path::Path) -> OverviewCheckState {
    doctor_check_state_from_report(crate::health::collect_doctor_report(root))
}

fn doctor_check_state_from_report(
    report: Result<crate::health::DoctorReport>,
) -> OverviewCheckState {
    match report {
        Ok(report) if report.is_healthy() && report.assessment.warnings.is_empty() => {
            OverviewCheckState::Healthy {
                summary: "ok".to_string(),
                details: Vec::new(),
            }
        }
        Ok(report) if report.is_healthy() => OverviewCheckState::Healthy {
            summary: format!("ok with {} warning(s)", report.assessment.warnings.len()),
            details: report.assessment.warnings.clone(),
        },
        Ok(report) => {
            let mut details = report.failure_lines();
            details.extend(
                report
                    .assessment
                    .warnings
                    .iter()
                    .map(|warning| format!("warning: {warning}")),
            );
            OverviewCheckState::Error {
                summary: format!("failed ({} issue(s))", details.len()),
                details,
            }
        }
        Err(err) => OverviewCheckState::Error {
            summary: "failed to run doctor".to_string(),
            details: vec![err.to_string()],
        },
    }
}

fn push_dirty_section(
    sections: &mut Vec<OverviewDirtySection>,
    name: &'static str,
    items: &BTreeSet<String>,
) {
    if items.is_empty() {
        return;
    }

    sections.push(OverviewDirtySection {
        name,
        items: items.iter().cloned().collect(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};

    #[test]
    fn overview_model_surfaces_context_and_dirty_sections() {
        let mut state = test_state("sudo-available");
        state.package_dirty_users.insert("alice".to_string());
        state.home_dirty_users.insert("alice".to_string());

        let model = state.overview_model();

        assert_eq!(model.context.current_host, "demo");
        assert_eq!(model.context.target_host, "demo");
        assert_eq!(model.context.current_user, "alice");
        assert_eq!(model.context.privilege_mode, "sudo-available");
        assert_eq!(model.dirty_sections.len(), 2);
        assert_eq!(model.dirty_sections[0].name, "Packages");
        assert_eq!(model.dirty_sections[0].items, vec!["alice".to_string()]);
        assert_eq!(model.dirty_sections[1].name, "Home");
        assert_eq!(
            model.primary_action.kind,
            OverviewPrimaryActionKind::PreviewApply
        );
        assert_eq!(
            model.repo_integrity,
            OverviewCheckState::Healthy {
                summary: "ok".to_string(),
                details: Vec::new()
            }
        );
        assert_eq!(model.doctor, OverviewCheckState::NotRun);
        assert_eq!(model.managed_guards.len(), 4);
        assert!(
            model
                .apply
                .blockers
                .iter()
                .any(|item| item.contains("Packages: alice"))
        );
        assert!(
            model
                .apply
                .blockers
                .iter()
                .any(|item| item.contains("Home: alice"))
        );
        assert!(!model.apply.can_apply_current_host);
        assert_eq!(
            model.primary_action.reason,
            "默认主路径：先进入 Apply 预览，确认 blocker / warning 和当前主机执行门槛。"
        );
    }

    #[test]
    fn managed_guard_snapshots_surface_blocked_package_targets() -> Result<()> {
        let unique = format!("{}-{}", std::process::id(), rand::random::<u64>());
        let root = std::env::temp_dir().join(format!("mcbctl-overview-guards-{unique}"));
        std::fs::create_dir_all(root.join("home/users/alice/managed/packages"))?;
        std::fs::write(
            root.join("home/users/alice/managed/packages/manual.nix"),
            "{ pkgs, ... }: { home.packages = [ pkgs.hello ]; }\n",
        )?;

        let mut state = test_state("sudo-available");
        state.context.repo_root = root.clone();

        let guards = state.managed_guard_snapshots();
        let packages = guards
            .iter()
            .find(|guard| guard.page == "Packages")
            .expect("packages guard snapshot");

        assert_eq!(packages.target, "alice");
        assert_eq!(packages.errors.len(), 1);
        assert!(packages.errors[0].contains("refusing to remove stale unmanaged package file"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn overview_model_reports_rootless_non_build_blocker_and_hardware_warning() {
        let mut state = test_state("rootless");
        state.deploy_action = DeployAction::Switch;

        let model = state.overview_model();

        assert_eq!(model.host_status, OverviewHostStatus::Ready);
        assert!(
            model
                .apply
                .blockers
                .iter()
                .any(|item| item.contains("rootless 模式下当前页只能直接执行 build"))
        );
        assert!(
            model
                .apply
                .warnings
                .iter()
                .any(|item| item.contains("hardware-configuration.nix"))
        );
        assert!(!model.apply.can_apply_current_host);
        assert_eq!(
            model.primary_action.kind,
            OverviewPrimaryActionKind::PreviewApply
        );
        assert_eq!(
            model.primary_action.reason,
            "默认主路径：先进入 Apply 预览，确认 blocker / warning 和当前主机执行门槛。"
        );
    }

    #[test]
    fn overview_model_reports_host_unavailability_and_handoff() {
        let mut state = test_state("sudo-available");
        state.host_settings_by_name.clear();
        state.host_settings_errors_by_name.insert(
            "demo".to_string(),
            "nix eval for host demo failed".to_string(),
        );
        state.deploy_source = DeploySource::RemoteHead;

        let model = state.overview_model();

        assert_eq!(
            model.host_status,
            OverviewHostStatus::Unavailable {
                message: "主机 demo 的配置读取失败：nix eval for host demo failed".to_string()
            }
        );
        assert!(
            model
                .apply
                .blockers
                .iter()
                .any(|item| item.contains("配置读取失败"))
        );
        assert!(
            model
                .apply
                .handoffs
                .iter()
                .any(|item| item.contains("远端最新版本"))
        );
        assert!(!model.apply.can_execute_directly);
        assert!(!model.apply.can_apply_current_host);
        assert_eq!(
            model.apply_summary.next_step,
            "在 Apply 先看 blocker / warning，再决定是否调整 Apply 项"
        );
        assert_eq!(
            model.primary_action.kind,
            OverviewPrimaryActionKind::PreviewApply
        );
        assert_eq!(
            model.primary_action.reason,
            "默认主路径：先进入 Apply 预览，确认 blocker / warning 和当前主机执行门槛。"
        );
    }

    #[test]
    fn overview_model_surfaces_handoff_next_step_when_apply_workspace_is_closed() {
        let mut state = test_state("sudo-available");
        state.deploy_source = DeploySource::RemoteHead;

        let model = state.overview_model();

        assert_eq!(
            model.apply_summary.next_step,
            "在 Apply 先看 handoff 预览；如需继续，切到 Advanced 执行 launch deploy wizard"
        );
        assert_eq!(
            model.primary_action.kind,
            OverviewPrimaryActionKind::PreviewApply
        );
        assert_eq!(
            model.primary_action.reason,
            "默认主路径：先进入 Apply 预览，确认 handoff 和当前主机执行门槛。"
        );
    }

    #[test]
    fn overview_model_exposes_sync_and_rebuild_previews() {
        let mut state = test_state("sudo-available");
        state.flake_update = true;

        let model = state.overview_model();

        assert_eq!(model.apply.source, DeploySource::CurrentRepo);
        assert_eq!(model.apply.action, DeployAction::Switch);
        assert!(model.apply.sync_preview.is_some());
        let rebuild_preview = model
            .apply
            .rebuild_preview
            .expect("direct apply should expose rebuild preview");
        assert!(rebuild_preview.contains("sudo -E"));
        assert!(rebuild_preview.contains("/etc/nixos#demo"));
        assert_eq!(
            model.apply_summary.next_step,
            "在 Apply 查看预览；确认后按 x 直接运行"
        );
        assert!(
            model
                .apply
                .warnings
                .iter()
                .any(|item| item.contains("--upgrade"))
        );
        assert!(
            model
                .apply
                .warnings
                .iter()
                .any(|item| item.contains("同步到 /etc/nixos"))
        );
        assert_eq!(
            model.primary_action.kind,
            OverviewPrimaryActionKind::PreviewApply
        );
        assert_eq!(
            model.primary_action.reason,
            "默认主路径：先进入 Apply 预览，确认无误后再执行当前主机。"
        );
    }

    #[test]
    fn overview_model_prefers_repo_integrity_review_when_health_has_failures() {
        let mut state = test_state("sudo-available");
        state.overview_repo_integrity = OverviewCheckState::Error {
            summary: "failed (1 finding(s))".to_string(),
            details: vec!["- [rule] path: detail".to_string()],
        };

        let model = state.overview_model();

        assert_eq!(model.health_focus, OverviewHealthFocus::RepoIntegrity);
        assert_eq!(
            model.primary_action.kind,
            OverviewPrimaryActionKind::PreviewApply
        );
        assert_eq!(
            model.primary_action.reason,
            "默认主路径：先进入 Apply 预览，确认 blocker / warning 和当前主机执行门槛。"
        );
    }

    #[test]
    fn overview_model_prefers_doctor_review_when_repo_integrity_is_clean() {
        let mut state = test_state("sudo-available");
        state.overview_doctor = OverviewCheckState::Error {
            summary: "failed (1 check(s))".to_string(),
            details: vec!["缺少 nixos-rebuild".to_string()],
        };

        let model = state.overview_model();

        assert_eq!(model.health_focus, OverviewHealthFocus::Doctor);
        assert_eq!(
            model.primary_action.kind,
            OverviewPrimaryActionKind::PreviewApply
        );
        assert_eq!(
            model.primary_action.reason,
            "默认主路径：先进入 Apply 预览，确认 blocker / warning 和当前主机执行门槛。"
        );
    }

    #[test]
    fn overview_model_primary_action_prefers_apply_scoped_completion_feedback() {
        let mut state = test_state("sudo-available");
        state.set_feedback_with_next_step(
            UiFeedbackLevel::Success,
            UiFeedbackScope::Apply,
            "Apply 已执行完成：switch demo",
            "回到 Overview 检查健康和下一步",
        );

        let model = state.overview_model();

        assert_eq!(
            model.primary_action.recent_feedback,
            "Apply 已执行完成：switch demo"
        );
        assert_eq!(
            model.primary_action.next_step,
            "回到 Overview 检查健康和下一步"
        );
        assert_eq!(
            model.apply_summary.next_step,
            "回到 Overview 检查健康和下一步"
        );
        assert_eq!(
            model.apply_summary.latest_result,
            "Apply 已执行完成：switch demo 下一步：回到 Overview 检查健康和下一步"
        );
    }

    #[test]
    fn overview_model_primary_action_ignores_inspect_scoped_feedback() {
        let mut state = test_state("sudo-available");
        state.overview_repo_integrity = OverviewCheckState::Error {
            summary: "failed (1 finding(s))".to_string(),
            details: vec!["- [rule] path: detail".to_string()],
        };
        state.set_feedback_with_next_step(
            UiFeedbackLevel::Success,
            UiFeedbackScope::Inspect,
            "flake check 已完成。",
            "留在 Inspect 复查健康详情",
        );

        let model = state.overview_model();

        assert_eq!(
            model.primary_action.kind,
            OverviewPrimaryActionKind::PreviewApply
        );
        assert_eq!(
            model.primary_action.recent_feedback,
            state.current_apply_feedback_summary().message
        );
        assert_eq!(
            model.primary_action.next_step,
            state.current_apply_feedback_summary().next_step
        );
    }

    #[test]
    fn overview_model_primary_action_ignores_advanced_scoped_feedback() {
        let mut state = test_state("sudo-available");
        state.deploy_source = DeploySource::RemotePinned;
        state.deploy_source_ref = "v5.0.0".to_string();
        state.set_feedback_with_next_step(
            UiFeedbackLevel::Success,
            UiFeedbackScope::Advanced,
            "完整部署向导已返回。",
            "回到 Advanced 继续核对 Deploy Parameters",
        );

        let model = state.overview_model();

        assert_eq!(
            model.primary_action.kind,
            OverviewPrimaryActionKind::PreviewApply
        );
        assert_eq!(
            model.primary_action.recent_feedback,
            state.current_apply_feedback_summary().message
        );
        assert_eq!(
            model.primary_action.next_step,
            state.current_apply_feedback_summary().next_step
        );
    }

    #[test]
    fn overview_model_primary_action_ignores_unrelated_feedback_scope_for_apply_actions() {
        let mut state = test_state("sudo-available");
        state.set_feedback_with_next_step(
            UiFeedbackLevel::Info,
            UiFeedbackScope::Packages,
            "Packages 已写入",
            "回到 Packages 查看结果",
        );

        let model = state.overview_model();

        assert_eq!(
            model.primary_action.kind,
            OverviewPrimaryActionKind::PreviewApply
        );
        assert_eq!(model.primary_action.recent_feedback, "当前组合可直接执行。");
        assert_eq!(
            model.primary_action.next_step,
            "在 Apply 查看预览；确认后按 x 直接运行"
        );
    }

    #[test]
    fn overview_model_prefers_managed_guard_review_when_health_is_clean() -> Result<()> {
        let unique = format!("{}-{}", std::process::id(), rand::random::<u64>());
        let root = std::env::temp_dir().join(format!("mcbctl-overview-action-guards-{unique}"));
        std::fs::create_dir_all(root.join("home/users/alice/managed/packages"))?;
        std::fs::write(
            root.join("home/users/alice/managed/packages/manual.nix"),
            "{ pkgs, ... }: { home.packages = [ pkgs.hello ]; }\n",
        )?;

        let mut state = test_state("sudo-available");
        state.context.repo_root = root.clone();

        let model = state.overview_model();

        assert_eq!(
            model.primary_action.kind,
            OverviewPrimaryActionKind::PreviewApply
        );
        assert_eq!(
            model.primary_action.reason,
            "默认主路径：先进入 Apply 预览，确认无误后再执行当前主机。"
        );

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn open_overview_primary_action_enters_apply_preview_even_when_dirty() {
        let mut state = test_state("sudo-available");
        state.package_dirty_users.insert("alice".to_string());
        state.home_dirty_users.insert("alice".to_string());

        state.open_overview_primary_action();

        assert_eq!(state.page(), Page::Deploy);
        assert_eq!(state.feedback.scope, UiFeedbackScope::Apply);
        assert!(
            state
                .feedback
                .message
                .contains("Overview 已进入 Apply 预览；当前组合仍有 blocker：")
        );
        assert_eq!(
            state.feedback.next_step.as_deref(),
            Some("在 Apply 先看 blocker / warning，再决定是否调整 Apply 项")
        );
    }

    #[test]
    fn open_overview_primary_action_keeps_health_failures_on_apply_preview_path() {
        let mut state = test_state("sudo-available");
        state.overview_repo_integrity = OverviewCheckState::Error {
            summary: "failed (1 finding(s))".to_string(),
            details: vec!["- [rule] path: detail".to_string()],
        };
        let overview = state.overview_model();

        state.open_overview_primary_action();

        assert_eq!(state.page(), Page::Deploy);
        assert_eq!(state.feedback.scope, UiFeedbackScope::Apply);
        assert_eq!(
            overview.primary_action.kind,
            OverviewPrimaryActionKind::PreviewApply
        );
        assert_eq!(
            state.feedback.message,
            "Overview 已进入 Apply 预览；当前组合仍有 blocker：repo-integrity 当前失败：failed (1 finding(s))。"
        );
        assert_eq!(
            state.feedback.next_step.as_deref(),
            Some("在 Apply 先看 blocker / warning，再决定是否调整 Apply 项")
        );
        assert_eq!(state.deploy_focus, 0);
    }

    #[test]
    fn open_overview_primary_action_keeps_doctor_failures_on_apply_preview_path() {
        let mut state = test_state("sudo-available");
        state.overview_doctor = OverviewCheckState::Error {
            summary: "failed (1 check(s))".to_string(),
            details: vec!["缺少 nixos-rebuild".to_string()],
        };

        state.open_overview_primary_action();

        assert_eq!(state.page(), Page::Deploy);
        assert_eq!(state.feedback.scope, UiFeedbackScope::Apply);
        assert_eq!(
            state.feedback.message,
            "Overview 已进入 Apply 预览；当前组合仍有 blocker：doctor 当前失败：failed (1 check(s))。"
        );
        assert_eq!(
            state.feedback.next_step.as_deref(),
            Some("在 Apply 先看 blocker / warning，再决定是否调整 Apply 项")
        );
    }

    #[test]
    fn open_overview_primary_action_keeps_package_guard_findings_on_apply_preview_path()
    -> Result<()> {
        let unique = format!("{}-{}", std::process::id(), rand::random::<u64>());
        let root = std::env::temp_dir().join(format!("mcbctl-overview-open-packages-{unique}"));
        std::fs::create_dir_all(root.join("home/users/alice/managed/packages"))?;
        std::fs::write(
            root.join("home/users/alice/managed/packages/manual.nix"),
            "{ pkgs, ... }: { home.packages = [ pkgs.hello ]; }\n",
        )?;

        let mut state = test_state("sudo-available");
        state.context.repo_root = root.clone();

        state.open_overview_primary_action();

        assert_eq!(state.page(), Page::Deploy);
        assert_eq!(state.feedback.scope, UiFeedbackScope::Apply);
        assert_eq!(
            state.feedback.next_step.as_deref(),
            Some("在 Apply 查看预览；确认后按 x 直接运行")
        );

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn open_overview_primary_action_keeps_runtime_guard_findings_on_apply_preview_path()
    -> Result<()> {
        let unique = format!("{}-{}", std::process::id(), rand::random::<u64>());
        let root = std::env::temp_dir().join(format!("mcbctl-overview-open-hosts-{unique}"));
        std::fs::create_dir_all(root.join("hosts/demo/managed"))?;
        std::fs::write(
            root.join("hosts/demo/managed/network.nix"),
            "{ lib, ... }: { mcb.proxyMode = lib.mkForce \"http\"; }\n",
        )?;

        let mut state = test_state("sudo-available");
        state.context.repo_root = root.clone();

        state.open_overview_primary_action();

        assert_eq!(state.page(), Page::Deploy);
        assert_eq!(state.feedback.scope, UiFeedbackScope::Apply);
        assert_eq!(
            state.feedback.next_step.as_deref(),
            Some("在 Apply 查看预览；确认后按 x 直接运行")
        );

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn open_overview_primary_action_keeps_user_guard_findings_on_apply_preview_path() -> Result<()>
    {
        let unique = format!("{}-{}", std::process::id(), rand::random::<u64>());
        let root = std::env::temp_dir().join(format!("mcbctl-overview-open-users-{unique}"));
        std::fs::create_dir_all(root.join("hosts/demo/managed"))?;
        std::fs::write(
            root.join("hosts/demo/managed/users.nix"),
            "{ lib, ... }: { users.users.alice.isNormalUser = true; }\n",
        )?;

        let mut state = test_state("sudo-available");
        state.context.repo_root = root.clone();

        state.open_overview_primary_action();

        assert_eq!(state.page(), Page::Deploy);
        assert_eq!(state.feedback.scope, UiFeedbackScope::Apply);
        assert_eq!(
            state.feedback.next_step.as_deref(),
            Some("在 Apply 查看预览；确认后按 x 直接运行")
        );

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn open_overview_primary_action_keeps_home_guard_findings_on_apply_preview_path() -> Result<()>
    {
        let unique = format!("{}-{}", std::process::id(), rand::random::<u64>());
        let root = std::env::temp_dir().join(format!("mcbctl-overview-open-home-{unique}"));
        std::fs::create_dir_all(root.join("home/users/alice/managed/settings"))?;
        std::fs::write(
            root.join("home/users/alice/managed/settings/desktop.nix"),
            "{ lib, ... }: { mcb.noctalia.barProfile = \"default\"; }\n",
        )?;

        let mut state = test_state("sudo-available");
        state.context.repo_root = root.clone();

        state.open_overview_primary_action();

        assert_eq!(state.page(), Page::Deploy);
        assert_eq!(state.feedback.scope, UiFeedbackScope::Apply);
        assert_eq!(
            state.feedback.next_step.as_deref(),
            Some("在 Apply 查看预览；确认后按 x 直接运行")
        );

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn open_overview_primary_action_routes_handoff_to_apply_preview() {
        let mut state = test_state("sudo-available");
        state.deploy_source = DeploySource::RemotePinned;
        state.deploy_source_ref = "v5.0.0".to_string();
        state.deploy_focus = 6;

        state.open_overview_primary_action();

        assert_eq!(state.page(), Page::Deploy);
        assert!(!state.advanced_path_active());
        assert_eq!(state.feedback.scope, UiFeedbackScope::Apply);
        assert_eq!(
            state.feedback.next_step.as_deref(),
            Some("在 Apply 先看 handoff 预览；如需继续，切到 Advanced 执行 launch deploy wizard")
        );
        assert_eq!(
            state.feedback.message,
            "Overview 已进入 Apply 预览；当前来源是远端固定版本；默认 Apply 不会直接执行，必须交给完整高级路径。"
        );
    }

    #[test]
    fn overview_handoff_reason_stays_on_preview_apply_path() {
        let mut state = test_state("sudo-available");
        state.deploy_source = DeploySource::RemoteHead;

        let overview = state.overview_model();
        assert_eq!(
            overview.primary_action.kind,
            OverviewPrimaryActionKind::PreviewApply
        );
        assert_eq!(
            overview.primary_action.reason,
            "默认主路径：先进入 Apply 预览，确认 handoff 和当前主机执行门槛。"
        );

        state.open_overview_primary_action();

        match state.deploy_page_model() {
            DeployPageModel::Apply(model) => {
                assert!(model.selection.recommendation.contains("切到 Advanced"));
                assert_eq!(
                    state.feedback.next_step.as_deref(),
                    Some(
                        "在 Apply 先看 handoff 预览；如需继续，切到 Advanced 执行 launch deploy wizard"
                    )
                );
            }
            other => panic!("expected apply page model, got {other:?}"),
        }
    }

    #[test]
    fn open_overview_primary_action_routes_to_apply_for_direct_apply() {
        let mut state = test_state("sudo-available");

        state.open_overview_primary_action();

        assert_eq!(state.page(), Page::Deploy);
        assert!(!state.advanced_path_active());
        assert_eq!(state.feedback.scope, UiFeedbackScope::Apply);
        assert!(state.status.contains("可直接执行"));
    }

    #[test]
    fn doctor_check_state_surfaces_warnings_without_marking_failure() {
        let state = doctor_check_state_from_report(Ok(crate::health::DoctorReport {
            repo_root: PathBuf::from("/repo"),
            remote_branch: "rust脚本分支".to_string(),
            tools: crate::health::DoctorToolStatus::default(),
            repo_hardware: "present".to_string(),
            legacy_root_hardware: false,
            current_user: "alice".to_string(),
            current_uid: "1000".to_string(),
            layout_error: None,
            integrity_clean: true,
            integrity_lines: vec!["repository integrity check passed".to_string()],
            assessment: crate::health::DoctorAssessment {
                blocking_issues: Vec::new(),
                warnings: vec!["缺少 cargo".to_string()],
            },
        }));

        assert_eq!(
            state,
            OverviewCheckState::Healthy {
                summary: "ok with 1 warning(s)".to_string(),
                details: vec!["缺少 cargo".to_string()],
            }
        );
    }

    #[test]
    fn doctor_check_state_surfaces_failures_and_warnings() {
        let state = doctor_check_state_from_report(Ok(crate::health::DoctorReport {
            repo_root: PathBuf::from("/repo"),
            remote_branch: "rust脚本分支".to_string(),
            tools: crate::health::DoctorToolStatus::default(),
            repo_hardware: "missing".to_string(),
            legacy_root_hardware: true,
            current_user: "alice".to_string(),
            current_uid: "1000".to_string(),
            layout_error: Some("layout broken".to_string()),
            integrity_clean: true,
            integrity_lines: vec!["repository integrity check passed".to_string()],
            assessment: crate::health::DoctorAssessment {
                blocking_issues: vec!["缺少 nix".to_string()],
                warnings: vec!["缺少 cargo".to_string()],
            },
        }));

        assert_eq!(
            state,
            OverviewCheckState::Error {
                summary: "failed (3 issue(s))".to_string(),
                details: vec![
                    "repo layout: layout broken".to_string(),
                    "deployment environment: 缺少 nix".to_string(),
                    "warning: 缺少 cargo".to_string(),
                ],
            }
        );
    }

    fn test_state(privilege_mode: &str) -> AppState {
        let mut host_settings_by_name = BTreeMap::new();
        host_settings_by_name.insert(
            "demo".to_string(),
            HostManagedSettings {
                primary_user: "alice".to_string(),
                users: vec!["alice".to_string()],
                admin_users: vec!["alice".to_string()],
                ..HostManagedSettings::default()
            },
        );

        AppState {
            context: AppContext {
                repo_root: PathBuf::from("/repo"),
                etc_root: PathBuf::from("/etc/nixos"),
                current_host: "demo".to_string(),
                current_system: "x86_64-linux".to_string(),
                current_user: "alice".to_string(),
                privilege_mode: privilege_mode.to_string(),
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
            deploy_action: if privilege_mode == "rootless" {
                DeployAction::Build
            } else {
                DeployAction::Switch
            },
            flake_update: false,
            advanced_target_host: "demo".to_string(),
            advanced_deploy_task: DeployTask::DirectDeploy,
            advanced_deploy_source: DeploySource::CurrentRepo,
            advanced_deploy_source_ref: String::new(),
            advanced_deploy_action: if privilege_mode == "rootless" {
                DeployAction::Build
            } else {
                DeployAction::Switch
            },
            advanced_flake_update: false,
            help_overlay_visible: false,
            deploy_text_mode: None,
            users_focus: 0,
            hosts_focus: 0,
            users_text_mode: None,
            hosts_text_mode: None,
            host_text_input: String::new(),
            host_settings_by_name,
            host_settings_errors_by_name: BTreeMap::new(),
            host_dirty_user_hosts: BTreeSet::new(),
            host_dirty_runtime_hosts: BTreeSet::new(),
            package_user_index: 0,
            package_mode: PackageDataMode::Search,
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
            inspect_action: crate::domain::tui::ActionItem::FlakeCheck,
            advanced_action: crate::domain::tui::ActionItem::FlakeUpdate,
            overview_repo_integrity: OverviewCheckState::Healthy {
                summary: "ok".to_string(),
                details: Vec::new(),
            },
            overview_doctor: OverviewCheckState::NotRun,
            feedback: UiFeedback::default(),
            status: String::new(),
        }
    }
}
