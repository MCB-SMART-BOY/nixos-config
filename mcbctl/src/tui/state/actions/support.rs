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
