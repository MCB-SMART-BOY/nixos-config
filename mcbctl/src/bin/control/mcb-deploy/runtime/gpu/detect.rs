use super::super::super::*;

impl App {
    pub(crate) fn reset_gpu_override(&mut self) {
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

    pub(crate) fn detect_host_gpu_profile(&mut self) {
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

    pub(crate) fn apply_detected_gpu_defaults(&mut self) -> bool {
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

    fn extract_bus_id_from_file(file: &Path, key: &str) -> Result<Option<String>> {
        let text = match fs::read_to_string(file) {
            Ok(text) => text,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(err) => {
                return Err(err)
                    .with_context(|| format!("读取 GPU Bus ID 候选文件 {} 失败", file.display()));
            }
        };
        for line in text.lines() {
            let l = strip_comment(line);
            if l.contains(key)
                && l.contains('"')
                && let Some(v) = first_quoted(l)
            {
                return Ok(Some(v));
            }
        }
        Ok(None)
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
            match Self::extract_bus_id_from_file(&file, key) {
                Ok(Some(v)) => return Some(v),
                Ok(None) => {}
                Err(err) => self.warn(&err.to_string()),
            }
        }
        Self::detect_bus_ids_from_lspci(vendor).into_iter().next()
    }

    pub(crate) fn bus_candidates_for_vendor(&self, vendor: &str) -> Vec<String> {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_bus_id_from_file_returns_none_for_missing_file() -> Result<()> {
        let temp_root = create_temp_dir("mcbctl-gpu-bus-missing")?;
        let missing = temp_root.join("missing.nix");

        assert_eq!(App::extract_bus_id_from_file(&missing, "intelBusId")?, None);
        Ok(())
    }

    #[test]
    fn extract_bus_id_from_file_reports_unreadable_path() -> Result<()> {
        let temp_root = create_temp_dir("mcbctl-gpu-bus-unreadable")?;
        let directory = temp_root.join("candidate.nix");
        fs::create_dir_all(&directory)?;

        let err = App::extract_bus_id_from_file(&directory, "intelBusId")
            .expect_err("directories should not be treated as missing files");

        assert!(err.to_string().contains("读取 GPU Bus ID 候选文件"));
        Ok(())
    }

    #[test]
    fn resolve_bus_id_default_skips_unreadable_candidate_files() -> Result<()> {
        let temp_root = create_temp_dir("mcbctl-gpu-bus-fallback")?;
        let host_dir = temp_root.join("hosts/demo");
        fs::create_dir_all(host_dir.join("local.nix"))?;
        fs::write(
            host_dir.join("default.nix"),
            r#"{ mcb.hardware.gpu.intelBusId = "PCI:0:2:0"; }"#,
        )?;
        let app = test_app(temp_root.clone());

        assert_eq!(
            app.resolve_bus_id_default("intel"),
            Some("PCI:0:2:0".to_string())
        );
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

    fn test_app(tmp_dir: PathBuf) -> App {
        App {
            repo_dir: tmp_dir.clone(),
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
            etc_dir: PathBuf::from("/tmp/etc-nixos"),
            dns_enabled: false,
            temp_dns_backend: String::new(),
            temp_dns_backup: None,
            temp_dns_iface: String::new(),
            tmp_dir: Some(tmp_dir),
            sudo_cmd: None,
            rootless: false,
            run_action: RunAction::Deploy,
            progress_total: 7,
            progress_current: 0,
            git_clone_timeout_sec: 90,
        }
    }
}
