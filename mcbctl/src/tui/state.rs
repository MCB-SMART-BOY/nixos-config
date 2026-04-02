use crate::domain::deploy::DeployPlan;
use crate::domain::tui::{
    ActionItem, CatalogEntry, DeployAction, DeploySource, DeployTask, GroupMeta,
    HomeManagedSettings, HomeOptionMeta, HostManagedSettings, HostsTextMode, ManagedBarProfile,
    ManagedToggle, PackageDataMode, PackageTextMode, Page, UsersTextMode,
};
use crate::store::catalog::{load_catalog, load_group_catalog, load_home_options_catalog};
use crate::store::context::{
    detect_hostname, detect_nix_system, detect_privilege_mode, detect_repo_root, list_hosts,
    list_users,
};
use crate::store::deploy::{
    NixosRebuildPlan, RepoSyncPlan, ensure_root_hardware_config, merged_nix_config,
    run_nixos_rebuild, run_repo_sync, run_root_command_ok,
};
use crate::store::home::{
    ensure_managed_settings_layout, load_home_user_settings, managed_home_desktop_path,
    render_managed_desktop_file,
};
use crate::store::hosts::{
    ensure_managed_host_layout, load_host_settings, managed_host_gpu_path,
    managed_host_network_path, managed_host_users_path, managed_host_virtualization_path,
    write_host_runtime_fragments, write_host_users_fragment,
};
use crate::store::packages::{
    ensure_managed_packages_layout, load_managed_package_entries, load_package_user_selections,
    managed_package_group_path, write_grouped_managed_packages,
};
use crate::store::search::search_catalog_entries;
use crate::{resolve_sibling_binary, write_file_atomic};
use anyhow::{Context, Result};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct AppContext {
    pub repo_root: PathBuf,
    pub etc_root: PathBuf,
    pub current_host: String,
    pub current_system: String,
    pub current_user: String,
    pub privilege_mode: String,
    pub hosts: Vec<String>,
    pub users: Vec<String>,
    pub catalog_path: PathBuf,
    pub catalog_groups_path: PathBuf,
    pub catalog_home_options_path: PathBuf,
    pub catalog_entries: Vec<CatalogEntry>,
    pub catalog_groups: BTreeMap<String, GroupMeta>,
    pub catalog_home_options: Vec<HomeOptionMeta>,
    pub catalog_categories: Vec<String>,
    pub catalog_sources: Vec<String>,
}

impl AppContext {
    pub fn detect() -> Result<Self> {
        let repo_root = detect_repo_root().context("failed to detect repo root")?;
        let etc_root = PathBuf::from("/etc/nixos");
        let current_host = detect_hostname();
        let current_system = detect_nix_system();
        let current_user = env::var("USER").unwrap_or_else(|_| "unknown".to_string());
        let privilege_mode = detect_privilege_mode();
        let hosts = list_hosts(&repo_root);
        let users = list_users(&repo_root);

        let catalog_path = repo_root.join("catalog/packages.toml");
        let catalog_groups_path = repo_root.join("catalog/groups.toml");
        let catalog_home_options_path = repo_root.join("catalog/home-options.toml");
        let (catalog_entries, catalog_categories, catalog_sources) = load_catalog(&catalog_path);
        let catalog_groups = load_group_catalog(&catalog_groups_path);
        let catalog_home_options = load_home_options_catalog(&catalog_home_options_path);

        Ok(Self {
            repo_root,
            etc_root,
            current_host,
            current_system,
            current_user,
            privilege_mode,
            hosts,
            users,
            catalog_path,
            catalog_groups_path,
            catalog_home_options_path,
            catalog_entries,
            catalog_groups,
            catalog_home_options,
            catalog_categories,
            catalog_sources,
        })
    }
}

#[derive(Debug)]
pub struct AppState {
    pub context: AppContext,
    pub active_page: usize,
    pub deploy_focus: usize,
    pub target_host: String,
    pub deploy_task: DeployTask,
    pub deploy_source: DeploySource,
    pub deploy_action: DeployAction,
    pub flake_update: bool,
    pub show_advanced: bool,
    pub users_focus: usize,
    pub hosts_focus: usize,
    pub users_text_mode: Option<UsersTextMode>,
    pub hosts_text_mode: Option<HostsTextMode>,
    pub host_text_input: String,
    pub host_settings_by_name: BTreeMap<String, HostManagedSettings>,
    pub host_dirty_user_hosts: BTreeSet<String>,
    pub host_dirty_runtime_hosts: BTreeSet<String>,
    pub package_user_index: usize,
    pub package_mode: PackageDataMode,
    pub package_cursor: usize,
    pub package_category_index: usize,
    pub package_group_filter: Option<String>,
    pub package_source_filter: Option<String>,
    pub package_search: String,
    pub package_search_result_indices: Vec<usize>,
    pub package_local_entry_ids: BTreeSet<String>,
    pub package_search_mode: bool,
    pub package_group_create_mode: bool,
    pub package_group_rename_mode: bool,
    pub package_group_rename_source: String,
    pub package_group_input: String,
    pub package_user_selections: BTreeMap<String, BTreeMap<String, String>>,
    pub package_dirty_users: BTreeSet<String>,
    pub home_user_index: usize,
    pub home_focus: usize,
    pub home_settings_by_user: BTreeMap<String, HomeManagedSettings>,
    pub home_dirty_users: BTreeSet<String>,
    pub actions_focus: usize,
    pub status: String,
}

impl AppState {
    pub fn new(mut context: AppContext) -> Self {
        let deploy_source = if context.repo_root == context.etc_root {
            DeploySource::EtcNixos
        } else {
            DeploySource::CurrentRepo
        };
        let deploy_action = if context.privilege_mode == "rootless" {
            DeployAction::Build
        } else {
            DeployAction::Switch
        };
        let target_host = default_target_host(&context);
        let mut package_local_entry_ids = context
            .catalog_entries
            .iter()
            .filter(|entry| is_local_overlay_entry(entry))
            .map(|entry| entry.id.clone())
            .collect::<BTreeSet<_>>();
        let managed_entries = load_managed_package_entries(
            &context.repo_root,
            &context.users,
            &context.catalog_entries,
        );
        for entry in managed_entries {
            if !context
                .catalog_entries
                .iter()
                .any(|existing| existing.id == entry.id)
            {
                package_local_entry_ids.insert(entry.id.clone());
                context.catalog_entries.push(entry);
            }
        }
        refresh_local_catalog_indexes(&mut context, &package_local_entry_ids);
        let host_settings_by_name = load_host_settings(&context.repo_root, &context.hosts);
        let package_user_index = default_package_user_index(&context);
        let package_user_selections = load_package_user_selections(
            &context.repo_root,
            &context.users,
            &context.catalog_entries,
        );
        let home_user_index = default_package_user_index(&context);
        let home_settings_by_user = load_home_user_settings(&context.repo_root, &context.users);

        Self {
            context,
            active_page: 0,
            deploy_focus: 0,
            target_host,
            deploy_task: DeployTask::DirectDeploy,
            deploy_source,
            deploy_action,
            flake_update: false,
            show_advanced: false,
            users_focus: 0,
            hosts_focus: 0,
            users_text_mode: None,
            hosts_text_mode: None,
            host_text_input: String::new(),
            host_settings_by_name,
            host_dirty_user_hosts: BTreeSet::new(),
            host_dirty_runtime_hosts: BTreeSet::new(),
            package_user_index,
            package_mode: PackageDataMode::Search,
            package_cursor: 0,
            package_category_index: 0,
            package_group_filter: None,
            package_source_filter: None,
            package_search: String::new(),
            package_search_result_indices: Vec::new(),
            package_local_entry_ids,
            package_search_mode: false,
            package_group_create_mode: false,
            package_group_rename_mode: false,
            package_group_rename_source: String::new(),
            package_group_input: String::new(),
            package_user_selections,
            package_dirty_users: BTreeSet::new(),
            home_user_index,
            home_focus: 0,
            home_settings_by_user,
            home_dirty_users: BTreeSet::new(),
            actions_focus: 0,
            status: "Packages 现在默认使用 nixpkgs 搜索；本地覆盖与已声明软件可按 f 切回查看。"
                .to_string(),
        }
    }

    pub fn page(&self) -> Page {
        Page::ALL[self.active_page]
    }

    pub fn next_page(&mut self) {
        self.active_page = (self.active_page + 1) % Page::ALL.len();
    }

    pub fn previous_page(&mut self) {
        self.active_page = if self.active_page == 0 {
            Page::ALL.len() - 1
        } else {
            self.active_page - 1
        };
    }

    pub fn next_deploy_field(&mut self) {
        self.deploy_focus = (self.deploy_focus + 1) % 6;
    }

    pub fn previous_deploy_field(&mut self) {
        self.deploy_focus = if self.deploy_focus == 0 {
            5
        } else {
            self.deploy_focus - 1
        };
    }

