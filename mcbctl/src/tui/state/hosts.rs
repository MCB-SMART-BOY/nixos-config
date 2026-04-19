use super::*;
use anyhow::Result;

use self::runtime::validate_host_runtime_settings;
use self::users::validate_host_user_settings;

mod runtime;
mod users;

impl AppState {
    pub(super) fn host_settings(&self, host: &str) -> Option<&HostManagedSettings> {
        self.host_settings_by_name.get(host)
    }

    pub(super) fn host_settings_load_error(&self, host: &str) -> Option<&str> {
        self.host_settings_errors_by_name
            .get(host)
            .map(String::as_str)
    }

    pub(super) fn host_settings_unavailable_message(&self, host: &str) -> String {
        if let Some(error) = self.host_settings_load_error(host) {
            format!("主机 {host} 的配置读取失败：{error}")
        } else {
            format!("主机 {host} 没有可用配置。")
        }
    }

    pub(super) fn current_host_settings_unavailable_message(&self) -> Option<String> {
        self.current_host_settings()
            .is_none()
            .then(|| self.host_settings_unavailable_message(&self.target_host))
    }

    pub(super) fn host_managed_guard_errors(&self, host: &str) -> Vec<String> {
        let managed_dir = self
            .context
            .repo_root
            .join("hosts")
            .join(host)
            .join("managed");
        [
            ("default.nix", "host-managed-default"),
            ("users.nix", "host-users"),
            ("network.nix", "host-network"),
            ("gpu.nix", "host-gpu"),
            ("virtualization.nix", "host-virtualization"),
        ]
        .into_iter()
        .filter_map(|(name, kind)| {
            crate::ensure_existing_managed_file(&managed_dir.join(name), kind)
                .err()
                .map(|err| err.to_string())
        })
        .collect()
    }

    pub(super) fn current_host_managed_guard_errors(&self) -> Vec<String> {
        self.host_managed_guard_errors(&self.target_host)
    }

    pub(crate) fn current_host_users_managed_guard_errors(&self) -> Vec<String> {
        self.current_host_managed_guard_errors()
            .into_iter()
            .filter(|error| error.contains("host-users") || error.contains("host-managed-default"))
            .collect()
    }

    pub(crate) fn current_host_runtime_managed_guard_errors(&self) -> Vec<String> {
        self.current_host_managed_guard_errors()
            .into_iter()
            .filter(|error| {
                error.contains("host-network")
                    || error.contains("host-gpu")
                    || error.contains("host-virtualization")
            })
            .collect()
    }

    pub(super) fn block_when_current_host_settings_unavailable(
        &mut self,
        scope: UiFeedbackScope,
        action: &str,
    ) -> bool {
        let Some(message) = self.current_host_settings_unavailable_message() else {
            return false;
        };
        self.set_feedback_with_next_step(
            UiFeedbackLevel::Error,
            scope,
            format!("{action}：{message}"),
            "先修复当前 host 的配置读取问题，再继续编辑。",
        );
        true
    }

    pub(super) fn current_host_unavailable_value(&self) -> Option<String> {
        self.current_host_settings_unavailable_message()
            .map(|message| {
                if message.contains("配置读取失败") {
                    "配置读取失败".to_string()
                } else {
                    "不可用".to_string()
                }
            })
    }

    pub(super) fn host_configuration_validation_errors(&self, host: &str) -> Vec<String> {
        if let Some(error) = self.host_settings_load_error(host) {
            return vec![format!("主机 {host} 的配置读取失败：{error}")];
        }
        let Some(settings) = self.host_settings(host) else {
            return vec![format!("主机 {host} 没有可用配置。")];
        };

        let mut errors = validate_host_user_settings(settings);
        errors.extend(validate_host_runtime_settings(settings));
        errors
    }

    pub(super) fn current_host_settings(&self) -> Option<&HostManagedSettings> {
        self.host_settings(&self.target_host)
    }

    fn current_host_settings_mut(&mut self) -> Option<&mut HostManagedSettings> {
        self.host_settings_by_name.get_mut(&self.target_host)
    }

    pub(crate) fn ensure_host_configuration_is_valid(&self, host: &str) -> Result<()> {
        let errors = self.host_configuration_validation_errors(host);
        if errors.is_empty() {
            return Ok(());
        }

        anyhow::bail!("主机 {host} 的 TUI 配置未通过校验：{}", errors.join("；"))
    }

    pub fn current_host_users_path(&self) -> Option<PathBuf> {
        let host = self
            .context
            .hosts
            .iter()
            .find(|name| *name == &self.target_host)?;
        Some(managed_host_users_path(&self.context.repo_root, host))
    }

    pub fn current_host_runtime_paths(&self) -> Vec<PathBuf> {
        let Some(host) = self
            .context
            .hosts
            .iter()
            .find(|name| *name == &self.target_host)
        else {
            return Vec::new();
        };

        vec![
            managed_host_network_path(&self.context.repo_root, host),
            managed_host_gpu_path(&self.context.repo_root, host),
            managed_host_virtualization_path(&self.context.repo_root, host),
        ]
    }

    pub fn switch_target_host(&mut self, delta: i8) {
        cycle_string(&mut self.target_host, &self.context.hosts, delta);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn host_configuration_validation_errors_surface_load_failures() {
        let mut state = test_state();
        state.host_settings_by_name.clear();
        state.host_settings_errors_by_name.insert(
            "demo".to_string(),
            "nix eval for host demo failed".to_string(),
        );

        let errors = state.host_configuration_validation_errors("demo");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("配置读取失败"));
        assert!(errors[0].contains("nix eval for host demo failed"));
    }

    #[test]
    fn ensure_host_configuration_is_valid_rejects_unavailable_hosts() {
        let mut state = test_state();
        state.host_settings_by_name.clear();
        state.host_settings_errors_by_name.insert(
            "demo".to_string(),
            "failed to parse evaluated host config for demo".to_string(),
        );

        let err = state
            .ensure_host_configuration_is_valid("demo")
            .expect_err("unavailable host settings should block validation");
        assert!(err.to_string().contains("配置未通过校验"));
        assert!(err.to_string().contains("配置读取失败"));
    }

    #[test]
    fn current_host_managed_guard_errors_surface_invalid_sibling_fragments() -> Result<()> {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let root = std::env::temp_dir().join(format!(
            "mcbctl-host-guard-errors-{}-{unique}",
            std::process::id()
        ));
        std::fs::create_dir_all(root.join("hosts/demo/managed"))?;
        std::fs::write(
            root.join("hosts/demo/managed/network.nix"),
            "{ lib, ... }: { mcb.proxyMode = lib.mkForce \"http\"; }\n",
        )?;

        let mut state = test_state();
        state.context.repo_root = root.clone();

        let errors = state.current_host_managed_guard_errors();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("host-network"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    fn test_state() -> AppState {
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
                privilege_mode: "sudo-available".to_string(),
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
            deploy_action: DeployAction::Switch,
            flake_update: false,
            advanced_target_host: "demo".to_string(),
            advanced_deploy_task: DeployTask::DirectDeploy,
            advanced_deploy_source: DeploySource::CurrentRepo,
            advanced_deploy_source_ref: String::new(),
            advanced_deploy_action: DeployAction::Switch,
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
            overview_repo_integrity: OverviewCheckState::NotRun,
            overview_doctor: OverviewCheckState::NotRun,
            feedback: UiFeedback::default(),
            status: String::new(),
        }
    }
}
