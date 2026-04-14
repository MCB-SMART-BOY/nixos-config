use super::*;

impl AppState {
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
        let len = self.home_desktop_options().len();
        if len == 0 {
            self.home_focus = 0;
            return;
        }
        self.home_focus = (self.home_focus + 1) % len;
    }

    pub fn previous_home_field(&mut self) {
        let len = self.home_desktop_options().len();
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

        let locked_noctalia_path = if option_id == "noctalia.barProfile" {
            self.current_home_user_noctalia_override_path()
                .filter(|path| path.is_file())
        } else {
            None
        };
        let settings = self.home_settings_by_user.entry(user.clone()).or_default();
        match option_id.as_str() {
            "noctalia.barProfile" => {
                if let Some(path) = locked_noctalia_path {
                    self.status = format!(
                        "用户 {user} 的 Noctalia 顶栏由 {} 接管；Home 页不会覆盖它。",
                        path.display()
                    );
                    return;
                }
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
        self.home_desktop_options()
            .into_iter()
            .map(|option| {
                let value = self.home_option_value(option.id.as_str(), &settings);
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
        let desktop_options = self.home_desktop_options();
        if desktop_options.is_empty() {
            lines.push("当前没有可用的 Home 元数据选项。".to_string());
        } else {
            for option in &desktop_options {
                let value = self.home_option_value(option.id.as_str(), &settings);
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
        let guard_errors = self.current_home_managed_guard_errors();
        if self.current_home_user().is_none() {
            lines.push("受管保护：无可用目标".to_string());
        } else if guard_errors.is_empty() {
            lines.push("受管保护：通过".to_string());
        } else {
            lines.push("受管保护：存在问题".to_string());
            for err in guard_errors {
                lines.push(format!("- {err}"));
            }
        }
        if let Some(path) = self.current_home_user_noctalia_override_path()
            && path.is_file()
        {
            lines.push(format!(
                "Noctalia：当前用户由 {} 提供自定义布局，Home 页不会覆盖顶栏 profile。",
                path.display()
            ));
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

        let settings = self
            .home_settings_by_user
            .get(&user)
            .cloned()
            .unwrap_or_default();
        let mut settings = settings;
        if user_has_custom_noctalia_layout(&self.context.repo_root, &user) {
            settings.bar_profile = ManagedBarProfile::Inherit;
        }
        let path = managed_dir.join("settings/desktop.nix");
        if let Err(err) = ensure_managed_settings_layout(&managed_dir).and_then(|()| {
            write_managed_file(
                &path,
                "home-settings-desktop",
                &render_managed_desktop_file(&settings),
                &["# 机器管理的桌面设置分片"],
            )
        }) {
            self.status = format!("Home 未写入：{err:#}");
            return Ok(());
        }
        self.home_dirty_users.remove(&user);
        self.status = if let Some(override_path) = self.current_home_user_noctalia_override_path() {
            if override_path.is_file() {
                format!(
                    "已写入 {}；Noctalia 顶栏仍由 {} 接管。",
                    path.display(),
                    override_path.display()
                )
            } else {
                format!("已写入 {}", path.display())
            }
        } else {
            format!("已写入 {}", path.display())
        };
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

    fn home_desktop_options(&self) -> Vec<&HomeOptionMeta> {
        self.home_options_for_area("desktop")
    }

    fn current_home_option_id(&self) -> Option<&str> {
        self.home_desktop_options()
            .get(self.home_focus)
            .map(|option| option.id.as_str())
    }

    fn home_target_desktop_path(&self) -> Option<PathBuf> {
        let user = self.current_home_user()?;
        Some(managed_home_desktop_path(&self.context.repo_root, user))
    }

    fn current_home_user_has_custom_noctalia_layout(&self) -> bool {
        self.current_home_user()
            .is_some_and(|user| user_has_custom_noctalia_layout(&self.context.repo_root, user))
    }

    fn current_home_user_noctalia_override_path(&self) -> Option<PathBuf> {
        let user = self.current_home_user()?;
        Some(user_noctalia_override_path(&self.context.repo_root, user))
    }

    pub(crate) fn current_home_managed_guard_errors(&self) -> Vec<String> {
        let Some(user) = self.current_home_user() else {
            return Vec::new();
        };
        let settings_dir = self
            .context
            .repo_root
            .join("home/users")
            .join(user)
            .join("managed/settings");

        [
            ("default.nix", "home-settings-default"),
            ("desktop.nix", "home-settings-desktop"),
            ("session.nix", "home-settings-session"),
            ("mime.nix", "home-settings-mime"),
        ]
        .into_iter()
        .filter_map(|(name, kind)| {
            crate::ensure_existing_managed_file(&settings_dir.join(name), kind)
                .err()
                .map(|err| err.to_string())
        })
        .collect()
    }

    fn home_option_value(&self, option_id: &str, settings: &HomeManagedSettings) -> String {
        match option_id {
            "noctalia.barProfile" if self.current_home_user_has_custom_noctalia_layout() => {
                "由自定义布局接管".to_string()
            }
            "noctalia.barProfile" => settings.bar_profile.label().to_string(),
            "desktop.enableZed" => settings.enable_zed_entry.label().to_string(),
            "desktop.enableYesPlayMusic" => settings.enable_yesplaymusic_entry.label().to_string(),
            _ => "未接入".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{managed_file_is_valid, managed_file_kind};
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn home_rows_show_custom_noctalia_layout_as_locked() -> Result<()> {
        let root = create_temp_repo("mcbctl-home-state")?;
        let mut state = test_state(&root);
        std::fs::write(root.join("home/users/alice/noctalia.nix"), "{ ... }: { }\n")?;
        state.home_settings_by_user.insert(
            "alice".to_string(),
            HomeManagedSettings {
                bar_profile: ManagedBarProfile::Default,
                ..HomeManagedSettings::default()
            },
        );

        let rows = state.home_rows();
        assert_eq!(rows[0].1, "由自定义布局接管");

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn adjust_home_field_refuses_to_edit_noctalia_profile_when_custom_layout_exists() -> Result<()>
    {
        let root = create_temp_repo("mcbctl-home-lock")?;
        std::fs::write(root.join("home/users/alice/noctalia.nix"), "{ ... }: { }\n")?;
        let mut state = test_state(&root);

        state.adjust_home_field(1);

        assert!(state.status.contains("Home 页不会覆盖它"));
        assert!(!state.home_dirty_users.contains("alice"));
        assert_eq!(
            state.home_settings_by_user["alice"].bar_profile,
            ManagedBarProfile::Inherit
        );

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn save_current_home_settings_for_custom_layout_forces_inherit() -> Result<()> {
        let root = create_temp_repo("mcbctl-home-save")?;
        std::fs::write(root.join("home/users/alice/noctalia.nix"), "{ ... }: { }\n")?;
        let mut state = test_state(&root);
        state.home_settings_by_user.insert(
            "alice".to_string(),
            HomeManagedSettings {
                bar_profile: ManagedBarProfile::Default,
                enable_zed_entry: ManagedToggle::Enabled,
                enable_yesplaymusic_entry: ManagedToggle::Inherit,
            },
        );
        state.home_dirty_users.insert("alice".to_string());

        state.save_current_home_settings()?;

        let desktop_path = managed_home_desktop_path(&root, "alice");
        let content = std::fs::read_to_string(desktop_path)?;
        assert_eq!(managed_file_kind(&content), Some("home-settings-desktop"));
        assert!(managed_file_is_valid(&content));
        assert!(content.contains("# managed-setting: noctalia.barProfile=inherit"));
        assert!(!content.contains("mcb.noctalia.barProfile = \"default\";"));
        assert!(content.contains("mcb.desktopEntries.enableZed = lib.mkForce true;"));
        assert!(state.status.contains("Noctalia 顶栏仍由"));
        assert!(!state.home_dirty_users.contains("alice"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn save_current_home_settings_rejects_tampered_managed_sibling_and_keeps_dirty() -> Result<()> {
        let root = create_temp_repo("mcbctl-home-save-tampered")?;
        let mut state = test_state(&root);
        let settings_dir = root.join("home/users/alice/managed/settings");
        std::fs::create_dir_all(&settings_dir)?;
        std::fs::write(
            settings_dir.join("session.nix"),
            "{ lib, ... }: { home.sessionVariables.DEMO = \"1\"; }\n",
        )?;
        state.home_dirty_users.insert("alice".to_string());

        state.save_current_home_settings()?;

        assert!(state.home_dirty_users.contains("alice"));
        assert!(state.status.contains("Home 未写入"));
        assert!(state.status.contains("home-settings-session"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn home_summary_lines_surface_managed_guard_errors_before_save() -> Result<()> {
        let root = create_temp_repo("mcbctl-home-summary-tampered")?;
        let settings_dir = root.join("home/users/alice/managed/settings");
        std::fs::create_dir_all(&settings_dir)?;
        std::fs::write(
            settings_dir.join("session.nix"),
            "{ lib, ... }: { home.sessionVariables.DEMO = \"1\"; }\n",
        )?;
        let state = test_state(&root);

        let lines = state.home_summary_lines();

        assert!(lines.iter().any(|line| line == "受管保护：存在问题"));
        assert!(
            lines
                .iter()
                .any(|line| line.contains("home-settings-session"))
        );

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn save_current_home_settings_without_override_keeps_selected_profile() -> Result<()> {
        let root = create_temp_repo("mcbctl-home-save-open")?;
        let mut state = test_state(&root);
        state.home_settings_by_user.insert(
            "alice".to_string(),
            HomeManagedSettings {
                bar_profile: ManagedBarProfile::Default,
                ..HomeManagedSettings::default()
            },
        );

        state.save_current_home_settings()?;

        let desktop_path = managed_home_desktop_path(&root, "alice");
        let content = std::fs::read_to_string(desktop_path)?;
        assert_eq!(managed_file_kind(&content), Some("home-settings-desktop"));
        assert!(content.contains("# managed-setting: noctalia.barProfile=default"));
        assert!(content.contains("mcb.noctalia.barProfile = \"default\";"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    fn test_state(root: &Path) -> AppState {
        let mut settings = BTreeMap::new();
        settings.insert("alice".to_string(), HomeManagedSettings::default());

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
                catalog_home_options: vec![
                    HomeOptionMeta {
                        id: "noctalia.barProfile".to_string(),
                        label: "Noctalia 顶栏".to_string(),
                        description: Some("test".to_string()),
                        area: "desktop".to_string(),
                        order: 10,
                    },
                    HomeOptionMeta {
                        id: "desktop.enableZed".to_string(),
                        label: "Zed 桌面入口".to_string(),
                        description: Some("test".to_string()),
                        area: "desktop".to_string(),
                        order: 20,
                    },
                ],
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
            host_settings_by_name: BTreeMap::new(),
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
            home_settings_by_user: settings,
            home_dirty_users: BTreeSet::new(),
            actions_focus: 0,
            overview_repo_integrity: OverviewCheckState::NotRun,
            overview_doctor: OverviewCheckState::NotRun,
            feedback: UiFeedback::default(),
            status: String::new(),
        }
    }

    fn create_temp_repo(prefix: &str) -> Result<PathBuf> {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let root = std::env::temp_dir().join(format!("{prefix}-{}-{unique}", std::process::id()));
        std::fs::create_dir_all(root.join("home/users/alice"))?;
        Ok(root)
    }
}
