use super::*;

impl App {
    pub(crate) fn select_existing_users_menu(&mut self, users: &[String]) -> Result<bool> {
        loop {
            let mut options = Vec::new();
            for user in users {
                if self.target_users.iter().any(|u| u == user) {
                    options.push(format!("[x] {user}"));
                } else {
                    options.push(format!("[ ] {user}"));
                }
            }
            options.push("完成".to_string());
            options.push("返回".to_string());
            let pick = self.menu_prompt("勾选已有用户（可重复切换）", 1, &options)?;
            if pick >= 1 && pick <= users.len() {
                self.toggle_target_user(&users[pick - 1]);
                continue;
            }
            if pick == users.len() + 1 {
                return Ok(true);
            }
            return Ok(false);
        }
    }

    pub(crate) fn select_admin_users_menu(&mut self) -> Result<bool> {
        loop {
            let mut options = Vec::new();
            for user in &self.target_users {
                if self.target_admin_users.iter().any(|u| u == user) {
                    options.push(format!("[x] {user}"));
                } else {
                    options.push(format!("[ ] {user}"));
                }
            }
            options.push("完成".to_string());
            options.push("返回".to_string());
            let pick = self.menu_prompt("勾选管理员用户（可重复切换）", 1, &options)?;
            if pick >= 1 && pick <= self.target_users.len() {
                let user = self.target_users[pick - 1].clone();
                self.toggle_admin_user(&user);
                continue;
            }
            if pick == self.target_users.len() + 1 {
                return Ok(true);
            }
            return Ok(false);
        }
    }

    pub(crate) fn prompt_users(&mut self, repo_dir: &Path) -> Result<WizardAction> {
        let default_user = self.resolve_default_user();
        if !self.is_tty() {
            if self.target_users.is_empty() {
                self.target_users = vec![default_user];
            }
            return Ok(WizardAction::Continue);
        }
        if self.target_users.is_empty() {
            self.target_users = vec![default_user.clone()];
        }

        loop {
            let current = if self.target_users.is_empty() {
                "未选择".to_string()
            } else {
                self.target_users.join(" ")
            };
            let pick = self.menu_prompt(
                &format!("选择用户（当前：{current}）"),
                1,
                &[
                    format!("仅使用默认用户 ({default_user})"),
                    "从已有 Home 用户中选择".to_string(),
                    "新增用户（手写用户名）".to_string(),
                    "清空已选用户".to_string(),
                    "完成".to_string(),
                    "返回".to_string(),
                    "退出".to_string(),
                ],
            )?;
            match pick {
                1 => {
                    self.target_users = vec![default_user.clone()];
                }
                2 => {
                    let mut existing = self.list_existing_home_users(repo_dir);
                    existing.sort();
                    existing.dedup();
                    if existing.is_empty() {
                        self.warn("未发现可选的已有 Home 用户目录。");
                        continue;
                    }
                    let _ = self.select_existing_users_menu(&existing)?;
                }
                3 => {
                    let input = self.prompt_line("输入新增用户名（留空取消）： ")?;
                    let input = input.trim();
                    if input.is_empty() {
                        continue;
                    }
                    if !is_valid_username(input) {
                        self.warn(&format!("用户名不合法：{input}"));
                        continue;
                    }
                    self.add_target_user(input);
                }
                4 => {
                    self.target_users.clear();
                }
                5 => {
                    if self.target_users.is_empty() {
                        self.warn("请至少选择一个用户。");
                        continue;
                    }
                    return Ok(WizardAction::Continue);
                }
                6 => return Ok(WizardAction::Back),
                7 => bail!("已退出"),
                _ => {}
            }
        }
    }

