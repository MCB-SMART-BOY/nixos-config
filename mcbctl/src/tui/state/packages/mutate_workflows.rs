use super::*;

impl AppState {
    pub fn open_current_workflow_missing_packages_confirm(&mut self) {
        if self.package_mode == PackageDataMode::Search {
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Info,
                "Packages workflow 批量加入只支持本地覆盖/已声明模式。",
                "按 f 切回本地覆盖/已声明后再操作 workflow。",
            );
            return;
        }

        let Some(user) = self.current_package_user().map(ToOwned::to_owned) else {
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Error,
                "Packages 页没有可操作的用户目录。",
                "先补可用 user 目标，或切到其他编辑页。",
            );
            return;
        };
        let Some(workflow) = self
            .current_package_workflow_filter()
            .map(ToOwned::to_owned)
        else {
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Info,
                "请先用 o / p 选中一个工作流过滤，再批量加入缺失软件。",
                self.package_browse_next_step(),
            );
            return;
        };
        let Some(missing_rows) = self.current_workflow_missing_package_rows() else {
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Info,
                "当前工作流没有可预览的软件。",
                self.package_browse_next_step(),
            );
            return;
        };
        if missing_rows.is_empty() {
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Info,
                format!(
                    "Packages 已选中工作流 {} 下的全部软件。",
                    self.package_workflow_display(&workflow)
                ),
                self.package_browse_next_step(),
            );
            return;
        }

        self.package_search_mode = false;
        self.package_group_create_mode = false;
        self.package_group_rename_mode = false;
        self.package_workflow_add_confirm_mode = true;
        self.set_package_feedback_with_next_step(
            UiFeedbackLevel::Info,
            format!(
                "Packages 准备为 {user} 批量加入工作流 {} 下的 {} 个未选软件。",
                self.package_workflow_display(&workflow),
                missing_rows.len()
            ),
            self.package_workflow_confirm_next_step(),
        );
    }

    pub fn handle_workflow_add_confirm_input(&mut self, code: crossterm::event::KeyCode) {
        match code {
            crossterm::event::KeyCode::Enter => self.confirm_current_workflow_missing_packages(),
            crossterm::event::KeyCode::Esc => {
                self.package_workflow_add_confirm_mode = false;
                self.set_package_feedback_with_next_step(
                    UiFeedbackLevel::Info,
                    "Packages 已取消 workflow 批量加入。",
                    self.package_browse_next_step(),
                );
            }
            _ => {}
        }
    }

    pub fn confirm_current_workflow_missing_packages(&mut self) {
        self.package_workflow_add_confirm_mode = false;
        self.add_current_workflow_missing_packages();
    }

    pub fn add_current_workflow_missing_packages(&mut self) {
        if self.package_mode == PackageDataMode::Search {
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Info,
                "Packages workflow 批量加入只支持本地覆盖/已声明模式。",
                "按 f 切回本地覆盖/已声明后再操作 workflow。",
            );
            return;
        }

        let Some(user) = self.current_package_user().map(ToOwned::to_owned) else {
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Error,
                "Packages 页没有可操作的用户目录。",
                "先补可用 user 目标，或切到其他编辑页。",
            );
            return;
        };
        let Some(workflow) = self
            .current_package_workflow_filter()
            .map(ToOwned::to_owned)
        else {
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Info,
                "请先用 o / p 选中一个工作流过滤，再批量加入缺失软件。",
                self.package_browse_next_step(),
            );
            return;
        };

        let workflow_entries = self
            .package_local_entries_for_workflow(&workflow)
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();
        if workflow_entries.is_empty() {
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Info,
                format!(
                    "当前工作流 {} 下没有可加入的软件。",
                    self.package_workflow_display(&workflow)
                ),
                self.package_browse_next_step(),
            );
            return;
        }

        let additions = workflow_entries
            .iter()
            .filter(|entry| {
                !self
                    .package_user_selections
                    .get(&user)
                    .is_some_and(|selection| selection.contains_key(&entry.id))
            })
            .map(|entry| (entry.id.clone(), self.default_group_for_entry(entry)))
            .collect::<Vec<_>>();

        let added_count = additions.len();
        if added_count == 0 {
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Info,
                format!(
                    "Packages 已选中工作流 {} 下的全部软件。",
                    self.package_workflow_display(&workflow)
                ),
                self.package_browse_next_step(),
            );
            return;
        }

        {
            let selection = self
                .package_user_selections
                .entry(user.clone())
                .or_default();
            for (id, group) in additions {
                selection.insert(id, group);
            }
        }

        self.package_dirty_users.insert(user.clone());
        self.ensure_valid_package_group_filter();
        self.clamp_package_cursor();
        self.set_package_feedback_with_next_step(
            UiFeedbackLevel::Success,
            format!(
                "Packages 已加入工作流 {} 下的 {added_count} 个未选软件。",
                self.package_workflow_display(&workflow)
            ),
            self.package_edit_next_step(),
        );
    }
}
