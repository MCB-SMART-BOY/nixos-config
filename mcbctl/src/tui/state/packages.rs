use super::*;

impl AppState {
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
}