    pub(crate) fn prompt_admin_users(&mut self) -> Result<WizardAction> {
        if self.target_users.is_empty() {
            bail!("用户列表为空，无法选择管理员。");
        }
        let default_admin = self.target_users[0].clone();
        if !self.is_tty() {
            if self.target_admin_users.is_empty() {
                self.target_admin_users = vec![default_admin];
            }
            return Ok(WizardAction::Continue);
        }
        if self.target_admin_users.is_empty() {
            self.target_admin_users = vec![default_admin.clone()];
        }

        loop {
            let current = if self.target_admin_users.is_empty() {
                "未选择".to_string()
            } else {
                self.target_admin_users.join(" ")
            };
            let pick = self.menu_prompt(
                &format!("管理员权限（wheel，当前：{current}）"),
                1,
                &[
                    format!("仅主用户 ({default_admin})"),
                    "所有用户".to_string(),
                    "自定义勾选管理员".to_string(),
                    "清空管理员".to_string(),
                    "完成".to_string(),
                    "返回".to_string(),
                    "退出".to_string(),
                ],
            )?;
            match pick {
                1 => self.target_admin_users = vec![default_admin.clone()],
                2 => self.target_admin_users = self.target_users.clone(),
                3 => {
                    let _ = self.select_admin_users_menu()?;
                }
                4 => self.target_admin_users.clear(),
                5 => {
                    if self.target_admin_users.is_empty() {
                        self.warn("至少需要一个管理员用户。");
                        continue;
                    }
                    return Ok(WizardAction::Continue);
                }
                6 => return Ok(WizardAction::Back),
                7 => bail!("已退出"),
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn prompt_users_non_tty_uses_detected_default_user() -> Result<()> {
        let repo_dir = create_temp_dir("mcbctl-prompt-users-default")?;
        let host_dir = repo_dir.join("hosts/demo");
        fs::create_dir_all(&host_dir)?;
        fs::write(host_dir.join("default.nix"), r#"{ mcb.user = "alice"; }"#)?;
        let mut app = test_app(repo_dir.clone());
        app.target_name = "demo".to_string();
        app.tmp_dir = Some(repo_dir.clone());

        let action = app.prompt_users(&repo_dir)?;

        assert_eq!(action, WizardAction::Continue);
        assert_eq!(app.target_users, vec!["alice".to_string()]);
        Ok(())
    }

    #[test]
    fn prompt_admin_users_non_tty_defaults_to_primary_user() -> Result<()> {
        let repo_dir = create_temp_dir("mcbctl-prompt-admin-default")?;
        let mut app = test_app(repo_dir);
        app.target_users = vec!["alice".to_string(), "bob".to_string()];

        let action = app.prompt_admin_users()?;

        assert_eq!(action, WizardAction::Continue);
        assert_eq!(app.target_admin_users, vec!["alice".to_string()]);
        Ok(())
    }

    #[test]
    fn prompt_users_tty_retries_invalid_username_and_then_accepts_valid_user() -> Result<()> {
        let repo_dir = create_temp_dir("mcbctl-prompt-users-invalid-then-valid")?;
        let host_dir = repo_dir.join("hosts/demo");
        fs::create_dir_all(&host_dir)?;
        fs::write(host_dir.join("default.nix"), r#"{ mcb.user = "demo"; }"#)?;
        let mut app = test_app(repo_dir.clone());
        app.target_name = "demo".to_string();
        app.tmp_dir = Some(repo_dir.clone());
        let _ui = App::install_test_ui(true, &["4", "3", "BadUser", "3", "alice", "5"]);

        let action = app.prompt_users(&repo_dir)?;

        assert_eq!(action, WizardAction::Continue);
        assert_eq!(app.target_users, vec!["alice".to_string()]);
        Ok(())
    }

    #[test]
    fn prompt_users_tty_requires_user_before_completion_after_clear() -> Result<()> {
        let repo_dir = create_temp_dir("mcbctl-prompt-users-clear-then-default")?;
        let host_dir = repo_dir.join("hosts/demo");
        fs::create_dir_all(&host_dir)?;
        fs::write(host_dir.join("default.nix"), r#"{ mcb.user = "alice"; }"#)?;
        let mut app = test_app(repo_dir.clone());
        app.target_name = "demo".to_string();
        app.tmp_dir = Some(repo_dir.clone());
        app.target_users = vec!["existing".to_string()];
        let _ui = App::install_test_ui(true, &["4", "5", "1", "5"]);

        let action = app.prompt_users(&repo_dir)?;

        assert_eq!(action, WizardAction::Continue);
        assert_eq!(app.target_users, vec!["alice".to_string()]);
        Ok(())
    }

    #[test]
    fn prompt_users_tty_emits_terminal_transcript_for_empty_finish_and_invalid_username()
    -> Result<()> {
        let repo_dir = create_temp_dir("mcbctl-prompt-users-transcript")?;
        let host_dir = repo_dir.join("hosts/demo");
        fs::create_dir_all(&host_dir)?;
        fs::write(host_dir.join("default.nix"), r#"{ mcb.user = "alice"; }"#)?;
        let mut app = test_app(repo_dir.clone());
        app.target_name = "demo".to_string();
        app.tmp_dir = Some(repo_dir.clone());
        app.target_users = vec!["existing".to_string()];
        let _ui = App::install_test_ui(true, &["4", "5", "3", "BadUser", "3", "alice", "5"]);

        let action = app.prompt_users(&repo_dir)?;
        let output = App::take_test_output();

        assert_eq!(action, WizardAction::Continue);
        assert_eq!(app.target_users, vec!["alice".to_string()]);
        assert!(output.contains("选择用户（当前：existing）"));
        assert!(output.contains("输入新增用户名（留空取消）： "));
        assert!(output.contains("[警告] 请至少选择一个用户。"));
        assert!(output.contains("[警告] 用户名不合法：BadUser"));
        assert!(output.contains("新增用户（手写用户名）"));
        Ok(())
    }

    #[test]
    fn prompt_admin_users_tty_requires_admin_before_completion() -> Result<()> {
        let repo_dir = create_temp_dir("mcbctl-prompt-admin-requires-selection")?;
        let mut app = test_app(repo_dir);
        app.target_users = vec!["alice".to_string(), "bob".to_string()];
        app.target_admin_users = vec!["alice".to_string()];
        let _ui = App::install_test_ui(true, &["4", "5", "2", "5"]);

        let action = app.prompt_admin_users()?;

        assert_eq!(action, WizardAction::Continue);
        assert_eq!(
            app.target_admin_users,
            vec!["alice".to_string(), "bob".to_string()]
        );
        Ok(())
    }

    #[test]
    fn prompt_admin_users_tty_emits_terminal_transcript_for_empty_finish_warning() -> Result<()> {
        let repo_dir = create_temp_dir("mcbctl-prompt-admin-transcript")?;
        let mut app = test_app(repo_dir);
        app.target_users = vec!["alice".to_string(), "bob".to_string()];
        app.target_admin_users = vec!["alice".to_string()];
        let _ui = App::install_test_ui(true, &["4", "5", "2", "5"]);

        let action = app.prompt_admin_users()?;
        let output = App::take_test_output();

        assert_eq!(action, WizardAction::Continue);
        assert_eq!(
            app.target_admin_users,
            vec!["alice".to_string(), "bob".to_string()]
        );
        assert!(output.contains("管理员权限（wheel，当前：alice）"));
        assert!(output.contains("清空管理员"));
        assert!(output.contains("[警告] 至少需要一个管理员用户。"));
        assert!(output.contains("所有用户"));
        Ok(())
    }

    #[test]
    fn prompt_admin_users_tty_defaults_to_current_primary_user_when_empty() -> Result<()> {
        let repo_dir = create_temp_dir("mcbctl-prompt-admin-current-primary")?;
        let mut app = test_app(repo_dir);
        app.target_users = vec!["charlie".to_string(), "dave".to_string()];
        let _ui = App::install_test_ui(true, &["5"]);

        let action = app.prompt_admin_users()?;

        assert_eq!(action, WizardAction::Continue);
        assert_eq!(app.target_admin_users, vec!["charlie".to_string()]);
        Ok(())
    }

    fn create_temp_dir(prefix: &str) -> Result<PathBuf> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let root = std::env::temp_dir().join(format!("{prefix}-{}-{unique}", std::process::id()));
        fs::create_dir_all(&root)?;
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
            host_profile_kind: HostProfileKind::Desktop,
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
}
