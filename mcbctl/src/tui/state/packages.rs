use super::*;

mod browse;
#[path = "packages/mutate_groups.rs"]
mod mutate_groups;
#[path = "packages/mutate_navigation.rs"]
mod mutate_navigation;
#[path = "packages/mutate_save.rs"]
mod mutate_save;
#[path = "packages/mutate_search.rs"]
mod mutate_search;

impl AppState {
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
