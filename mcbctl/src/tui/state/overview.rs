use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OverviewModel {
    pub(crate) context: OverviewContext,
    pub(crate) dirty_sections: Vec<OverviewDirtySection>,
    pub(crate) host_status: OverviewHostStatus,
    pub(crate) repo_integrity: OverviewCheckState,
    pub(crate) doctor: OverviewCheckState,
    pub(crate) apply: OverviewApplySummary,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum OverviewCheckState {
    NotRun,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OverviewApplySummary {
    pub(crate) task: DeployTask,
    pub(crate) source: DeploySource,
    pub(crate) action: DeployAction,
    pub(crate) flake_update: bool,
    pub(crate) advanced: bool,
    pub(crate) sync_preview: Option<String>,
    pub(crate) rebuild_preview: Option<String>,
    pub(crate) can_execute_directly: bool,
    pub(crate) can_apply_current_host: bool,
    pub(crate) blockers: Vec<String>,
    pub(crate) warnings: Vec<String>,
    pub(crate) handoffs: Vec<String>,
    pub(crate) infos: Vec<String>,
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
            repo_integrity: OverviewCheckState::NotRun,
            doctor: OverviewCheckState::NotRun,
            apply: self.overview_apply_summary(),
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

    fn overview_apply_summary(&self) -> OverviewApplySummary {
        let can_execute_directly = self.can_execute_deploy_directly();
        let sync_preview = self
            .deploy_sync_plan_for_execution()
            .map(|plan| plan.command_preview());
        let rebuild_preview = self
            .deploy_rebuild_plan_for_execution()
            .map(|plan| plan.command_preview(self.should_use_sudo()));

        let mut blockers = Vec::new();
        for section in self.overview_dirty_sections() {
            blockers.push(format!(
                "{} 存在未保存修改：{}",
                section.name,
                section.items.join(", ")
            ));
        }

        match self.overview_host_status() {
            OverviewHostStatus::Ready => {}
            OverviewHostStatus::Unavailable { message } => blockers.push(message),
            OverviewHostStatus::Invalid { errors } => blockers.extend(
                errors
                    .into_iter()
                    .map(|error| format!("主机 {} 配置未通过校验：{error}", self.target_host)),
            ),
        }

        if self.context.privilege_mode == "rootless" && self.deploy_action != DeployAction::Build {
            blockers.push(
                "rootless 模式下只能直接执行 build；如需 switch/test/boot，请切换 sudo/root 或进入 Advanced。"
                    .to_string(),
            );
        }

        let mut warnings = Vec::new();
        if let Some(preview) = &sync_preview {
            warnings.push(format!("当前组合会先把仓库同步到 /etc/nixos：{preview}"));
        }
        if self.flake_update {
            warnings.push("当前组合会以 --upgrade 执行重建。".to_string());
        }
        if self.should_use_sudo() {
            warnings.push("当前组合会使用 sudo -E 执行受权命令。".to_string());
        }
        let needs_real_hardware = !(self.context.privilege_mode == "rootless"
            && self.deploy_action == DeployAction::Build);
        if needs_real_hardware {
            warnings.push(format!(
                "当前组合要求 {} 存在真实 hardware-configuration.nix。",
                host_hardware_config_path(&self.context.etc_root, &self.target_host).display()
            ));
        }

        let mut handoffs = Vec::new();
        match self.deploy_source {
            DeploySource::RemotePinned => {
                handoffs.push("远端固定版本必须交给 Advanced Deploy 处理。".to_string())
            }
            DeploySource::RemoteHead => {
                handoffs.push("远端最新版本必须交给 Advanced Deploy 处理。".to_string())
            }
            DeploySource::CurrentRepo | DeploySource::EtcNixos => {}
        }
        if self.show_advanced {
            handoffs.push("当前已打开高级选项，应交给 Advanced Deploy 处理。".to_string());
        }

        let mut infos = Vec::new();
        if !can_execute_directly {
            infos.push("当前组合不会直接执行，而是回退到完整 deploy wizard。".to_string());
        }
        infos.push("repo-integrity 还未刷新。".to_string());
        infos.push("doctor 还未刷新。".to_string());

        OverviewApplySummary {
            task: self.deploy_task,
            source: self.deploy_source,
            action: self.deploy_action,
            flake_update: self.flake_update,
            advanced: self.show_advanced,
            sync_preview,
            rebuild_preview,
            can_execute_directly,
            can_apply_current_host: can_execute_directly && blockers.is_empty(),
            blockers,
            warnings,
            handoffs,
            infos,
        }
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
        assert_eq!(model.repo_integrity, OverviewCheckState::NotRun);
        assert_eq!(model.doctor, OverviewCheckState::NotRun);
        assert!(
            model
                .apply
                .blockers
                .iter()
                .any(|item| item.contains("Packages 存在未保存修改"))
        );
        assert!(
            model
                .apply
                .blockers
                .iter()
                .any(|item| item.contains("Home 存在未保存修改"))
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
                .any(|item| item.contains("rootless 模式下只能直接执行 build"))
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
            status: String::new(),
        }
    }
}
