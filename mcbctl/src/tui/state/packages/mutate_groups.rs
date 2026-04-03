use super::*;

impl AppState {
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
}
