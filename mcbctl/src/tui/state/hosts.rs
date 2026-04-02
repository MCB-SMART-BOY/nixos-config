use super::*;

impl AppState {
    pub(super) fn current_host_settings(&self) -> Option<&HostManagedSettings> {
        self.host_settings_by_name.get(&self.target_host)
    }

    fn current_host_settings_mut(&mut self) -> Option<&mut HostManagedSettings> {
        self.host_settings_by_name.get_mut(&self.target_host)
    }

    pub fn current_host_users_path(&self) -> Option<PathBuf> {
        let host = self
            .context
            .hosts
            .iter()
            .find(|name| *name == &self.target_host)?;
        Some(managed_host_users_path(&self.context.repo_root, host))
    }

    pub fn current_host_runtime_paths(&self) -> Vec<PathBuf> {
        let Some(host) = self
            .context
            .hosts
            .iter()
            .find(|name| *name == &self.target_host)
        else {
            return Vec::new();
        };

        vec![
            managed_host_network_path(&self.context.repo_root, host),
            managed_host_gpu_path(&self.context.repo_root, host),
            managed_host_virtualization_path(&self.context.repo_root, host),
        ]
    }

    pub fn users_rows(&self) -> Vec<(String, String)> {
        let settings = self.current_host_settings().cloned().unwrap_or_default();
        vec![
            ("主机".to_string(), self.target_host.clone()),
            ("主用户".to_string(), settings.primary_user),
            ("托管用户".to_string(), format_string_list(&settings.users)),
            (
                "管理员".to_string(),
                format_string_list(&settings.admin_users),
            ),
            ("主机角色".to_string(), settings.host_role),
            (
                "用户 linger".to_string(),
                bool_label(settings.user_linger).to_string(),
            ),
        ]
    }

    pub fn hosts_rows(&self) -> Vec<(String, String)> {
        let settings = self.current_host_settings().cloned().unwrap_or_default();
        vec![
            ("主机".to_string(), self.target_host.clone()),
            ("缓存策略".to_string(), settings.cache_profile),
            ("代理模式".to_string(), settings.proxy_mode),
            ("代理 URL".to_string(), nonempty_label(&settings.proxy_url)),
            (
                "主 TUN 接口".to_string(),
                nonempty_label(&settings.tun_interface),
            ),
            (
                "Per-user TUN".to_string(),
                bool_label(settings.per_user_tun_enable).to_string(),
            ),
            (
                "用户接口映射".to_string(),
                format_string_map(&settings.per_user_tun_interfaces),
            ),
            (
                "用户 DNS 端口".to_string(),
                format_u16_map(&settings.per_user_tun_dns_ports),
            ),
            ("GPU 模式".to_string(), settings.gpu_mode),
            ("iGPU 厂商".to_string(), settings.gpu_igpu_vendor),
            ("PRIME 模式".to_string(), settings.gpu_prime_mode),
            (
                "Intel Bus ID".to_string(),
                nonempty_opt_label(settings.gpu_intel_bus.as_deref()),
            ),
            (
                "AMD Bus ID".to_string(),
                nonempty_opt_label(settings.gpu_amd_bus.as_deref()),
            ),
            (
                "NVIDIA Bus ID".to_string(),
                nonempty_opt_label(settings.gpu_nvidia_bus.as_deref()),
            ),
            (
                "NVIDIA Open".to_string(),
                bool_label(settings.gpu_nvidia_open).to_string(),
            ),
            (
                "GPU 特化".to_string(),
                bool_label(settings.gpu_specialisations_enable).to_string(),
            ),
            (
                "特化模式".to_string(),
                format_string_list(&settings.gpu_specialisation_modes),
            ),
            (
                "Docker".to_string(),
                bool_label(settings.docker_enable).to_string(),
            ),
            (
                "Libvirtd".to_string(),
                bool_label(settings.libvirtd_enable).to_string(),
            ),
        ]
    }

