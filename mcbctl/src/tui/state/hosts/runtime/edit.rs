use super::*;

impl AppState {
    pub fn next_hosts_field(&mut self) {
        self.hosts_focus = (self.hosts_focus + 1) % 19;
    }

    pub fn previous_hosts_field(&mut self) {
        self.hosts_focus = if self.hosts_focus == 0 {
            18
        } else {
            self.hosts_focus - 1
        };
    }

    pub fn adjust_hosts_field(&mut self, delta: i8) {
        match self.hosts_focus {
            0 => self.switch_target_host(delta),
            1 => {
                let options = vec![
                    "cn".to_string(),
                    "global".to_string(),
                    "official-only".to_string(),
                    "custom".to_string(),
                ];
                self.adjust_current_host_string_field(delta, &options, |settings| {
                    &mut settings.cache_profile
                });
            }
            2 => {
                let options = vec!["tun".to_string(), "http".to_string(), "off".to_string()];
                self.adjust_current_host_string_field(delta, &options, |settings| {
                    &mut settings.proxy_mode
                });
            }
            5 => {
                if let Some(settings) = self.current_host_settings_mut() {
                    settings.per_user_tun_enable = !settings.per_user_tun_enable;
                }
                self.host_dirty_runtime_hosts
                    .insert(self.target_host.clone());
                self.status = "Per-user TUN 开关已切换。".to_string();
            }
            8 => {
                let options = vec!["igpu".to_string(), "hybrid".to_string(), "dgpu".to_string()];
                self.adjust_current_host_string_field(delta, &options, |settings| {
                    &mut settings.gpu_mode
                });
            }
            9 => {
                let options = vec!["intel".to_string(), "amd".to_string()];
                self.adjust_current_host_string_field(delta, &options, |settings| {
                    &mut settings.gpu_igpu_vendor
                });
            }
            10 => {
                let options = vec![
                    "offload".to_string(),
                    "sync".to_string(),
                    "reverseSync".to_string(),
                ];
                self.adjust_current_host_string_field(delta, &options, |settings| {
                    &mut settings.gpu_prime_mode
                });
            }
            14 => {
                if let Some(settings) = self.current_host_settings_mut() {
                    settings.gpu_nvidia_open = !settings.gpu_nvidia_open;
                }
                self.host_dirty_runtime_hosts
                    .insert(self.target_host.clone());
                self.status = "NVIDIA Open 开关已切换。".to_string();
            }
            15 => {
                if let Some(settings) = self.current_host_settings_mut() {
                    settings.gpu_specialisations_enable = !settings.gpu_specialisations_enable;
                }
                self.host_dirty_runtime_hosts
                    .insert(self.target_host.clone());
                self.status = "GPU 特化开关已切换。".to_string();
            }
            17 => {
                if let Some(settings) = self.current_host_settings_mut() {
                    settings.docker_enable = !settings.docker_enable;
                }
                self.host_dirty_runtime_hosts
                    .insert(self.target_host.clone());
                self.status = "Docker 开关已切换。".to_string();
            }
            18 => {
                if let Some(settings) = self.current_host_settings_mut() {
                    settings.libvirtd_enable = !settings.libvirtd_enable;
                }
                self.host_dirty_runtime_hosts
                    .insert(self.target_host.clone());
                self.status = "Libvirtd 开关已切换。".to_string();
            }
            _ => {}
        }
    }

