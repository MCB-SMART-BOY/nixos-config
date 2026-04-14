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

    pub(super) fn block_when_current_host_settings_unavailable(&mut self, action: &str) -> bool {
        let Some(message) = self.current_host_settings_unavailable_message() else {
            return false;
        };
        self.status = format!("{action}：{message}");
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
            deploy_action: DeployAction::Switch,
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
            overview_repo_integrity: OverviewCheckState::NotRun,
            overview_doctor: OverviewCheckState::NotRun,
            status: String::new(),
        }
    }
}
