use super::*;
use crate::repo::ensure_repository_integrity;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ApplyModel {
    pub(crate) target_host: String,
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

impl ApplyModel {
    pub(crate) fn summary_lines(&self) -> Vec<String> {
        let mut lines = vec![
            format!("目标主机：{}", self.target_host),
            format!("任务：{}", self.task.label()),
            format!("来源：{}", self.source.label()),
            format!("动作：{}", self.action.label()),
            format!("flake update：{}", bool_label(self.flake_update)),
            format!("高级项：{}", bool_label(self.advanced)),
        ];

        lines.push(format!(
            "同步预览：{}",
            self.sync_preview
                .as_deref()
                .unwrap_or("当前组合不需要同步 /etc/nixos")
        ));
        lines.push(format!(
            "命令预览：{}",
            self.rebuild_preview
                .as_deref()
                .unwrap_or("当前来源会转交给完整部署向导处理。")
        ));

        if self.blockers.is_empty() {
            lines.push("阻塞项：无".to_string());
        } else {
            lines.push(format!("阻塞项：{}", self.blockers.join(" | ")));
        }

        if self.warnings.is_empty() {
            lines.push("警告项：无".to_string());
        } else {
            lines.push(format!("警告项：{}", self.warnings.join(" | ")));
        }

        if self.handoffs.is_empty() {
            lines.push("交接项：无".to_string());
        } else {
            lines.push(format!("交接项：{}", self.handoffs.join(" | ")));
        }

        if self.infos.is_empty() {
            lines.push("信息：无".to_string());
        } else {
            lines.push(format!("信息：{}", self.infos.join(" | ")));
        }

        lines.push(format!(
            "执行路径：{}",
            if self.can_apply_current_host {
                "当前页可直接执行；按 x 立即运行。"
            } else if self.can_execute_directly {
                "当前页仍是 direct apply，但存在 blocker。"
            } else {
                "当前页会调起完整部署向导。"
            }
        ));

        lines
    }
}

impl AppState {
    pub fn next_deploy_field(&mut self) {
        self.deploy_focus = (self.deploy_focus + 1) % 6;
    }

    pub fn previous_deploy_field(&mut self) {
        self.deploy_focus = if self.deploy_focus == 0 {
            5
        } else {
            self.deploy_focus - 1
        };
    }

    pub fn adjust_deploy_field(&mut self, delta: i8) {
        match self.deploy_focus {
            0 => cycle_string(&mut self.target_host, &self.context.hosts, delta),
            1 => cycle_enum(&mut self.deploy_task, &DeployTask::ALL, delta),
            2 => cycle_enum(&mut self.deploy_source, &DeploySource::ALL, delta),
            3 => cycle_enum(&mut self.deploy_action, &DeployAction::ALL, delta),
            4 => self.flake_update = !self.flake_update,
            5 => self.show_advanced = !self.show_advanced,
            _ => {}
        }
    }

    pub fn deploy_rows(&self) -> Vec<(String, String)> {
        vec![
            ("目标主机".to_string(), self.target_host.clone()),
            ("任务".to_string(), self.deploy_task.label().to_string()),
            ("来源".to_string(), self.deploy_source.label().to_string()),
            ("动作".to_string(), self.deploy_action.label().to_string()),
            (
                "flake update".to_string(),
                bool_label(self.flake_update).to_string(),
            ),
            (
                "高级选项".to_string(),
                bool_label(self.show_advanced).to_string(),
            ),
        ]
    }

    pub fn can_execute_deploy_directly(&self) -> bool {
        !matches!(
            self.deploy_source,
            DeploySource::RemotePinned | DeploySource::RemoteHead
        ) && !self.show_advanced
    }

    pub(crate) fn apply_model(&self) -> ApplyModel {
        let can_execute_directly = self.can_execute_deploy_directly();
        let sync_preview = self
            .deploy_sync_plan_for_execution()
            .map(|plan| plan.command_preview());
        let rebuild_preview = self
            .deploy_rebuild_plan_for_execution()
            .map(|plan| plan.command_preview(self.should_use_sudo()));

        let mut blockers = Vec::new();
        if let Err(err) = self.ensure_no_unsaved_changes_for_execution() {
            blockers.push(err.to_string());
        }
        let host_errors = self.host_configuration_validation_errors(&self.target_host);
        blockers.extend(
            host_errors
                .into_iter()
                .map(|error| format!("主机 {} 的 TUI 配置未通过校验：{error}", self.target_host)),
        );
        if self.context.privilege_mode == "rootless" && self.deploy_action != DeployAction::Build {
            blockers.push(
                "rootless 模式下当前页只能直接执行 build；如需 switch/test/boot，请使用 sudo/root 或退回 deploy wizard。"
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
        infos.push(format!("检测 hostname：{}", self.context.current_host));

        ApplyModel {
            target_host: self.target_host.clone(),
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

    pub fn execute_deploy(&mut self) -> Result<()> {
        self.ensure_no_unsaved_changes_for_execution()?;
        ensure_repository_integrity(&self.context.repo_root)?;
        self.ensure_host_configuration_is_valid(&self.target_host)?;

        if !self.can_execute_deploy_directly() {
            let mut args = Vec::new();
            if matches!(self.deploy_task, DeployTask::Maintenance) {
                args.push("--mode".to_string());
                args.push("update-existing".to_string());
            }
            let status = self.run_sibling_in_repo("mcb-deploy", &args)?;
            if status.success() {
                self.status = "已返回完整部署向导。".to_string();
                return Ok(());
            }
            anyhow::bail!("mcb-deploy exited with {}", status.code().unwrap_or(1));
        }

        if self.context.privilege_mode == "rootless" && self.deploy_action != DeployAction::Build {
            anyhow::bail!(
                "rootless 模式下当前页只能直接执行 build；如需 switch/test/boot，请使用 sudo/root 或退回 deploy wizard。"
            );
        }

        let use_sudo = self.should_use_sudo();
        let needs_root_hw = !(self.context.privilege_mode == "rootless"
            && self.deploy_action == DeployAction::Build);
        if needs_root_hw {
            ensure_host_hardware_config(&self.context.etc_root, &self.target_host, use_sudo)?;
        }

        let sync_plan = self.deploy_sync_plan_for_execution();
        let rebuild_plan = self
            .deploy_rebuild_plan_for_execution()
            .context("当前 Deploy 组合还没有可执行的重建计划")?;

        if let Some(plan) = sync_plan {
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
        }

        let status = run_nixos_rebuild(&rebuild_plan, use_sudo)?;
        if !status.success() {
            anyhow::bail!("nixos-rebuild exited with {}", status.code().unwrap_or(1));
        }

        self.status = format!(
            "Deploy 已执行完成：{} {}",
            rebuild_plan.action.label(),
            rebuild_plan.target_host
        );
        Ok(())
    }

    pub(crate) fn deploy_rebuild_plan_for_execution(&self) -> Option<NixosRebuildPlan> {
        let flake_root = match self.deploy_source {
            DeploySource::CurrentRepo if self.should_sync_current_repo_before_rebuild() => {
                self.context.etc_root.clone()
            }
            DeploySource::CurrentRepo => self.context.repo_root.clone(),
            DeploySource::EtcNixos => self.context.etc_root.clone(),
            DeploySource::RemotePinned | DeploySource::RemoteHead => return None,
        };

        Some(NixosRebuildPlan {
            action: self.deploy_action,
            upgrade: self.flake_update,
            flake_root,
            target_host: self.target_host.clone(),
        })
    }

    pub(crate) fn deploy_sync_plan_for_execution(&self) -> Option<RepoSyncPlan> {
        match self.deploy_source {
            DeploySource::CurrentRepo if self.should_sync_current_repo_before_rebuild() => {
                Some(RepoSyncPlan {
                    source_dir: self.context.repo_root.clone(),
                    destination_dir: self.context.etc_root.clone(),
                    delete_extra: true,
                })
            }
            _ => None,
        }
    }

    fn should_sync_current_repo_before_rebuild(&self) -> bool {
        self.deploy_source == DeploySource::CurrentRepo
            && self.context.repo_root != self.context.etc_root
            && self.deploy_action != DeployAction::Build
            && self.context.privilege_mode != "rootless"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn current_repo_switch_uses_sync_plan_and_etc_rebuild_target() {
        let state = test_state("/repo", "/etc/nixos", "sudo-available");

        assert!(state.can_execute_deploy_directly());
        let sync = state
            .deploy_sync_plan_for_execution()
            .expect("current repo switch should sync into /etc/nixos");
        assert_eq!(sync.source_dir, PathBuf::from("/repo"));
        assert_eq!(sync.destination_dir, PathBuf::from("/etc/nixos"));
        assert!(sync.delete_extra);

        let rebuild = state
            .deploy_rebuild_plan_for_execution()
            .expect("current repo switch should produce rebuild plan");
        assert_eq!(rebuild.action, DeployAction::Switch);
        assert_eq!(rebuild.flake_root, PathBuf::from("/etc/nixos"));
        assert_eq!(rebuild.target_host, "demo");
    }

    #[test]
    fn rootless_build_stays_on_repo_and_skips_sync() {
        let mut state = test_state("/repo", "/etc/nixos", "rootless");
        state.deploy_action = DeployAction::Build;

        assert!(state.can_execute_deploy_directly());
        assert!(state.deploy_sync_plan_for_execution().is_none());

        let rebuild = state
            .deploy_rebuild_plan_for_execution()
            .expect("rootless build should still produce rebuild plan");
        assert_eq!(rebuild.action, DeployAction::Build);
        assert_eq!(rebuild.flake_root, PathBuf::from("/repo"));
    }

    #[test]
    fn remote_sources_fall_back_to_wizard() {
        let mut state = test_state("/repo", "/etc/nixos", "sudo-available");
        state.deploy_source = DeploySource::RemoteHead;

        assert!(!state.can_execute_deploy_directly());
        let model = state.apply_model();
        assert!(!model.can_execute_directly);
        assert!(
            model
                .handoffs
                .iter()
                .any(|item| item.contains("远端最新版本"))
        );
        assert!(state.deploy_sync_plan_for_execution().is_none());
        assert!(state.deploy_rebuild_plan_for_execution().is_none());
    }

    #[test]
    fn apply_model_surfaces_blockers_warnings_and_previews() {
        let mut state = test_state("/repo", "/etc/nixos", "sudo-available");
        state.package_dirty_users.insert("alice".to_string());
        state.flake_update = true;

        let model = state.apply_model();

        assert_eq!(model.target_host, "demo");
        assert!(model.can_execute_directly);
        assert!(!model.can_apply_current_host);
        assert!(
            model
                .blockers
                .iter()
                .any(|item| item.contains("Packages: alice"))
        );
        assert!(
            model
                .warnings
                .iter()
                .any(|item| item.contains("同步到 /etc/nixos"))
        );
        assert!(model.warnings.iter().any(|item| item.contains("--upgrade")));
        assert!(model.sync_preview.is_some());
        assert!(
            model
                .rebuild_preview
                .as_deref()
                .is_some_and(|preview| preview.contains("/etc/nixos#demo"))
        );
    }

    #[test]
    fn apply_model_reports_rootless_direct_build_without_sync() {
        let mut state = test_state("/repo", "/etc/nixos", "rootless");
        state.deploy_action = DeployAction::Build;

        let model = state.apply_model();

        assert!(model.can_execute_directly);
        assert!(model.can_apply_current_host);
        assert!(model.sync_preview.is_none());
        assert!(model.blockers.is_empty());
        assert!(model.warnings.iter().all(|item| !item.contains("sudo -E")));
    }

    #[test]
    fn execute_deploy_rejects_unsaved_changes_before_other_checks() {
        let mut state = test_state("/definitely/missing/repo", "/etc/nixos", "sudo-available");
        state.home_dirty_users.insert("alice".to_string());
        state.package_dirty_users.insert("alice".to_string());

        let err = state
            .execute_deploy()
            .expect_err("unsaved changes should block execution immediately");
        let text = err.to_string();
        assert!(text.contains("仍有未保存修改"));
        assert!(text.contains("Packages: alice"));
        assert!(text.contains("Home: alice"));
    }

    #[test]
    fn execute_deploy_rejects_rootless_non_build_before_external_commands() -> Result<()> {
        let repo_root = create_temp_dir("mcbctl-deploy-state")?;
        let mut state = test_state(
            repo_root.to_string_lossy().as_ref(),
            "/etc/nixos",
            "rootless",
        );
        state.deploy_action = DeployAction::Switch;

        let err = state
            .execute_deploy()
            .expect_err("rootless direct switch should be rejected");
        assert!(
            err.to_string()
                .contains("rootless 模式下当前页只能直接执行 build")
        );

        std::fs::remove_dir_all(repo_root)?;
        Ok(())
    }

    fn test_state(repo_root: &str, etc_root: &str, privilege_mode: &str) -> AppState {
        let context = AppContext {
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
        };

        let mut host_settings_by_name = BTreeMap::new();
        host_settings_by_name.insert("demo".to_string(), valid_host_settings());

        AppState {
            context,
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

    fn create_temp_dir(prefix: &str) -> Result<PathBuf> {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let root = std::env::temp_dir().join(format!("{prefix}-{}-{unique}", std::process::id()));
        std::fs::create_dir_all(&root)?;
        Ok(root)
    }
}
