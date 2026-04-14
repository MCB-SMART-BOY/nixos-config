use super::*;

impl AppState {
    pub(crate) fn should_use_sudo(&self) -> bool {
        matches!(
            self.context.privilege_mode.as_str(),
            "sudo-session" | "sudo-available"
        )
    }

    pub(crate) fn manual_repo_sync_plan(&self) -> Option<RepoSyncPlan> {
        (self.context.repo_root != self.context.etc_root).then(|| RepoSyncPlan {
            source_dir: self.context.repo_root.clone(),
            destination_dir: self.context.etc_root.clone(),
            delete_extra: true,
        })
    }

    pub(crate) fn ensure_no_unsaved_changes_for_execution(&self) -> Result<()> {
        let mut dirty = Vec::new();
        if !self.host_dirty_user_hosts.is_empty() {
            dirty.push(format!(
                "Users: {}",
                self.host_dirty_user_hosts
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if !self.host_dirty_runtime_hosts.is_empty() {
            dirty.push(format!(
                "Hosts: {}",
                self.host_dirty_runtime_hosts
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if !self.package_dirty_users.is_empty() {
            dirty.push(format!(
                "Packages: {}",
                self.package_dirty_users
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if !self.home_dirty_users.is_empty() {
            dirty.push(format!(
                "Home: {}",
                self.home_dirty_users
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        if dirty.is_empty() {
            return Ok(());
        }

        anyhow::bail!("仍有未保存修改；请先保存后再执行：{}", dirty.join(" | "))
    }

    pub(crate) fn clean_etc_dir_keep_hardware(&self) -> Result<()> {
        if self.context.etc_root.as_os_str().is_empty()
            || self.context.etc_root.as_path() == std::path::Path::new("/")
        {
            anyhow::bail!(
                "ETC_ROOT 无效，拒绝清理：{}",
                self.context.etc_root.display()
            );
        }
        if !self.context.etc_root.is_dir() {
            return Ok(());
        }

        let preserve = std::env::temp_dir().join(format!(
            "mcbctl-hw-preserve-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0)
        ));
        fs::create_dir_all(&preserve)
            .with_context(|| format!("failed to create {}", preserve.display()))?;

        let etc_hw = host_hardware_config_path(&self.context.etc_root, &self.context.current_host);
        if etc_hw.is_file() {
            let preserved = preserve
                .join("hosts")
                .join(&self.context.current_host)
                .join("hardware-configuration.nix");
            if let Some(parent) = preserved.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create {}", parent.display()))?;
            }
            fs::copy(&etc_hw, &preserved)
                .with_context(|| format!("failed to preserve {}", etc_hw.display()))?;
        }

        for entry in fs::read_dir(&self.context.etc_root)
            .with_context(|| format!("failed to read {}", self.context.etc_root.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            let is_hw = path == etc_hw;
            if is_hw {
                continue;
            }
            if path.is_dir() {
                fs::remove_dir_all(&path)
                    .with_context(|| format!("failed to remove {}", path.display()))?;
            } else {
                fs::remove_file(&path)
                    .with_context(|| format!("failed to remove {}", path.display()))?;
            }
        }

        let preserved_root = preserve
            .join("hosts")
            .join(&self.context.current_host)
            .join("hardware-configuration.nix");
        if preserved_root.is_file() {
            if let Some(parent) = etc_hw.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create {}", parent.display()))?;
            }
            fs::copy(&preserved_root, &etc_hw)
                .with_context(|| format!("failed to restore {}", etc_hw.display()))?;
        }
        fs::remove_dir_all(preserve).ok();
        Ok(())
    }

    pub(crate) fn run_sibling_in_repo(
        &self,
        name: &str,
        args: &[String],
    ) -> Result<std::process::ExitStatus> {
        let binary = resolve_sibling_binary(name)?;
        std::process::Command::new(&binary)
            .args(args)
            .current_dir(&self.context.repo_root)
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status()
            .with_context(|| format!("failed to run {}", binary.display()))
    }

    pub(crate) fn action_available(&self, action: ActionItem) -> bool {
        match action {
            ActionItem::SyncRepoToEtc => {
                self.context.repo_root != self.context.etc_root
                    && self.context.privilege_mode != "rootless"
            }
            ActionItem::RebuildCurrentHost => !self.context.current_host.is_empty(),
            _ => true,
        }
    }

    pub(crate) fn action_command_preview(&self, action: ActionItem) -> Option<String> {
        match action {
            ActionItem::FlakeCheck => Some(format!(
                "nix --extra-experimental-features 'nix-command flakes' flake check path:{}",
                self.context.repo_root.display()
            )),
            ActionItem::FlakeUpdate => Some(format!(
                "nix --extra-experimental-features 'nix-command flakes' flake update --flake {}",
                self.context.repo_root.display()
            )),
            ActionItem::UpdateUpstreamCheck => Some("update-upstream-apps --check".to_string()),
            ActionItem::UpdateUpstreamPins => Some("update-upstream-apps".to_string()),
            ActionItem::SyncRepoToEtc => self
                .manual_repo_sync_plan()
                .map(|plan| plan.command_preview()),
            ActionItem::RebuildCurrentHost => {
                let action = if self.context.privilege_mode == "rootless" {
                    DeployAction::Build
                } else {
                    DeployAction::Switch
                };
                let plan = NixosRebuildPlan {
                    action,
                    upgrade: false,
                    flake_root: if self.context.repo_root == self.context.etc_root {
                        self.context.repo_root.clone()
                    } else {
                        self.context.etc_root.clone()
                    },
                    target_host: self.context.current_host.clone(),
                };
                Some(plan.command_preview(self.should_use_sudo()))
            }
            ActionItem::LaunchDeployWizard => Some("mcb-deploy".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};

    #[test]
    fn should_use_sudo_only_for_sudo_capable_modes() {
        assert!(test_state("sudo-session").should_use_sudo());
        assert!(test_state("sudo-available").should_use_sudo());
        assert!(!test_state("root").should_use_sudo());
        assert!(!test_state("rootless").should_use_sudo());
    }

    #[test]
    fn ensure_no_unsaved_changes_reports_all_dirty_sections() {
        let mut state = test_state("sudo-available");
        state.host_dirty_user_hosts.insert("demo".to_string());
        state.host_dirty_runtime_hosts.insert("demo".to_string());
        state.package_dirty_users.insert("alice".to_string());
        state.home_dirty_users.insert("alice".to_string());

        let err = state
            .ensure_no_unsaved_changes_for_execution()
            .expect_err("dirty state should block execution");
        let text = err.to_string();
        assert!(text.contains("Users: demo"));
        assert!(text.contains("Hosts: demo"));
        assert!(text.contains("Packages: alice"));
        assert!(text.contains("Home: alice"));
    }

    #[test]
    fn sync_action_availability_depends_on_repo_location_and_privilege() {
        let state = test_state("sudo-available");
        assert!(state.action_available(ActionItem::SyncRepoToEtc));

        let rootless = test_state("rootless");
        assert!(!rootless.action_available(ActionItem::SyncRepoToEtc));

        let same_root = test_state_with_paths("sudo-available", "/repo", "/repo");
        assert!(!same_root.action_available(ActionItem::SyncRepoToEtc));
    }

    #[test]
    fn rebuild_current_host_preview_uses_build_for_rootless() {
        let state = test_state("rootless");
        let preview = state
            .action_command_preview(ActionItem::RebuildCurrentHost)
            .expect("preview should exist");

        assert!(preview.contains("nixos-rebuild build"));
        assert!(preview.contains("/etc/nixos#demo"));
        assert!(!preview.starts_with("sudo "));
    }

    #[test]
    fn rebuild_current_host_preview_uses_switch_and_sudo_when_available() {
        let state = test_state("sudo-available");
        let preview = state
            .action_command_preview(ActionItem::RebuildCurrentHost)
            .expect("preview should exist");

        assert!(preview.contains("sudo -E env"));
        assert!(preview.contains("nixos-rebuild switch"));
        assert!(preview.contains("/etc/nixos#demo"));
    }

    #[test]
    fn sync_action_preview_disappears_when_repo_is_already_etc() {
        let state = test_state_with_paths("sudo-available", "/repo", "/repo");
        assert!(
            state
                .action_command_preview(ActionItem::SyncRepoToEtc)
                .is_none()
        );
    }

    fn test_state(privilege_mode: &str) -> AppState {
        test_state_with_paths(privilege_mode, "/repo", "/etc/nixos")
    }

    fn test_state_with_paths(privilege_mode: &str, repo_root: &str, etc_root: &str) -> AppState {
        AppState {
            context: AppContext {
                repo_root: PathBuf::from(repo_root),
                etc_root: PathBuf::from(etc_root),
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
            deploy_action: DeployAction::Switch,
            flake_update: false,
            show_advanced: false,
            users_focus: 0,
            hosts_focus: 0,
            users_text_mode: None,
            hosts_text_mode: None,
            host_text_input: String::new(),
            host_settings_by_name: BTreeMap::new(),
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