    pub fn users_summary_lines(&self) -> Vec<String> {
        let mut lines = vec![
            format!("当前主机：{}", self.target_host),
            format!("目标文件：{}", display_path(self.current_host_users_path())),
            format!(
                "仓库内可选用户：{}",
                format_string_list(&self.context.users)
            ),
        ];

        if self.host_dirty_user_hosts.contains(&self.target_host) {
            lines.push("状态：当前主机的用户结构分片有未保存修改".to_string());
        } else {
            lines.push("状态：当前主机的用户结构分片没有未保存修改".to_string());
        }

        let errors = self.current_host_user_validation_errors();
        if errors.is_empty() {
            lines.push("校验：通过".to_string());
        } else {
            lines.push("校验：存在问题".to_string());
            for err in errors {
                lines.push(format!("- {err}"));
            }
        }

        lines.push(String::new());
        lines.push("当前页说明：".to_string());
        lines.push("- 这里只管理主机级 users.nix 分片".to_string());
        lines.push("- 不会创建新的 home/users/<name> 目录".to_string());
        lines.push("- 新用户模板生成仍应走 deploy / template 流程".to_string());
        lines
    }

    pub fn hosts_summary_lines(&self) -> Vec<String> {
        let mut lines = vec![format!("当前主机：{}", self.target_host)];
        let runtime_paths = self.current_host_runtime_paths();
        if runtime_paths.is_empty() {
            lines.push("目标文件：无".to_string());
        } else {
            lines.push("目标分片：".to_string());
            for path in runtime_paths {
                lines.push(format!("- {}", path.display()));
            }
        }

        if self.host_dirty_runtime_hosts.contains(&self.target_host) {
            lines.push("状态：当前主机的运行时分片有未保存修改".to_string());
        } else {
            lines.push("状态：当前主机的运行时分片没有未保存修改".to_string());
        }

        let errors = self.current_host_runtime_validation_errors();
        if errors.is_empty() {
            lines.push("校验：通过".to_string());
        } else {
            lines.push("校验：存在问题".to_string());
            for err in errors {
                lines.push(format!("- {err}"));
            }
        }

        lines.push(String::new());
        lines.push("当前页说明：".to_string());
        lines.push("- 这里只写 network.nix / gpu.nix / virtualization.nix".to_string());
        lines.push("- 不会直接改手写 hosts/<host>/default.nix".to_string());
        lines.push("- 文本字段用 Enter 编辑，枚举/布尔用 h/l 或 Space 调整".to_string());
        lines
    }

    pub fn next_users_field(&mut self) {
        self.users_focus = (self.users_focus + 1) % 6;
    }

    pub fn previous_users_field(&mut self) {
        self.users_focus = if self.users_focus == 0 {
            5
        } else {
            self.users_focus - 1
        };
    }

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

    pub fn switch_target_host(&mut self, delta: i8) {
        cycle_string(&mut self.target_host, &self.context.hosts, delta);
    }

