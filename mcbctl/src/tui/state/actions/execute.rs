use super::*;

impl AppState {
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
}