    pub fn adjust_deploy_field(&mut self, delta: i8) {
        match self.deploy_focus {
            0 => cycle_string(&mut self.target_host, &self.context.hosts, delta),
            1 => cycle_enum(&mut self.deploy_task, &DeployTask::ALL, delta),
            2 => cycle_enum(&mut self.deploy_source, &DeploySource::ALL, delta),
            3 => cycle_enum(&mut self.deploy_action, &DeployAction::ALL, delta),
            4 => self.flake_update = !self.flake_update,
            5 => self.show_advanced = !self.show_advanced,
            _ => {}
        }
    }

    pub fn deploy_rows(&self) -> Vec<(String, String)> {
        vec![
            ("目标主机".to_string(), self.target_host.clone()),
            ("任务".to_string(), self.deploy_task.label().to_string()),
            ("来源".to_string(), self.deploy_source.label().to_string()),
            ("动作".to_string(), self.deploy_action.label().to_string()),
            (
                "flake update".to_string(),
                bool_label(self.flake_update).to_string(),
            ),
            (
                "高级选项".to_string(),
                bool_label(self.show_advanced).to_string(),
            ),
        ]
    }

    pub fn deploy_summary(&self) -> Vec<String> {
        self.deploy_plan().summary_lines()
    }

    pub fn can_execute_deploy_directly(&self) -> bool {
        !matches!(
            self.deploy_source,
            DeploySource::RemotePinned | DeploySource::RemoteHead
        ) && !self.show_advanced
    }

    pub fn execute_deploy(&mut self) -> Result<()> {
        self.ensure_no_unsaved_changes_for_execution()?;

        if !self.can_execute_deploy_directly() {
            let mut args = Vec::new();
            if matches!(self.deploy_task, DeployTask::Maintenance) {
                args.push("--mode".to_string());
                args.push("update-existing".to_string());
            }
            let status = self.run_sibling_in_repo("mcb-deploy", &args)?;
            if status.success() {
                self.status = "已返回完整部署向导。".to_string();
                return Ok(());
            }
            anyhow::bail!("mcb-deploy exited with {}", status.code().unwrap_or(1));
        }

        if self.context.privilege_mode == "rootless" && self.deploy_action != DeployAction::Build {
            anyhow::bail!("rootless 模式下当前页只能直接执行 build；如需 switch/test/boot，请使用 sudo/root 或退回 deploy wizard。");
        }

        let use_sudo = self.should_use_sudo();
        let needs_root_hw = !(self.context.privilege_mode == "rootless"
            && self.deploy_action == DeployAction::Build);
        if needs_root_hw {
            ensure_root_hardware_config(&self.context.etc_root, use_sudo)?;
        }

        let sync_plan = self.deploy_sync_plan_for_execution();
        let rebuild_plan = self
            .deploy_rebuild_plan_for_execution()
            .context("当前 Deploy 组合还没有可执行的重建计划")?;

        if let Some(plan) = sync_plan {
            run_repo_sync(
                &plan,
                |cmd, args| {
                    let status = std::process::Command::new(cmd)
                        .args(args)
                        .stdin(std::process::Stdio::inherit())
                        .stdout(std::process::Stdio::inherit())
                        .stderr(std::process::Stdio::inherit())
                        .status()
                        .with_context(|| format!("failed to run {cmd}"))?;
                    if status.success() {
                        Ok(())
                    } else {
                        anyhow::bail!("{cmd} failed with {}", status.code().unwrap_or(1));
                    }
                },
                |cmd, args| run_root_command_ok(cmd, args, use_sudo),
                || self.clean_etc_dir_keep_hardware(),
            )?;
        }

        let status = run_nixos_rebuild(&rebuild_plan, use_sudo)?;
        if !status.success() {
            anyhow::bail!("nixos-rebuild exited with {}", status.code().unwrap_or(1));
        }

        self.status = format!(
            "Deploy 已执行完成：{} {}",
            rebuild_plan.action.label(),
            rebuild_plan.target_host
        );
        Ok(())
    }

    pub fn current_host_settings(&self) -> Option<&HostManagedSettings> {
        self.host_settings_by_name.get(&self.target_host)
    }

    fn deploy_plan(&self) -> DeployPlan {
        let mut notes = vec![format!("flake update：{}", bool_label(self.flake_update))];
        if let Some(sync_plan) = self.deploy_sync_plan_for_execution() {
            notes.push(format!("同步预览：{}", sync_plan.command_preview()));
        } else {
            notes.push("同步预览：当前组合不需要同步 /etc/nixos".to_string());
        }
        if self.show_advanced {
            notes.push("高级项：当前会退回完整部署向导处理。".to_string());
        } else {
            notes.push("高级项：关闭".to_string());
        }

        if let Some(rebuild_plan) = self.deploy_rebuild_plan_for_execution() {
            notes.push(format!(
                "命令预览：{}",
                rebuild_plan.command_preview(self.should_use_sudo())
            ));
        } else {
            notes.push(
                "命令预览：当前来源会转交给完整部署向导处理。".to_string(),
            );
        }
        if self.can_execute_deploy_directly() {
            notes.push("执行路径：当前页可直接执行；按 x 立即运行。".to_string());
        } else {
            notes.push("执行路径：当前页会调起完整部署向导。".to_string());
        }

        DeployPlan {
            task: self.deploy_task,
            detected_host: Some(self.context.current_host.clone()),
            target_host: self.target_host.clone(),
            source: self.deploy_source,
            source_detail: None,
            action: self.deploy_action,
            notes,
        }
    }

    fn deploy_rebuild_plan_for_execution(&self) -> Option<NixosRebuildPlan> {
        let flake_root = match self.deploy_source {
            DeploySource::CurrentRepo if self.should_sync_current_repo_before_rebuild() => {
                self.context.etc_root.clone()
            }
            DeploySource::CurrentRepo => self.context.repo_root.clone(),
            DeploySource::EtcNixos => self.context.etc_root.clone(),
            DeploySource::RemotePinned | DeploySource::RemoteHead => return None,
        };

        Some(NixosRebuildPlan {
            action: self.deploy_action,
            upgrade: self.flake_update,
            flake_root,
            target_host: self.target_host.clone(),
        })
    }

