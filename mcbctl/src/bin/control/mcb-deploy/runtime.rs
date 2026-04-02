use super::*;

impl App {
    pub(super) fn reset_tun_maps(&mut self) {
        self.user_tun.clear();
        self.user_dns.clear();
    }

    pub(super) fn reset_admin_users(&mut self) {
        self.target_admin_users.clear();
    }

    pub(super) fn reset_server_overrides(&mut self) {
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

    pub(super) fn reset_gpu_override(&mut self) {
        self.gpu_override = false;
        self.gpu_override_from_detection = false;
        self.gpu_mode.clear();
        self.gpu_igpu_vendor.clear();
        self.gpu_prime_mode.clear();
        self.gpu_intel_bus.clear();
        self.gpu_amd_bus.clear();
        self.gpu_nvidia_bus.clear();
        self.gpu_nvidia_open.clear();
        self.gpu_specialisations_enabled = false;
        self.gpu_specialisations_set = false;
        self.gpu_specialisation_modes.clear();
    }

    pub(super) fn detect_host_gpu_profile(&mut self) {
        let intel = Self::detect_bus_ids_from_lspci("intel");
        let amd = Self::detect_bus_ids_from_lspci("amd");
        let nvidia = Self::detect_bus_ids_from_lspci("nvidia");

        let intel_bus = intel
            .first()
            .cloned()
            .or_else(|| self.resolve_bus_id_default("intel"));
        let amd_bus = amd
            .first()
            .cloned()
            .or_else(|| self.resolve_bus_id_default("amd"));
        let nvidia_bus = nvidia
            .first()
            .cloned()
            .or_else(|| self.resolve_bus_id_default("nvidia"));

        let topology = if nvidia_bus.is_some() && (intel_bus.is_some() || amd_bus.is_some()) {
            DetectedGpuTopology::MultiGpu
        } else if nvidia_bus.is_some() {
            DetectedGpuTopology::DgpuOnly
        } else if intel_bus.is_some() || amd_bus.is_some() {
            DetectedGpuTopology::IgpuOnly
        } else {
            DetectedGpuTopology::Unknown
        };

        self.detected_gpu = DetectedGpuProfile {
            topology: Some(topology),
            igpu_vendor: if intel_bus.is_some() {
                "intel".to_string()
            } else if amd_bus.is_some() {
                "amd".to_string()
            } else {
                String::new()
            },
            intel_bus: intel_bus.unwrap_or_default(),
            amd_bus: amd_bus.unwrap_or_default(),
            nvidia_bus: nvidia_bus.unwrap_or_default(),
        };
    }

    fn apply_detected_gpu_defaults(&mut self) -> bool {
        let topology = self.detected_gpu.topology();
        if topology == DetectedGpuTopology::Unknown {
            return false;
        }

        self.gpu_override = true;
        self.gpu_override_from_detection = true;
        self.gpu_mode = topology.recommended_mode().to_string();
        self.gpu_igpu_vendor = self.detected_gpu.igpu_vendor.clone();
        self.gpu_prime_mode.clear();
        self.gpu_intel_bus = self.detected_gpu.intel_bus.clone();
        self.gpu_amd_bus = self.detected_gpu.amd_bus.clone();
        self.gpu_nvidia_bus = self.detected_gpu.nvidia_bus.clone();
        self.gpu_nvidia_open.clear();
        self.gpu_specialisations_set = true;

        match topology {
            DetectedGpuTopology::IgpuOnly => {
                self.gpu_prime_mode.clear();
                self.gpu_nvidia_bus.clear();
                self.gpu_nvidia_open.clear();
                self.gpu_specialisations_enabled = false;
                self.gpu_specialisation_modes.clear();
            }
            DetectedGpuTopology::MultiGpu => {
                self.gpu_mode = "hybrid".to_string();
                self.gpu_prime_mode = "offload".to_string();
                self.gpu_nvidia_open = "true".to_string();
                self.gpu_specialisations_enabled = true;
                self.gpu_specialisation_modes =
                    vec!["igpu".to_string(), "hybrid".to_string(), "dgpu".to_string()];
            }
            DetectedGpuTopology::DgpuOnly => {
                self.gpu_mode = "dgpu".to_string();
                self.gpu_igpu_vendor.clear();
                self.gpu_prime_mode.clear();
                self.gpu_intel_bus.clear();
                self.gpu_amd_bus.clear();
                self.gpu_nvidia_open = "true".to_string();
                self.gpu_specialisations_enabled = false;
                self.gpu_specialisation_modes.clear();
            }
            DetectedGpuTopology::Unknown => return false,
        }

        true
    }

    pub(super) fn configure_per_user_tun(&mut self) -> Result<WizardAction> {
        if !self.is_tty() {
            return Ok(WizardAction::Continue);
        }
        self.section("Per-user TUN 配置");
        self.note("检测到当前主机已启用 per-user TUN。");
        self.note("请为每个用户指定独立 TUN 名称与 DNS 端口。");
        loop {
            self.user_tun.clear();
            self.user_dns.clear();
            for (idx, user) in self.target_users.iter().enumerate() {
                let default_iface = format!("tun{}", idx + 1);
                print!("用户 {user} 的 TUN 接口（默认 {default_iface}）： ");
                io::stdout().flush().ok();
                let mut iface = String::new();
                io::stdin().read_line(&mut iface).ok();
                let iface = iface.trim();
                let iface = if iface.is_empty() {
                    &default_iface
                } else {
                    iface
                };
                self.user_tun.insert(user.clone(), iface.to_string());

                let default_dns = 1053u16 + (idx as u16);
                print!("用户 {user} 的 DNS 端口（默认 {default_dns}）： ");
                io::stdout().flush().ok();
                let mut dns = String::new();
                io::stdin().read_line(&mut dns).ok();
                let dns = dns.trim();
                let port = if dns.is_empty() {
                    default_dns
                } else if let Ok(v) = dns.parse::<u16>() {
                    v
                } else {
                    self.warn("端口无效，请重新输入这一轮。");
                    self.user_tun.clear();
                    self.user_dns.clear();
                    continue;
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

    fn strip_leading_zeros(v: &str) -> String {
        v.trim_start_matches('0').to_string()
    }

    fn normalize_pci_bus_id(addr: &str) -> Option<String> {
        let addr = addr.trim();
        let raw = if let Some(rest) = addr.strip_prefix("0000:") {
            rest
        } else {
            addr
        };
        let parts: Vec<&str> = raw.split(':').collect();
        if parts.len() != 2 {
            return None;
        }
        let bus = Self::strip_leading_zeros(parts[0]);
        let rest: Vec<&str> = parts[1].split('.').collect();
        if rest.len() != 2 {
            return None;
        }
        let device = Self::strip_leading_zeros(rest[0]);
        let function = Self::strip_leading_zeros(rest[1]);
        Some(format!("PCI:{bus}:{device}:{function}"))
    }

    fn detect_bus_ids_from_lspci(vendor: &str) -> Vec<String> {
        let mut out = Vec::new();
        let Some(text) = run_capture_allow_fail("lspci", &["-D"]) else {
            return out;
        };
        for line in text.lines() {
            let line_lc = line.to_lowercase();
            let matches = match vendor {
                "intel" => {
                    line_lc.contains("vga compatible controller") && line_lc.contains("intel")
                }
                "amd" => {
                    line_lc.contains("vga compatible controller")
                        && (line_lc.contains("advanced micro devices")
                            || line_lc.contains("amd/ati")
                            || line_lc.contains("amd "))
                }
                "nvidia" => line_lc.contains("3d controller") || line_lc.contains("nvidia"),
                _ => false,
            };
            if !matches {
                continue;
            }
            let addr = line.split_whitespace().next().unwrap_or_default();
            if let Some(bus_id) = Self::normalize_pci_bus_id(addr)
                && !out.contains(&bus_id)
            {
                out.push(bus_id);
            }
        }
        out
    }

    fn extract_bus_id_from_file(file: &Path, key: &str) -> Option<String> {
        let text = fs::read_to_string(file).ok()?;
        for line in text.lines() {
            let l = strip_comment(line);
            if l.contains(key)
                && l.contains('"')
                && let Some(v) = first_quoted(l)
            {
                return Some(v);
            }
        }
        None
    }

    fn resolve_bus_id_default(&self, vendor: &str) -> Option<String> {
        let key = match vendor {
            "intel" => "intelBusId",
            "amd" => "amdgpuBusId",
            "nvidia" => "nvidiaBusId",
            _ => return None,
        };
        let mut files = Vec::new();
        if let Some(tmp_dir) = &self.tmp_dir {
            files.push(
                tmp_dir
                    .join("hosts")
                    .join(&self.target_name)
                    .join("local.nix"),
            );
            files.push(
                tmp_dir
                    .join("hosts")
                    .join(&self.target_name)
                    .join("default.nix"),
            );
        }
        files.push(
            self.etc_dir
                .join("hosts")
                .join(&self.target_name)
                .join("local.nix"),
        );
        files.push(
            self.etc_dir
                .join("hosts")
                .join(&self.target_name)
                .join("default.nix"),
        );
        for file in files {
            if let Some(v) = Self::extract_bus_id_from_file(&file, key) {
                return Some(v);
            }
        }
        Self::detect_bus_ids_from_lspci(vendor).into_iter().next()
    }

    fn bus_candidates_for_vendor(&self, vendor: &str) -> Vec<String> {
        let mut out = Vec::new();
        if let Some(v) = self.resolve_bus_id_default(vendor) {
            out.push(v);
        }
        for v in Self::detect_bus_ids_from_lspci(vendor) {
            if !out.contains(&v) {
                out.push(v);
            }
        }
        out
    }

    pub(super) fn configure_gpu(&mut self) -> Result<WizardAction> {
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
            if !candidates.is_empty() {
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
                    print!("{label}： ");
                    io::stdout().flush().ok();
                    let mut input = String::new();
                    io::stdin().read_line(&mut input).ok();
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
                print!("NVIDIA Bus ID： ");
                io::stdout().flush().ok();
                let mut input = String::new();
                io::stdin().read_line(&mut input).ok();
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

    pub(super) fn configure_server_overrides(&mut self) -> Result<WizardAction> {
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
