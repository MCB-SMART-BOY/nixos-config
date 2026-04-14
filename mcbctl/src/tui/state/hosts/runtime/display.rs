use super::*;

impl AppState {
    pub fn hosts_rows(&self) -> Vec<(String, String)> {
        let Some(settings) = self.current_host_settings().cloned() else {
            let unavailable = self
                .current_host_unavailable_value()
                .unwrap_or_else(|| "不可用".to_string());
            return vec![
                ("主机".to_string(), self.target_host.clone()),
                ("缓存策略".to_string(), unavailable.clone()),
                ("自定义 substituters".to_string(), unavailable.clone()),
                ("自定义 trusted keys".to_string(), unavailable.clone()),
                ("代理模式".to_string(), unavailable.clone()),
                ("代理 URL".to_string(), unavailable.clone()),
                ("主 TUN 接口".to_string(), unavailable.clone()),
                ("额外 TUN 接口".to_string(), unavailable.clone()),
                ("全局代理 DNS".to_string(), unavailable.clone()),
                ("代理 DNS 地址".to_string(), unavailable.clone()),
                ("代理 DNS 端口".to_string(), unavailable.clone()),
                ("Per-user TUN".to_string(), unavailable.clone()),
                ("兼容全局服务 Socket".to_string(), unavailable.clone()),
                ("Per-user DNS 重定向".to_string(), unavailable.clone()),
                ("用户接口映射".to_string(), unavailable.clone()),
                ("用户 DNS 端口".to_string(), unavailable.clone()),
                ("路由表基值".to_string(), unavailable.clone()),
                ("规则优先级基值".to_string(), unavailable.clone()),
                ("GPU 模式".to_string(), unavailable.clone()),
                ("iGPU 厂商".to_string(), unavailable.clone()),
                ("PRIME 模式".to_string(), unavailable.clone()),
                ("Intel Bus ID".to_string(), unavailable.clone()),
                ("AMD Bus ID".to_string(), unavailable.clone()),
                ("NVIDIA Bus ID".to_string(), unavailable.clone()),
                ("NVIDIA Open".to_string(), unavailable.clone()),
                ("GPU 特化".to_string(), unavailable.clone()),
                ("特化模式".to_string(), unavailable.clone()),
                ("Docker".to_string(), unavailable.clone()),
                ("Libvirtd".to_string(), unavailable),
            ];
        };
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

        if let Some(message) = self.current_host_settings_unavailable_message() {
            lines.push(format!("状态：{message}"));
        } else {
            if self.host_dirty_runtime_hosts.contains(&self.target_host) {
                lines.push("状态：当前主机的运行时分片有未保存修改".to_string());
            } else {
                lines.push("状态：当前主机的运行时分片没有未保存修改".to_string());
            }
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
        let guard_errors = self.current_host_managed_guard_errors();
        if guard_errors.is_empty() {
            lines.push("受管保护：通过".to_string());
        } else {
            lines.push("受管保护：存在问题".to_string());
            for err in guard_errors {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn hosts_summary_lines_surface_managed_guard_errors_before_save() -> Result<()> {
        let root = create_temp_repo("mcbctl-host-runtime-summary")?;
        let managed_dir = root.join("hosts/demo/managed");
        std::fs::create_dir_all(&managed_dir)?;
        std::fs::write(
            managed_dir.join("users.nix"),
            "{ lib, ... }: { mcb.user = lib.mkForce \"alice\"; }\n",
        )?;
        let state = test_state(&root);

        let lines = state.hosts_summary_lines();

        assert!(lines.iter().any(|line| line == "受管保护：存在问题"));
        assert!(lines.iter().any(|line| line.contains("host-users")));

        std::fs::remove_dir_all(root)?;
        Ok(())
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
                users: vec!["alice".to_string()],
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
            feedback: UiFeedback::default(),
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

    fn create_temp_repo(prefix: &str) -> Result<PathBuf> {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let root = std::env::temp_dir().join(format!("{prefix}-{}-{unique}", std::process::id()));
        std::fs::create_dir_all(&root)?;
        Ok(root)
    }
}
