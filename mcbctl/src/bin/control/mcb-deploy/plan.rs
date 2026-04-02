use super::*;

impl App {
    pub(super) fn print_summary(&mut self) {
        self.section("部署概要");
        for line in self.build_deploy_plan().summary_lines() {
            println!("{line}");
        }
    }

    pub(super) fn build_deploy_plan(&self) -> DeployPlan {
        let source = self.deploy_plan_source();
        let mut notes = Vec::new();

        if source == DeploySource::CurrentRepo
            && let Some(local_repo) = self.detect_local_repo_dir()
        {
            notes.push(format!("本地仓库：{}", local_repo.display()));
        }

        if self.deploy_mode == DeployMode::UpdateExisting {
            notes.push("用户/权限：保持当前主机 local.nix".to_string());
        } else {
            notes.push(format!("用户：{}", self.target_users.join(" ")));
            notes.push(format!("管理员：{}", self.target_admin_users.join(" ")));
        }

        if !self.source_commit.is_empty() {
            notes.push(format!("源提交：{}", self.source_commit));
        }

        notes.push(format!(
            "覆盖策略：{}",
            match self.overwrite_mode {
                OverwriteMode::Ask => "ask",
                OverwriteMode::Backup => "backup",
                OverwriteMode::Overwrite => "overwrite",
            }
        ));
        notes.push(format!(
            "依赖升级：{}",
            if self.rebuild_upgrade {
                "启用"
            } else {
                "关闭"
            }
        ));

        if self.deploy_mode != DeployMode::UpdateExisting {
            if self.per_user_tun_enabled {
                if self.user_tun.is_empty() {
                    notes.push("Per-user TUN：已启用（沿用主机配置）".to_string());
                } else {
                    notes.push("Per-user TUN：已启用".to_string());
                    for user in &self.target_users {
                        let iface = self.user_tun.get(user).cloned().unwrap_or_default();
                        let dns = self.user_dns.get(user).copied().unwrap_or_default();
                        notes.push(format!("  - {user} -> {iface} (DNS {dns})"));
                    }
                }
            } else {
                notes.push("Per-user TUN：未启用".to_string());
            }

            if self.gpu_override {
                notes.push(format!("GPU：{}", self.gpu_mode));
                if !self.gpu_igpu_vendor.is_empty() {
                    notes.push(format!("  - iGPU 厂商：{}", self.gpu_igpu_vendor));
                }
                if !self.gpu_prime_mode.is_empty() {
                    notes.push(format!("  - PRIME：{}", self.gpu_prime_mode));
                }
                if !self.gpu_intel_bus.is_empty() {
                    notes.push(format!("  - Intel busId：{}", self.gpu_intel_bus));
                }
                if !self.gpu_amd_bus.is_empty() {
                    notes.push(format!("  - AMD busId：{}", self.gpu_amd_bus));
                }
                if !self.gpu_nvidia_bus.is_empty() {
                    notes.push(format!("  - NVIDIA busId：{}", self.gpu_nvidia_bus));
                }
                if !self.gpu_nvidia_open.is_empty() {
                    notes.push(format!("  - NVIDIA open：{}", self.gpu_nvidia_open));
                }
                if self.gpu_specialisations_enabled {
                    notes.push(format!(
                        "  - specialisation：启用 ({})",
                        self.gpu_specialisation_modes.join(" ")
                    ));
                }
            } else {
                notes.push("GPU：沿用主机配置".to_string());
            }

            if self.server_overrides_enabled {
                notes.push("服务器软件覆盖：已启用".to_string());
                notes.push(format!(
                    "  - enableNetworkCli={}",
                    self.server_enable_network_cli
                ));
                notes.push(format!(
                    "  - enableNetworkGui={}",
                    self.server_enable_network_gui
                ));
                notes.push(format!(
                    "  - enableShellTools={}",
                    self.server_enable_shell_tools
                ));
                notes.push(format!(
                    "  - enableWaylandTools={}",
                    self.server_enable_wayland_tools
                ));
                notes.push(format!(
                    "  - enableSystemTools={}",
                    self.server_enable_system_tools
                ));
                notes.push(format!(
                    "  - enableGeekTools={}",
                    self.server_enable_geek_tools
                ));
                notes.push(format!("  - enableGaming={}", self.server_enable_gaming));
                notes.push(format!(
                    "  - enableInsecureTools={}",
                    self.server_enable_insecure_tools
                ));
                notes.push(format!("  - docker.enable={}", self.server_enable_docker));
                notes.push(format!(
                    "  - libvirtd.enable={}",
                    self.server_enable_libvirtd
                ));
            }
        }

        DeployPlan {
            task: self.deploy_plan_task(),
            detected_host: None,
            target_host: self.target_name.clone(),
            source,
            source_detail: self.deploy_plan_source_detail(),
            action: self.deploy_plan_action(),
            notes,
        }
    }

    pub(super) fn rebuild_plan(&self) -> NixosRebuildPlan {
        NixosRebuildPlan {
            action: self.deploy_plan_action(),
            upgrade: self.rebuild_upgrade,
            flake_root: self.etc_dir.clone(),
            target_host: self.target_name.clone(),
        }
    }

    pub(super) fn repo_sync_plan(&self, repo_dir: &Path) -> RepoSyncPlan {
        RepoSyncPlan {
            source_dir: repo_dir.to_path_buf(),
            destination_dir: self.etc_dir.clone(),
            delete_extra: matches!(
                self.overwrite_mode,
                OverwriteMode::Overwrite | OverwriteMode::Backup
            ),
        }
    }

    pub(super) fn deploy_plan_task(&self) -> DeployTask {
        match self.deploy_mode {
            DeployMode::ManageUsers => DeployTask::AdjustStructure,
            DeployMode::UpdateExisting => DeployTask::DirectDeploy,
        }
    }

    pub(super) fn deploy_plan_source(&self) -> DeploySource {
        if self.force_remote_source {
            if self.allow_remote_head {
                DeploySource::RemoteHead
            } else {
                DeploySource::RemotePinned
            }
        } else if self.etc_dir == self.repo_dir {
            DeploySource::EtcNixos
        } else {
            DeploySource::CurrentRepo
        }
    }

    pub(super) fn deploy_plan_source_detail(&self) -> Option<String> {
        match self.deploy_plan_source() {
            DeploySource::RemotePinned => {
                if self.source_ref.is_empty() {
                    None
                } else {
                    Some(self.source_ref.clone())
                }
            }
            DeploySource::RemoteHead => Some(self.branch.clone()),
            DeploySource::CurrentRepo => self
                .detect_local_repo_dir()
                .map(|path| path.display().to_string()),
            DeploySource::EtcNixos => Some(self.etc_dir.display().to_string()),
        }
    }

    pub(super) fn deploy_plan_action(&self) -> DeployAction {
        DeployAction::from_rebuild_mode(&self.mode)
    }
}
