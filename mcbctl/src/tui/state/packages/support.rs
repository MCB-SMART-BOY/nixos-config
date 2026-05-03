use super::*;

impl AppState {
    pub(super) fn package_edit_next_step(&self) -> &'static str {
        "继续调整列表，完成后按 s 保存。"
    }

    pub(super) fn package_browse_next_step(&self) -> &'static str {
        "继续浏览列表，或切换其他过滤条件。"
    }

    pub(super) fn package_search_next_step(&self) -> &'static str {
        "继续输入关键词，或按 Enter / r 刷新结果。"
    }

    pub(super) fn package_result_next_step(&self) -> &'static str {
        "继续编辑 Packages，或切到 Apply / Overview 复查。"
    }

    pub(super) fn package_group_input_next_step(&self) -> &'static str {
        "继续输入组名，按 Enter 确认，Esc 取消。"
    }

    pub(super) fn package_workflow_confirm_next_step(&self) -> &'static str {
        "按 Enter 确认批量加入，或 Esc 取消。"
    }

    pub(super) fn set_package_feedback_with_next_step(
        &mut self,
        level: UiFeedbackLevel,
        message: impl Into<String>,
        next_step: impl Into<String>,
    ) {
        self.set_feedback_with_next_step(level, UiFeedbackScope::Packages, message, next_step);
    }

    pub(super) fn current_package_action_summary_model(&self) -> Option<PackageActionSummaryModel> {
        if self.feedback.scope != UiFeedbackScope::Packages || self.feedback.message.is_empty() {
            return None;
        }

        let next_step = self
            .feedback
            .next_step
            .clone()
            .unwrap_or_else(|| self.package_browse_next_step().to_string());

        if self.feedback.level == UiFeedbackLevel::Info && self.feedback.next_step.is_none() {
            return None;
        }

        Some(PackageActionSummaryModel {
            latest_result: self.feedback.message.clone(),
            next_step,
        })
    }

    pub(super) fn package_selection_status(&self) -> String {
        let Some(user) = self.current_package_user() else {
            return "无可用用户".to_string();
        };

        if !self.current_package_managed_guard_errors().is_empty() {
            return "受管保护".to_string();
        }
        if self.package_dirty_users.contains(user) {
            return "未保存".to_string();
        }
        if self.package_search_mode {
            return "搜索输入".to_string();
        }
        if self.package_mode == PackageDataMode::Search {
            let query = self.package_search.trim();
            if query.is_empty() {
                return "搜索待输入".to_string();
            }
            return format!("搜索：{query}");
        }
        if let Some(workflow) = self.current_package_workflow_filter() {
            return format!("流程过滤：{}", self.package_workflow_display(workflow));
        }
        if let Some(group) = self.current_package_group_filter() {
            return format!("组过滤：{}", self.package_group_display(group));
        }
        if let Some(source) = self.current_package_source_filter() {
            return format!("来源过滤：{source}");
        }
        if self.package_filtered_count() == 0 {
            return "无匹配项".to_string();
        }

        "就绪".to_string()
    }

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

    pub(super) fn package_workflow_meta(&self, workflow: &str) -> Option<&WorkflowMeta> {
        self.context.catalog_workflows.get(workflow)
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

    pub fn package_workflow_display(&self, workflow: &str) -> String {
        let label = self.package_workflow_label(workflow);
        if label == workflow {
            label
        } else {
            format!("{label} [{workflow}]")
        }
    }

    pub fn package_workflow_label(&self, workflow: &str) -> String {
        self.package_workflow_meta(workflow)
            .map(|meta| meta.label.clone())
            .unwrap_or_else(|| workflow.to_string())
    }

    pub fn package_workflow_description(&self, workflow: &str) -> Option<&str> {
        self.package_workflow_meta(workflow)
            .and_then(|meta| meta.description.as_deref())
    }

    pub fn package_entry_workflow_labels(&self, entry: &CatalogEntry) -> Vec<String> {
        let mut workflows = entry
            .workflow_tags
            .iter()
            .map(|workflow| self.package_workflow_display(workflow))
            .collect::<Vec<_>>();
        workflows.sort();
        workflows
    }

    pub(super) fn compare_package_workflows(&self, left: &str, right: &str) -> Ordering {
        let left_meta = self.package_workflow_meta(left);
        let right_meta = self.package_workflow_meta(right);

        left_meta
            .map(|meta| meta.order)
            .unwrap_or(u32::MAX)
            .cmp(&right_meta.map(|meta| meta.order).unwrap_or(u32::MAX))
            .then_with(|| {
                self.package_workflow_label(left)
                    .cmp(&self.package_workflow_label(right))
            })
            .then_with(|| left.cmp(right))
    }

    pub(crate) fn package_workflows(&self) -> Vec<String> {
        let mut workflows: Vec<String> = self
            .context
            .catalog_entries
            .iter()
            .filter(|entry| self.package_local_entry_ids.contains(&entry.id))
            .flat_map(|entry| entry.workflow_tags.iter().cloned())
            .collect::<BTreeSet<String>>()
            .into_iter()
            .collect();
        workflows.sort_by(|left, right| self.compare_package_workflows(left, right));
        workflows
    }

    pub(super) fn package_local_entries_for_workflow(&self, workflow: &str) -> Vec<&CatalogEntry> {
        self.context
            .catalog_entries
            .iter()
            .filter(|entry| self.package_local_entry_ids.contains(&entry.id))
            .filter(|entry| entry.workflow_tags.iter().any(|tag| tag == workflow))
            .collect()
    }

    pub(crate) fn current_workflow_missing_package_rows(
        &self,
    ) -> Option<Vec<PackageWorkflowEntryRow>> {
        let workflow = self.current_package_workflow_filter()?;
        let selected = self.current_user_selection();

        Some(
            self.package_local_entries_for_workflow(workflow)
                .into_iter()
                .filter(|entry| {
                    !selected.is_some_and(|selection| selection.contains_key(&entry.id))
                })
                .map(|entry| PackageWorkflowEntryRow {
                    name: entry.name.clone(),
                    category: entry.category.clone(),
                    group_label: self.package_group_display(&self.default_group_for_entry(entry)),
                })
                .collect(),
        )
    }

    pub(crate) fn package_workflows_overview(&self) -> Vec<(String, usize, usize)> {
        let selected = self.current_user_selection();

        self.package_workflows()
            .into_iter()
            .map(|workflow| {
                let workflow_entries = self.package_local_entries_for_workflow(&workflow);
                let total_count = workflow_entries.len();
                let selected_count = workflow_entries
                    .into_iter()
                    .filter(|entry| {
                        selected.is_some_and(|selection| selection.contains_key(&entry.id))
                    })
                    .count();
                (workflow, total_count, selected_count)
            })
            .collect()
    }

    pub(crate) fn current_package_workflow_overview(&self) -> Option<(String, usize, usize)> {
        let workflow = self.current_package_workflow_filter()?;
        self.package_workflows_overview()
            .into_iter()
            .find(|(candidate, _, _)| candidate == workflow)
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

    pub(crate) fn package_groups_for_user(&self, user: &str) -> Vec<String> {
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

    pub(crate) fn clamp_package_cursor(&mut self) {
        let len = self.package_filtered_count();
        if len == 0 {
            self.package_cursor = 0;
        } else if self.package_cursor >= len {
            self.package_cursor = len - 1;
        }
    }
}
