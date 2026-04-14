use super::*;

impl AppState {
    pub fn next_hosts_field(&mut self) {
        self.hosts_focus = (self.hosts_focus + 1) % 29;
    }

    pub fn previous_hosts_field(&mut self) {
        self.hosts_focus = if self.hosts_focus == 0 {
            28
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
            4 => {
                let options = vec!["tun".to_string(), "http".to_string(), "off".to_string()];
                self.adjust_current_host_string_field(delta, &options, |settings| {
                    &mut settings.proxy_mode
                });
            }
            8 => {
                self.toggle_current_host_bool_field(
                    |settings| &mut settings.enable_proxy_dns,
                    "全局代理 DNS 开关已切换。",
                );
            }
            11 => {
                self.toggle_current_host_bool_field(
                    |settings| &mut settings.per_user_tun_enable,
                    "Per-user TUN 开关已切换。",
                );
            }
            12 => {
                self.toggle_current_host_bool_field(
                    |settings| &mut settings.per_user_tun_compat_global_service_socket,
                    "兼容全局服务 Socket 开关已切换。",
                );
            }
            13 => {
                self.toggle_current_host_bool_field(
                    |settings| &mut settings.per_user_tun_redirect_dns,
                    "Per-user DNS 重定向开关已切换。",
                );
            }
            18 => {
                let options = vec!["igpu".to_string(), "hybrid".to_string(), "dgpu".to_string()];
                self.adjust_current_host_string_field(delta, &options, |settings| {
                    &mut settings.gpu_mode
                });
            }
            19 => {
                let options = vec!["intel".to_string(), "amd".to_string()];
                self.adjust_current_host_string_field(delta, &options, |settings| {
                    &mut settings.gpu_igpu_vendor
                });
            }
            20 => {
                let options = vec![
                    "offload".to_string(),
                    "sync".to_string(),
                    "reverseSync".to_string(),
                ];
                self.adjust_current_host_string_field(delta, &options, |settings| {
                    &mut settings.gpu_prime_mode
                });
            }
            24 => {
                self.toggle_current_host_bool_field(
                    |settings| &mut settings.gpu_nvidia_open,
                    "NVIDIA Open 开关已切换。",
                );
            }
            25 => {
                self.toggle_current_host_bool_field(
                    |settings| &mut settings.gpu_specialisations_enable,
                    "GPU 特化开关已切换。",
                );
            }
            27 => {
                self.toggle_current_host_bool_field(
                    |settings| &mut settings.docker_enable,
                    "Docker 开关已切换。",
                );
            }
            28 => {
                self.toggle_current_host_bool_field(
                    |settings| &mut settings.libvirtd_enable,
                    "Libvirtd 开关已切换。",
                );
            }
            _ => {}
        }
    }

