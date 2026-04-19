use super::*;

impl AppState {
    pub fn toggle_current_package(&mut self) {
        let Some(user) = self.current_package_user().map(ToOwned::to_owned) else {
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Error,
                "Packages 页没有可操作的用户目录。",
                "先补可用 user 目标，或切到其他编辑页。",
            );
            return;
        };
        let Some(entry) = self.current_package_entry().cloned() else {
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Info,
                "当前过滤条件下没有可切换的软件。",
                self.package_browse_next_step(),
            );
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
        self.set_package_feedback_with_next_step(
            UiFeedbackLevel::Success,
            if enabled {
                format!("Packages 已选中 {}：{}", user, entry.name)
            } else {
                format!("Packages 已取消 {}：{}", user, entry.name)
            },
            self.package_edit_next_step(),
        );
    }

    pub fn open_package_group_creation(&mut self) {
        let Some(entry_name) = self.current_package_entry().map(|entry| entry.name.clone()) else {
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Info,
                "当前过滤条件下没有可新建分组的软件。",
                self.package_browse_next_step(),
            );
            return;
        };

        self.package_search_mode = false;
        self.package_group_rename_mode = false;
        self.package_group_rename_source.clear();
        self.package_group_create_mode = true;
        self.package_group_input.clear();
        self.set_package_feedback_with_next_step(
            UiFeedbackLevel::Info,
            format!("Packages 准备为 {entry_name} 创建新组。"),
            self.package_group_input_next_step(),
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
                self.set_package_feedback_with_next_step(
                    UiFeedbackLevel::Info,
                    "Packages 已取消组编辑。",
                    self.package_browse_next_step(),
                );
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
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Error,
                "Packages 页没有可操作的用户目录。",
                "先补可用 user 目标，或切到其他编辑页。",
            );
            return;
        };
        let Some(entry) = self.current_package_entry().cloned() else {
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Info,
                "当前过滤条件下没有可重命名分组的软件。",
                self.package_browse_next_step(),
            );
            return;
        };

        let Some(current_group) = self
            .package_user_selections
            .get(&user)
            .and_then(|selection| selection.get(&entry.id))
            .cloned()
        else {
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Info,
                "请先选中当前软件，再重命名它所在的组。",
                self.package_edit_next_step(),
            );
            return;
        };

        self.package_search_mode = false;
        self.package_group_create_mode = false;
        self.package_group_rename_mode = true;
        self.package_group_rename_source = current_group.clone();
        self.package_group_input = current_group.clone();
        self.set_package_feedback_with_next_step(
            UiFeedbackLevel::Info,
            format!("Packages 准备重命名组：{current_group}"),
            self.package_group_input_next_step(),
        );
    }

    pub fn adjust_current_package_group(&mut self, delta: i8) {
        let Some(user) = self.current_package_user().map(ToOwned::to_owned) else {
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Error,
                "Packages 页没有可操作的用户目录。",
                "先补可用 user 目标，或切到其他编辑页。",
            );
            return;
        };
        let Some(entry) = self.current_package_entry().cloned() else {
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Info,
                "当前过滤条件下没有可调整分组的软件。",
                self.package_browse_next_step(),
            );
            return;
        };

        let groups = self.package_groups_for_user(&user);
        if groups.is_empty() {
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Info,
                "当前用户没有可用的软件组。",
                self.package_browse_next_step(),
            );
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
        self.set_package_feedback_with_next_step(
            UiFeedbackLevel::Success,
            format!(
                "Packages 已把 {} 调整到组 {}。",
                entry.name,
                self.package_group_display(&next_group)
            ),
            self.package_edit_next_step(),
        );
    }

    pub fn move_current_selected_group(&mut self, delta: i8) {
        let Some(user) = self.current_package_user().map(ToOwned::to_owned) else {
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Error,
                "Packages 页没有可操作的用户目录。",
                "先补可用 user 目标，或切到其他编辑页。",
            );
            return;
        };
        let Some(current_group) = self.current_selected_group_name() else {
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Info,
                "请先选中当前软件，再整组移动它所在的组。",
                self.package_edit_next_step(),
            );
            return;
        };

        let groups = self.package_groups_for_user(&user);
        if groups.len() < 2 {
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Info,
                "当前用户只有一个可用组，无法整组移动。",
                self.package_browse_next_step(),
            );
            return;
        }

        let Some(next_group) = cycle_string_value(&current_group, &groups, delta) else {
            return;
        };
        if next_group == current_group {
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Info,
                format!(
                    "Packages 当前组未变化：{}",
                    self.package_group_display(&current_group)
                ),
                self.package_browse_next_step(),
            );
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
        self.set_package_feedback_with_next_step(
            UiFeedbackLevel::Success,
            format!(
                "Packages 已将组 {} 整体移动到 {}，影响 {moved} 个软件。",
                self.package_group_display(&current_group),
                self.package_group_display(&next_group)
            ),
            self.package_edit_next_step(),
        );
    }

    pub fn package_group_input_preview(&self) -> String {
        normalize_package_group_name(&self.package_group_input)
    }
}
