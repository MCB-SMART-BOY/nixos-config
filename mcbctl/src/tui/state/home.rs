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
        ensure_managed_settings_layout(&managed_dir)?;

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
        write_managed_file(
            &path,
            "home-settings-desktop",
            &render_managed_desktop_file(&settings),
            &["# 机器管理的桌面设置分片"],
        )?;
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
