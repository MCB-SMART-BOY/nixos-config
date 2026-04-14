use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OverviewModel {
    pub(crate) context: OverviewContext,
    pub(crate) dirty_sections: Vec<OverviewDirtySection>,
    pub(crate) host_status: OverviewHostStatus,
    pub(crate) repo_integrity: OverviewCheckState,
    pub(crate) doctor: OverviewCheckState,
    pub(crate) managed_guards: Vec<ManagedGuardSnapshot>,
    pub(crate) apply: ApplyModel,
    pub(crate) primary_action: OverviewPrimaryAction,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OverviewPrimaryAction {
    pub(crate) kind: OverviewPrimaryActionKind,
    pub(crate) reason: String,
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
    SaveDirtyPages,
    ReviewInspect,
    ReviewManagedGuards,
    OpenAdvancedApply,
    ReviewApply,
    ApplyCurrentHost,
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
        let primary_action = overview_primary_action(
            &dirty_sections,
            &repo_integrity,
            &doctor,
            &managed_guards,
            &apply,
        );

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
            repo_integrity: repo_integrity.clone(),
            doctor: doctor.clone(),
            managed_guards,
            apply: apply.clone(),
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
        let overview = self.overview_model();
        match overview.primary_action.kind {
            OverviewPrimaryActionKind::SaveDirtyPages => {
                let Some(section) = overview.dirty_sections.first() else {
                    return;
                };
                let page = overview_dirty_page(section.name);
                self.set_page(page);
                self.set_feedback_with_next_step(
                    UiFeedbackLevel::Info,
                    overview_dirty_scope(section.name),
                    format!(
                        "Overview 检测到 {} 页仍有未保存修改：{}。",
                        section.name,
                        section.items.join(", ")
                    ),
                    format!("先在 {} 页保存，再回到 Overview / Apply", section.name),
                );
            }
            OverviewPrimaryActionKind::ReviewInspect => {
                self.set_page(Page::Inspect);
                self.ensure_inspect_action_focus();
                self.set_feedback_with_next_step(
                    UiFeedbackLevel::Info,
                    UiFeedbackScope::Inspect,
                    "Overview 推荐先进入 Inspect 查看失败的健康检查项。",
                    "在 Inspect 查看详情或执行检查命令",
                );
            }
            OverviewPrimaryActionKind::ReviewManagedGuards => {
                let Some(route) = preferred_managed_guard_route(&overview.managed_guards) else {
                    return;
                };
                self.set_page(route.page);
                let focus_label = self.apply_managed_guard_focus(&route.focus);
                self.set_feedback_with_next_step(
                    UiFeedbackLevel::Info,
                    route.scope,
                    format!(
                        "Overview 发现 {}[{target}] 的受管保护阻塞：{reason}",
                        route.label,
                        target = route.target,
                        reason = route.reason
                    ),
                    format!("先在 {} 页查看 {focus_label} 附近的摘要并处理受管分片阻塞", route.label),
                );
            }
            OverviewPrimaryActionKind::OpenAdvancedApply => {
                self.set_page(Page::Deploy);
                self.show_advanced = true;
                self.ensure_advanced_action_focus();
                self.set_feedback_with_next_step(
                    UiFeedbackLevel::Info,
                    UiFeedbackScope::Advanced,
                    "Overview 推荐先打开 Apply 的高级模式。",
                    "在 Apply 的 Advanced Workspace 里继续复杂部署或维护",
                );
            }
            OverviewPrimaryActionKind::ReviewApply => {
                self.set_page(Page::Deploy);
                self.show_advanced = false;
                self.set_feedback_with_next_step(
                    UiFeedbackLevel::Info,
                    UiFeedbackScope::Apply,
                    "Overview 推荐先进入 Apply 检查当前 blocker / warning。",
                    "在 Apply 查看预览并修复阻塞项",
                );
            }
            OverviewPrimaryActionKind::ApplyCurrentHost => {
                self.set_page(Page::Deploy);
                self.show_advanced = false;
                self.set_feedback_with_next_step(
                    UiFeedbackLevel::Info,
                    UiFeedbackScope::Apply,
                    "Overview 已把你带到 Apply；当前组合可直接执行。",
                    "在 Apply 查看预览，或按 a / x 直接运行",
                );
            }
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

    fn apply_managed_guard_focus(&mut self, focus: &ManagedGuardFocus) -> &'static str {
        match focus {
            ManagedGuardFocus::Packages { group, label } => {
                self.package_mode = PackageDataMode::Local;
                self.package_search_mode = false;
                self.package_group_create_mode = false;
                self.package_group_rename_mode = false;
                self.package_group_rename_source.clear();
                self.package_group_input.clear();
                self.package_group_filter = group.clone().and_then(|group| {
                    self.current_package_user().and_then(|user| {
                        self.package_groups_for_user(user)
                            .contains(&group)
                            .then_some(group)
                    })
                });
                self.ensure_valid_package_group_filter();
                self.clamp_package_cursor();
                label
            }
            ManagedGuardFocus::Home { index, label } => {
                let max = self.home_rows().len().saturating_sub(1);
                self.home_focus = (*index).min(max);
                label
            }
            ManagedGuardFocus::Users { index, label } => {
                self.users_focus = (*index).min(5);
                label
            }
            ManagedGuardFocus::Hosts { index, label } => {
                self.hosts_focus = (*index).min(28);
                label
            }
        }
    }
}

fn overview_dirty_page(section: &str) -> Page {
    match section {
        "Users" => Page::Users,
        "Hosts" => Page::Hosts,
        "Packages" => Page::Packages,
        "Home" => Page::Home,
        _ => Page::Dashboard,
    }
}

fn overview_dirty_scope(section: &str) -> UiFeedbackScope {
    match section {
        "Users" => UiFeedbackScope::Users,
        "Hosts" => UiFeedbackScope::Hosts,
        "Packages" => UiFeedbackScope::Packages,
        "Home" => UiFeedbackScope::Home,
        _ => UiFeedbackScope::Overview,
    }
}

fn overview_primary_action(
    dirty_sections: &[OverviewDirtySection],
    repo_integrity: &OverviewCheckState,
    doctor: &OverviewCheckState,
    managed_guards: &[ManagedGuardSnapshot],
    apply: &ApplyModel,
) -> OverviewPrimaryAction {
    if !dirty_sections.is_empty() {
        return OverviewPrimaryAction {
            kind: OverviewPrimaryActionKind::SaveDirtyPages,
            reason: format!(
                "存在未保存修改：{}。",
                dirty_sections
                    .iter()
                    .map(|section| format!("{}: {}", section.name, section.items.join(", ")))
                    .collect::<Vec<_>>()
                    .join(" | ")
            ),
        };
    }

    if matches!(repo_integrity, OverviewCheckState::Error { .. })
        || matches!(doctor, OverviewCheckState::Error { .. })
    {
        return OverviewPrimaryAction {
            kind: OverviewPrimaryActionKind::ReviewInspect,
            reason: "健康检查存在失败项；应先进入 Inspect 查看详情。".to_string(),
        };
    }

    if let Some(route) = preferred_managed_guard_route(managed_guards) {
        return OverviewPrimaryAction {
            kind: OverviewPrimaryActionKind::ReviewManagedGuards,
            reason: format!(
                "{}[{target}] 的受管保护存在阻塞：{reason}",
                route.label,
                target = route.target,
                reason = route.reason
            ),
        };
    }

    if !apply.handoffs.is_empty() {
        return OverviewPrimaryAction {
            kind: OverviewPrimaryActionKind::OpenAdvancedApply,
            reason: apply.handoffs.join(" | "),
        };
    }

    if apply.can_apply_current_host {
        return OverviewPrimaryAction {
            kind: OverviewPrimaryActionKind::ApplyCurrentHost,
            reason: "当前组合可直接执行 Apply。".to_string(),
        };
    }

    OverviewPrimaryAction {
        kind: OverviewPrimaryActionKind::ReviewApply,
        reason: if apply.blockers.is_empty() {
            "请先进入 Apply 查看当前组合详情。".to_string()
        } else {
            format!("当前组合仍有阻塞项：{}。", apply.blockers.join(" | "))
        },
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ManagedGuardRoute {
    page: Page,
    scope: UiFeedbackScope,
    label: &'static str,
    target: String,
    reason: String,
    focus: ManagedGuardFocus,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ManagedGuardFocus {
    Packages { group: Option<String>, label: &'static str },
    Home { index: usize, label: &'static str },
    Users { index: usize, label: &'static str },
    Hosts { index: usize, label: &'static str },
}

fn preferred_managed_guard_route(guards: &[ManagedGuardSnapshot]) -> Option<ManagedGuardRoute> {
    if let Some(guard) = guards
        .iter()
        .find(|guard| guard.available && guard.page == "Packages" && !guard.errors.is_empty())
    {
        let reason = guard.errors[0].clone();
        return Some(ManagedGuardRoute {
            page: Page::Packages,
            scope: UiFeedbackScope::Packages,
            label: "Packages",
            target: guard.target.clone(),
            focus: ManagedGuardFocus::Packages {
                group: package_focus_group_from_error(&reason),
                label: "软件组过滤",
            },
            reason,
        });
    }

    if let Some(guard) = guards
        .iter()
        .find(|guard| guard.available && guard.page == "Home" && !guard.errors.is_empty())
    {
        let reason = guard.errors[0].clone();
        return Some(ManagedGuardRoute {
            page: Page::Home,
            scope: UiFeedbackScope::Home,
            label: "Home",
            target: guard.target.clone(),
            focus: ManagedGuardFocus::Home {
                index: home_focus_index_for_guard_error(&reason),
                label: "设置列表",
            },
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
        return Some(ManagedGuardRoute {
            page: Page::Hosts,
            scope: UiFeedbackScope::Hosts,
            label: "Hosts",
            target: guard.target.clone(),
            focus: ManagedGuardFocus::Hosts {
                index: hosts_focus_index_for_guard_error(&reason),
                label: hosts_focus_label_for_guard_error(&reason),
            },
            reason,
        });
    }

    if let Some(guard) = users_guard.filter(|guard| {
        guard
            .errors
            .iter()
            .any(|error| is_users_guard_error(error))
    }) {
        let reason = first_matching_guard_error(&guard.errors, is_users_guard_error);
        return Some(ManagedGuardRoute {
            page: Page::Users,
            scope: UiFeedbackScope::Users,
            label: "Users",
            target: guard.target.clone(),
            focus: ManagedGuardFocus::Users {
                index: users_focus_index_for_guard_error(&reason),
                label: users_focus_label_for_guard_error(&reason),
            },
            reason,
        });
    }

    users_guard
        .map(|guard| ManagedGuardRoute {
            page: Page::Users,
            scope: UiFeedbackScope::Users,
            label: "Users",
            target: guard.target.clone(),
            focus: ManagedGuardFocus::Users {
                index: users_focus_index_for_guard_error(&guard.errors[0]),
                label: users_focus_label_for_guard_error(&guard.errors[0]),
            },
            reason: guard.errors[0].clone(),
        })
        .or_else(|| {
            hosts_guard.map(|guard| ManagedGuardRoute {
                page: Page::Hosts,
                scope: UiFeedbackScope::Hosts,
                label: "Hosts",
                target: guard.target.clone(),
                focus: ManagedGuardFocus::Hosts {
                    index: hosts_focus_index_for_guard_error(&guard.errors[0]),
                    label: hosts_focus_label_for_guard_error(&guard.errors[0]),
                },
                reason: guard.errors[0].clone(),
            })
        })
}

fn first_matching_guard_error(
    errors: &[String],
    predicate: impl Fn(&str) -> bool,
) -> String {
    errors
        .iter()
        .find(|error| predicate(error))
        .cloned()
        .unwrap_or_else(|| errors.first().cloned().unwrap_or_else(|| "unknown".to_string()))
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
            OverviewPrimaryActionKind::SaveDirtyPages
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
        assert!(model.primary_action.reason.contains("Packages: alice"));
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
        assert!(
            packages.errors[0].contains("refusing to remove stale unmanaged package file")
        );

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
            OverviewPrimaryActionKind::ReviewApply
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
        state.show_advanced = true;

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
        assert!(
            model
                .apply
                .handoffs
                .iter()
                .any(|item| item.contains("高级选项"))
        );
        assert!(!model.apply.can_execute_directly);
        assert!(!model.apply.can_apply_current_host);
        assert_eq!(
            model.primary_action.kind,
            OverviewPrimaryActionKind::OpenAdvancedApply
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
            OverviewPrimaryActionKind::ApplyCurrentHost
        );
    }

    #[test]
    fn overview_model_prefers_inspect_when_health_has_failures() {
        let mut state = test_state("sudo-available");
        state.overview_repo_integrity = OverviewCheckState::Error {
            summary: "failed (1 finding(s))".to_string(),
            details: vec!["- [rule] path: detail".to_string()],
        };

        let model = state.overview_model();

        assert_eq!(
            model.primary_action,
            OverviewPrimaryAction {
                kind: OverviewPrimaryActionKind::ReviewInspect,
                reason: "健康检查存在失败项；应先进入 Inspect 查看详情。".to_string(),
            }
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
            OverviewPrimaryActionKind::ReviewManagedGuards
        );
        assert!(model.primary_action.reason.contains("Packages[alice]"));
        assert!(model.primary_action.reason.contains("受管保护存在阻塞"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn open_overview_primary_action_routes_to_first_dirty_page() {
        let mut state = test_state("sudo-available");
        state.package_dirty_users.insert("alice".to_string());
        state.home_dirty_users.insert("alice".to_string());

        state.open_overview_primary_action();

        assert_eq!(state.page(), Page::Packages);
        assert_eq!(state.feedback.scope, UiFeedbackScope::Packages);
        assert!(state.status.contains("Packages 页仍有未保存修改"));
    }

    #[test]
    fn open_overview_primary_action_routes_to_inspect_for_health_failures() {
        let mut state = test_state("sudo-available");
        state.overview_repo_integrity = OverviewCheckState::Error {
            summary: "failed (1 finding(s))".to_string(),
            details: vec!["- [rule] path: detail".to_string()],
        };
        state.actions_focus = 4;

        state.open_overview_primary_action();

        assert_eq!(state.page(), Page::Inspect);
        assert_eq!(state.current_inspect_action(), ActionItem::FlakeCheck);
        assert_eq!(state.feedback.scope, UiFeedbackScope::Inspect);
        assert!(state.status.contains("Inspect"));
    }

    #[test]
    fn open_overview_primary_action_routes_to_packages_for_package_guard_blockers() -> Result<()> {
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

        assert_eq!(state.page(), Page::Packages);
        assert_eq!(state.feedback.scope, UiFeedbackScope::Packages);
        assert!(state.status.contains("受管保护阻塞"));
        assert_eq!(state.package_mode, PackageDataMode::Local);
        assert_eq!(state.package_group_filter.as_deref(), Some("manual"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn open_overview_primary_action_routes_runtime_guard_blockers_to_hosts() -> Result<()> {
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

        assert_eq!(state.page(), Page::Hosts);
        assert_eq!(state.feedback.scope, UiFeedbackScope::Hosts);
        assert!(state.status.contains("host-network"));
        assert_eq!(state.hosts_focus, 4);

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn open_overview_primary_action_routes_user_guard_blockers_to_users() -> Result<()> {
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

        assert_eq!(state.page(), Page::Users);
        assert_eq!(state.feedback.scope, UiFeedbackScope::Users);
        assert!(state.status.contains("host-users"));
        assert_eq!(state.users_focus, 2);

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn open_overview_primary_action_routes_home_guard_blockers_to_home() -> Result<()> {
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

        assert_eq!(state.page(), Page::Home);
        assert_eq!(state.feedback.scope, UiFeedbackScope::Home);
        assert!(state.status.contains("home-settings-desktop"));
        assert_eq!(state.home_focus, 0);

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn open_overview_primary_action_routes_to_advanced_apply_when_handoff_exists() {
        let mut state = test_state("sudo-available");
        state.deploy_source = DeploySource::RemoteHead;

        state.open_overview_primary_action();

        assert_eq!(state.page(), Page::Deploy);
        assert!(state.show_advanced);
        assert_eq!(state.current_advanced_action(), ActionItem::FlakeUpdate);
        assert_eq!(state.feedback.scope, UiFeedbackScope::Advanced);
        assert!(state.status.contains("Advanced Workspace"));
    }

    #[test]
    fn open_overview_primary_action_routes_to_apply_for_direct_apply() {
        let mut state = test_state("sudo-available");

        state.open_overview_primary_action();

        assert_eq!(state.page(), Page::Deploy);
        assert!(!state.show_advanced);
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
                catalog_entries: Vec::new(),
                catalog_groups: BTreeMap::new(),
                catalog_home_options: Vec::new(),
                catalog_categories: Vec::new(),
                catalog_sources: Vec::new(),
            },
            active_page: 0,
            deploy_focus: 0,
            target_host: "demo".to_string(),
            deploy_task: DeployTask::DirectDeploy,
            deploy_source: DeploySource::CurrentRepo,
            deploy_action: if privilege_mode == "rootless" {
                DeployAction::Build
            } else {
                DeployAction::Switch
            },
            flake_update: false,
            show_advanced: false,
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
