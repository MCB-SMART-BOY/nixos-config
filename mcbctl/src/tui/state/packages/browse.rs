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
}
