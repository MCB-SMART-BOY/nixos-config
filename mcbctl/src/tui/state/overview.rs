use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OverviewModel {
    pub(crate) context: OverviewContext,
    pub(crate) dirty_sections: Vec<OverviewDirtySection>,
    pub(crate) host_status: OverviewHostStatus,
    pub(crate) repo_integrity: OverviewCheckState,
    pub(crate) doctor: OverviewCheckState,
    pub(crate) apply: ApplyModel,
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
        OverviewModel {
            context: OverviewContext {
                current_host: self.context.current_host.clone(),
                target_host: self.target_host.clone(),
                current_user: self.context.current_user.clone(),
                privilege_mode: self.context.privilege_mode.clone(),
                repo_root: self.context.repo_root.clone(),
                etc_root: self.context.etc_root.clone(),
            },
            dirty_sections: self.overview_dirty_sections(),
            host_status: self.overview_host_status(),
            repo_integrity: self.overview_repo_integrity.clone(),
            doctor: self.overview_doctor.clone(),
            apply: self.apply_model(),
        }
    }

    pub(crate) fn refresh_overview_repo_integrity(&mut self) {
        self.overview_repo_integrity = OverviewCheckState::Running;
        let snapshot = repo_integrity_check_state(&self.context.repo_root);
        self.status = format!(
            "Overview: repo-integrity 已刷新（{}）。",
            snapshot.short_outcome()
        );
        self.overview_repo_integrity = snapshot;
    }

    pub(crate) fn refresh_overview_doctor(&mut self) {
        self.overview_doctor = OverviewCheckState::Running;
        let snapshot = doctor_check_state(&self.context.repo_root);
        self.status = format!("Overview: doctor 已刷新（{}）。", snapshot.short_outcome());
        self.overview_doctor = snapshot;
    }

    pub(crate) fn refresh_overview_health(&mut self) {
        self.overview_repo_integrity = OverviewCheckState::Running;
        self.overview_doctor = OverviewCheckState::Running;
        let repo_snapshot = repo_integrity_check_state(&self.context.repo_root);
        let doctor_snapshot = doctor_check_state(&self.context.repo_root);
        self.status = format!(
            "Overview 健康项已刷新：repo-integrity={}，doctor={}。",
            repo_snapshot.short_outcome(),
            doctor_snapshot.short_outcome()
        );
        self.overview_repo_integrity = repo_snapshot;
        self.overview_doctor = doctor_snapshot;
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
            model.repo_integrity,
            OverviewCheckState::Healthy {
                summary: "ok".to_string(),
                details: Vec::new()
            }
        );
        assert_eq!(model.doctor, OverviewCheckState::NotRun);
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
                blocking_issues: vec!["缺少 nixos-rebuild".to_string()],
                warnings: vec!["缺少 cargo".to_string()],
            },
        }));

        assert_eq!(
            state,
            OverviewCheckState::Error {
                summary: "failed (3 issue(s))".to_string(),
                details: vec![
                    "repo layout: layout broken".to_string(),
                    "deployment environment: 缺少 nixos-rebuild".to_string(),
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
            status: String::new(),
        }
    }
}
