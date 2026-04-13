use super::*;

impl AppState {
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

    fn current_host_user_validation_errors(&self) -> Vec<String> {
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
