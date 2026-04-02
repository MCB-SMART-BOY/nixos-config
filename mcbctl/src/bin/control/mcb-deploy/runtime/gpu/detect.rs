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
