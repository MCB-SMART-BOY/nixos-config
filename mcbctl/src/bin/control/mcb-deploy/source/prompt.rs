use super::*;

impl App {
    pub(crate) fn prompt_source_strategy(&mut self) -> Result<()> {
        if self.source_choice_set {
            return Ok(());
        }
        let local_repo = self.detect_local_repo_dir();
        if !self.is_tty() {
            if self.deploy_mode == DeployMode::UpdateExisting {
                self.force_remote_source = true;
                self.allow_remote_head = true;
                self.source_ref.clear();
            } else if local_repo.is_some() {
                self.force_remote_source = false;
                self.allow_remote_head = false;
                self.source_ref.clear();
            } else {
                self.force_remote_source = true;
                self.allow_remote_head = false;
            }
            self.source_choice_set = true;
            return Ok(());
        }

        let mut options = Vec::<String>::new();
        let mut default_index = 1usize;
        if let Some(local) = &local_repo {
            options.push(format!("使用本地仓库（推荐）: {}", local.display()));
        }
        options.push("使用网络仓库固定版本（输入 commit/tag）".to_string());
        options.push("使用网络仓库最新版本（HEAD）".to_string());
        if self.deploy_mode == DeployMode::UpdateExisting {
            default_index = options.len();
        }
        let pick = self.menu_prompt("选择配置来源", default_index, &options)?;

        if local_repo.is_some() && pick == 1 {
            self.force_remote_source = false;
            self.allow_remote_head = false;
            self.source_ref.clear();
        } else {
            let mut remote_pick = pick;
            if local_repo.is_some() {
                remote_pick = pick.saturating_sub(1);
            }
            match remote_pick {
                1 => {
                    self.force_remote_source = true;
                    self.allow_remote_head = false;
                    loop {
                        let line = self.prompt_line("请输入远端固定版本（commit/tag）： ")?;
                        let v = line.trim();
                        if !v.is_empty() {
                            self.source_ref = v.to_string();
                            break;
                        }
                        println!("版本不能为空，请重试。");
                    }
                }
                2 => {
                    self.force_remote_source = true;
                    self.allow_remote_head = true;
                    self.source_ref.clear();
                }
                _ => {}
            }
        }
        self.source_choice_set = true;
        Ok(())
    }

    pub(crate) fn validate_mode_conflicts(&self) -> Result<()> {
        if self.deploy_mode == DeployMode::UpdateExisting && !self.target_users.is_empty() {
            bail!("仅更新模式不允许修改用户列表；该模式会保留现有用户与权限。");
        }
        Ok(())
    }

