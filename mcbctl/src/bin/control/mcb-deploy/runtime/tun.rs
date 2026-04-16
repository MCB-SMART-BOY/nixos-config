use super::super::*;

impl App {
    pub(crate) fn reset_tun_maps(&mut self) {
        self.user_tun.clear();
        self.user_dns.clear();
    }

    pub(crate) fn configure_per_user_tun(&mut self) -> Result<WizardAction> {
        if !self.is_tty() {
            return Ok(WizardAction::Continue);
        }
        self.section("Per-user TUN 配置");
        self.note("检测到当前主机已启用 per-user TUN。");
        self.note("请为每个用户指定独立 TUN 名称与 DNS 端口。");
        'retry: loop {
            self.user_tun.clear();
            self.user_dns.clear();
            for (idx, user) in self.target_users.iter().enumerate() {
                let default_iface = format!("tun{}", idx + 1);
                let iface = self.prompt_line(&format!(
                    "用户 {user} 的 TUN 接口（默认 {default_iface}）： "
                ))?;
                let iface = iface.trim();
                let iface = if iface.is_empty() {
                    &default_iface
                } else {
                    iface
                };
                self.user_tun.insert(user.clone(), iface.to_string());

                let default_dns = 1053u16 + (idx as u16);
                let dns =
                    self.prompt_line(&format!("用户 {user} 的 DNS 端口（默认 {default_dns}）： "))?;
                let dns = dns.trim();
                let port = if dns.is_empty() {
                    default_dns
                } else if let Ok(v) = dns.parse::<u16>() {
                    v
                } else {
                    self.warn("端口无效，请重新输入这一轮。");
                    self.user_tun.clear();
                    self.user_dns.clear();
                    continue 'retry;
                };
                self.user_dns.insert(user.clone(), port);
            }
            self.note("Per-user TUN 配置预览：");
            for user in &self.target_users {
                let iface = self.user_tun.get(user).cloned().unwrap_or_default();
                let dns = self.user_dns.get(user).copied().unwrap_or_default();
                self.note(&format!("  - {user}: {iface}, DNS {dns}"));
            }
            match self.wizard_back_or_quit("确认 Per-user TUN 配置？")? {
                WizardAction::Back => return Ok(WizardAction::Back),
                WizardAction::Continue => return Ok(WizardAction::Continue),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn configure_per_user_tun_tty_uses_defaults_for_each_user() -> Result<()> {
        let repo_dir = create_temp_dir("mcbctl-tun-defaults")?;
        let mut app = test_app(repo_dir);
        app.target_users = vec!["alice".to_string(), "bob".to_string()];
        let _ui = App::install_test_ui(true, &["", "", "", "", ""]);

        let action = app.configure_per_user_tun()?;

        assert_eq!(action, WizardAction::Continue);
        assert_eq!(app.user_tun.get("alice"), Some(&"tun1".to_string()));
        assert_eq!(app.user_dns.get("alice"), Some(&1053));
        assert_eq!(app.user_tun.get("bob"), Some(&"tun2".to_string()));
        assert_eq!(app.user_dns.get("bob"), Some(&1054));
        Ok(())
    }

    #[test]
    fn configure_per_user_tun_tty_emits_terminal_transcript_for_default_round() -> Result<()> {
        let repo_dir = create_temp_dir("mcbctl-tun-defaults-transcript")?;
        let mut app = test_app(repo_dir);
        app.target_users = vec!["alice".to_string(), "bob".to_string()];
        let _ui = App::install_test_ui(true, &["", "", "", "", ""]);

        let action = app.configure_per_user_tun()?;
        let output = App::take_test_output();

        assert_eq!(action, WizardAction::Continue);
        assert!(output.contains("Per-user TUN 配置"));
        assert!(output.contains("检测到当前主机已启用 per-user TUN。"));
        assert!(output.contains("请为每个用户指定独立 TUN 名称与 DNS 端口。"));
        assert!(output.contains("用户 alice 的 TUN 接口（默认 tun1）： "));
        assert!(output.contains("用户 alice 的 DNS 端口（默认 1053）： "));
        assert!(output.contains("用户 bob 的 TUN 接口（默认 tun2）： "));
        assert!(output.contains("用户 bob 的 DNS 端口（默认 1054）： "));
        assert!(output.contains("Per-user TUN 配置预览："));
        assert!(output.contains("  - alice: tun1, DNS 1053"));
        assert!(output.contains("  - bob: tun2, DNS 1054"));
        assert!(output.contains("确认 Per-user TUN 配置？ [c继续/b返回/q退出]（默认 c）： "));
        Ok(())
    }

    #[test]
    fn configure_per_user_tun_tty_restarts_whole_round_after_invalid_dns() -> Result<()> {
        let repo_dir = create_temp_dir("mcbctl-tun-invalid-dns-retry")?;
        let mut app = test_app(repo_dir);
        app.target_users = vec!["alice".to_string(), "bob".to_string()];
        let _ui = App::install_test_ui(true, &["tap0", "bad", "tap1", "2053", "", "", ""]);

        let action = app.configure_per_user_tun()?;

        assert_eq!(action, WizardAction::Continue);
        assert_eq!(app.user_tun.len(), 2);
        assert_eq!(app.user_dns.len(), 2);
        assert_eq!(app.user_tun.get("alice"), Some(&"tap1".to_string()));
        assert_eq!(app.user_dns.get("alice"), Some(&2053));
        assert_eq!(app.user_tun.get("bob"), Some(&"tun2".to_string()));
        assert_eq!(app.user_dns.get("bob"), Some(&1054));
        Ok(())
    }

    #[test]
    fn configure_per_user_tun_tty_emits_terminal_transcript_for_invalid_dns_retry() -> Result<()> {
        let repo_dir = create_temp_dir("mcbctl-tun-invalid-dns-transcript")?;
        let mut app = test_app(repo_dir);
        app.target_users = vec!["alice".to_string(), "bob".to_string()];
        let _ui = App::install_test_ui(true, &["tap0", "bad", "tap1", "2053", "", "", ""]);

        let action = app.configure_per_user_tun()?;
        let output = App::take_test_output();

        assert_eq!(action, WizardAction::Continue);
        assert!(output.contains("[警告] 端口无效，请重新输入这一轮。"));
        assert!(
            output
                .matches("用户 alice 的 TUN 接口（默认 tun1）： ")
                .count()
                >= 2
        );
        assert!(
            output
                .matches("用户 alice 的 DNS 端口（默认 1053）： ")
                .count()
                >= 2
        );
        assert!(output.contains("  - alice: tap1, DNS 2053"));
        assert!(output.contains("  - bob: tun2, DNS 1054"));
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
            source_dir_override: None,
            repo_urls: Vec::new(),
            branch: "rust脚本分支".to_string(),
            source_ref: String::new(),
            allow_remote_head: false,
            source_commit: String::new(),
            source_choice_set: false,
            target_name: "demo".to_string(),
            target_users: Vec::new(),
            target_admin_users: Vec::new(),
            deploy_mode: DeployMode::ManageUsers,
            deploy_mode_set: false,
            force_remote_source: false,
            overwrite_mode: OverwriteMode::Ask,
            overwrite_mode_set: false,
            per_user_tun_enabled: true,
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
            rebuild_upgrade_set: false,
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