    fn deploy_sync_plan_for_execution(&self) -> Option<RepoSyncPlan> {
        match self.deploy_source {
            DeploySource::CurrentRepo if self.should_sync_current_repo_before_rebuild() => {
                Some(RepoSyncPlan {
                    source_dir: self.context.repo_root.clone(),
                    destination_dir: self.context.etc_root.clone(),
                    delete_extra: true,
                })
            }
            _ => None,
        }
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

    pub fn current_home_user(&self) -> Option<&str> {
        self.context
            .users
            .get(self.home_user_index)
            .map(String::as_str)
    }

    pub fn next_home_user(&mut self) {
        if self.context.users.is_empty() {
            return;
        }
        self.home_user_index = (self.home_user_index + 1) % self.context.users.len();
    }

    pub fn previous_home_user(&mut self) {
        if self.context.users.is_empty() {
            return;
        }
        self.home_user_index = if self.home_user_index == 0 {
            self.context.users.len() - 1
        } else {
            self.home_user_index - 1
        };
    }

    pub fn next_home_field(&mut self) {
        let len = self.home_options_for_area("desktop").len();
        if len == 0 {
            self.home_focus = 0;
            return;
        }
        self.home_focus = (self.home_focus + 1) % len;
    }

    pub fn previous_home_field(&mut self) {
        let len = self.home_options_for_area("desktop").len();
        if len == 0 {
            self.home_focus = 0;
            return;
        }
        self.home_focus = if self.home_focus == 0 {
            len - 1
        } else {
            self.home_focus - 1
        };
    }

    pub fn adjust_home_field(&mut self, delta: i8) {
        let Some(user) = self.current_home_user().map(ToOwned::to_owned) else {
            self.status = "Home 页没有可操作的用户目录。".to_string();
            return;
        };

        let Some(option_id) = self.current_home_option_id().map(ToOwned::to_owned) else {
            self.status = "Home 页当前没有可编辑的结构化选项。".to_string();
            return;
        };

        let settings = self.home_settings_by_user.entry(user.clone()).or_default();
        match option_id.as_str() {
            "noctalia.barProfile" => {
                cycle_enum(&mut settings.bar_profile, &ManagedBarProfile::ALL, delta)
            }
            "desktop.enableZed" => {
                cycle_enum(&mut settings.enable_zed_entry, &ManagedToggle::ALL, delta)
            }
            "desktop.enableYesPlayMusic" => cycle_enum(
                &mut settings.enable_yesplaymusic_entry,
                &ManagedToggle::ALL,
                delta,
            ),
            _ => {
                self.status = format!("Home 选项 {option_id} 还没有接入可编辑实现。");
                return;
            }
        }
        self.home_dirty_users.insert(user.clone());
        self.status = format!("已更新用户 {user} 的 Home 结构化设置。");
    }

    pub fn home_rows(&self) -> Vec<(String, String)> {
        let settings = self.current_home_settings().cloned().unwrap_or_default();
        self.home_options_for_area("desktop")
            .into_iter()
            .map(|option| {
                let value = match option.id.as_str() {
                    "noctalia.barProfile" => settings.bar_profile.label().to_string(),
                    "desktop.enableZed" => settings.enable_zed_entry.label().to_string(),
                    "desktop.enableYesPlayMusic" => {
                        settings.enable_yesplaymusic_entry.label().to_string()
                    }
                    _ => "未接入".to_string(),
                };
                (option.label.clone(), value)
            })
            .collect()
    }

    pub fn home_summary_lines(&self) -> Vec<String> {
        let mut lines = vec![
            format!(
                "当前用户：{}",
                self.current_home_user().unwrap_or("无可用用户")
            ),
            format!(
                "目标文件：{}",
                display_path(self.home_target_desktop_path())
            ),
        ];

        let settings = self.current_home_settings().cloned().unwrap_or_default();
        let desktop_options = self.home_options_for_area("desktop");
        if desktop_options.is_empty() {
            lines.push("当前没有可用的 Home 元数据选项。".to_string());
        } else {
            for option in &desktop_options {
                let value = match option.id.as_str() {
                    "noctalia.barProfile" => settings.bar_profile.label(),
                    "desktop.enableZed" => settings.enable_zed_entry.label(),
                    "desktop.enableYesPlayMusic" => settings.enable_yesplaymusic_entry.label(),
                    _ => "未接入",
                };
                lines.push(format!("{}：{value}", option.label));
            }
        }

        if let Some(user) = self.current_home_user()
            && self.home_dirty_users.contains(user)
        {
            lines.push("状态：当前用户有未保存的 Home 设置修改".to_string());
        } else {
            lines.push("状态：当前用户没有未保存的 Home 设置修改".to_string());
        }

        lines.push(String::new());
        lines.push("当前阶段已接入的结构化设置：".to_string());
        for option in desktop_options {
            if let Some(description) = &option.description {
                lines.push(format!("- {}：{description}", option.label));
            } else {
                lines.push(format!("- {}", option.label));
            }
        }
        lines.push(String::new());
        lines.push(
            "这些内容只会写入 managed/settings/desktop.nix，不会直接改你的手写 config/。"
                .to_string(),
        );
        lines
    }

    pub fn save_current_home_settings(&mut self) -> Result<()> {
        let Some(user) = self.current_home_user().map(ToOwned::to_owned) else {
            self.status = "没有可保存的用户。".to_string();
            return Ok(());
        };

        let managed_dir = self
            .context
            .repo_root
            .join("home/users")
            .join(&user)
            .join("managed");
        ensure_managed_settings_layout(&managed_dir)?;

        let settings = self
            .home_settings_by_user
            .get(&user)
            .cloned()
            .unwrap_or_default();
        let path = managed_dir.join("settings/desktop.nix");
        write_file_atomic(&path, &render_managed_desktop_file(&settings))?;
        self.home_dirty_users.remove(&user);
        self.status = format!("已写入 {}", path.display());
        Ok(())
    }

    pub fn next_action_item(&mut self) {
        self.actions_focus = (self.actions_focus + 1) % ActionItem::ALL.len();
    }

    pub fn previous_action_item(&mut self) {
        self.actions_focus = if self.actions_focus == 0 {
            ActionItem::ALL.len() - 1
        } else {
            self.actions_focus - 1
        };
    }

    pub fn current_action_item(&self) -> ActionItem {
        ActionItem::ALL[self.actions_focus]
    }

    pub fn actions_rows(&self) -> Vec<(String, String)> {
        ActionItem::ALL
            .iter()
            .map(|item| {
                (
                    item.label().to_string(),
                    if self.action_available(*item) {
                        "可执行".to_string()
                    } else {
                        "需切换场景".to_string()
                    },
                )
            })
            .collect()
    }

    pub fn actions_summary_lines(&self) -> Vec<String> {
        let action = self.current_action_item();
        let mut lines = vec![
            format!("当前动作：{}", action.label()),
            format!("说明：{}", action.description()),
            format!(
                "当前仓库：{}",
                self.context.repo_root.display()
            ),
            format!("/etc/nixos：{}", self.context.etc_root.display()),
            format!("当前主机：{}", self.target_host),
            format!(
                "权限：{}",
                match self.context.privilege_mode.as_str() {
                    "root" => "root",
                    "sudo-session" => "sudo session",
                    "sudo-available" => "sudo available",
                    _ => "rootless",
                }
            ),
        ];

        if let Some(preview) = self.action_command_preview(action) {
            lines.push(format!("命令预览：{preview}"));
        }
        if self.action_available(action) {
            lines.push("状态：当前环境可以直接执行".to_string());
        } else {
            lines.push("状态：当前环境不适合直接执行；请改用 Deploy 页或切换权限".to_string());
        }

        lines.push(String::new());
        lines.push("当前页说明：".to_string());
        lines.push("- 这里只放高频维护动作，不处理复杂初始化向导".to_string());
        lines.push("- 直接执行外部命令前，会临时退出 TUI，执行完成后再返回".to_string());
        lines.push("- 如需远端来源、模板生成、复杂交互，请使用 deploy wizard".to_string());
        lines
    }

    pub fn execute_current_action(&mut self) -> Result<()> {
        self.ensure_no_unsaved_changes_for_execution()?;
        let action = self.current_action_item();
        if !self.action_available(action) {
            anyhow::bail!("当前环境暂不适合直接执行动作：{}", action.label());
        }
        let use_sudo = self.should_use_sudo();

        match action {
            ActionItem::FlakeCheck => {
                let mut cmd = std::process::Command::new("nix");
                cmd.arg("--extra-experimental-features")
                    .arg("nix-command flakes")
                    .arg("flake")
                    .arg("check")
                    .arg(format!("path:{}", self.context.repo_root.display()))
                    .env("NIX_CONFIG", merged_nix_config())
                    .stdin(std::process::Stdio::inherit())
                    .stdout(std::process::Stdio::inherit())
                    .stderr(std::process::Stdio::inherit());
                let status = cmd.status().context("failed to run nix flake check")?;
                if !status.success() {
                    anyhow::bail!("flake check exited with {}", status.code().unwrap_or(1));
                }
                self.status = "flake check 已完成。".to_string();
            }
            ActionItem::FlakeUpdate => {
                let mut cmd = std::process::Command::new("nix");
                cmd.arg("--extra-experimental-features")
                    .arg("nix-command flakes")
                    .arg("flake")
                    .arg("update")
                    .arg("--flake")
                    .arg(self.context.repo_root.display().to_string())
                    .env("NIX_CONFIG", merged_nix_config())
                    .stdin(std::process::Stdio::inherit())
                    .stdout(std::process::Stdio::inherit())
                    .stderr(std::process::Stdio::inherit());
                let status = cmd.status().context("failed to run nix flake update")?;
                if !status.success() {
                    anyhow::bail!("flake update exited with {}", status.code().unwrap_or(1));
                }
                self.status = "flake update 已完成。".to_string();
            }
            ActionItem::UpdateUpstreamCheck => {
                let status = self.run_sibling_in_repo(
                    "update-upstream-apps",
                    &["--check".to_string()],
                )?;
                if !status.success() {
                    anyhow::bail!(
                        "update-upstream-apps --check exited with {}",
                        status.code().unwrap_or(1)
                    );
                }
                self.status = "上游 pin 检查已完成。".to_string();
            }
            ActionItem::UpdateUpstreamPins => {
                let status = self.run_sibling_in_repo("update-upstream-apps", &[])?;
                if !status.success() {
                    anyhow::bail!(
                        "update-upstream-apps exited with {}",
                        status.code().unwrap_or(1)
                    );
                }
                self.status = "上游 pin 刷新已完成。".to_string();
            }
            ActionItem::SyncRepoToEtc => {
                let plan = self
                    .manual_repo_sync_plan()
                    .context("当前仓库已经是 /etc/nixos，无需同步")?;
                run_repo_sync(
                    &plan,
                    |cmd, args| {
                        let status = std::process::Command::new(cmd)
                            .args(args)
                            .stdin(std::process::Stdio::inherit())
                            .stdout(std::process::Stdio::inherit())
                            .stderr(std::process::Stdio::inherit())
                            .status()
                            .with_context(|| format!("failed to run {cmd}"))?;
                        if status.success() {
                            Ok(())
                        } else {
                            anyhow::bail!("{cmd} failed with {}", status.code().unwrap_or(1));
                        }
                    },
                    |cmd, args| run_root_command_ok(cmd, args, use_sudo),
                    || self.clean_etc_dir_keep_hardware(),
                )?;
                self.status = "仓库已同步到 /etc/nixos。".to_string();
            }
            ActionItem::RebuildCurrentHost => {
                let action = if self.context.privilege_mode == "rootless" {
                    DeployAction::Build
                } else {
                    DeployAction::Switch
                };
                if action != DeployAction::Build {
                    ensure_root_hardware_config(&self.context.etc_root, use_sudo)?;
                }
                let plan = NixosRebuildPlan {
                    action,
                    upgrade: false,
                    flake_root: if self.context.repo_root == self.context.etc_root {
                        self.context.repo_root.clone()
                    } else {
                        self.context.etc_root.clone()
                    },
                    target_host: self.context.current_host.clone(),
                };
                let status = run_nixos_rebuild(&plan, use_sudo)?;
                if !status.success() {
                    anyhow::bail!("nixos-rebuild exited with {}", status.code().unwrap_or(1));
                }
                self.status = format!(
                    "当前主机 {} 已完成一次 {}。",
                    self.context.current_host,
                    action.label()
                );
            }
            ActionItem::LaunchDeployWizard => {
                let status = self.run_sibling_in_repo("mcb-deploy", &[])?;
                if !status.success() {
                    anyhow::bail!("mcb-deploy exited with {}", status.code().unwrap_or(1));
                }
                self.status = "已返回 deploy wizard。".to_string();
            }
        }

        Ok(())
    }

    pub fn captures_text_input(&self) -> bool {
        self.package_search_mode
            || self.package_group_create_mode
            || self.package_group_rename_mode
            || self.users_text_mode.is_some()
            || self.hosts_text_mode.is_some()
    }

    pub fn active_package_text_mode(&self) -> Option<PackageTextMode> {
        if self.package_group_rename_mode {
            Some(PackageTextMode::RenameGroup)
        } else if self.package_group_create_mode {
            Some(PackageTextMode::CreateGroup)
        } else if self.package_search_mode {
            Some(PackageTextMode::Search)
        } else {
            None
        }
    }

    pub fn active_users_text_mode(&self) -> Option<UsersTextMode> {
        self.users_text_mode
    }

    pub fn active_hosts_text_mode(&self) -> Option<HostsTextMode> {
        self.hosts_text_mode
    }

    pub fn current_package_user(&self) -> Option<&str> {
        self.context
            .users
            .get(self.package_user_index)
            .map(String::as_str)
    }

    pub fn current_package_mode(&self) -> PackageDataMode {
        self.package_mode
    }

    pub fn current_package_mode_label(&self) -> &'static str {
        self.package_mode.label()
    }

