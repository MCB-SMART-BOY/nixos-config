use super::*;

impl AppState {
    pub fn open_package_search(&mut self) {
        self.package_group_create_mode = false;
        self.package_group_rename_mode = false;
        self.package_group_rename_source.clear();
        self.package_group_input.clear();
        self.package_search_mode = true;
        self.set_package_feedback_with_next_step(
            UiFeedbackLevel::Info,
            "Packages 搜索输入已打开。",
            self.package_search_next_step(),
        );
    }

    pub fn handle_search_input(&mut self, code: crossterm::event::KeyCode) {
        match code {
            crossterm::event::KeyCode::Enter => {
                self.package_search_mode = false;
                if self.package_mode == PackageDataMode::Search {
                    self.refresh_package_search_results();
                } else {
                    self.clamp_package_cursor();
                    self.set_package_feedback_with_next_step(
                        UiFeedbackLevel::Info,
                        "Packages 搜索输入结束。",
                        self.package_browse_next_step(),
                    );
                }
            }
            crossterm::event::KeyCode::Esc => {
                self.package_search_mode = false;
                self.clamp_package_cursor();
                self.set_package_feedback_with_next_step(
                    UiFeedbackLevel::Info,
                    "Packages 搜索输入结束。",
                    self.package_browse_next_step(),
                );
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
        self.set_package_feedback_with_next_step(
            UiFeedbackLevel::Info,
            "Packages 已清空搜索条件。",
            self.package_browse_next_step(),
        );
    }

    pub fn toggle_package_mode(&mut self) {
        self.package_mode = match self.package_mode {
            PackageDataMode::Local => PackageDataMode::Search,
            PackageDataMode::Search => PackageDataMode::Local,
        };
        self.package_category_index = 0;
        self.package_source_filter = None;
        self.package_workflow_filter = None;
        if self.package_mode == PackageDataMode::Search {
            if self.package_search.trim().is_empty() {
                self.set_package_feedback_with_next_step(
                    UiFeedbackLevel::Info,
                    "Packages 已切到 nixpkgs 搜索模式。",
                    self.package_search_next_step(),
                );
            } else {
                self.refresh_package_search_results();
                return;
            }
        } else {
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Info,
                "Packages 已切回本地覆盖/已声明视图。",
                self.package_browse_next_step(),
            );
        }
        self.clamp_package_cursor();
    }

    pub fn refresh_package_search_results(&mut self) {
        if self.package_mode != PackageDataMode::Search {
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Info,
                "Packages 当前不在 nixpkgs 搜索模式。",
                "按 f 切到搜索模式后再刷新。",
            );
            return;
        }
        let query = self.package_search.trim().to_string();
        if query.is_empty() {
            self.package_search_result_indices.clear();
            self.set_package_feedback_with_next_step(
                UiFeedbackLevel::Warning,
                "Packages 请输入关键词后再刷新 nixpkgs 搜索。",
                self.package_search_next_step(),
            );
            self.clamp_package_cursor();
            return;
        }

        match search_catalog_entries("nixpkgs", &query, &self.context.current_system) {
            Ok(entries) => {
                let count = entries.len();
                self.package_search_result_indices = self.merge_catalog_entries(entries, false);
                self.clamp_package_cursor();
                self.set_package_feedback_with_next_step(
                    UiFeedbackLevel::Success,
                    format!("Packages 已刷新 nixpkgs 搜索：{query}（{count} 条结果）。"),
                    "继续浏览结果，或修改关键词后再按 Enter / r 刷新。",
                );
            }
            Err(err) => {
                self.package_search_result_indices.clear();
                self.clamp_package_cursor();
                self.set_package_feedback_with_next_step(
                    UiFeedbackLevel::Error,
                    format!("Packages nixpkgs 搜索失败：{err}"),
                    "检查网络或搜索词后重试。",
                );
            }
        }
    }
}
