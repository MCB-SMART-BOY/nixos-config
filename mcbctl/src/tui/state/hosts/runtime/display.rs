use super::*;

impl AppState {
    pub fn hosts_rows(&self) -> Vec<(String, String)> {
        let settings = self.current_host_settings().cloned().unwrap_or_default();
        vec![
            ("主机".to_string(), self.target_host.clone()),
            ("缓存策略".to_string(), settings.cache_profile),
            (
                "自定义 substituters".to_string(),
                format_string_list(&settings.custom_substituters),
            ),
            (
                "自定义 trusted keys".to_string(),
                format_string_list(&settings.custom_trusted_public_keys),
            ),
            ("代理模式".to_string(), settings.proxy_mode),
            ("代理 URL".to_string(), nonempty_label(&settings.proxy_url)),
            (
                "主 TUN 接口".to_string(),
                nonempty_label(&settings.tun_interface),
            ),
            (
                "额外 TUN 接口".to_string(),
                format_string_list(&settings.tun_interfaces),
            ),
            (
                "全局代理 DNS".to_string(),
                bool_label(settings.enable_proxy_dns).to_string(),
            ),
            (
                "代理 DNS 地址".to_string(),
                nonempty_label(&settings.proxy_dns_addr),
            ),
            (
                "代理 DNS 端口".to_string(),
                settings.proxy_dns_port.to_string(),
            ),
            (
                "Per-user TUN".to_string(),
                bool_label(settings.per_user_tun_enable).to_string(),
            ),
            (
                "兼容全局服务 Socket".to_string(),
                bool_label(settings.per_user_tun_compat_global_service_socket).to_string(),
            ),
            (
                "Per-user DNS 重定向".to_string(),
                bool_label(settings.per_user_tun_redirect_dns).to_string(),
            ),
            (
                "用户接口映射".to_string(),
                format_string_map(&settings.per_user_tun_interfaces),
            ),
            (
                "用户 DNS 端口".to_string(),
                format_u16_map(&settings.per_user_tun_dns_ports),
            ),
            (
                "路由表基值".to_string(),
                settings.per_user_tun_table_base.to_string(),
            ),
            (
                "规则优先级基值".to_string(),
                settings.per_user_tun_priority_base.to_string(),
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
        lines.push(
            "- 这里的校验会尽量对齐 modules/networking.nix 和 modules/hardware/gpu.nix 的关键断言"
                .to_string(),
        );
        lines
    }
}
