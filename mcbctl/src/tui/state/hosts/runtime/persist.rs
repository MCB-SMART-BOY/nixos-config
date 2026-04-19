use super::*;

impl AppState {
    pub fn save_current_host_runtime(&mut self) -> Result<()> {
        let host = self.target_host.clone();
        let errors = self.host_configuration_validation_errors(&host);
        if !errors.is_empty() {
            self.set_feedback_with_next_step(
                UiFeedbackLevel::Error,
                UiFeedbackScope::Hosts,
                format!(
                    "当前主机的整机配置未通过校验，运行时分片未写入：{}",
                    errors.join("；")
                ),
                "先处理 Hosts Summary 里的校验或受管保护，再重试保存。",
            );
            return Ok(());
        }

        let Some(settings) = self.current_host_settings().cloned() else {
            self.set_feedback_with_next_step(
                UiFeedbackLevel::Error,
                UiFeedbackScope::Hosts,
                "Hosts 没有可保存的主机运行时配置。",
                "先补可用 host 配置，再继续编辑。",
            );
            return Ok(());
        };

        let host_dir = self.context.repo_root.join("hosts").join(&host);
        let managed_dir = host_dir.join("managed");
        let paths = match ensure_managed_host_layout(&managed_dir)
            .and_then(|()| write_host_runtime_fragments(&managed_dir, &settings))
        {
            Ok(paths) => paths,
            Err(err) => {
                self.set_feedback_with_next_step(
                    UiFeedbackLevel::Error,
                    UiFeedbackScope::Hosts,
                    format!("Hosts 未写入：{err:#}"),
                    "先处理 Hosts Summary 里的受管保护，再重试保存。",
                );
                return Ok(());
            }
        };
        self.host_dirty_runtime_hosts.remove(&host);
        self.set_feedback_with_next_step(
            UiFeedbackLevel::Success,
            UiFeedbackScope::Hosts,
            format!(
                "Hosts 已写入 {}",
                paths
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>()
                    .join("、")
            ),
            "继续编辑 Hosts，或切到 Apply / Overview 复查。",
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{managed_file_is_valid, managed_file_kind};
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn save_current_host_runtime_rejects_invalid_combined_configuration() -> Result<()> {
        let root = create_temp_repo("mcbctl-host-runtime-invalid")?;
        let mut state = test_state(&root);
        if let Some(settings) = state.host_settings_by_name.get_mut("demo") {
            settings.primary_user.clear();
        }
        state.host_dirty_runtime_hosts.insert("demo".to_string());

        state.save_current_host_runtime()?;

        let paths = state.current_host_runtime_paths();
        assert!(paths.iter().all(|path| !path.exists()));
        assert!(state.host_dirty_runtime_hosts.contains("demo"));
        assert!(state.status.contains("整机配置未通过校验"));
        assert!(state.status.contains("主用户不能为空"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn save_current_host_runtime_writes_all_fragments_and_clears_dirty() -> Result<()> {
        let root = create_temp_repo("mcbctl-host-runtime-save")?;
        let mut state = test_state(&root);
        if let Some(settings) = state.host_settings_by_name.get_mut("demo") {
            settings.proxy_mode = "http".to_string();
            settings.proxy_url = "http://127.0.0.1:7890".to_string();
            settings.enable_proxy_dns = false;
            settings.gpu_mode = "dgpu".to_string();
            settings.docker_enable = true;
        }
        state.host_dirty_runtime_hosts.insert("demo".to_string());

        state.save_current_host_runtime()?;

        let paths = state.current_host_runtime_paths();
        let contents = paths
            .iter()
            .map(std::fs::read_to_string)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        assert_eq!(managed_file_kind(&contents[0]), Some("host-network"));
        assert_eq!(managed_file_kind(&contents[1]), Some("host-gpu"));
        assert_eq!(managed_file_kind(&contents[2]), Some("host-virtualization"));
        assert!(
            contents
                .iter()
                .all(|content| managed_file_is_valid(content))
        );
        assert!(contents[0].contains("mcb.proxyMode = lib.mkForce \"http\";"));
        assert!(contents[0].contains("mcb.proxyUrl = lib.mkForce \"http://127.0.0.1:7890\";"));
        assert!(contents[1].contains("mcb.hardware.gpu.mode = lib.mkForce \"dgpu\";"));
        assert!(contents[2].contains("mcb.virtualisation.docker.enable = lib.mkForce true;"));
        assert!(!state.host_dirty_runtime_hosts.contains("demo"));
        assert!(state.status.contains("network.nix"));
        assert!(state.status.contains("gpu.nix"));
        assert!(state.status.contains("virtualization.nix"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn save_current_host_runtime_rejects_tampered_sibling_fragment_and_keeps_dirty() -> Result<()> {
        let root = create_temp_repo("mcbctl-host-runtime-tampered")?;
        let mut state = test_state(&root);
        let managed_dir = root.join("hosts/demo/managed");
        std::fs::create_dir_all(&managed_dir)?;
        std::fs::write(
            managed_dir.join("users.nix"),
            "{ lib, ... }: { mcb.user = lib.mkForce \"alice\"; }\n",
        )?;
        state.host_dirty_runtime_hosts.insert("demo".to_string());

        state.save_current_host_runtime()?;

        let paths = state.current_host_runtime_paths();
        assert!(paths.iter().all(|path| !path.exists()));
        assert!(state.host_dirty_runtime_hosts.contains("demo"));
        assert!(state.status.contains("Hosts 未写入"));
        assert!(state.status.contains("host-users"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    fn test_state(root: &Path) -> AppState {
        let mut host_settings_by_name = BTreeMap::new();
        host_settings_by_name.insert("demo".to_string(), valid_host_settings());

        AppState {
            context: AppContext {
                repo_root: root.to_path_buf(),
                etc_root: PathBuf::from("/etc/nixos"),
                current_host: "demo".to_string(),
                current_system: "x86_64-linux".to_string(),
                current_user: "alice".to_string(),
                privilege_mode: "sudo-available".to_string(),
                hosts: vec!["demo".to_string()],
                users: vec!["alice".to_string()],
                catalog_path: root.join("catalog/packages"),
                catalog_groups_path: root.join("catalog/groups.toml"),
                catalog_home_options_path: root.join("catalog/home-options.toml"),
                catalog_workflows_path: root.join("catalog/workflows.toml"),
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

    fn valid_host_settings() -> HostManagedSettings {
        HostManagedSettings {
            primary_user: "alice".to_string(),
            users: vec!["alice".to_string()],
            admin_users: vec!["alice".to_string()],
            ..HostManagedSettings::default()
        }
    }

    fn create_temp_repo(prefix: &str) -> Result<PathBuf> {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let root = std::env::temp_dir().join(format!("{prefix}-{}-{unique}", std::process::id()));
        std::fs::create_dir_all(&root)?;
        Ok(root)
    }
}
