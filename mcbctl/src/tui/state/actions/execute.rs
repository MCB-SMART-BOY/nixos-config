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
                self.set_feedback_with_next_step(
                    UiFeedbackLevel::Success,
                    UiFeedbackScope::Inspect,
                    "flake check 已完成。",
                    "切到 Inspect 查看检查结果",
                );
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
                self.set_feedback_with_next_step(
                    UiFeedbackLevel::Success,
                    UiFeedbackScope::Advanced,
                    "flake update 已完成。",
                    "继续在 Advanced 处理后续仓库维护",
                );
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
                self.set_feedback_with_next_step(
                    UiFeedbackLevel::Success,
                    UiFeedbackScope::Inspect,
                    "上游 pin 检查已完成。",
                    "切到 Inspect 查看 pin 状态",
                );
            }
            ActionItem::UpdateUpstreamPins => {
                let status = self.run_sibling_in_repo("update-upstream-apps", &[])?;
                if !status.success() {
                    anyhow::bail!(
                        "update-upstream-apps exited with {}",
                        status.code().unwrap_or(1)
                    );
                }
                self.set_feedback_with_next_step(
                    UiFeedbackLevel::Success,
                    UiFeedbackScope::Advanced,
                    "上游 pin 刷新已完成。",
                    "继续在 Advanced 完成后续维护",
                );
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
                self.set_feedback_with_next_step(
                    UiFeedbackLevel::Success,
                    UiFeedbackScope::Apply,
                    "仓库已同步到 /etc/nixos。",
                    "回到 Apply 或 Overview 继续后续重建",
                );
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
                self.set_feedback_with_next_step(
                    UiFeedbackLevel::Success,
                    UiFeedbackScope::Apply,
                    format!(
                        "当前主机 {} 已完成一次 {}。",
                        self.context.current_host,
                        action.label()
                    ),
                    "回到 Overview 检查健康和下一步",
                );
            }
            ActionItem::LaunchDeployWizard => {
                let status = self.run_sibling_in_repo("mcb-deploy", &[])?;
                if !status.success() {
                    anyhow::bail!("mcb-deploy exited with {}", status.code().unwrap_or(1));
                }
                self.set_feedback_with_next_step(
                    UiFeedbackLevel::Info,
                    UiFeedbackScope::Advanced,
                    "已返回 deploy wizard。",
                    "继续在 Advanced 完成复杂部署",
                );
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
    fn actions_execute_shortcut_routes_advanced_actions_into_apply_workspace() {
        let mut state = test_state();
        state.actions_focus = 4;

        state
            .execute_current_action_from_actions()
            .expect("route advanced");

        assert_eq!(state.page(), Page::Deploy);
        assert!(state.show_advanced);
        assert_eq!(state.current_advanced_action(), ActionItem::FlakeUpdate);
        assert!(state.status.contains("Advanced Workspace"));
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
            active_page: Page::ALL
                .iter()
                .position(|page| *page == Page::Actions)
                .expect("actions page index"),
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
            feedback: UiFeedback::default(),
            status: String::new(),
        }
    }
}