    pub fn current_package_category(&self) -> Option<&str> {
        if self.package_mode == PackageDataMode::Search {
            return None;
        }
        if self.package_category_index == 0 {
            None
        } else {
            self.context
                .catalog_categories
                .get(self.package_category_index - 1)
                .map(String::as_str)
        }
    }

    pub fn current_package_category_label(&self) -> &str {
        if self.package_mode == PackageDataMode::Search {
            "搜索结果"
        } else {
            self.current_package_category().unwrap_or("全部")
        }
    }

    pub fn current_package_group_filter(&self) -> Option<&str> {
        self.package_group_filter.as_deref()
    }

    pub fn current_package_group_filter_label(&self) -> String {
        self.current_package_group_filter()
            .map(|group| self.package_group_label(group))
            .unwrap_or_else(|| "全部".to_string())
    }

    pub fn current_package_source_filter(&self) -> Option<&str> {
        self.package_source_filter.as_deref()
    }

    pub fn current_package_source_filter_label(&self) -> String {
        if self.package_mode == PackageDataMode::Search {
            "nixpkgs".to_string()
        } else {
            self.current_package_source_filter()
                .unwrap_or("全部")
                .to_string()
        }
    }

    pub fn package_filtered_indices(&self) -> Vec<usize> {
        let group_filter = self.package_group_filter.clone();
        let source_filter = self.package_source_filter.clone();
        let current_user = self.current_package_user().map(ToOwned::to_owned);
        self.package_base_indices()
            .into_iter()
            .filter_map(|(index, entry)| {
                let matches_group = if let Some(group_filter) = &group_filter {
                    let effective_group = current_user
                        .as_deref()
                        .map(|user| self.package_group_for_user(user, entry))
                        .unwrap_or_else(|| entry.group_key().to_string());
                    effective_group == *group_filter
                } else {
                    true
                };

                let matches_source = if let Some(source_filter) = &source_filter {
                    self.package_mode != PackageDataMode::Search
                        && entry.source_label() == source_filter
                } else {
                    true
                };

                (entry.matches(self.current_package_category(), &self.package_search)
                    && matches_group
                    && matches_source)
                    .then_some(index)
            })
            .collect()
    }

    pub fn package_filtered_count(&self) -> usize {
        self.package_filtered_indices().len()
    }

    pub fn package_selected_count(&self) -> usize {
        self.current_user_selection().map_or(0, BTreeMap::len)
    }

    pub fn package_dirty_count(&self) -> usize {
        self.package_dirty_users.len()
    }

    pub fn package_target_dir_path(&self) -> Option<PathBuf> {
        let user = self.current_package_user()?;
        Some(
            self.context
                .repo_root
                .join("home/users")
                .join(user)
                .join("managed/packages"),
        )
    }

    pub fn current_package_entry(&self) -> Option<&CatalogEntry> {
        let filtered = self.package_filtered_indices();
        let index = *filtered.get(self.package_cursor)?;
        self.context.catalog_entries.get(index)
    }

    pub fn current_package_target_path(&self) -> Option<PathBuf> {
        let user = self.current_package_user()?;
        let entry = self.current_package_entry()?;
        let group = self.package_group_for_user(user, entry);
        Some(managed_package_group_path(
            &self.context.repo_root,
            user,
            &group,
        ))
    }

