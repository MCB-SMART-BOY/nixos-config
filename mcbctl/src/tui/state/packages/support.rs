use super::*;

impl AppState {
    pub(super) fn current_user_selection(&self) -> Option<&BTreeMap<String, String>> {
        let user = self.current_package_user()?;
        self.package_user_selections.get(user)
    }

    pub(super) fn package_base_indices(&self) -> Vec<(usize, &CatalogEntry)> {
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

    pub(super) fn package_total_count(&self) -> usize {
        match self.package_mode {
            PackageDataMode::Local => self.package_local_entry_ids.len(),
            PackageDataMode::Search => self.package_search_result_indices.len(),
        }
    }

    pub(super) fn merge_catalog_entries(
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

    pub(super) fn package_group_for_user(&self, user: &str, entry: &CatalogEntry) -> String {
        self.package_user_selections
            .get(user)
            .and_then(|selection| selection.get(&entry.id))
            .cloned()
            .unwrap_or_else(|| entry.group_key().to_string())
    }

    pub(super) fn default_group_for_entry(&self, entry: &CatalogEntry) -> String {
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

    pub(super) fn package_group_meta(&self, group: &str) -> Option<&GroupMeta> {
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

    pub(super) fn compare_package_groups(&self, left: &str, right: &str) -> Ordering {
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

    pub(super) fn package_groups_for_user(&self, user: &str) -> Vec<String> {
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
                if !path.is_file() || path.extension().is_none_or(|ext| ext != "nix") {
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
                if !path.is_file() || path.extension().is_none_or(|ext| ext != "nix") {
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

    pub(super) fn package_id_selected_anywhere(&self, entry_id: &str) -> bool {
        self.package_user_selections
            .values()
            .any(|selection| selection.contains_key(entry_id))
    }

    pub(super) fn sync_local_catalog_membership(&mut self, entry_id: &str) {
        let keep_local = self
            .context
            .catalog_entries
            .iter()
            .find(|entry| entry.id == entry_id)
            .is_some_and(is_local_overlay_entry)
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

    pub(super) fn clamp_package_cursor(&mut self) {
        let len = self.package_filtered_count();
        if len == 0 {
            self.package_cursor = 0;
        } else if self.package_cursor >= len {
            self.package_cursor = len - 1;
        }
    }
}
