use super::super::*;

impl App {
    pub(crate) fn reset_server_overrides(&mut self) {
        self.server_overrides_enabled = false;
        self.server_enable_network_cli.clear();
        self.server_enable_network_gui.clear();
        self.server_enable_shell_tools.clear();
        self.server_enable_wayland_tools.clear();
        self.server_enable_system_tools.clear();
        self.server_enable_geek_tools.clear();
        self.server_enable_gaming.clear();
        self.server_enable_insecure_tools.clear();
        self.server_enable_docker.clear();
        self.server_enable_libvirtd.clear();
    }

    pub(crate) fn configure_server_overrides(&mut self) -> Result<WizardAction> {
        if !self.is_tty() {
            self.reset_server_overrides();
            return Ok(WizardAction::Continue);
        }

        let pick = self.menu_prompt(
            "服务器软件覆盖",
            2,
            &[
                "启用服务器包组覆盖".to_string(),
                "沿用主机现有配置".to_string(),
                "返回".to_string(),
            ],
        )?;

        match pick {
            1 => self.server_overrides_enabled = true,
            2 => {
                self.reset_server_overrides();
                return Ok(WizardAction::Continue);
            }
            3 => return Ok(WizardAction::Back),
            _ => {}
        }

        let ask = |app: &App, name: &str, default: bool| -> Result<String> {
            Ok(if app.ask_bool(&format!("{name}？"), default)? {
                "true".to_string()
            } else {
                "false".to_string()
            })
        };

        self.server_enable_network_cli = ask(self, "启用网络 CLI 包", true)?;
        self.server_enable_network_gui = ask(self, "启用网络 GUI 包", false)?;
        self.server_enable_shell_tools = ask(self, "启用 Shell 工具", true)?;
        self.server_enable_wayland_tools = ask(self, "启用 Wayland 工具", false)?;
        self.server_enable_system_tools = ask(self, "启用系统工具", true)?;
        self.server_enable_geek_tools = ask(self, "启用 Geek 工具", true)?;
        self.server_enable_gaming = ask(self, "启用游戏工具", false)?;
        self.server_enable_insecure_tools = ask(self, "启用不安全工具", false)?;
        self.server_enable_docker = ask(self, "启用 Docker", true)?;
        self.server_enable_libvirtd = ask(self, "启用 Libvirtd", false)?;

        Ok(WizardAction::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn configure_server_overrides_non_tty_resets_existing_values() -> Result<()> {
        let repo_dir = create_temp_dir("mcbctl-server-overrides-default")?;
        let mut app = test_app(repo_dir);
        app.server_overrides_enabled = true;
        app.server_enable_network_cli = "true".to_string();
        app.server_enable_docker = "true".to_string();

        let action = app.configure_server_overrides()?;

        assert_eq!(action, WizardAction::Continue);
        assert!(!app.server_overrides_enabled);
        assert!(app.server_enable_network_cli.is_empty());
        assert!(app.server_enable_docker.is_empty());
        Ok(())
    }

    #[test]
    fn configure_server_overrides_tty_applies_answer_matrix() -> Result<()> {
        let repo_dir = create_temp_dir("mcbctl-server-overrides-tty-enable")?;
        let mut app = test_app(repo_dir);
        let _ui = App::install_test_ui(
            true,
            &["1", "2", "1", "2", "1", "1", "2", "1", "2", "2", "1"],
        );

        let action = app.configure_server_overrides()?;

        assert_eq!(action, WizardAction::Continue);
        assert!(app.server_overrides_enabled);
        assert_eq!(app.server_enable_network_cli, "false");
        assert_eq!(app.server_enable_network_gui, "true");
        assert_eq!(app.server_enable_shell_tools, "false");
        assert_eq!(app.server_enable_wayland_tools, "true");
        assert_eq!(app.server_enable_system_tools, "true");
        assert_eq!(app.server_enable_geek_tools, "false");
        assert_eq!(app.server_enable_gaming, "true");
        assert_eq!(app.server_enable_insecure_tools, "false");
        assert_eq!(app.server_enable_docker, "false");
        assert_eq!(app.server_enable_libvirtd, "true");
        Ok(())
    }

    #[test]
    fn configure_server_overrides_tty_emits_terminal_transcript_for_enable_matrix() -> Result<()> {
        let repo_dir = create_temp_dir("mcbctl-server-overrides-transcript-enable")?;
        let mut app = test_app(repo_dir);
        let _ui = App::install_test_ui(
            true,
            &["1", "2", "1", "2", "1", "1", "2", "1", "2", "2", "1"],
        );

        let action = app.configure_server_overrides()?;
        let output = App::take_test_output();

        assert_eq!(action, WizardAction::Continue);
        assert!(output.contains("服务器软件覆盖"));
        assert!(output.contains("启用服务器包组覆盖"));
        assert!(output.contains("沿用主机现有配置"));
        assert!(output.contains("启用网络 CLI 包？"));
        assert!(output.contains("启用网络 GUI 包？"));
        assert!(output.contains("启用 Shell 工具？"));
        assert!(output.contains("启用 Wayland 工具？"));
        assert!(output.contains("启用系统工具？"));
        assert!(output.contains("启用 Geek 工具？"));
        assert!(output.contains("启用游戏工具？"));
        assert!(output.contains("启用不安全工具？"));
        assert!(output.contains("启用 Docker？"));
        assert!(output.contains("启用 Libvirtd？"));
        Ok(())
    }

    #[test]
    fn configure_server_overrides_tty_existing_config_clears_stale_values() -> Result<()> {
        let repo_dir = create_temp_dir("mcbctl-server-overrides-tty-existing")?;
        let mut app = test_app(repo_dir);
        app.server_overrides_enabled = true;
        app.server_enable_network_cli = "true".to_string();
        app.server_enable_network_gui = "true".to_string();
        app.server_enable_docker = "false".to_string();
        let _ui = App::install_test_ui(true, &["2"]);

        let action = app.configure_server_overrides()?;

        assert_eq!(action, WizardAction::Continue);
        assert!(!app.server_overrides_enabled);
        assert!(app.server_enable_network_cli.is_empty());
        assert!(app.server_enable_network_gui.is_empty());
        assert!(app.server_enable_docker.is_empty());
        Ok(())
    }

    #[test]
    fn configure_server_overrides_tty_existing_config_emits_short_terminal_transcript() -> Result<()>
    {
        let repo_dir = create_temp_dir("mcbctl-server-overrides-transcript-existing")?;
        let mut app = test_app(repo_dir);
        app.server_overrides_enabled = true;
        app.server_enable_network_cli = "true".to_string();
        app.server_enable_docker = "false".to_string();
        let _ui = App::install_test_ui(true, &["2"]);

        let action = app.configure_server_overrides()?;
        let output = App::take_test_output();

        assert_eq!(action, WizardAction::Continue);
        assert!(output.contains("服务器软件覆盖"));
        assert!(output.contains("沿用主机现有配置"));
        assert!(!output.contains("启用网络 CLI 包？"));
        assert!(!output.contains("启用 Docker？"));
        Ok(())
    }

    #[test]
    fn configure_server_overrides_tty_back_preserves_existing_state() -> Result<()> {
        let repo_dir = create_temp_dir("mcbctl-server-overrides-tty-back")?;
        let mut app = test_app(repo_dir);
        app.server_overrides_enabled = true;
        app.server_enable_network_cli = "true".to_string();
        app.server_enable_docker = "false".to_string();
        let _ui = App::install_test_ui(true, &["3"]);

        let action = app.configure_server_overrides()?;

        assert_eq!(action, WizardAction::Back);
        assert!(app.server_overrides_enabled);
        assert_eq!(app.server_enable_network_cli, "true");
        assert_eq!(app.server_enable_docker, "false");
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
            branch: "main".to_string(),
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
            per_user_tun_enabled: false,
            host_profile_kind: HostProfileKind::Server,
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