    pub fn package_selected_entries(&self) -> Vec<&CatalogEntry> {
        let mut entries = self
            .current_user_selection()
            .into_iter()
            .flat_map(|selected| {
                self.context
                    .catalog_entries
                    .iter()
                    .filter(move |entry| selected.contains_key(&entry.id))
            })
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| {
            self.compare_package_groups(
                &self.effective_selected_group(left),
                &self.effective_selected_group(right),
            )
            .then_with(|| left.category.cmp(&right.category))
            .then_with(|| left.name.cmp(&right.name))
        });
        entries
    }

    pub fn package_group_for_current_entry(&self) -> Option<String> {
        let user = self.current_package_user()?;
        let entry = self.current_package_entry()?;
        Some(self.package_group_for_user(user, entry))
    }

    pub fn current_selected_group_name(&self) -> Option<String> {
        let user = self.current_package_user()?;
        let entry = self.current_package_entry()?;
        self.package_user_selections
            .get(user)
            .and_then(|selection| selection.get(&entry.id))
            .cloned()
    }

    pub fn effective_selected_group(&self, entry: &CatalogEntry) -> String {
        self.current_package_user()
            .map(|user| self.package_group_for_user(user, entry))
            .unwrap_or_else(|| entry.group_key().to_string())
    }

    pub fn package_group_counts(&self) -> Vec<(String, usize)> {
        let Some(user) = self.current_package_user() else {
            return Vec::new();
        };
        let Some(selection) = self.package_user_selections.get(user) else {
            return Vec::new();
        };

        let mut counts = BTreeMap::<String, usize>::new();
        for group in selection.values() {
            *counts.entry(group.clone()).or_insert(0) += 1;
        }
        let mut pairs = counts.into_iter().collect::<Vec<_>>();
        pairs.sort_by(|(left, _), (right, _)| self.compare_package_groups(left, right));
        pairs
    }

    pub fn current_selected_group_member_count(&self) -> usize {
        let Some(current_group) = self.current_selected_group_name() else {
            return 0;
        };
        self.package_group_counts()
            .into_iter()
            .find(|(group, _)| group == &current_group)
            .map(|(_, count)| count)
            .unwrap_or(0)
    }

    pub fn package_summary_lines(&self) -> Vec<String> {
        let mut lines = vec![
            format!("数据源：{}", self.current_package_mode_label()),
            format!(
                "当前用户：{}",
                self.current_package_user().unwrap_or("无可用用户")
            ),
            format!("目标目录：{}", display_path(self.package_target_dir_path())),
            format!("分类过滤：{}", self.current_package_category_label()),
            format!("组过滤：{}", self.current_package_group_filter_label()),
            format!("来源过滤：{}", self.current_package_source_filter_label()),
            format!(
                "搜索：{}",
                if self.package_search.is_empty() {
                    "无".to_string()
                } else {
                    self.package_search.clone()
                }
            ),
            format!("目录总数：{}", self.package_total_count()),
            format!("过滤后数量：{}", self.package_filtered_count()),
            format!("当前用户已选：{}", self.package_selected_count()),
            format!("未保存用户：{}", self.package_dirty_count()),
            format!(
                "可用组数：{}",
                self.current_package_user()
                    .map(|user| self.package_groups_for_user(user).len())
                    .unwrap_or(0)
            ),
        ];

        if let Some(path) = self.current_package_target_path() {
            lines.push(format!("当前组落点：{}", path.display()));
        }
        if let Some(group) = self.current_selected_group_name() {
            lines.push(format!(
                "当前已选组：{}（{} 个软件）",
                self.package_group_label(&group),
                self.current_selected_group_member_count()
            ));
            if let Some(description) = self.package_group_description(&group) {
                lines.push(format!("组说明：{description}"));
            }
        }

        if let Some(user) = self.current_package_user()
            && self.package_dirty_users.contains(user)
        {
            lines.push("状态：当前用户有未保存修改".to_string());
        }
        lines
    }

    pub fn next_package_user(&mut self) {
        if self.context.users.is_empty() {
            return;
        }
        self.package_user_index = (self.package_user_index + 1) % self.context.users.len();
        self.ensure_valid_package_group_filter();
        self.clamp_package_cursor();
    }

    pub fn previous_package_user(&mut self) {
        if self.context.users.is_empty() {
            return;
        }
        self.package_user_index = if self.package_user_index == 0 {
            self.context.users.len() - 1
        } else {
            self.package_user_index - 1
        };
        self.ensure_valid_package_group_filter();
        self.clamp_package_cursor();
    }

    pub fn next_package_item(&mut self) {
        let len = self.package_filtered_count();
        if len == 0 {
            self.package_cursor = 0;
            return;
        }
        self.package_cursor = (self.package_cursor + 1) % len;
    }

    pub fn previous_package_item(&mut self) {
        let len = self.package_filtered_count();
        if len == 0 {
            self.package_cursor = 0;
            return;
        }
        self.package_cursor = if self.package_cursor == 0 {
            len - 1
        } else {
            self.package_cursor - 1
        };
    }

    pub fn next_package_category(&mut self) {
        if self.package_mode == PackageDataMode::Search {
            return;
        }
        let len = self.context.catalog_categories.len() + 1;
        if len == 0 {
            return;
        }
        self.package_category_index = (self.package_category_index + 1) % len;
        self.clamp_package_cursor();
    }

    pub fn previous_package_category(&mut self) {
        if self.package_mode == PackageDataMode::Search {
            return;
        }
        let len = self.context.catalog_categories.len() + 1;
        if len == 0 {
            return;
        }
        self.package_category_index = if self.package_category_index == 0 {
            len - 1
        } else {
            self.package_category_index - 1
        };
        self.clamp_package_cursor();
    }

    pub fn adjust_package_source_filter(&mut self, delta: i8) {
        if self.package_mode == PackageDataMode::Search {
            self.status = "nixpkgs 搜索模式不使用本地来源过滤。".to_string();
            return;
        }
        let mut options = vec![String::new()];
        options.extend(self.context.catalog_sources.clone());

        let current = self.package_source_filter.clone().unwrap_or_default();
        let Some(next) = cycle_string_value(&current, &options, delta) else {
            return;
        };

        if next.is_empty() {
            self.package_source_filter = None;
            self.status = "已清空软件来源过滤。".to_string();
        } else {
            self.package_source_filter = Some(next.clone());
            self.status = format!("当前软件来源过滤：{next}");
        }
        self.clamp_package_cursor();
    }

    pub fn adjust_package_group_filter(&mut self, delta: i8) {
        let Some(user) = self.current_package_user() else {
            self.status = "Packages 页没有可操作的用户目录。".to_string();
            return;
        };

        let groups = self.package_groups_for_user(user);
        let mut options = vec![String::new()];
        options.extend(groups);

        let current = self.package_group_filter.clone().unwrap_or_default();
        let Some(next) = cycle_string_value(&current, &options, delta) else {
            return;
        };

        if next.is_empty() {
            self.package_group_filter = None;
            self.status = "已清空软件组过滤。".to_string();
        } else {
            self.package_group_filter = Some(next.clone());
            self.status = format!("当前软件组过滤：{next}");
        }
        self.clamp_package_cursor();
    }

    pub fn focus_current_selected_group(&mut self) {
        let Some(group) = self.package_group_for_current_entry() else {
            self.status = "当前过滤条件下没有可聚焦分组的软件。".to_string();
            return;
        };

        self.package_group_filter = Some(group.clone());
        self.clamp_package_cursor();
        self.status = format!("已聚焦到软件组：{group}");
    }

    pub fn clear_package_group_filter(&mut self) {
        if self.package_group_filter.is_none() {
            self.status = "当前没有启用软件组过滤。".to_string();
            return;
        }

        self.package_group_filter = None;
        self.clamp_package_cursor();
        self.status = "已清空软件组过滤。".to_string();
    }

    pub fn toggle_current_package(&mut self) {
        let Some(user) = self.current_package_user().map(ToOwned::to_owned) else {
            self.status = "Packages 页没有可操作的用户目录。".to_string();
            return;
        };
        let Some(entry) = self.current_package_entry().cloned() else {
            self.status = "当前过滤条件下没有可切换的软件。".to_string();
            return;
        };

        let default_group = self.default_group_for_entry(&entry);
        let enabled = {
            let selection = self
                .package_user_selections
                .entry(user.clone())
                .or_default();
            if selection.contains_key(&entry.id) {
                selection.remove(&entry.id);
                false
            } else {
                selection.insert(entry.id.clone(), default_group);
                true
            }
        };
        self.sync_local_catalog_membership(&entry.id);
        self.package_dirty_users.insert(user.clone());
        self.ensure_valid_package_group_filter();
        self.clamp_package_cursor();
        self.status = if enabled {
            format!("已为用户 {user} 选中软件：{}", entry.name)
        } else {
            format!("已为用户 {user} 取消软件：{}", entry.name)
        };
    }

    pub fn open_package_search(&mut self) {
        self.package_group_create_mode = false;
        self.package_group_rename_mode = false;
        self.package_group_rename_source.clear();
        self.package_group_input.clear();
        self.package_search_mode = true;
        self.status =
            "Packages 搜索已进入输入模式；搜索模式下 Enter 会刷新 nixpkgs 结果，Esc 退出。"
                .to_string();
    }

    pub fn handle_search_input(&mut self, code: crossterm::event::KeyCode) {
        match code {
            crossterm::event::KeyCode::Enter => {
                self.package_search_mode = false;
                if self.package_mode == PackageDataMode::Search {
                    self.refresh_package_search_results();
                } else {
                    self.clamp_package_cursor();
                    self.status = "Packages 搜索输入结束。".to_string();
                }
            }
            crossterm::event::KeyCode::Esc => {
                self.package_search_mode = false;
                self.clamp_package_cursor();
                self.status = "Packages 搜索输入结束。".to_string();
            }
            crossterm::event::KeyCode::Backspace => {
                self.package_search.pop();
                self.clamp_package_cursor();
            }
            crossterm::event::KeyCode::Char(ch) => {
                self.package_search.push(ch);
                self.clamp_package_cursor();
            }
            _ => {}
        }
    }

    pub fn open_package_group_creation(&mut self) {
        let Some(entry_name) = self.current_package_entry().map(|entry| entry.name.clone()) else {
            self.status = "当前过滤条件下没有可新建分组的软件。".to_string();
            return;
        };

        self.package_search_mode = false;
        self.package_group_rename_mode = false;
        self.package_group_rename_source.clear();
        self.package_group_create_mode = true;
        self.package_group_input.clear();
        self.status = format!(
            "开始为软件 {} 创建新组；输入组名后按 Enter，Esc 取消。",
            entry_name
        );
    }

    pub fn handle_group_input(&mut self, code: crossterm::event::KeyCode) {
        match code {
            crossterm::event::KeyCode::Enter => {
                if self.package_group_rename_mode {
                    self.confirm_package_group_rename();
                } else {
                    self.confirm_package_group_creation();
                }
            }
            crossterm::event::KeyCode::Esc => {
                self.package_group_create_mode = false;
                self.package_group_rename_mode = false;
                self.package_group_rename_source.clear();
                self.package_group_input.clear();
                self.status = "已取消软件组编辑。".to_string();
            }
            crossterm::event::KeyCode::Backspace => {
                self.package_group_input.pop();
            }
            crossterm::event::KeyCode::Char(ch) => {
                self.package_group_input.push(ch);
            }
            _ => {}
        }
    }

    pub fn clear_package_search(&mut self) {
        if self.package_search.is_empty() {
            return;
        }
        self.package_search.clear();
        self.package_search_mode = false;
        if self.package_mode == PackageDataMode::Search {
            self.package_search_result_indices.clear();
        }
        self.clamp_package_cursor();
        self.status = "已清空 Packages 搜索条件。".to_string();
    }

    pub fn toggle_package_mode(&mut self) {
        self.package_mode = match self.package_mode {
            PackageDataMode::Local => PackageDataMode::Search,
            PackageDataMode::Search => PackageDataMode::Local,
        };
        self.package_category_index = 0;
        self.package_source_filter = None;
        if self.package_mode == PackageDataMode::Search {
            if self.package_search.trim().is_empty() {
                self.status =
                    "已切到 nixpkgs 搜索模式；按 / 输入关键词，Enter 刷新搜索结果。".to_string();
            } else {
                self.refresh_package_search_results();
                return;
            }
        } else {
            self.status = "已切回本地覆盖层视图。".to_string();
        }
        self.clamp_package_cursor();
    }

    pub fn refresh_package_search_results(&mut self) {
        if self.package_mode != PackageDataMode::Search {
            self.status = "当前不在 nixpkgs 搜索模式。".to_string();
            return;
        }
        let query = self.package_search.trim().to_string();
        if query.is_empty() {
            self.package_search_result_indices.clear();
            self.status = "请输入关键词后再刷新 nixpkgs 搜索。".to_string();
            self.clamp_package_cursor();
            return;
        }

        match search_catalog_entries("nixpkgs", &query, &self.context.current_system) {
            Ok(entries) => {
                let count = entries.len();
                self.package_search_result_indices = self.merge_catalog_entries(entries, false);
                self.clamp_package_cursor();
                self.status = format!("nixpkgs 搜索完成：{query}，得到 {count} 条结果。");
            }
            Err(err) => {
                self.package_search_result_indices.clear();
                self.clamp_package_cursor();
                self.status = format!("nixpkgs 搜索失败：{err}");
            }
        }
    }

    pub fn open_package_group_rename(&mut self) {
        let Some(user) = self.current_package_user().map(ToOwned::to_owned) else {
            self.status = "Packages 页没有可操作的用户目录。".to_string();
            return;
        };
        let Some(entry) = self.current_package_entry().cloned() else {
            self.status = "当前过滤条件下没有可重命名分组的软件。".to_string();
            return;
        };

        let Some(current_group) = self
            .package_user_selections
            .get(&user)
            .and_then(|selection| selection.get(&entry.id))
            .cloned()
        else {
            self.status = "请先为当前用户选中这个软件，再重命名它所在的组。".to_string();
            return;
        };

        self.package_search_mode = false;
        self.package_group_create_mode = false;
        self.package_group_rename_mode = true;
        self.package_group_rename_source = current_group.clone();
        self.package_group_input = current_group.clone();
        self.status = format!("开始重命名组 {current_group}；输入新组名后按 Enter，Esc 取消。");
    }

    pub fn adjust_current_package_group(&mut self, delta: i8) {
        let Some(user) = self.current_package_user().map(ToOwned::to_owned) else {
            self.status = "Packages 页没有可操作的用户目录。".to_string();
            return;
        };
        let Some(entry) = self.current_package_entry().cloned() else {
            self.status = "当前过滤条件下没有可调整分组的软件。".to_string();
            return;
        };

        let groups = self.package_groups_for_user(&user);
        if groups.is_empty() {
            self.status = "当前用户没有可用的软件组。".to_string();
            return;
        }

        let current = self
            .package_user_selections
            .get(&user)
            .and_then(|selection| selection.get(&entry.id))
            .cloned()
            .unwrap_or_else(|| entry.group_key().to_string());
        let Some(next_group) = cycle_string_value(&current, &groups, delta) else {
            return;
        };

        self.package_user_selections
            .entry(user.clone())
            .or_default()
            .insert(entry.id.clone(), next_group.clone());
        self.package_dirty_users.insert(user.clone());
        self.ensure_valid_package_group_filter();
        self.clamp_package_cursor();
        self.status = format!(
            "已将用户 {user} 的软件 {} 调整到组：{next_group}",
            entry.name
        );
    }

    pub fn move_current_selected_group(&mut self, delta: i8) {
        let Some(user) = self.current_package_user().map(ToOwned::to_owned) else {
            self.status = "Packages 页没有可操作的用户目录。".to_string();
            return;
        };
        let Some(current_group) = self.current_selected_group_name() else {
            self.status = "请先选中当前软件，再整组移动它所在的组。".to_string();
            return;
        };

        let groups = self.package_groups_for_user(&user);
        if groups.len() < 2 {
            self.status = "当前用户只有一个可用组，无法整组移动。".to_string();
            return;
        }

        let Some(next_group) = cycle_string_value(&current_group, &groups, delta) else {
            return;
        };
        if next_group == current_group {
            self.status = format!("当前组未变化：{current_group}");
            return;
        }

        let mut moved = 0usize;
        if let Some(selection) = self.package_user_selections.get_mut(&user) {
            for group in selection.values_mut() {
                if *group == current_group {
                    *group = next_group.clone();
                    moved += 1;
                }
            }
        }

        self.package_dirty_users.insert(user.clone());
        if self.package_group_filter.as_deref() == Some(current_group.as_str()) {
            self.package_group_filter = Some(next_group.clone());
        } else {
            self.ensure_valid_package_group_filter();
        }
        self.clamp_package_cursor();
        self.status = format!(
            "已将用户 {user} 的组 {current_group} 整体移动到 {next_group}，影响 {moved} 个软件"
        );
    }

    pub fn package_group_input_preview(&self) -> String {
        normalize_package_group_name(&self.package_group_input)
    }

    pub fn save_current_user_packages(&mut self) -> Result<()> {
        let Some(user) = self.current_package_user().map(ToOwned::to_owned) else {
            self.status = "没有可保存的用户。".to_string();
            return Ok(());
        };

        let selected = self
            .package_user_selections
            .get(&user)
            .cloned()
            .unwrap_or_default();
        let managed_dir = self
            .context
            .repo_root
            .join("home/users")
            .join(&user)
            .join("managed");
        ensure_managed_packages_layout(&managed_dir)?;
        write_grouped_managed_packages(&managed_dir, &self.context.catalog_entries, &selected)?;
        self.package_dirty_users.remove(&user);
        self.status = format!("已写入 {}", managed_dir.join("packages").display());
        Ok(())
    }

    fn current_home_settings(&self) -> Option<&HomeManagedSettings> {
        let user = self.current_home_user()?;
        self.home_settings_by_user.get(user)
    }

    fn home_options_for_area(&self, area: &str) -> Vec<&HomeOptionMeta> {
        self.context
            .catalog_home_options
            .iter()
            .filter(|option| option.area == area)
            .collect()
    }

    fn current_home_option_id(&self) -> Option<&str> {
        self.home_options_for_area("desktop")
            .get(self.home_focus)
            .map(|option| option.id.as_str())
    }

    fn home_target_desktop_path(&self) -> Option<PathBuf> {
        let user = self.current_home_user()?;
        Some(managed_home_desktop_path(&self.context.repo_root, user))
    }

    fn current_user_selection(&self) -> Option<&BTreeMap<String, String>> {
        let user = self.current_package_user()?;
        self.package_user_selections.get(user)
    }

    fn package_base_indices(&self) -> Vec<(usize, &CatalogEntry)> {
        match self.package_mode {
            PackageDataMode::Local => self
                .context
                .catalog_entries
                .iter()
                .enumerate()
                .filter(|(_, entry)| self.package_local_entry_ids.contains(&entry.id))
                .collect(),
            PackageDataMode::Search => self
                .package_search_result_indices
                .iter()
                .filter_map(|index| {
                    self.context
                        .catalog_entries
                        .get(*index)
                        .map(|entry| (*index, entry))
                })
                .collect(),
        }
    }

    fn package_total_count(&self) -> usize {
        match self.package_mode {
            PackageDataMode::Local => self.package_local_entry_ids.len(),
            PackageDataMode::Search => self.package_search_result_indices.len(),
        }
    }

    fn merge_catalog_entries(
        &mut self,
        entries: Vec<CatalogEntry>,
        include_in_local: bool,
    ) -> Vec<usize> {
        let mut indices = Vec::new();

        for entry in entries {
            if let Some(index) = self
                .context
                .catalog_entries
                .iter()
                .position(|existing| existing.id == entry.id)
            {
                if include_in_local {
                    self.package_local_entry_ids.insert(entry.id.clone());
                }
                indices.push(index);
            } else {
                let id = entry.id.clone();
                self.context.catalog_entries.push(entry);
                let index = self.context.catalog_entries.len() - 1;
                if include_in_local {
                    self.package_local_entry_ids.insert(id);
                }
                indices.push(index);
            }
        }

        if include_in_local {
            refresh_local_catalog_indexes(&mut self.context, &self.package_local_entry_ids);
        }
        indices
    }

    fn package_group_for_user(&self, user: &str, entry: &CatalogEntry) -> String {
        self.package_user_selections
            .get(user)
            .and_then(|selection| selection.get(&entry.id))
            .cloned()
            .unwrap_or_else(|| entry.group_key().to_string())
    }

    fn default_group_for_entry(&self, entry: &CatalogEntry) -> String {
        if let Some(group) = self.package_group_filter.as_deref()
            && !group.trim().is_empty()
        {
            return group.to_string();
        }

        let group = entry.group_key();
        if group == "search" {
            "misc".to_string()
        } else {
            group.to_string()
        }
    }

    fn package_group_meta(&self, group: &str) -> Option<&GroupMeta> {
        self.context.catalog_groups.get(group)
    }

    pub fn package_group_label(&self, group: &str) -> String {
        self.package_group_meta(group)
            .map(|meta| meta.label.clone())
            .unwrap_or_else(|| group.to_string())
    }

    pub fn package_group_description(&self, group: &str) -> Option<&str> {
        self.package_group_meta(group)
            .and_then(|meta| meta.description.as_deref())
    }

    pub fn package_group_display(&self, group: &str) -> String {
        let label = self.package_group_label(group);
        if label == group {
            label
        } else {
            format!("{label} [{group}]")
        }
    }

    fn compare_package_groups(&self, left: &str, right: &str) -> Ordering {
        let left_meta = self.package_group_meta(left);
        let right_meta = self.package_group_meta(right);

        left_meta
            .map(|meta| meta.order)
            .unwrap_or(u32::MAX)
            .cmp(&right_meta.map(|meta| meta.order).unwrap_or(u32::MAX))
            .then_with(|| {
                self.package_group_label(left)
                    .cmp(&self.package_group_label(right))
            })
            .then_with(|| left.cmp(right))
    }

    fn package_groups_for_user(&self, user: &str) -> Vec<String> {
        let mut groups = BTreeSet::new();

        for entry in &self.context.catalog_entries {
            if self.package_local_entry_ids.contains(&entry.id) {
                groups.insert(entry.group_key().to_string());
            }
        }

        if let Some(selection) = self.package_user_selections.get(user) {
            for group in selection.values() {
                groups.insert(group.clone());
            }
        }

        let hand_written_dir = self
            .context
            .repo_root
            .join("home/users")
            .join(user)
            .join("packages");
        if let Ok(entries) = fs::read_dir(hand_written_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_file() || !path.extension().is_some_and(|ext| ext == "nix") {
                    continue;
                }
                if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
                    groups.insert(stem.to_string());
                }
            }
        }

        let managed_dir = self
            .context
            .repo_root
            .join("home/users")
            .join(user)
            .join("managed/packages");
        if let Ok(entries) = fs::read_dir(managed_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_file() || !path.extension().is_some_and(|ext| ext == "nix") {
                    continue;
                }
                if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
                    groups.insert(stem.to_string());
                }
            }
        }

        let mut ordered = groups.into_iter().collect::<Vec<_>>();
        ordered.sort_by(|left, right| self.compare_package_groups(left, right));
        ordered
    }

    fn package_id_selected_anywhere(&self, entry_id: &str) -> bool {
        self.package_user_selections
            .values()
            .any(|selection| selection.contains_key(entry_id))
    }

    fn sync_local_catalog_membership(&mut self, entry_id: &str) {
        let keep_local = self
            .context
            .catalog_entries
            .iter()
            .find(|entry| entry.id == entry_id)
            .is_some_and(|entry| is_local_overlay_entry(entry))
            || self.package_id_selected_anywhere(entry_id);

        let changed = if keep_local {
            self.package_local_entry_ids.insert(entry_id.to_string())
        } else {
            self.package_local_entry_ids.remove(entry_id)
        };

        if changed {
            refresh_local_catalog_indexes(&mut self.context, &self.package_local_entry_ids);
        }
    }

    pub fn package_groups_overview(&self) -> Vec<(String, usize)> {
        let Some(user) = self.current_package_user() else {
            return Vec::new();
        };

        let counts = self
            .package_group_counts()
            .into_iter()
            .collect::<BTreeMap<_, _>>();

        self.package_groups_for_user(user)
            .into_iter()
            .map(|group| {
                let count = counts.get(&group).copied().unwrap_or(0);
                (group, count)
            })
            .collect()
    }

    fn clamp_package_cursor(&mut self) {
        let len = self.package_filtered_count();
        if len == 0 {
            self.package_cursor = 0;
        } else if self.package_cursor >= len {
            self.package_cursor = len - 1;
        }
    }

    fn confirm_package_group_creation(&mut self) {
        let Some(user) = self.current_package_user().map(ToOwned::to_owned) else {
            self.package_group_create_mode = false;
            self.package_group_input.clear();
            self.status = "Packages 页没有可操作的用户目录。".to_string();
            return;
        };
        let Some(entry) = self.current_package_entry().cloned() else {
            self.package_group_create_mode = false;
            self.package_group_input.clear();
            self.status = "当前过滤条件下没有可新建分组的软件。".to_string();
            return;
        };

        let normalized = normalize_package_group_name(&self.package_group_input);
        if normalized.is_empty() {
            self.status =
                "组名不能为空；建议使用字母、数字和连字符，例如 research-writing。".to_string();
            return;
        }

        let existed = self.package_groups_for_user(&user).contains(&normalized);
        self.package_user_selections
            .entry(user.clone())
            .or_default()
            .insert(entry.id.clone(), normalized.clone());
        self.package_dirty_users.insert(user.clone());
        self.package_group_filter = Some(normalized.clone());
        self.clamp_package_cursor();
        self.package_group_create_mode = false;
        self.package_group_input.clear();
        self.status = if existed {
            format!(
                "已将用户 {user} 的软件 {} 指向现有组：{normalized}",
                entry.name
            )
        } else {
            format!(
                "已为用户 {user} 新建组：{normalized}，并将软件 {} 分配到该组",
                entry.name
            )
        };
    }

    fn confirm_package_group_rename(&mut self) {
        let Some(user) = self.current_package_user().map(ToOwned::to_owned) else {
            self.reset_package_group_edit_state();
            self.status = "Packages 页没有可操作的用户目录。".to_string();
            return;
        };

        let old_group = self.package_group_rename_source.clone();
        let normalized = normalize_package_group_name(&self.package_group_input);
        if normalized.is_empty() {
            self.status =
                "组名不能为空；建议使用字母、数字和连字符，例如 database-tools。".to_string();
            return;
        }
        if normalized == old_group {
            self.reset_package_group_edit_state();
            self.status = format!("组名未变化：{old_group}");
            return;
        }

        let mut renamed_count = 0usize;
        if let Some(selection) = self.package_user_selections.get_mut(&user) {
            for group in selection.values_mut() {
                if *group == old_group {
                    *group = normalized.clone();
                    renamed_count += 1;
                }
            }
        }

        self.package_dirty_users.insert(user.clone());
        if self.package_group_filter.as_deref() == Some(old_group.as_str()) {
            self.package_group_filter = Some(normalized.clone());
        } else {
            self.ensure_valid_package_group_filter();
        }
        self.clamp_package_cursor();
        self.reset_package_group_edit_state();
        self.status = format!(
            "已将用户 {user} 的组 {old_group} 重命名为 {normalized}，影响 {renamed_count} 个软件"
        );
    }

    fn reset_package_group_edit_state(&mut self) {
        self.package_group_create_mode = false;
        self.package_group_rename_mode = false;
        self.package_group_rename_source.clear();
        self.package_group_input.clear();
    }

    fn ensure_valid_package_group_filter(&mut self) {
        let Some(filter) = self.package_group_filter.clone() else {
            return;
        };
        let Some(user) = self.current_package_user() else {
            self.package_group_filter = None;
            return;
        };

        if !self.package_groups_for_user(user).contains(&filter) {
            self.package_group_filter = None;
        }
    }

    fn should_use_sudo(&self) -> bool {
        matches!(
            self.context.privilege_mode.as_str(),
            "sudo-session" | "sudo-available"
        )
    }

    fn should_sync_current_repo_before_rebuild(&self) -> bool {
        self.deploy_source == DeploySource::CurrentRepo
            && self.context.repo_root != self.context.etc_root
            && self.deploy_action != DeployAction::Build
            && self.context.privilege_mode != "rootless"
    }

    fn manual_repo_sync_plan(&self) -> Option<RepoSyncPlan> {
        (self.context.repo_root != self.context.etc_root).then(|| RepoSyncPlan {
            source_dir: self.context.repo_root.clone(),
            destination_dir: self.context.etc_root.clone(),
            delete_extra: true,
        })
    }

    fn ensure_no_unsaved_changes_for_execution(&self) -> Result<()> {
        let mut dirty = Vec::new();
        if !self.host_dirty_user_hosts.is_empty() {
            dirty.push(format!(
                "Users: {}",
                self.host_dirty_user_hosts.iter().cloned().collect::<Vec<_>>().join(", ")
            ));
        }
        if !self.host_dirty_runtime_hosts.is_empty() {
            dirty.push(format!(
                "Hosts: {}",
                self.host_dirty_runtime_hosts
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if !self.package_dirty_users.is_empty() {
            dirty.push(format!(
                "Packages: {}",
                self.package_dirty_users.iter().cloned().collect::<Vec<_>>().join(", ")
            ));
        }
        if !self.home_dirty_users.is_empty() {
            dirty.push(format!(
                "Home: {}",
                self.home_dirty_users.iter().cloned().collect::<Vec<_>>().join(", ")
            ));
        }

        if dirty.is_empty() {
            return Ok(());
        }

        anyhow::bail!(
            "仍有未保存修改；请先保存后再执行：{}",
            dirty.join(" | ")
        )
    }

    fn clean_etc_dir_keep_hardware(&self) -> Result<()> {
        if self.context.etc_root.as_os_str().is_empty()
            || self.context.etc_root == PathBuf::from("/")
        {
            anyhow::bail!(
                "ETC_ROOT 无效，拒绝清理：{}",
                self.context.etc_root.display()
            );
        }
        if !self.context.etc_root.is_dir() {
            return Ok(());
        }

        let preserve = std::env::temp_dir().join(format!(
            "mcbctl-hw-preserve-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0)
        ));
        fs::create_dir_all(&preserve)
            .with_context(|| format!("failed to create {}", preserve.display()))?;

        let etc_hw = self.context.etc_root.join("hardware-configuration.nix");
        if etc_hw.is_file() {
            fs::copy(&etc_hw, preserve.join("hardware-configuration.nix"))
                .with_context(|| format!("failed to preserve {}", etc_hw.display()))?;
        }

        for entry in fs::read_dir(&self.context.etc_root)
            .with_context(|| format!("failed to read {}", self.context.etc_root.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            let is_hw = path.file_name().and_then(|name| name.to_str())
                == Some("hardware-configuration.nix");
            if is_hw {
                continue;
            }
            if path.is_dir() {
                fs::remove_dir_all(&path)
                    .with_context(|| format!("failed to remove {}", path.display()))?;
            } else {
                fs::remove_file(&path)
                    .with_context(|| format!("failed to remove {}", path.display()))?;
            }
        }

        let preserved_root = preserve.join("hardware-configuration.nix");
        if preserved_root.is_file() {
            fs::copy(&preserved_root, &etc_hw)
                .with_context(|| format!("failed to restore {}", etc_hw.display()))?;
        }
        fs::remove_dir_all(preserve).ok();
        Ok(())
    }

    fn run_sibling_in_repo(
        &self,
        name: &str,
        args: &[String],
    ) -> Result<std::process::ExitStatus> {
        let binary = resolve_sibling_binary(name)?;
        std::process::Command::new(&binary)
            .args(args)
            .current_dir(&self.context.repo_root)
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status()
            .with_context(|| format!("failed to run {}", binary.display()))
    }

    fn action_available(&self, action: ActionItem) -> bool {
        match action {
            ActionItem::SyncRepoToEtc => {
                self.context.repo_root != self.context.etc_root
                    && self.context.privilege_mode != "rootless"
            }
            ActionItem::RebuildCurrentHost => !self.context.current_host.is_empty(),
            _ => true,
        }
    }

    fn action_command_preview(&self, action: ActionItem) -> Option<String> {
        match action {
            ActionItem::FlakeCheck => Some(format!(
                "nix --extra-experimental-features 'nix-command flakes' flake check path:{}",
                self.context.repo_root.display()
            )),
            ActionItem::FlakeUpdate => Some(format!(
                "nix --extra-experimental-features 'nix-command flakes' flake update --flake {}",
                self.context.repo_root.display()
            )),
            ActionItem::UpdateUpstreamCheck => Some("update-upstream-apps --check".to_string()),
            ActionItem::UpdateUpstreamPins => Some("update-upstream-apps".to_string()),
            ActionItem::SyncRepoToEtc => self
                .manual_repo_sync_plan()
                .map(|plan| plan.command_preview()),
            ActionItem::RebuildCurrentHost => {
                let action = if self.context.privilege_mode == "rootless" {
                    DeployAction::Build
                } else {
                    DeployAction::Switch
                };
                let plan = NixosRebuildPlan {
                    action,
                    upgrade: false,
                    flake_root: if self.context.repo_root == self.context.etc_root {
                        self.context.repo_root.clone()
                    } else {
                        self.context.etc_root.clone()
                    },
                    target_host: self.context.current_host.clone(),
                };
                Some(plan.command_preview(self.should_use_sudo()))
            }
            ActionItem::LaunchDeployWizard => Some("mcb-deploy".to_string()),
        }
    }
}

fn cycle_enum<T: Copy + Eq>(current: &mut T, all: &[T], delta: i8) {
    let Some(index) = all.iter().position(|item| item == current) else {
        return;
    };
    let len = all.len() as isize;
    let next = (index as isize + delta as isize).rem_euclid(len) as usize;
    *current = all[next];
}

fn bool_label(value: bool) -> &'static str {
    if value { "开启" } else { "关闭" }
}

fn default_target_host(context: &AppContext) -> String {
    if context
        .hosts
        .iter()
        .any(|host| host == &context.current_host)
    {
        return context.current_host.clone();
    }
    if context.hosts.iter().any(|host| host == "nixos") {
        return "nixos".to_string();
    }
    context
        .hosts
        .first()
        .cloned()
        .unwrap_or_else(|| context.current_host.clone())
}

fn default_package_user_index(context: &AppContext) -> usize {
    if let Some(index) = context
        .users
        .iter()
        .position(|user| user == &context.current_user)
    {
        return index;
    }
    if let Some(index) = context.users.iter().position(|user| user == "mcbnixos") {
        return index;
    }
    0
}

fn format_string_list(items: &[String]) -> String {
    if items.is_empty() {
        "无".to_string()
    } else {
        items.join(", ")
    }
}

fn serialize_string_list(items: &[String]) -> String {
    items.join(", ")
}

fn format_string_map(items: &BTreeMap<String, String>) -> String {
    if items.is_empty() {
        "无".to_string()
    } else {
        items
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn serialize_string_map(items: &BTreeMap<String, String>) -> String {
    items
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_u16_map(items: &BTreeMap<String, u16>) -> String {
    if items.is_empty() {
        "无".to_string()
    } else {
        items
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn serialize_u16_map(items: &BTreeMap<String, u16>) -> String {
    items
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn parse_string_list(raw: &str) -> Vec<String> {
    dedup_string_list(
        raw.split(',')
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(ToOwned::to_owned)
            .collect(),
    )
}

fn dedup_string_list(items: Vec<String>) -> Vec<String> {
    let mut output = Vec::new();
    for item in items {
        if !output.contains(&item) {
            output.push(item);
        }
    }
    output
}

fn has_duplicates(items: &[String]) -> bool {
    let mut seen = BTreeSet::new();
    for item in items {
        if !seen.insert(item) {
            return true;
        }
    }
    false
}

fn parse_string_map(raw: &str) -> Result<BTreeMap<String, String>> {
    let mut output = BTreeMap::new();
    if raw.trim().is_empty() {
        return Ok(output);
    }

    for part in raw.split(',') {
        let piece = part.trim();
        if piece.is_empty() {
            continue;
        }
        let Some((key, value)) = piece.split_once('=') else {
            anyhow::bail!("映射项必须是 user=value 形式：{piece}");
        };
        let key = key.trim();
        let value = value.trim();
        if key.is_empty() || value.is_empty() {
            anyhow::bail!("映射项不能为空：{piece}");
        }
        output.insert(key.to_string(), value.to_string());
    }

    Ok(output)
}

fn parse_u16_map(raw: &str) -> Result<BTreeMap<String, u16>> {
    let mut output = BTreeMap::new();
    if raw.trim().is_empty() {
        return Ok(output);
    }

    for part in raw.split(',') {
        let piece = part.trim();
        if piece.is_empty() {
            continue;
        }
        let Some((key, value)) = piece.split_once('=') else {
            anyhow::bail!("端口映射必须是 user=1053 形式：{piece}");
        };
        let key = key.trim();
        let value = value.trim();
        if key.is_empty() || value.is_empty() {
            anyhow::bail!("端口映射项不能为空：{piece}");
        }
        let port = value
            .parse::<u16>()
            .with_context(|| format!("无效端口：{value}"))?;
        output.insert(key.to_string(), port);
    }

    Ok(output)
}

fn parse_gpu_modes(raw: &str) -> Result<Vec<String>> {
    let modes = parse_string_list(raw);
    for mode in &modes {
        if !matches!(mode.as_str(), "igpu" | "hybrid" | "dgpu") {
            anyhow::bail!("无效 GPU 特化模式：{mode}");
        }
    }
    Ok(modes)
}

fn empty_to_none(value: &str) -> Option<String> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value.trim().to_string())
    }
}

fn nonempty_label(value: &str) -> String {
    if value.trim().is_empty() {
        "无".to_string()
    } else {
        value.to_string()
    }
}

fn nonempty_opt_label(value: Option<&str>) -> String {
    value
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| "无".to_string())
}

fn normalize_package_group_name(input: &str) -> String {
    let mut output = String::new();
    let mut last_was_dash = false;

    for ch in input.chars().flat_map(char::to_lowercase) {
        let mapped = match ch {
            'a'..='z' | '0'..='9' => Some(ch),
            '-' | '_' | ' ' | '/' | '.' => Some('-'),
            _ => None,
        };

        let Some(ch) = mapped else {
            continue;
        };

        if ch == '-' {
            if output.is_empty() || last_was_dash {
                continue;
            }
            last_was_dash = true;
            output.push(ch);
        } else {
            last_was_dash = false;
            output.push(ch);
        }
    }

    while output.ends_with('-') {
        output.pop();
    }

    output
}

fn display_path(path: Option<PathBuf>) -> String {
    path.map(|path| path.display().to_string())
        .unwrap_or_else(|| "无".to_string())
}

fn is_local_overlay_entry(entry: &CatalogEntry) -> bool {
    let source = entry.source_label();
    source.starts_with("local/") || source.starts_with("overlay/") || source.starts_with("managed/")
}

fn refresh_local_catalog_indexes(context: &mut AppContext, local_entry_ids: &BTreeSet<String>) {
    let mut categories = BTreeSet::new();
    let mut sources = BTreeSet::new();

    for entry in &context.catalog_entries {
        if !local_entry_ids.contains(&entry.id) {
            continue;
        }
        categories.insert(entry.category.clone());
        sources.insert(entry.source_label().to_string());
    }

    context.catalog_categories = categories.into_iter().collect();
    context.catalog_sources = sources.into_iter().collect();
}

fn cycle_string_value(current: &str, all: &[String], delta: i8) -> Option<String> {
    if all.is_empty() {
        return None;
    }
    let index = all.iter().position(|item| item == current).unwrap_or(0);
    let len = all.len() as isize;
    let next = (index as isize + delta as isize).rem_euclid(len) as usize;
    Some(all[next].clone())
}

fn cycle_string(current: &mut String, all: &[String], delta: i8) {
    if all.is_empty() {
        return;
    }
    let index = all.iter().position(|item| item == current).unwrap_or(0);
    let len = all.len() as isize;
    let next = (index as isize + delta as isize).rem_euclid(len) as usize;
    *current = all[next].clone();
}
