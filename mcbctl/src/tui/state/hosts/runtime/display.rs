use super::*;

impl AppState {
    pub(crate) fn hosts_page_model(&self) -> EditPageModel {
        EditPageModel {
            rows: self.hosts_rows(),
            selected: self.hosts_focus,
            summary: self.hosts_summary_model(),
        }
    }

    pub(crate) fn hosts_rows(&self) -> Vec<EditRow> {
        let Some(settings) = self.current_host_settings().cloned() else {
            let unavailable = self
                .current_host_unavailable_value()
                .unwrap_or_else(|| "不可用".to_string());
            return vec![
                EditRow {
                    label: "主机".to_string(),
                    value: self.target_host.clone(),
                },
                EditRow {
                    label: "缓存策略".to_string(),
                    value: unavailable.clone(),
                },
                EditRow {
                    label: "自定义 substituters".to_string(),
                    value: unavailable.clone(),
                },
                EditRow {
                    label: "自定义 trusted keys".to_string(),
                    value: unavailable.clone(),
                },
                EditRow {
                    label: "代理模式".to_string(),
                    value: unavailable.clone(),
                },
                EditRow {
                    label: "代理 URL".to_string(),
                    value: unavailable.clone(),
                },
                EditRow {
                    label: "主 TUN 接口".to_string(),
                    value: unavailable.clone(),
                },
                EditRow {
                    label: "额外 TUN 接口".to_string(),
                    value: unavailable.clone(),
                },
                EditRow {
                    label: "全局代理 DNS".to_string(),
                    value: unavailable.clone(),
                },
                EditRow {
                    label: "代理 DNS 地址".to_string(),
                    value: unavailable.clone(),
                },
                EditRow {
                    label: "代理 DNS 端口".to_string(),
                    value: unavailable.clone(),
                },
                EditRow {
                    label: "Per-user TUN".to_string(),
                    value: unavailable.clone(),
                },
                EditRow {
                    label: "兼容全局服务 Socket".to_string(),
                    value: unavailable.clone(),
                },
                EditRow {
                    label: "Per-user DNS 重定向".to_string(),
                    value: unavailable.clone(),
                },
                EditRow {
                    label: "用户接口映射".to_string(),
                    value: unavailable.clone(),
                },
                EditRow {
                    label: "用户 DNS 端口".to_string(),
                    value: unavailable.clone(),
                },
                EditRow {
                    label: "路由表基值".to_string(),
                    value: unavailable.clone(),
                },
                EditRow {
                    label: "规则优先级基值".to_string(),
                    value: unavailable.clone(),
                },
                EditRow {
                    label: "GPU 模式".to_string(),
                    value: unavailable.clone(),
                },
                EditRow {
                    label: "iGPU 厂商".to_string(),
                    value: unavailable.clone(),
                },
                EditRow {
                    label: "PRIME 模式".to_string(),
                    value: unavailable.clone(),
                },
                EditRow {
                    label: "Intel Bus ID".to_string(),
                    value: unavailable.clone(),
                },
                EditRow {
                    label: "AMD Bus ID".to_string(),
                    value: unavailable.clone(),
                },
                EditRow {
                    label: "NVIDIA Bus ID".to_string(),
                    value: unavailable.clone(),
                },
                EditRow {
                    label: "NVIDIA Open".to_string(),
                    value: unavailable.clone(),
                },
                EditRow {
                    label: "GPU 特化".to_string(),
                    value: unavailable.clone(),
                },
                EditRow {
                    label: "特化模式".to_string(),
                    value: unavailable.clone(),
                },
                EditRow {
                    label: "Docker".to_string(),
                    value: unavailable.clone(),
                },
                EditRow {
                    label: "Libvirtd".to_string(),
                    value: unavailable,
                },
            ];
        };
        vec![
            EditRow {
                label: "主机".to_string(),
                value: self.target_host.clone(),
            },
            EditRow {
                label: "缓存策略".to_string(),
                value: settings.cache_profile,
            },
            EditRow {
                label: "自定义 substituters".to_string(),
                value: format_string_list(&settings.custom_substituters),
            },
            EditRow {
                label: "自定义 trusted keys".to_string(),
                value: format_string_list(&settings.custom_trusted_public_keys),
            },
            EditRow {
                label: "代理模式".to_string(),
                value: settings.proxy_mode,
            },
            EditRow {
                label: "代理 URL".to_string(),
                value: nonempty_label(&settings.proxy_url),
            },
            EditRow {
                label: "主 TUN 接口".to_string(),
                value: nonempty_label(&settings.tun_interface),
            },
            EditRow {
                label: "额外 TUN 接口".to_string(),
                value: format_string_list(&settings.tun_interfaces),
            },
            EditRow {
                label: "全局代理 DNS".to_string(),
                value: bool_label(settings.enable_proxy_dns).to_string(),
            },
            EditRow {
                label: "代理 DNS 地址".to_string(),
                value: nonempty_label(&settings.proxy_dns_addr),
            },
            EditRow {
                label: "代理 DNS 端口".to_string(),
                value: settings.proxy_dns_port.to_string(),
            },
            EditRow {
                label: "Per-user TUN".to_string(),
                value: bool_label(settings.per_user_tun_enable).to_string(),
            },
            EditRow {
                label: "兼容全局服务 Socket".to_string(),
                value: bool_label(settings.per_user_tun_compat_global_service_socket).to_string(),
            },
            EditRow {
                label: "Per-user DNS 重定向".to_string(),
                value: bool_label(settings.per_user_tun_redirect_dns).to_string(),
            },
            EditRow {
                label: "用户接口映射".to_string(),
                value: format_string_map(&settings.per_user_tun_interfaces),
            },
            EditRow {
                label: "用户 DNS 端口".to_string(),
                value: format_u16_map(&settings.per_user_tun_dns_ports),
            },
            EditRow {
                label: "路由表基值".to_string(),
                value: settings.per_user_tun_table_base.to_string(),
            },
            EditRow {
                label: "规则优先级基值".to_string(),
                value: settings.per_user_tun_priority_base.to_string(),
            },
            EditRow {
                label: "GPU 模式".to_string(),
                value: settings.gpu_mode,
            },
            EditRow {
                label: "iGPU 厂商".to_string(),
                value: settings.gpu_igpu_vendor,
            },
            EditRow {
                label: "PRIME 模式".to_string(),
                value: settings.gpu_prime_mode,
            },
            EditRow {
                label: "Intel Bus ID".to_string(),
                value: nonempty_opt_label(settings.gpu_intel_bus.as_deref()),
            },
            EditRow {
                label: "AMD Bus ID".to_string(),
                value: nonempty_opt_label(settings.gpu_amd_bus.as_deref()),
            },
            EditRow {
                label: "NVIDIA Bus ID".to_string(),
                value: nonempty_opt_label(settings.gpu_nvidia_bus.as_deref()),
            },
            EditRow {
                label: "NVIDIA Open".to_string(),
                value: bool_label(settings.gpu_nvidia_open).to_string(),
            },
            EditRow {
                label: "GPU 特化".to_string(),
                value: bool_label(settings.gpu_specialisations_enable).to_string(),
            },
            EditRow {
                label: "特化模式".to_string(),
                value: format_string_list(&settings.gpu_specialisation_modes),
            },
            EditRow {
                label: "Docker".to_string(),
                value: bool_label(settings.docker_enable).to_string(),
            },
            EditRow {
                label: "Libvirtd".to_string(),
                value: bool_label(settings.libvirtd_enable).to_string(),
            },
        ]
    }

