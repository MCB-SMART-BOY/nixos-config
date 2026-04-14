use super::*;

impl AppState {
    pub fn users_rows(&self) -> Vec<(String, String)> {
        let Some(settings) = self.current_host_settings().cloned() else {
            let unavailable = self
                .current_host_unavailable_value()
                .unwrap_or_else(|| "不可用".to_string());
            return vec![
                ("主机".to_string(), self.target_host.clone()),
                ("主用户".to_string(), unavailable.clone()),
                ("托管用户".to_string(), unavailable.clone()),
                ("管理员".to_string(), unavailable.clone()),
                ("主机角色".to_string(), unavailable.clone()),
                ("用户 linger".to_string(), unavailable),
            ];
        };
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

    pub fn users_summary_lines(&self) -> Vec<String> {
        let mut lines = vec![
            format!("当前主机：{}", self.target_host),
            format!("目标文件：{}", display_path(self.current_host_users_path())),
            format!(
                "仓库内可选用户：{}",
                format_string_list(&self.context.users)
            ),
        ];

        if let Some(message) = self.current_host_settings_unavailable_message() {
            lines.push(format!("状态：{message}"));
        } else {
            if self.host_dirty_user_hosts.contains(&self.target_host) {
                lines.push("状态：当前主机的用户结构分片有未保存修改".to_string());
            } else {
                lines.push("状态：当前主机的用户结构分片没有未保存修改".to_string());
            }
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

    pub fn adjust_users_field(&mut self, delta: i8) {
        match self.users_focus {
            0 => self.switch_target_host(delta),
            1 => {
                if self.block_when_current_host_settings_unavailable("无法调整主用户") {
                    return;
                }
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
                let Some(settings) = self.current_host_settings_mut() else {
                    self.status = format!(
                        "无法调整主用户：{}",
                        self.host_settings_unavailable_message(&self.target_host)
                    );
                    return;
                };
                settings.primary_user = next.clone();
                if !settings.users.contains(&next) {
                    settings.users.insert(0, next.clone());
                }
                self.host_dirty_user_hosts.insert(self.target_host.clone());
                self.status = format!("当前主用户已切换为：{next}");
            }
            4 => {
                if self.block_when_current_host_settings_unavailable("无法调整主机角色") {
                    return;
                }
                let options = vec!["desktop".to_string(), "server".to_string()];
                let current = self
                    .current_host_settings()
                    .map(|settings| settings.host_role.clone())
                    .unwrap_or_else(|| "desktop".to_string());
                let Some(next) = cycle_string_value(&current, &options, delta) else {
                    return;
                };
                let Some(settings) = self.current_host_settings_mut() else {
                    self.status = format!(
                        "无法调整主机角色：{}",
                        self.host_settings_unavailable_message(&self.target_host)
                    );
                    return;
                };
                settings.host_role = next.clone();
                self.host_dirty_user_hosts.insert(self.target_host.clone());
                self.status = format!("当前主机角色已切换为：{next}");
            }
            5 => {
                let Some(settings) = self.current_host_settings_mut() else {
                    self.status = format!(
                        "无法调整 user linger：{}",
                        self.host_settings_unavailable_message(&self.target_host)
                    );
                    return;
                };
                settings.user_linger = !settings.user_linger;
                self.host_dirty_user_hosts.insert(self.target_host.clone());
                self.status = "当前主机的 user linger 已切换。".to_string();
            }
            _ => {}
        }
    }

    pub fn open_users_text_edit(&mut self) {
        let Some(settings) = self.current_host_settings().cloned() else {
            self.status = format!(
                "无法编辑用户结构：{}",
                self.host_settings_unavailable_message(&self.target_host)
            );
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

    pub fn save_current_host_users(&mut self) -> Result<()> {
        let host = self.target_host.clone();
        let errors = self.host_configuration_validation_errors(&host);
        if !errors.is_empty() {
            self.status = format!(
                "当前主机的整机配置未通过校验，users 分片未写入：{}",
                errors.join("；")
            );
            return Ok(());
        }

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

    fn confirm_users_text_edit(&mut self) {
        let Some(mode) = self.users_text_mode else {
            return;
        };
        let parsed = parse_string_list(&self.host_text_input);
        let Some(settings) = self.current_host_settings_mut() else {
            self.users_text_mode = None;
            self.host_text_input.clear();
            self.status = format!(
                "无法确认用户结构编辑：{}",
                self.host_settings_unavailable_message(&self.target_host)
            );
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

    fn current_host_user_validation_errors(&self) -> Vec<String> {
        if let Some(error) = self.current_host_settings_unavailable_message() {
            return vec![error];
        }
        let Some(settings) = self.current_host_settings() else {
            return vec!["当前主机没有可用设置。".to_string()];
        };

        validate_host_user_settings(settings)
    }
}

pub(super) fn validate_host_user_settings(settings: &HostManagedSettings) -> Vec<String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{managed_file_is_valid, managed_file_kind};
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn confirm_managed_users_edit_reassigns_primary_and_filters_admins() {
        let mut state = test_state(Path::new("/repo"));
        state.users_text_mode = Some(UsersTextMode::ManagedUsers);
        state.host_text_input = "bob, carol, bob".to_string();

        state.confirm_users_text_edit();

        let settings = &state.host_settings_by_name["demo"];
        assert_eq!(settings.users, vec!["bob".to_string(), "carol".to_string()]);
        assert_eq!(settings.primary_user, "bob");
        assert_eq!(settings.admin_users, vec!["bob".to_string()]);
        assert!(state.host_dirty_user_hosts.contains("demo"));
        assert!(state.users_text_mode.is_none());
        assert!(state.host_text_input.is_empty());
        assert_eq!(state.status, "用户结构字段已更新。");
    }

    #[test]
    fn confirm_admin_users_edit_filters_unknown_entries() {
        let mut state = test_state(Path::new("/repo"));
        state.users_text_mode = Some(UsersTextMode::AdminUsers);
        state.host_text_input = "bob, carol, alice, bob".to_string();

        state.confirm_users_text_edit();

        assert_eq!(
            state.host_settings_by_name["demo"].admin_users,
            vec!["bob".to_string(), "alice".to_string()]
        );
        assert!(state.host_dirty_user_hosts.contains("demo"));
    }

    #[test]
    fn save_current_host_users_rejects_invalid_combined_configuration() -> Result<()> {
        let root = create_temp_repo("mcbctl-host-users-invalid")?;
        let mut state = test_state(&root);
        if let Some(settings) = state.host_settings_by_name.get_mut("demo") {
            settings.proxy_mode = "tun".to_string();
            settings.tun_interface.clear();
        }
        state.host_dirty_user_hosts.insert("demo".to_string());

        state.save_current_host_users()?;

        let users_path = managed_host_users_path(&root, "demo");
        assert!(!users_path.exists());
        assert!(state.host_dirty_user_hosts.contains("demo"));
        assert!(state.status.contains("整机配置未通过校验"));
        assert!(state.status.contains("主 TUN 接口不能为空"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn save_current_host_users_writes_managed_fragment_and_clears_dirty() -> Result<()> {
        let root = create_temp_repo("mcbctl-host-users-save")?;
        let mut state = test_state(&root);
        state.host_dirty_user_hosts.insert("demo".to_string());

        state.save_current_host_users()?;

        let users_path = managed_host_users_path(&root, "demo");
        let content = std::fs::read_to_string(&users_path)?;
        assert_eq!(managed_file_kind(&content), Some("host-users"));
        assert!(managed_file_is_valid(&content));
        assert!(content.contains("mcb.user = lib.mkForce \"alice\";"));
        assert!(content.contains("mcb.adminUsers = lib.mkForce [ \"alice\" \"bob\" ];"));
        assert!(!state.host_dirty_user_hosts.contains("demo"));
        assert!(state.status.contains(&users_path.display().to_string()));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn adjust_users_field_refuses_to_modify_unavailable_host_settings() {
        let mut state = test_state(Path::new("/repo"));
        state.host_settings_by_name.clear();
        state.host_settings_errors_by_name.insert(
            "demo".to_string(),
            "nix eval for host demo failed".to_string(),
        );
        state.users_focus = 1;

        state.adjust_users_field(1);

        assert!(!state.host_dirty_user_hosts.contains("demo"));
        assert!(state.status.contains("无法调整主用户"));
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
                users: vec!["alice".to_string(), "bob".to_string(), "carol".to_string()],
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
            users: vec!["alice".to_string(), "bob".to_string()],
            admin_users: vec!["alice".to_string(), "bob".to_string()],
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