    pub fn adjust_users_field(&mut self, delta: i8) {
        match self.users_focus {
            0 => self.switch_target_host(delta),
            1 => {
                let candidates = self
                    .current_host_settings()
                    .map(|settings| {
                        if settings.users.is_empty() {
                            self.context.users.clone()
                        } else {
                            settings.users.clone()
                        }
                    })
                    .unwrap_or_default();
                if candidates.is_empty() {
                    self.status = "当前没有可选用户。".to_string();
                    return;
                }
                let current = self
                    .current_host_settings()
                    .map(|settings| settings.primary_user.clone())
                    .unwrap_or_default();
                let Some(next) = cycle_string_value(&current, &candidates, delta) else {
                    return;
                };
                if let Some(settings) = self.current_host_settings_mut() {
                    settings.primary_user = next.clone();
                    if !settings.users.contains(&next) {
                        settings.users.insert(0, next.clone());
                    }
                }
                self.host_dirty_user_hosts.insert(self.target_host.clone());
                self.status = format!("当前主用户已切换为：{next}");
            }
            4 => {
                let options = vec!["desktop".to_string(), "server".to_string()];
                let current = self
                    .current_host_settings()
                    .map(|settings| settings.host_role.clone())
                    .unwrap_or_else(|| "desktop".to_string());
                let Some(next) = cycle_string_value(&current, &options, delta) else {
                    return;
                };
                if let Some(settings) = self.current_host_settings_mut() {
                    settings.host_role = next.clone();
                }
                self.host_dirty_user_hosts.insert(self.target_host.clone());
                self.status = format!("当前主机角色已切换为：{next}");
            }
            5 => {
                if let Some(settings) = self.current_host_settings_mut() {
                    settings.user_linger = !settings.user_linger;
                }
                self.host_dirty_user_hosts.insert(self.target_host.clone());
                self.status = "当前主机的 user linger 已切换。".to_string();
            }
            _ => {}
        }
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

    pub fn open_users_text_edit(&mut self) {
        let Some(settings) = self.current_host_settings().cloned() else {
            self.status = "当前主机没有可编辑的用户结构。".to_string();
            return;
        };

        match self.users_focus {
            2 => {
                self.users_text_mode = Some(UsersTextMode::ManagedUsers);
                self.host_text_input = serialize_string_list(&settings.users);
                self.status = "开始编辑托管用户列表；使用逗号分隔。".to_string();
            }
            3 => {
                self.users_text_mode = Some(UsersTextMode::AdminUsers);
                self.host_text_input = serialize_string_list(&settings.admin_users);
                self.status = "开始编辑管理员用户列表；使用逗号分隔。".to_string();
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

    pub fn handle_users_text_input(&mut self, code: crossterm::event::KeyCode) {
        match code {
            crossterm::event::KeyCode::Enter => self.confirm_users_text_edit(),
            crossterm::event::KeyCode::Esc => {
                self.users_text_mode = None;
                self.host_text_input.clear();
                self.status = "已取消用户结构编辑。".to_string();
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

    pub fn save_current_host_users(&mut self) -> Result<()> {
        let errors = self.current_host_user_validation_errors();
        if !errors.is_empty() {
            self.status = format!("当前主机的 users 分片未通过校验：{}", errors.join("；"));
            return Ok(());
        }

        let host = self.target_host.clone();
        let Some(settings) = self.current_host_settings().cloned() else {
            self.status = "没有可保存的主机用户结构。".to_string();
            return Ok(());
        };

        let host_dir = self.context.repo_root.join("hosts").join(&host);
        let managed_dir = host_dir.join("managed");
        ensure_managed_host_layout(&managed_dir)?;
        let users_path = write_host_users_fragment(&managed_dir, &settings)?;
        self.host_dirty_user_hosts.remove(&host);
        self.status = format!("已写入 {}", users_path.display());
        Ok(())
    }

    pub fn save_current_host_runtime(&mut self) -> Result<()> {
        let errors = self.current_host_runtime_validation_errors();
        if !errors.is_empty() {
            self.status = format!("当前主机的运行时分片未通过校验：{}", errors.join("；"));
            return Ok(());
        }

        let host = self.target_host.clone();
        let Some(settings) = self.current_host_settings().cloned() else {
            self.status = "没有可保存的主机运行时配置。".to_string();
            return Ok(());
        };

        let host_dir = self.context.repo_root.join("hosts").join(&host);
        let managed_dir = host_dir.join("managed");
        ensure_managed_host_layout(&managed_dir)?;
        let paths = write_host_runtime_fragments(&managed_dir, &settings)?;
        self.host_dirty_runtime_hosts.remove(&host);
        self.status = format!(
            "已写入 {}",
            paths
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join("、")
        );
        Ok(())
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

    fn confirm_users_text_edit(&mut self) {
        let Some(mode) = self.users_text_mode else {
            return;
        };
        let parsed = parse_string_list(&self.host_text_input);
        let Some(settings) = self.current_host_settings_mut() else {
            self.users_text_mode = None;
            self.host_text_input.clear();
            self.status = "当前主机没有可编辑的用户结构。".to_string();
            return;
        };

        match mode {
            UsersTextMode::ManagedUsers => {
                settings.users = parsed;
                if !settings.users.contains(&settings.primary_user)
                    && let Some(first) = settings.users.first()
                {
                    settings.primary_user = first.clone();
                }
                settings
                    .admin_users
                    .retain(|user| settings.users.contains(user));
            }
            UsersTextMode::AdminUsers => {
                settings.admin_users = parsed
                    .into_iter()
                    .filter(|user| settings.users.contains(user))
                    .collect();
            }
        }

        self.host_dirty_user_hosts.insert(self.target_host.clone());
        self.users_text_mode = None;
        self.host_text_input.clear();
        self.status = "用户结构字段已更新。".to_string();
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

    fn current_host_user_validation_errors(&self) -> Vec<String> {
        let Some(settings) = self.current_host_settings() else {
            return vec!["当前主机没有可用设置。".to_string()];
        };

        let mut errors = Vec::new();
        if settings.users.is_empty() {
            errors.push("托管用户列表不能为空。".to_string());
        }
        if settings.primary_user.trim().is_empty() {
            errors.push("主用户不能为空。".to_string());
        } else if !settings.users.contains(&settings.primary_user) {
            errors.push("主用户必须包含在托管用户列表中。".to_string());
        }
        if has_duplicates(&settings.users) {
            errors.push("托管用户列表不能包含重复项。".to_string());
        }
        if has_duplicates(&settings.admin_users) {
            errors.push("管理员列表不能包含重复项。".to_string());
        }
        if settings
            .admin_users
            .iter()
            .any(|user| !settings.users.contains(user))
        {
            errors.push("管理员列表必须是托管用户列表的子集。".to_string());
        }
        errors
    }

    fn current_host_runtime_validation_errors(&self) -> Vec<String> {
        let Some(settings) = self.current_host_settings() else {
            return vec!["当前主机没有可用设置。".to_string()];
        };

        let mut errors = Vec::new();
        if settings.proxy_mode == "http" && settings.proxy_url.trim().is_empty() {
            errors.push("proxyMode = http 时，代理 URL 不能为空。".to_string());
        }
        if settings.proxy_mode == "tun"
            && !settings.per_user_tun_enable
            && settings.tun_interface.trim().is_empty()
        {
            errors.push(
                "proxyMode = tun 且未开启 per-user TUN 时，主 TUN 接口不能为空。".to_string(),
            );
        }
        if settings.per_user_tun_enable && settings.proxy_mode != "tun" {
            errors.push("启用 per-user TUN 时，proxyMode 必须为 tun。".to_string());
        }
        if settings.per_user_tun_enable {
            for user in &settings.users {
                if !settings.per_user_tun_interfaces.contains_key(user) {
                    errors.push(format!("per-user TUN 接口映射缺少用户：{user}"));
                }
            }
        }
        if settings.gpu_mode == "hybrid" {
            let has_igpu = if settings.gpu_igpu_vendor == "amd" {
                settings
                    .gpu_amd_bus
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty())
            } else {
                settings
                    .gpu_intel_bus
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty())
            };
            let has_nvidia = settings
                .gpu_nvidia_bus
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty());
            if !has_igpu || !has_nvidia {
                errors
                    .push("GPU hybrid 模式要求设置 nvidiaBusId 和匹配的 iGPU Bus ID。".to_string());
            }
        }
        if settings.gpu_specialisations_enable
            && settings
                .gpu_specialisation_modes
                .iter()
                .any(|mode| mode == "hybrid")
        {
            let has_igpu = if settings.gpu_igpu_vendor == "amd" {
                settings
                    .gpu_amd_bus
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty())
            } else {
                settings
                    .gpu_intel_bus
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty())
            };
            let has_nvidia = settings
                .gpu_nvidia_bus
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty());
            if !has_igpu || !has_nvidia {
                errors.push("GPU 特化包含 hybrid 时，需要配置完整的 PRIME Bus ID。".to_string());
            }
        }

        errors
    }
}