    fn current_hosts_row(&self) -> Option<EditRow> {
        self.hosts_rows().get(self.hosts_focus).cloned()
    }

    pub(crate) fn hosts_summary_model(&self) -> EditSummaryModel {
        let mut header_lines = vec![format!("当前主机：{}", self.target_host)];
        let runtime_paths = self.current_host_runtime_paths();
        if runtime_paths.is_empty() {
            header_lines.push("目标文件：无".to_string());
        } else {
            header_lines.push("目标分片：".to_string());
            for path in runtime_paths {
                header_lines.push(format!("- {}", path.display()));
            }
        }
        let focused_row = self.current_hosts_row();
        let status = if let Some(message) = self.current_host_settings_unavailable_message() {
            format!("状态：{message}")
        } else {
            if self.host_dirty_runtime_hosts.contains(&self.target_host) {
                "状态：当前主机的运行时分片有未保存修改".to_string()
            } else {
                "状态：当前主机的运行时分片没有未保存修改".to_string()
            }
        };
        let action_summary = self.edit_action_summary(
            UiFeedbackScope::Hosts,
            "复查 Hosts Summary，确认后按 s 保存。",
        );

        let errors = self.current_host_runtime_validation_errors();
        let validation = if errors.is_empty() {
            EditCheckModel {
                summary: "校验：通过".to_string(),
                details: Vec::new(),
            }
        } else {
            EditCheckModel {
                summary: "校验：存在问题".to_string(),
                details: errors.into_iter().map(|err| format!("- {err}")).collect(),
            }
        };
        let guard_errors = self.current_host_managed_guard_errors();
        let managed_guard = if guard_errors.is_empty() {
            EditCheckModel {
                summary: "受管保护：通过".to_string(),
                details: Vec::new(),
            }
        } else {
            EditCheckModel {
                summary: "受管保护：存在问题".to_string(),
                details: guard_errors
                    .into_iter()
                    .map(|err| format!("- {err}"))
                    .collect(),
            }
        };

        let notes = vec![
            String::new(),
            "当前页说明：".to_string(),
            "- 这里只写 network.nix / gpu.nix / virtualization.nix".to_string(),
            "- 不会直接改手写 hosts/<host>/default.nix".to_string(),
            "- 文本字段用 Enter 编辑，枚举/布尔用 h/l 或 Space 调整".to_string(),
            "- 这里的校验会尽量对齐 modules/networking.nix 和 modules/hardware/gpu.nix 的关键断言"
                .to_string(),
        ];
        EditSummaryModel {
            header_lines,
            focused_row,
            field_lines: Vec::new(),
            detail: EditDetailModel {
                status,
                action_summary,
                validation: Some(validation),
                managed_guard,
                notes,
            },
        }
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

        let lines = state.hosts_summary_model().lines();

        assert!(lines.iter().any(|line| line == "受管保护：存在问题"));
        assert!(lines.iter().any(|line| line.contains("host-users")));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn hosts_page_model_assembles_rows_selection_and_summary() -> Result<()> {
        let root = create_temp_repo("mcbctl-host-runtime-page-model")?;
        let mut state = test_state(&root);
        state.hosts_focus = 18;

        let model = state.hosts_page_model();

        assert_eq!(model.selected, 18);
        assert_eq!(model.rows.len(), state.hosts_rows().len());
        assert_eq!(model.summary.focused_row, model.rows.get(18).cloned());

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn hosts_summary_lines_surface_current_focus_row() -> Result<()> {
        let root = create_temp_repo("mcbctl-host-runtime-focus")?;
        let mut state = test_state(&root);
        state.hosts_focus = 18;

        let model = state.hosts_summary_model();

        assert_eq!(
            model.focused_row,
            Some(EditRow {
                label: "GPU 模式".to_string(),
                value: "igpu".to_string(),
            })
        );

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
                catalog_workflows_path: root.join("catalog/workflows.toml"),
                catalog_entries: Vec::new(),
                catalog_groups: BTreeMap::new(),
                catalog_home_options: Vec::new(),
                catalog_workflows: BTreeMap::new(),
                catalog_categories: Vec::new(),
                catalog_sources: Vec::new(),
            },
            active_page: 0,
            active_edit_page: 0,
            deploy_focus: 0,
            advanced_deploy_focus: 0,
            target_host: "demo".to_string(),
            deploy_task: DeployTask::DirectDeploy,
            deploy_source: DeploySource::CurrentRepo,
            deploy_source_ref: String::new(),
            deploy_action: DeployAction::Switch,
            flake_update: false,
            advanced_target_host: "demo".to_string(),
            advanced_deploy_task: DeployTask::DirectDeploy,
            advanced_deploy_source: DeploySource::CurrentRepo,
            advanced_deploy_source_ref: String::new(),
            advanced_deploy_action: DeployAction::Switch,
            advanced_flake_update: false,
            help_overlay_visible: false,
            deploy_text_mode: None,
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
            package_workflow_filter: None,
            package_search: String::new(),
            package_search_result_indices: Vec::new(),
            package_local_entry_ids: BTreeSet::new(),
            package_search_mode: false,
            package_group_create_mode: false,
            package_group_rename_mode: false,
            package_workflow_add_confirm_mode: false,
            package_group_rename_source: String::new(),
            package_group_input: String::new(),
            package_user_selections: BTreeMap::new(),
            package_dirty_users: BTreeSet::new(),
            home_user_index: 0,
            home_focus: 0,
            home_settings_by_user: BTreeMap::new(),
            home_dirty_users: BTreeSet::new(),
            inspect_action: crate::domain::tui::ActionItem::FlakeCheck,
            advanced_action: crate::domain::tui::ActionItem::FlakeUpdate,
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