    pub fn open_hosts_text_edit(&mut self) {
        let Some(settings) = self.current_host_settings().cloned() else {
            self.status = "当前主机没有可编辑的主机设置。".to_string();
            return;
        };

        let (mode, value, message) = match self.hosts_focus {
            3 => (
                Some(HostsTextMode::ProxyUrl),
                settings.proxy_url.clone(),
                "开始编辑代理 URL。",
            ),
            4 => (
                Some(HostsTextMode::TunInterface),
                settings.tun_interface.clone(),
                "开始编辑主 TUN 接口。",
            ),
            6 => (
                Some(HostsTextMode::PerUserTunInterfaces),
                serialize_string_map(&settings.per_user_tun_interfaces),
                "开始编辑 per-user TUN 接口映射，格式为 user=iface。",
            ),
            7 => (
                Some(HostsTextMode::PerUserTunDnsPorts),
                serialize_u16_map(&settings.per_user_tun_dns_ports),
                "开始编辑 per-user DNS 端口映射，格式为 user=1053。",
            ),
            11 => (
                Some(HostsTextMode::IntelBusId),
                settings.gpu_intel_bus.clone().unwrap_or_default(),
                "开始编辑 Intel Bus ID。",
            ),
            12 => (
                Some(HostsTextMode::AmdBusId),
                settings.gpu_amd_bus.clone().unwrap_or_default(),
                "开始编辑 AMD Bus ID。",
            ),
            13 => (
                Some(HostsTextMode::NvidiaBusId),
                settings.gpu_nvidia_bus.clone().unwrap_or_default(),
                "开始编辑 NVIDIA Bus ID。",
            ),
            16 => (
                Some(HostsTextMode::SpecialisationModes),
                serialize_string_list(&settings.gpu_specialisation_modes),
                "开始编辑 GPU 特化模式列表；使用逗号分隔。",
            ),
            _ => (None, String::new(), ""),
        };

        if let Some(mode) = mode {
            self.hosts_text_mode = Some(mode);
            self.host_text_input = value;
            self.status = message.to_string();
        }
    }

    pub fn handle_hosts_text_input(&mut self, code: crossterm::event::KeyCode) {
        match code {
            crossterm::event::KeyCode::Enter => self.confirm_hosts_text_edit(),
            crossterm::event::KeyCode::Esc => {
                self.hosts_text_mode = None;
                self.host_text_input.clear();
                self.status = "已取消主机设置编辑。".to_string();
            }
            crossterm::event::KeyCode::Backspace => {
                self.host_text_input.pop();
            }
            crossterm::event::KeyCode::Char(ch) => {
                self.host_text_input.push(ch);
            }
            _ => {}
        }
    }

    fn adjust_current_host_string_field<F>(&mut self, delta: i8, options: &[String], mut field: F)
    where
        F: FnMut(&mut HostManagedSettings) -> &mut String,
    {
        let current = self
            .current_host_settings()
            .cloned()
            .map(|mut settings| field(&mut settings).clone())
            .unwrap_or_default();
        let Some(next) = cycle_string_value(&current, options, delta) else {
            return;
        };
        if let Some(settings) = self.current_host_settings_mut() {
            *field(settings) = next.clone();
        }
        self.host_dirty_runtime_hosts
            .insert(self.target_host.clone());
        self.status = format!("当前字段已切换为：{next}");
    }

    fn confirm_hosts_text_edit(&mut self) {
        let Some(mode) = self.hosts_text_mode else {
            return;
        };

        let raw = self.host_text_input.trim().to_string();
        let Some(settings) = self.current_host_settings_mut() else {
            self.hosts_text_mode = None;
            self.host_text_input.clear();
            self.status = "当前主机没有可编辑的主机设置。".to_string();
            return;
        };

        let result: Result<()> = match mode {
            HostsTextMode::ProxyUrl => {
                settings.proxy_url = raw;
                Ok(())
            }
            HostsTextMode::TunInterface => {
                settings.tun_interface = raw;
                Ok(())
            }
            HostsTextMode::PerUserTunInterfaces => parse_string_map(&raw).map(|value| {
                settings.per_user_tun_interfaces = value;
            }),
            HostsTextMode::PerUserTunDnsPorts => parse_u16_map(&raw).map(|value| {
                settings.per_user_tun_dns_ports = value;
            }),
            HostsTextMode::IntelBusId => {
                settings.gpu_intel_bus = empty_to_none(&raw);
                Ok(())
            }
            HostsTextMode::AmdBusId => {
                settings.gpu_amd_bus = empty_to_none(&raw);
                Ok(())
            }
            HostsTextMode::NvidiaBusId => {
                settings.gpu_nvidia_bus = empty_to_none(&raw);
                Ok(())
            }
            HostsTextMode::SpecialisationModes => parse_gpu_modes(&raw).map(|value| {
                settings.gpu_specialisation_modes = value;
            }),
        };

        match result {
            Ok(()) => {
                self.host_dirty_runtime_hosts
                    .insert(self.target_host.clone());
                self.hosts_text_mode = None;
                self.host_text_input.clear();
                self.status = "主机字段已更新。".to_string();
            }
            Err(err) => {
                self.status = format!("输入无效：{err}");
            }
        }
    }
}
