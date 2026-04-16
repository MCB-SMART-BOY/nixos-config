use super::super::super::*;

impl App {
    pub(crate) fn configure_gpu(&mut self) -> Result<WizardAction> {
        let topology = self.detected_gpu.topology();

        if !self.is_tty() {
            if self.host_profile_kind == HostProfileKind::Desktop {
                let _ = self.apply_detected_gpu_defaults();
            } else {
                self.reset_gpu_override();
            }
            return Ok(WizardAction::Continue);
        }

        self.section("GPU 自动识别");
        self.note(&format!(
            "检测到当前主机：{}",
            self.detected_gpu.summary_line()
        ));

        match topology {
            DetectedGpuTopology::IgpuOnly => {
                self.note(
                    "当前是单集显主机，部署将直接按识别结果写入 igpu 模式，不再进入 GPU 切换问答。",
                );
                let _ = self.apply_detected_gpu_defaults();
                return Ok(WizardAction::Continue);
            }
            DetectedGpuTopology::DgpuOnly => {
                self.note(
                    "当前是独显主机，部署将直接按识别结果写入 dgpu 模式，不再进入 GPU 切换问答。",
                );
                let _ = self.apply_detected_gpu_defaults();
                return Ok(WizardAction::Continue);
            }
            DetectedGpuTopology::Unknown => {
                self.warn("未能自动识别当前主机 GPU 拓扑，将退回手动 GPU 配置。");
            }
            DetectedGpuTopology::MultiGpu => {
                let pick = self.menu_prompt(
                    "GPU 方案",
                    1,
                    &[
                        "使用自动识别结果（推荐：hybrid）".to_string(),
                        "沿用主机现有 GPU 配置".to_string(),
                        "手动指定 GPU 模式".to_string(),
                        "返回".to_string(),
                    ],
                )?;

                match pick {
                    1 => {
                        let _ = self.apply_detected_gpu_defaults();
                        return Ok(WizardAction::Continue);
                    }
                    2 => {
                        self.reset_gpu_override();
                        return Ok(WizardAction::Continue);
                    }
                    4 => return Ok(WizardAction::Back),
                    _ => {}
                }
            }
        }

        let pick = self.menu_prompt(
            "手动 GPU 方案",
            1,
            &[
                "沿用主机现有 GPU 配置".to_string(),
                "独显直通（dgpu）".to_string(),
                "混合显卡（hybrid）".to_string(),
                "仅核显（igpu）".to_string(),
                "返回".to_string(),
            ],
        )?;

        match pick {
            1 => {
                self.reset_gpu_override();
                return Ok(WizardAction::Continue);
            }
            5 => return Ok(WizardAction::Back),
            _ => {}
        }

        self.gpu_override = true;
        self.gpu_override_from_detection = false;
        self.gpu_mode = match pick {
            2 => "dgpu".to_string(),
            3 => "hybrid".to_string(),
            4 => "igpu".to_string(),
            _ => self.gpu_mode.clone(),
        };

        if self.gpu_mode == "hybrid" || self.gpu_mode == "igpu" {
            let igpu_pick = self.menu_prompt(
                "iGPU 厂商",
                1,
                &["Intel".to_string(), "AMD".to_string(), "返回".to_string()],
            )?;
            match igpu_pick {
                1 => self.gpu_igpu_vendor = "intel".to_string(),
                2 => self.gpu_igpu_vendor = "amd".to_string(),
                3 => return Ok(WizardAction::Back),
                _ => {}
            }
        }

        if self.gpu_mode == "hybrid" {
            let prime_pick = self.menu_prompt(
                "PRIME 模式",
                1,
                &[
                    "offload".to_string(),
                    "sync".to_string(),
                    "reverseSync".to_string(),
                    "返回".to_string(),
                ],
            )?;
            match prime_pick {
                1 => self.gpu_prime_mode = "offload".to_string(),
                2 => self.gpu_prime_mode = "sync".to_string(),
                3 => self.gpu_prime_mode = "reverseSync".to_string(),
                4 => return Ok(WizardAction::Back),
                _ => {}
            }
        } else {
            self.gpu_prime_mode.clear();
        }

        let intel_candidates = self.bus_candidates_for_vendor("intel");
        let amd_candidates = self.bus_candidates_for_vendor("amd");
        let nvidia_candidates = self.bus_candidates_for_vendor("nvidia");

        if self.gpu_mode == "hybrid" || self.gpu_mode == "igpu" {
            let candidates = if self.gpu_igpu_vendor == "amd" {
                &amd_candidates
            } else {
                &intel_candidates
            };
            let label = if self.gpu_igpu_vendor == "amd" {
                "AMD iGPU Bus ID"
            } else {
                "Intel iGPU Bus ID"
            };
            let mut options = candidates.clone();
            options.push("手动输入".to_string());
            options.push("返回".to_string());
            let pick = self.menu_prompt(label, 1, &options)?;
            if pick == options.len() {
                return Ok(WizardAction::Back);
            }
            if pick == options.len() - 1 {
                let input = self.prompt_line(&format!("{label}： "))?;
                let value = input.trim().to_string();
                if self.gpu_igpu_vendor == "amd" {
                    self.gpu_amd_bus = value;
                    self.gpu_intel_bus.clear();
                } else {
                    self.gpu_intel_bus = value;
                    self.gpu_amd_bus.clear();
                }
            } else {
                let value = candidates[pick - 1].clone();
                if self.gpu_igpu_vendor == "amd" {
                    self.gpu_amd_bus = value;
                    self.gpu_intel_bus.clear();
                } else {
                    self.gpu_intel_bus = value;
                    self.gpu_amd_bus.clear();
                }
            }
        } else {
            self.gpu_intel_bus.clear();
            self.gpu_amd_bus.clear();
        }

        if self.gpu_mode == "hybrid" || self.gpu_mode == "dgpu" {
            let mut options = nvidia_candidates.clone();
            options.push("手动输入".to_string());
            options.push("返回".to_string());
            let pick = self.menu_prompt("NVIDIA Bus ID", 1, &options)?;
            if pick == options.len() {
                return Ok(WizardAction::Back);
            }
            if pick == options.len() - 1 {
                let input = self.prompt_line("NVIDIA Bus ID： ")?;
                self.gpu_nvidia_bus = input.trim().to_string();
            } else {
                self.gpu_nvidia_bus = nvidia_candidates[pick - 1].clone();
            }

            let pick = self.menu_prompt(
                "NVIDIA Open 内核模块",
                1,
                &["开启".to_string(), "关闭".to_string(), "返回".to_string()],
            )?;
            match pick {
                1 => self.gpu_nvidia_open = "true".to_string(),
                2 => self.gpu_nvidia_open = "false".to_string(),
                3 => return Ok(WizardAction::Back),
                _ => {}
            }
        } else {
            self.gpu_nvidia_bus.clear();
            self.gpu_nvidia_open.clear();
        }

        let pick = self.menu_prompt(
            "是否启用 GPU specialisation",
            2,
            &["开启".to_string(), "关闭".to_string(), "返回".to_string()],
        )?;
        match pick {
            1 => {
                self.gpu_specialisations_set = true;
                self.gpu_specialisations_enabled = true;
                self.gpu_specialisation_modes = vec!["igpu".to_string(), "dgpu".to_string()];
                if self.gpu_mode == "hybrid" {
                    self.gpu_specialisation_modes.push("hybrid".to_string());
                }
            }
            2 => {
                self.gpu_specialisations_set = true;
                self.gpu_specialisations_enabled = false;
                self.gpu_specialisation_modes.clear();
            }
            3 => return Ok(WizardAction::Back),
            _ => {}
        }

        Ok(WizardAction::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn configure_gpu_non_tty_applies_detected_defaults_for_desktop() -> Result<()> {
        let repo_dir = create_temp_dir("mcbctl-gpu-prompt-default")?;
        let mut app = test_app(repo_dir);
        app.host_profile_kind = HostProfileKind::Desktop;
        app.detected_gpu = DetectedGpuProfile {
            topology: Some(DetectedGpuTopology::MultiGpu),
            igpu_vendor: "intel".to_string(),
            intel_bus: "PCI:0:2:0".to_string(),
            amd_bus: String::new(),
            nvidia_bus: "PCI:1:0:0".to_string(),
        };

        let action = app.configure_gpu()?;

        assert_eq!(action, WizardAction::Continue);
        assert!(app.gpu_override);
        assert_eq!(app.gpu_mode, "hybrid");
        assert_eq!(app.gpu_prime_mode, "offload");
        assert_eq!(app.gpu_intel_bus, "PCI:0:2:0");
        assert_eq!(app.gpu_nvidia_bus, "PCI:1:0:0");
        assert!(app.gpu_specialisations_enabled);
        Ok(())
    }

    #[test]
    fn configure_gpu_unknown_topology_allows_manual_bus_ids_without_candidates() -> Result<()> {
        let repo_dir = create_temp_dir("mcbctl-gpu-prompt-manual-unknown")?;
        let etc_dir = write_host_gpu_defaults(
            &repo_dir,
            r#"{ mcb.hardware.gpu.nvidiaBusId = "PCI:1:0:0"; }"#,
        )?;
        let mut app = test_app(repo_dir);
        app.etc_dir = etc_dir;
        app.detected_gpu = DetectedGpuProfile::default();
        let _ui = App::install_test_ui(true, &["3", "1", "1", "1", "PCI:0:2:0", "1", "2", "2"]);

        let action = app.configure_gpu()?;

        assert_eq!(action, WizardAction::Continue);
        assert!(app.gpu_override);
        assert!(!app.gpu_override_from_detection);
        assert_eq!(app.gpu_mode, "hybrid");
        assert_eq!(app.gpu_igpu_vendor, "intel");
        assert_eq!(app.gpu_prime_mode, "offload");
        assert_eq!(app.gpu_intel_bus, "PCI:0:2:0");
        assert_eq!(app.gpu_nvidia_bus, "PCI:1:0:0");
        assert_eq!(app.gpu_nvidia_open, "false");
        assert!(!app.gpu_specialisations_enabled);
        Ok(())
    }

    #[test]
    fn configure_gpu_unknown_topology_emits_terminal_transcript_for_manual_flow() -> Result<()> {
        let repo_dir = create_temp_dir("mcbctl-gpu-prompt-transcript-unknown")?;
        let etc_dir = write_host_gpu_defaults(
            &repo_dir,
            r#"{ mcb.hardware.gpu.nvidiaBusId = "PCI:1:0:0"; }"#,
        )?;
        let mut app = test_app(repo_dir);
        app.etc_dir = etc_dir;
        app.detected_gpu = DetectedGpuProfile::default();
        let _ui = App::install_test_ui(true, &["3", "1", "1", "1", "PCI:0:2:0", "1", "2", "2"]);

        let action = app.configure_gpu()?;
        let output = App::take_test_output();

        assert_eq!(action, WizardAction::Continue);
        assert!(output.contains("GPU 自动识别"));
        assert!(output.contains("检测到当前主机：未识别"));
        assert!(output.contains("[警告] 未能自动识别当前主机 GPU 拓扑，将退回手动 GPU 配置。"));
        assert!(output.contains("手动 GPU 方案"));
        assert!(output.contains("iGPU 厂商"));
        assert!(output.contains("PRIME 模式"));
        assert!(output.contains("Intel iGPU Bus ID"));
        assert!(output.contains("Intel iGPU Bus ID： "));
        assert!(output.contains("NVIDIA Bus ID"));
        assert!(output.contains("NVIDIA Open 内核模块"));
        assert!(output.contains("是否启用 GPU specialisation"));
        Ok(())
    }

    #[test]
    fn configure_gpu_manual_hybrid_returns_back_from_deep_nested_prompt() -> Result<()> {
        let repo_dir = create_temp_dir("mcbctl-gpu-prompt-back-deep")?;
        let etc_dir = write_host_gpu_defaults(
            &repo_dir,
            r#"{
                mcb.hardware.gpu.intelBusId = "PCI:0:2:0";
                mcb.hardware.gpu.nvidiaBusId = "PCI:1:0:0";
            }"#,
        )?;
        let mut app = test_app(repo_dir);
        app.etc_dir = etc_dir;
        app.detected_gpu = DetectedGpuProfile {
            topology: Some(DetectedGpuTopology::MultiGpu),
            igpu_vendor: "intel".to_string(),
            intel_bus: "PCI:0:2:0".to_string(),
            amd_bus: String::new(),
            nvidia_bus: "PCI:1:0:0".to_string(),
        };
        let _ui = App::install_test_ui(true, &["3", "3", "1", "1", "1", "1", "3"]);

        let action = app.configure_gpu()?;

        assert_eq!(action, WizardAction::Back);
        assert!(app.gpu_override);
        assert_eq!(app.gpu_mode, "hybrid");
        assert_eq!(app.gpu_igpu_vendor, "intel");
        assert_eq!(app.gpu_prime_mode, "offload");
        assert_eq!(app.gpu_intel_bus, "PCI:0:2:0");
        assert_eq!(app.gpu_nvidia_bus, "PCI:1:0:0");
        assert!(app.gpu_nvidia_open.is_empty());
        Ok(())
    }

    #[test]
    fn configure_gpu_manual_hybrid_back_path_emits_deep_menu_transcript() -> Result<()> {
        let repo_dir = create_temp_dir("mcbctl-gpu-prompt-transcript-back")?;
        let etc_dir = write_host_gpu_defaults(
            &repo_dir,
            r#"{
                mcb.hardware.gpu.intelBusId = "PCI:0:2:0";
                mcb.hardware.gpu.nvidiaBusId = "PCI:1:0:0";
            }"#,
        )?;
        let mut app = test_app(repo_dir);
        app.etc_dir = etc_dir;
        app.detected_gpu = DetectedGpuProfile {
            topology: Some(DetectedGpuTopology::MultiGpu),
            igpu_vendor: "intel".to_string(),
            intel_bus: "PCI:0:2:0".to_string(),
            amd_bus: String::new(),
            nvidia_bus: "PCI:1:0:0".to_string(),
        };
        let _ui = App::install_test_ui(true, &["3", "3", "1", "1", "1", "1", "3"]);

        let action = app.configure_gpu()?;
        let output = App::take_test_output();

        assert_eq!(action, WizardAction::Back);
        assert!(output.contains("GPU 自动识别"));
        assert!(output.contains("检测到当前主机：多显卡主机"));
        assert!(output.contains("GPU 方案"));
        assert!(output.contains("使用自动识别结果（推荐：hybrid）"));
        assert!(output.contains("手动 GPU 方案"));
        assert!(output.contains("iGPU 厂商"));
        assert!(output.contains("PRIME 模式"));
        assert!(output.contains("Intel iGPU Bus ID"));
        assert!(output.contains("NVIDIA Bus ID"));
        assert!(output.contains("NVIDIA Open 内核模块"));
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

    fn write_host_gpu_defaults(repo_dir: &Path, body: &str) -> Result<PathBuf> {
        let etc_dir = repo_dir.join("etc-nixos");
        let host_dir = etc_dir.join("hosts").join("demo");
        fs::create_dir_all(&host_dir)?;
        fs::write(host_dir.join("default.nix"), body)?;
        Ok(etc_dir)
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