    pub fn open_hosts_text_edit(&mut self) {
        let Some(settings) = self.current_host_settings().cloned() else {
            self.status = format!(
                "无法编辑主机设置：{}",
                self.host_settings_unavailable_message(&self.target_host)
            );
            return;
        };

        let (mode, value, message) = match self.hosts_focus {
            2 => (
                Some(HostsTextMode::CustomSubstituters),
                serialize_string_list(&settings.custom_substituters),
                "开始编辑 custom substituters；使用逗号分隔。",
            ),
            3 => (
                Some(HostsTextMode::CustomTrustedPublicKeys),
                serialize_string_list(&settings.custom_trusted_public_keys),
                "开始编辑 custom trusted-public-keys；使用逗号分隔。",
            ),
            5 => (
                Some(HostsTextMode::ProxyUrl),
                settings.proxy_url.clone(),
                "开始编辑代理 URL。",
            ),
            6 => (
                Some(HostsTextMode::TunInterface),
                settings.tun_interface.clone(),
                "开始编辑主 TUN 接口。",
            ),
            7 => (
                Some(HostsTextMode::TunInterfaces),
                serialize_string_list(&settings.tun_interfaces),
                "开始编辑额外 TUN 接口列表；使用逗号分隔。",
            ),
            9 => (
                Some(HostsTextMode::ProxyDnsAddr),
                settings.proxy_dns_addr.clone(),
                "开始编辑代理 DNS 地址。",
            ),
            10 => (
                Some(HostsTextMode::ProxyDnsPort),
                settings.proxy_dns_port.to_string(),
                "开始编辑代理 DNS 端口。",
            ),
            14 => (
                Some(HostsTextMode::PerUserTunInterfaces),
                serialize_string_map(&settings.per_user_tun_interfaces),
                "开始编辑 per-user TUN 接口映射，格式为 user=iface。",
            ),
            15 => (
                Some(HostsTextMode::PerUserTunDnsPorts),
                serialize_u16_map(&settings.per_user_tun_dns_ports),
                "开始编辑 per-user DNS 端口映射，格式为 user=1053。",
            ),
            16 => (
                Some(HostsTextMode::PerUserTunTableBase),
                settings.per_user_tun_table_base.to_string(),
                "开始编辑 per-user 路由表基值。",
            ),
            17 => (
                Some(HostsTextMode::PerUserTunPriorityBase),
                settings.per_user_tun_priority_base.to_string(),
                "开始编辑 per-user 规则优先级基值。",
            ),
            21 => (
                Some(HostsTextMode::IntelBusId),
                settings.gpu_intel_bus.clone().unwrap_or_default(),
                "开始编辑 Intel Bus ID。",
            ),
            22 => (
                Some(HostsTextMode::AmdBusId),
                settings.gpu_amd_bus.clone().unwrap_or_default(),
                "开始编辑 AMD Bus ID。",
            ),
            23 => (
                Some(HostsTextMode::NvidiaBusId),
                settings.gpu_nvidia_bus.clone().unwrap_or_default(),
                "开始编辑 NVIDIA Bus ID。",
            ),
            26 => (
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
        let Some(mut current_settings) = self.current_host_settings().cloned() else {
            self.status = format!(
                "无法调整当前主机字段：{}",
                self.host_settings_unavailable_message(&self.target_host)
            );
            return;
        };
        let current = field(&mut current_settings).clone();
        let Some(next) = cycle_string_value(&current, options, delta) else {
            return;
        };
        let Some(settings) = self.current_host_settings_mut() else {
            self.status = format!(
                "无法调整当前主机字段：{}",
                self.host_settings_unavailable_message(&self.target_host)
            );
            return;
        };
        *field(settings) = next.clone();
        self.host_dirty_runtime_hosts
            .insert(self.target_host.clone());
        self.status = format!("当前字段已切换为：{next}");
    }

    fn toggle_current_host_bool_field<F>(&mut self, mut field: F, message: &str)
    where
        F: FnMut(&mut HostManagedSettings) -> &mut bool,
    {
        let Some(settings) = self.current_host_settings_mut() else {
            self.status = format!(
                "无法调整当前主机字段：{}",
                self.host_settings_unavailable_message(&self.target_host)
            );
            return;
        };
        let value = field(settings);
        *value = !*value;
        self.host_dirty_runtime_hosts
            .insert(self.target_host.clone());
        self.status = message.to_string();
    }

    fn confirm_hosts_text_edit(&mut self) {
        let Some(mode) = self.hosts_text_mode else {
            return;
        };

        let raw = self.host_text_input.trim().to_string();
        let Some(settings) = self.current_host_settings_mut() else {
            self.hosts_text_mode = None;
            self.host_text_input.clear();
            self.status = format!(
                "无法确认主机设置编辑：{}",
                self.host_settings_unavailable_message(&self.target_host)
            );
            return;
        };

        let result: Result<()> = match mode {
            HostsTextMode::CustomSubstituters => {
                settings.custom_substituters = parse_string_list(&raw);
                Ok(())
            }
            HostsTextMode::CustomTrustedPublicKeys => {
                settings.custom_trusted_public_keys = parse_string_list(&raw);
                Ok(())
            }
            HostsTextMode::ProxyUrl => {
                settings.proxy_url = raw;
                Ok(())
            }
            HostsTextMode::TunInterface => {
                settings.tun_interface = raw;
                Ok(())
            }
            HostsTextMode::TunInterfaces => {
                settings.tun_interfaces = parse_string_list(&raw);
                Ok(())
            }
            HostsTextMode::ProxyDnsAddr => {
                settings.proxy_dns_addr = raw;
                Ok(())
            }
            HostsTextMode::ProxyDnsPort => raw
                .parse::<u16>()
                .with_context(|| format!("无效端口：{raw}"))
                .map(|value| {
                    settings.proxy_dns_port = value;
                }),
            HostsTextMode::PerUserTunInterfaces => parse_string_map(&raw).map(|value| {
                settings.per_user_tun_interfaces = value;
            }),
            HostsTextMode::PerUserTunDnsPorts => parse_u16_map(&raw).map(|value| {
                settings.per_user_tun_dns_ports = value;
            }),
            HostsTextMode::PerUserTunTableBase => raw
                .parse::<i64>()
                .with_context(|| format!("无效整数：{raw}"))
                .map(|value| {
                    settings.per_user_tun_table_base = value;
                }),
            HostsTextMode::PerUserTunPriorityBase => raw
                .parse::<i64>()
                .with_context(|| format!("无效整数：{raw}"))
                .map(|value| {
                    settings.per_user_tun_priority_base = value;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::Path;

    #[test]
    fn confirm_hosts_text_edit_keeps_invalid_input_open() {
        let mut state = test_state(Path::new("/repo"));
        state.hosts_text_mode = Some(HostsTextMode::ProxyDnsPort);
        state.host_text_input = "not-a-port".to_string();

        state.confirm_hosts_text_edit();

        assert_eq!(state.host_settings_by_name["demo"].proxy_dns_port, 53);
        assert_eq!(state.hosts_text_mode, Some(HostsTextMode::ProxyDnsPort));
        assert_eq!(state.host_text_input, "not-a-port");
        assert!(!state.host_dirty_runtime_hosts.contains("demo"));
        assert!(state.status.contains("输入无效"));
        assert!(state.status.contains("无效端口"));
    }

    #[test]
    fn confirm_hosts_text_edit_updates_runtime_field_and_marks_dirty() {
        let mut state = test_state(Path::new("/repo"));
        state.hosts_text_mode = Some(HostsTextMode::ProxyDnsPort);
        state.host_text_input = "1053".to_string();

        state.confirm_hosts_text_edit();

        assert_eq!(state.host_settings_by_name["demo"].proxy_dns_port, 1053);
        assert!(state.host_dirty_runtime_hosts.contains("demo"));
        assert!(state.hosts_text_mode.is_none());
        assert!(state.host_text_input.is_empty());
        assert_eq!(state.status, "主机字段已更新。");
    }

    #[test]
    fn adjust_hosts_field_refuses_to_modify_unavailable_host_settings() {
        let mut state = test_state(Path::new("/repo"));
        state.host_settings_by_name.clear();
        state.host_settings_errors_by_name.insert(
            "demo".to_string(),
            "nix eval for host demo failed".to_string(),
        );
        state.hosts_focus = 4;

        state.adjust_hosts_field(1);

        assert!(!state.host_dirty_runtime_hosts.contains("demo"));
        assert!(state.status.contains("无法调整当前主机字段"));
        assert!(state.status.contains("配置读取失败"));
    }

    fn test_state(root: &Path) -> AppState {
        let mut host_settings_by_name = BTreeMap::new();
        host_settings_by_name.insert("demo".to_string(), valid_host_settings());

        AppState {
            context: AppContext {
                repo_root: root.to_path_buf(),
                etc_root: PathBuf::from("/etc/nixos"),
                current_host: "demo".to_string(),
                current_system: "x86_64-linux".to_string(),
                current_user: "alice".to_string(),
                privilege_mode: "sudo-available".to_string(),
                hosts: vec!["demo".to_string()],
                users: vec!["alice".to_string(), "bob".to_string()],
                catalog_path: root.join("catalog/packages"),
                catalog_groups_path: root.join("catalog/groups.toml"),
                catalog_home_options_path: root.join("catalog/home-options.toml"),
                catalog_entries: Vec::new(),
                catalog_groups: BTreeMap::new(),
                catalog_home_options: Vec::new(),
                catalog_categories: Vec::new(),
                catalog_sources: Vec::new(),
            },
            active_page: 0,
            deploy_focus: 0,
            target_host: "demo".to_string(),
            deploy_task: DeployTask::DirectDeploy,
            deploy_source: DeploySource::CurrentRepo,
            deploy_action: DeployAction::Switch,
            flake_update: false,
            show_advanced: false,
            users_focus: 0,
            hosts_focus: 0,
            users_text_mode: None,
            hosts_text_mode: None,
            host_text_input: String::new(),
            host_settings_by_name,
            host_settings_errors_by_name: BTreeMap::new(),
            host_dirty_user_hosts: BTreeSet::new(),
            host_dirty_runtime_hosts: BTreeSet::new(),
            package_user_index: 0,
            package_mode: PackageDataMode::Search,
            package_cursor: 0,
            package_category_index: 0,
            package_group_filter: None,
            package_source_filter: None,
            package_search: String::new(),
            package_search_result_indices: Vec::new(),
            package_local_entry_ids: BTreeSet::new(),
            package_search_mode: false,
            package_group_create_mode: false,
            package_group_rename_mode: false,
            package_group_rename_source: String::new(),
            package_group_input: String::new(),
            package_user_selections: BTreeMap::new(),
            package_dirty_users: BTreeSet::new(),
            home_user_index: 0,
            home_focus: 0,
            home_settings_by_user: BTreeMap::new(),
            home_dirty_users: BTreeSet::new(),
            actions_focus: 0,
            overview_repo_integrity: OverviewCheckState::NotRun,
            overview_doctor: OverviewCheckState::NotRun,
            status: String::new(),
        }
    }

    fn valid_host_settings() -> HostManagedSettings {
        HostManagedSettings {
            primary_user: "alice".to_string(),
            users: vec!["alice".to_string()],
            admin_users: vec!["alice".to_string()],
            ..HostManagedSettings::default()
        }
    }
}
