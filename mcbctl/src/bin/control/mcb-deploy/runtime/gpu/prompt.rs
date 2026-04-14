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
