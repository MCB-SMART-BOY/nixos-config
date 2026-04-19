use super::*;

impl AppState {
    pub(super) fn confirm_package_group_creation(&mut self) {
        let Some(user) = self.current_package_user().map(ToOwned::to_owned) else {
            self.package_group_create_mode = false;
            self.package_group_input.clear();
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Error,
                "Packages 页没有可操作的用户目录。",
                "先补可用 user 目标，或切到其他编辑页。",
            );
            return;
        };
        let Some(entry) = self.current_package_entry().cloned() else {
            self.package_group_create_mode = false;
            self.package_group_input.clear();
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Info,
                "当前过滤条件下没有可新建分组的软件。",
                self.package_browse_next_step(),
            );
            return;
        };

        let normalized = normalize_package_group_name(&self.package_group_input);
        if normalized.is_empty() {
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Error,
                "Packages 组名不能为空；建议使用字母、数字和连字符，例如 research-writing。",
                self.package_group_input_next_step(),
            );
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
        self.set_package_feedback_with_next_step(
            UiFeedbackLevel::Success,
            if existed {
                format!(
                    "Packages 已将 {} 指向现有组 {}。",
                    entry.name,
                    self.package_group_display(&normalized)
                )
            } else {
                format!(
                    "Packages 已新建组 {}，并把 {} 放进去。",
                    self.package_group_display(&normalized),
                    entry.name
                )
            },
            self.package_edit_next_step(),
        );
    }

    pub(super) fn confirm_package_group_rename(&mut self) {
        let Some(user) = self.current_package_user().map(ToOwned::to_owned) else {
            self.reset_package_group_edit_state();
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Error,
                "Packages 页没有可操作的用户目录。",
                "先补可用 user 目标，或切到其他编辑页。",
            );
            return;
        };

        let old_group = self.package_group_rename_source.clone();
        let normalized = normalize_package_group_name(&self.package_group_input);
        if normalized.is_empty() {
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Error,
                "Packages 组名不能为空；建议使用字母、数字和连字符，例如 database-tools。",
                self.package_group_input_next_step(),
            );
            return;
        }
        if normalized == old_group {
            self.reset_package_group_edit_state();
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Info,
                format!(
                    "Packages 组名未变化：{}",
                    self.package_group_display(&old_group)
                ),
                self.package_browse_next_step(),
            );
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
        self.set_package_feedback_with_next_step(
            UiFeedbackLevel::Success,
            format!(
                "Packages 已将组 {} 重命名为 {}，影响 {renamed_count} 个软件。",
                self.package_group_display(&old_group),
                self.package_group_display(&normalized)
            ),
            self.package_edit_next_step(),
        );
    }

    pub(super) fn reset_package_group_edit_state(&mut self) {
        self.package_group_create_mode = false;
        self.package_group_rename_mode = false;
        self.package_group_rename_source.clear();
        self.package_group_input.clear();
    }

    pub(crate) fn ensure_valid_package_group_filter(&mut self) {
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

    pub(crate) fn ensure_valid_package_workflow_filter(&mut self) {
        let Some(filter) = self.package_workflow_filter.clone() else {
            return;
        };

        if self.package_mode == PackageDataMode::Search
            || !self.package_workflows().contains(&filter)
        {
            self.package_workflow_filter = None;
        }
    }
}
