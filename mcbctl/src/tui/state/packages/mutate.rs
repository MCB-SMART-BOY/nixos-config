use super::*;

impl AppState {
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
}
