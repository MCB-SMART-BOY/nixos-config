use super::*;

impl AppState {
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
}
