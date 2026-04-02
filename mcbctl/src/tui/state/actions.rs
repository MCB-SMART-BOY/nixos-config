use super::*;

impl AppState {
    pub fn next_action_item(&mut self) {
        self.actions_focus = (self.actions_focus + 1) % ActionItem::ALL.len();
    }

    pub fn previous_action_item(&mut self) {
        self.actions_focus = if self.actions_focus == 0 {
            ActionItem::ALL.len() - 1
        } else {
            self.actions_focus - 1
        };
    }

    pub fn current_action_item(&self) -> ActionItem {
        ActionItem::ALL[self.actions_focus]
    }

    pub fn actions_rows(&self) -> Vec<(String, String)> {
        ActionItem::ALL
            .iter()
            .map(|item| {
                (
                    item.label().to_string(),
                    if self.action_available(*item) {
                        "可执行".to_string()
                    } else {
                        "需切换场景".to_string()
                    },
                )
            })
            .collect()
    }

    pub fn actions_summary_lines(&self) -> Vec<String> {
        let action = self.current_action_item();
        let mut lines = vec![
            format!("当前动作：{}", action.label()),
            format!("说明：{}", action.description()),
            format!("当前仓库：{}", self.context.repo_root.display()),
            format!("/etc/nixos：{}", self.context.etc_root.display()),
            format!("当前主机：{}", self.target_host),
            format!(
                "权限：{}",
                match self.context.privilege_mode.as_str() {
                    "root" => "root",
                    "sudo-session" => "sudo session",
                    "sudo-available" => "sudo available",
                    _ => "rootless",
                }
            ),
        ];

        if let Some(preview) = self.action_command_preview(action) {
            lines.push(format!("命令预览：{preview}"));
        }
        if self.action_available(action) {
            lines.push("状态：当前环境可以直接执行".to_string());
        } else {
            lines.push("状态：当前环境不适合直接执行；请改用 Deploy 页或切换权限".to_string());
        }

        lines.push(String::new());
        lines.push("当前页说明：".to_string());
        lines.push("- 这里只放高频维护动作，不处理复杂初始化向导".to_string());
        lines.push("- 直接执行外部命令前，会临时退出 TUI，执行完成后再返回".to_string());
        lines.push("- 如需远端来源、模板生成、复杂交互，请使用 deploy wizard".to_string());
        lines
    }

    pub fn execute_current_action(&mut self) -> Result<()> {
        self.ensure_no_unsaved_changes_for_execution()?;
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
                self.status = "flake check 已完成。".to_string();
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
                self.status = "flake update 已完成。".to_string();
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
                self.status = "上游 pin 检查已完成。".to_string();
            }
            ActionItem::UpdateUpstreamPins => {
                let status = self.run_sibling_in_repo("update-upstream-apps", &[])?;
                if !status.success() {
                    anyhow::bail!(
                        "update-upstream-apps exited with {}",
                        status.code().unwrap_or(1)
                    );
                }
                self.status = "上游 pin 刷新已完成。".to_string();
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
                self.status = "仓库已同步到 /etc/nixos。".to_string();
            }
            ActionItem::RebuildCurrentHost => {
                let action = if self.context.privilege_mode == "rootless" {
                    DeployAction::Build
                } else {
                    DeployAction::Switch
                };
                if action != DeployAction::Build {
                    ensure_root_hardware_config(&self.context.etc_root, use_sudo)?;
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
                self.status = format!(
                    "当前主机 {} 已完成一次 {}。",
                    self.context.current_host,
                    action.label()
                );
            }
            ActionItem::LaunchDeployWizard => {
                let status = self.run_sibling_in_repo("mcb-deploy", &[])?;
                if !status.success() {
                    anyhow::bail!("mcb-deploy exited with {}", status.code().unwrap_or(1));
                }
                self.status = "已返回 deploy wizard。".to_string();
            }
        }

        Ok(())
    }

    pub(super) fn should_use_sudo(&self) -> bool {
        matches!(
            self.context.privilege_mode.as_str(),
            "sudo-session" | "sudo-available"
        )
    }

    pub(super) fn manual_repo_sync_plan(&self) -> Option<RepoSyncPlan> {
        (self.context.repo_root != self.context.etc_root).then(|| RepoSyncPlan {
            source_dir: self.context.repo_root.clone(),
            destination_dir: self.context.etc_root.clone(),
            delete_extra: true,
        })
    }

    pub(super) fn ensure_no_unsaved_changes_for_execution(&self) -> Result<()> {
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

    pub(super) fn clean_etc_dir_keep_hardware(&self) -> Result<()> {
        if self.context.etc_root.as_os_str().is_empty()
            || self.context.etc_root == PathBuf::from("/")
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

        let etc_hw = self.context.etc_root.join("hardware-configuration.nix");
        if etc_hw.is_file() {
            fs::copy(&etc_hw, preserve.join("hardware-configuration.nix"))
                .with_context(|| format!("failed to preserve {}", etc_hw.display()))?;
        }

        for entry in fs::read_dir(&self.context.etc_root)
            .with_context(|| format!("failed to read {}", self.context.etc_root.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            let is_hw = path.file_name().and_then(|name| name.to_str())
                == Some("hardware-configuration.nix");
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

        let preserved_root = preserve.join("hardware-configuration.nix");
        if preserved_root.is_file() {
            fs::copy(&preserved_root, &etc_hw)
                .with_context(|| format!("failed to restore {}", etc_hw.display()))?;
        }
        fs::remove_dir_all(preserve).ok();
        Ok(())
    }

    pub(super) fn run_sibling_in_repo(
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

    pub(super) fn action_available(&self, action: ActionItem) -> bool {
        match action {
            ActionItem::SyncRepoToEtc => {
                self.context.repo_root != self.context.etc_root
                    && self.context.privilege_mode != "rootless"
            }
            ActionItem::RebuildCurrentHost => !self.context.current_host.is_empty(),
            _ => true,
        }
    }

    pub(super) fn action_command_preview(&self, action: ActionItem) -> Option<String> {
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