    pub(crate) fn require_remote_source_pin(&self) -> Result<()> {
        if self.allow_remote_head {
            self.warn("当前将跟随远端分支最新提交（存在供应链风险）。");
            return Ok(());
        }
        if self.source_ref.is_empty() {
            bail!(
                "未检测到本地仓库，且未选择远端固定版本；请在向导中选择固定版本或明确选择远端最新版本。"
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::sync::{Mutex, MutexGuard, OnceLock};

    #[test]
    fn prompt_source_strategy_non_tty_uses_local_repo_for_manage_users() -> Result<()> {
        let repo_dir = create_temp_repo_dir("mcbctl-source-default-local")?;
        let mut app = test_app(repo_dir);
        app.deploy_mode = DeployMode::ManageUsers;

        app.prompt_source_strategy()?;

        assert!(!app.force_remote_source);
        assert!(!app.allow_remote_head);
        assert!(app.source_ref.is_empty());
        assert!(app.source_choice_set);
        Ok(())
    }

    #[test]
    fn prompt_source_strategy_non_tty_uses_remote_head_for_update_existing() -> Result<()> {
        let repo_dir = create_temp_repo_dir("mcbctl-source-default-update")?;
        let mut app = test_app(repo_dir);
        app.deploy_mode = DeployMode::UpdateExisting;
        app.source_ref = "old-pin".to_string();

        app.prompt_source_strategy()?;

        assert!(app.force_remote_source);
        assert!(app.allow_remote_head);
        assert!(app.source_ref.is_empty());
        assert!(app.source_choice_set);
        Ok(())
    }

    #[test]
    fn require_remote_source_pin_rejects_unpinned_remote_source() {
        let app = test_app(PathBuf::from("/tmp/non-repo"));

        let err = app
            .require_remote_source_pin()
            .expect_err("empty remote pin should fail");

        assert!(err.to_string().contains("未检测到本地仓库"));
    }

    #[test]
    fn detect_local_repo_dir_prefers_repo_dir_when_cwd_is_not_repo() -> Result<()> {
        let _guard = cwd_lock();
        let repo_dir = create_temp_repo_dir("mcbctl-source-detect-local")?;
        let outside =
            std::env::temp_dir().join(format!("mcbctl-source-outside-{}", std::process::id()));
        fs::create_dir_all(&outside)?;
        let previous = std::env::current_dir()?;
        std::env::set_current_dir(&outside)?;
        let app = test_app(repo_dir.clone());

        let detected = app.detect_local_repo_dir();

        std::env::set_current_dir(previous)?;
        assert_eq!(detected, Some(repo_dir));
        Ok(())
    }

    fn create_temp_repo_dir(prefix: &str) -> Result<PathBuf> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let root = std::env::temp_dir().join(format!("{prefix}-{}-{unique}", std::process::id()));
        fs::create_dir_all(root.join("hosts"))?;
        fs::create_dir_all(root.join("modules"))?;
        fs::create_dir_all(root.join("home"))?;
        fs::write(root.join("flake.nix"), "{ }")?;
        Ok(root)
    }

    fn test_app(repo_dir: PathBuf) -> App {
        App {
            repo_dir,
            repo_urls: Vec::new(),
            branch: "rust脚本分支".to_string(),
            source_ref: String::new(),
            allow_remote_head: false,
            source_commit: String::new(),
            source_choice_set: false,
            target_name: String::new(),
            target_users: Vec::new(),
            target_admin_users: Vec::new(),
            deploy_mode: DeployMode::ManageUsers,
            deploy_mode_set: false,
            force_remote_source: false,
            overwrite_mode: OverwriteMode::Ask,
            overwrite_mode_set: false,
            per_user_tun_enabled: false,
            host_profile_kind: HostProfileKind::Unknown,
            user_tun: BTreeMap::new(),
            user_dns: BTreeMap::new(),
            server_overrides_enabled: false,
            server_enable_network_cli: String::new(),
            server_enable_network_gui: String::new(),
            server_enable_shell_tools: String::new(),
            server_enable_wayland_tools: String::new(),
            server_enable_system_tools: String::new(),
            server_enable_geek_tools: String::new(),
            server_enable_gaming: String::new(),
            server_enable_insecure_tools: String::new(),
            server_enable_docker: String::new(),
            server_enable_libvirtd: String::new(),
            created_home_users: Vec::new(),
            gpu_override: false,
            gpu_override_from_detection: false,
            gpu_mode: String::new(),
            gpu_igpu_vendor: String::new(),
            gpu_prime_mode: String::new(),
            gpu_intel_bus: String::new(),
            gpu_amd_bus: String::new(),
            gpu_nvidia_bus: String::new(),
            gpu_nvidia_open: String::new(),
            gpu_specialisations_enabled: false,
            gpu_specialisations_set: false,
            gpu_specialisation_modes: Vec::new(),
            detected_gpu: DetectedGpuProfile::default(),
            mode: "switch".to_string(),
            rebuild_upgrade: false,
            etc_dir: PathBuf::from("/tmp/etc-nixos"),
            dns_enabled: false,
            temp_dns_backend: String::new(),
            temp_dns_backup: None,
            temp_dns_iface: String::new(),
            tmp_dir: None,
            sudo_cmd: None,
            rootless: false,
            run_action: RunAction::Deploy,
            progress_total: 7,
            progress_current: 0,
            git_clone_timeout_sec: 90,
        }
    }

    fn cwd_lock() -> MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
