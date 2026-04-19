use super::*;

impl AppState {
    pub fn next_package_user(&mut self) {
        if self.context.users.is_empty() {
            return;
        }
        self.package_user_index = (self.package_user_index + 1) % self.context.users.len();
        self.ensure_valid_package_group_filter();
        self.ensure_valid_package_workflow_filter();
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
        self.ensure_valid_package_workflow_filter();
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
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Info,
                "Packages 在 nixpkgs 搜索模式下不使用来源过滤。",
                "按 f 切回本地覆盖/已声明后再调整来源。",
            );
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
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Info,
                "Packages 已清空来源过滤。",
                self.package_browse_next_step(),
            );
        } else {
            self.package_source_filter = Some(next.clone());
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Info,
                format!("Packages 来源过滤：{next}"),
                self.package_browse_next_step(),
            );
        }
        self.clamp_package_cursor();
    }

    pub fn adjust_package_group_filter(&mut self, delta: i8) {
        let Some(user) = self.current_package_user() else {
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Error,
                "Packages 页没有可操作的用户目录。",
                "先补可用 user 目标，或切到其他编辑页。",
            );
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
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Info,
                "Packages 已清空组过滤。",
                self.package_browse_next_step(),
            );
        } else {
            self.package_group_filter = Some(next.clone());
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Info,
                format!("Packages 组过滤：{}", self.package_group_display(&next)),
                self.package_browse_next_step(),
            );
        }
        self.clamp_package_cursor();
    }

    pub fn adjust_package_workflow_filter(&mut self, delta: i8) {
        if self.package_mode == PackageDataMode::Search {
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Info,
                "Packages 在 nixpkgs 搜索模式下不使用工作流过滤。",
                "按 f 切回本地覆盖/已声明后再调整 workflow。",
            );
            return;
        }

        let mut options = vec![String::new()];
        options.extend(self.package_workflows());

        let current = self.package_workflow_filter.clone().unwrap_or_default();
        let Some(next) = cycle_string_value(&current, &options, delta) else {
            return;
        };

        if next.is_empty() {
            self.package_workflow_filter = None;
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Info,
                "Packages 已清空工作流过滤。",
                self.package_browse_next_step(),
            );
        } else {
            self.package_workflow_filter = Some(next.clone());
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Info,
                format!(
                    "Packages 流程过滤：{}",
                    self.package_workflow_display(&next)
                ),
                self.package_browse_next_step(),
            );
        }
        self.clamp_package_cursor();
    }

    pub fn focus_current_selected_group(&mut self) {
        let Some(group) = self.package_group_for_current_entry() else {
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Info,
                "当前过滤条件下没有可聚焦分组的软件。",
                self.package_browse_next_step(),
            );
            return;
        };

        self.package_group_filter = Some(group.clone());
        self.clamp_package_cursor();
        self.set_package_feedback_with_next_step(
            UiFeedbackLevel::Info,
            format!(
                "Packages 已聚焦到组：{}",
                self.package_group_display(&group)
            ),
            self.package_browse_next_step(),
        );
    }

    pub fn clear_package_group_filter(&mut self) {
        if self.package_group_filter.is_none() {
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Info,
                "Packages 当前没有启用组过滤。",
                self.package_browse_next_step(),
            );
            return;
        }

        self.package_group_filter = None;
        self.clamp_package_cursor();
        self.set_package_feedback_with_next_step(
            UiFeedbackLevel::Info,
            "Packages 已清空组过滤。",
            self.package_browse_next_step(),
        );
    }
}
