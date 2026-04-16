use super::*;
use crate::repo::ensure_repository_integrity;

impl AppState {
    pub fn execute_current_action_from_actions(&mut self) -> Result<()> {
        self.open_current_action_destination();
        Ok(())
    }

    pub fn execute_current_action(&mut self) -> Result<()> {
        self.ensure_no_unsaved_changes_for_execution()?;
        ensure_repository_integrity(&self.context.repo_root)?;
        let action = self.current_action_item();
        if !self.action_available(action) {
            anyhow::bail!("当前环境暂不适合直接执行动作：{}", action.label());
        }
        let use_sudo = self.should_use_sudo();

        match action {
            ActionItem::FlakeCheck => {
                let mut cmd = std::process::Command::new("nix");
                cmd.arg("--extra-experimental-features")
                    .arg("nix-command flakes")
                    .arg("flake")
                    .arg("check")
                    .arg(format!("path:{}", self.context.repo_root.display()))
                    .env("NIX_CONFIG", merged_nix_config())
                    .stdin(std::process::Stdio::inherit())
                    .stdout(std::process::Stdio::inherit())
                    .stderr(std::process::Stdio::inherit());
                let status = cmd.status().context("failed to run nix flake check")?;
                if !status.success() {
                    anyhow::bail!("flake check exited with {}", status.code().unwrap_or(1));
                }
                self.set_inspect_completion_feedback(ActionItem::FlakeCheck);
            }
            ActionItem::FlakeUpdate => {
                let mut cmd = std::process::Command::new("nix");
                cmd.arg("--extra-experimental-features")
                    .arg("nix-command flakes")
                    .arg("flake")
                    .arg("update")
                    .arg("--flake")
                    .arg(self.context.repo_root.display().to_string())
                    .env("NIX_CONFIG", merged_nix_config())
                    .stdin(std::process::Stdio::inherit())
                    .stdout(std::process::Stdio::inherit())
                    .stderr(std::process::Stdio::inherit());
                let status = cmd.status().context("failed to run nix flake update")?;
                if !status.success() {
                    anyhow::bail!("flake update exited with {}", status.code().unwrap_or(1));
                }
                self.set_advanced_maintenance_completion_feedback(ActionItem::FlakeUpdate);
            }
            ActionItem::UpdateUpstreamCheck => {
                let status =
                    self.run_sibling_in_repo("update-upstream-apps", &["--check".to_string()])?;
                if !status.success() {
                    anyhow::bail!(
                        "update-upstream-apps --check exited with {}",
                        status.code().unwrap_or(1)
                    );
                }
                self.set_inspect_completion_feedback(ActionItem::UpdateUpstreamCheck);
            }
            ActionItem::UpdateUpstreamPins => {
                let status = self.run_sibling_in_repo("update-upstream-apps", &[])?;
                if !status.success() {
                    anyhow::bail!(
                        "update-upstream-apps exited with {}",
                        status.code().unwrap_or(1)
                    );
                }
                self.set_advanced_maintenance_completion_feedback(ActionItem::UpdateUpstreamPins);
            }
            ActionItem::SyncRepoToEtc => {
                let plan = self
                    .manual_repo_sync_plan()
                    .context("当前仓库已经是 /etc/nixos，无需同步")?;
                run_repo_sync(
                    &plan,
                    |cmd, args| {
                        let status = std::process::Command::new(cmd)
                            .args(args)
                            .stdin(std::process::Stdio::inherit())
                            .stdout(std::process::Stdio::inherit())
                            .stderr(std::process::Stdio::inherit())
                            .status()
                            .with_context(|| format!("failed to run {cmd}"))?;
                        if status.success() {
                            Ok(())
                        } else {
                            anyhow::bail!("{cmd} failed with {}", status.code().unwrap_or(1));
                        }
                    },
                    |cmd, args| run_root_command_ok(cmd, args, use_sudo),
                    || self.clean_etc_dir_keep_hardware(),
                )?;
                self.set_sync_repo_completion_feedback();
            }
            ActionItem::RebuildCurrentHost => {
                self.ensure_host_configuration_is_valid(&self.context.current_host)?;
                let action = if self.context.privilege_mode == "rootless" {
                    DeployAction::Build
                } else {
                    DeployAction::Switch
                };
                if action != DeployAction::Build {
                    ensure_host_hardware_config(
                        &self.context.etc_root,
                        &self.context.current_host,
                        use_sudo,
                    )?;
                }
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
                let status = run_nixos_rebuild(&plan, use_sudo)?;
                if !status.success() {
                    anyhow::bail!("nixos-rebuild exited with {}", status.code().unwrap_or(1));
                }
                self.set_current_host_rebuild_completion_feedback(&plan);
            }
            ActionItem::LaunchDeployWizard => {
                if let Some(error) = self.current_deploy_wizard_validation_error() {
                    anyhow::bail!("{error}");
                }
                let status =
                    self.run_sibling_in_repo("mcb-deploy", &self.current_deploy_wizard_args())?;
                if !status.success() {
                    anyhow::bail!("mcb-deploy exited with {}", status.code().unwrap_or(1));
                }
                self.set_deploy_wizard_return_feedback();
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::tui::Page;
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::PathBuf;

    #[test]
    fn actions_execute_shortcut_routes_inspect_actions_instead_of_running_them() {
        let mut state = test_state();

        state
            .execute_current_action_from_actions()
            .expect("route inspect");

        assert_eq!(state.page(), Page::Inspect);
        assert!(state.status.contains("Inspect"));
    }

    #[test]
    fn actions_execute_shortcut_routes_apply_actions_instead_of_running_them() {
        let mut state = test_state();
        state.actions_focus = 2;

        state
            .execute_current_action_from_actions()
            .expect("route apply");

        assert_eq!(state.page(), Page::Deploy);
        assert!(!state.show_advanced);
        assert!(state.status.contains("Apply"));
    }

    #[test]
    fn actions_execute_shortcut_routes_advanced_actions_into_advanced_area() {
        let mut state = test_state();
        state.actions_focus = 4;

        state
            .execute_current_action_from_actions()
            .expect("route advanced");

        assert_eq!(state.page(), Page::Advanced);
        assert!(state.advanced_workspace_visible());
        assert!(!state.show_advanced);
        assert_eq!(state.current_advanced_action(), ActionItem::FlakeUpdate);
        assert!(state.status.contains("对准 flake update"));
    }

    #[test]
    fn actions_execute_shortcut_routes_wizard_action_with_aligned_focus() {
        let mut state = test_state();
        state.actions_focus = 6;
        state.deploy_source = DeploySource::RemotePinned;
        state.deploy_source_ref = "v5.0.0".to_string();
        state.deploy_focus = 6;

        state
            .execute_current_action_from_actions()
            .expect("route advanced wizard");

        assert_eq!(state.page(), Page::Advanced);
        assert_eq!(
            state.current_advanced_action(),
            ActionItem::LaunchDeployWizard
        );
        assert_eq!(state.advanced_deploy_focus, 3);
        assert!(state.status.contains("对准 launch deploy wizard"));
    }

    fn test_state() -> AppState {
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
            active_page: TopLevelPage::ALL
                .iter()
                .position(|page| *page == TopLevelPage::Advanced)
                .expect("advanced page index"),
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
            show_advanced: false,
            deploy_text_mode: None,
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
            feedback: UiFeedback::default(),
            status: String::new(),
        }
    }
}
