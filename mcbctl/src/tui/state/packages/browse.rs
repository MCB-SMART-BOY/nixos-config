use super::*;

impl AppState {
    pub(crate) fn package_page_model(&self) -> PackagePageModel {
        PackagePageModel {
            summary: self.package_summary_model(),
            list: self.package_list_model(),
            selection: self.package_selection_model(),
        }
    }

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

    pub fn current_package_workflow_filter(&self) -> Option<&str> {
        self.package_workflow_filter.as_deref()
    }

    pub fn current_package_workflow_filter_label(&self) -> String {
        if self.package_mode == PackageDataMode::Search {
            "不适用".to_string()
        } else {
            self.current_package_workflow_filter()
                .map(|workflow| self.package_workflow_display(workflow))
                .unwrap_or_else(|| "全部".to_string())
        }
    }

    pub fn package_filtered_indices(&self) -> Vec<usize> {
        let group_filter = self.package_group_filter.clone();
        let source_filter = self.package_source_filter.clone();
        let workflow_filter = self.package_workflow_filter.clone();
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

                let matches_workflow = if let Some(workflow_filter) = &workflow_filter {
                    self.package_mode != PackageDataMode::Search
                        && entry.workflow_tags.iter().any(|tag| tag == workflow_filter)
                } else {
                    true
                };

                (entry.matches(self.current_package_category(), &self.package_search)
                    && matches_group
                    && matches_source
                    && matches_workflow)
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

    pub(crate) fn package_summary_model(&self) -> EditSummaryModel {
        let header_lines = vec![
            format!("数据源：{}", self.current_package_mode_label()),
            format!(
                "当前用户：{}",
                self.current_package_user().unwrap_or("无可用用户")
            ),
            format!("目标目录：{}", display_path(self.package_target_dir_path())),
        ];

        let mut field_lines = vec![
            format!("分类过滤：{}", self.current_package_category_label()),
            format!("组过滤：{}", self.current_package_group_filter_label()),
            format!("来源过滤：{}", self.current_package_source_filter_label()),
            format!(
                "工作流过滤：{}",
                self.current_package_workflow_filter_label()
            ),
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

        if let Some((workflow, total_count, selected_count)) =
            self.current_package_workflow_overview()
        {
            field_lines.push(format!(
                "当前工作流：{}",
                self.package_workflow_display(&workflow)
            ));
            field_lines.push(format!("工作流可选：{total_count}"));
            field_lines.push(format!("工作流已选：{selected_count}"));
            if let Some(description) = self.package_workflow_description(&workflow) {
                field_lines.push(format!("工作流说明：{description}"));
            }
        }

        if let Some(path) = self.current_package_target_path() {
            field_lines.push(format!("当前组落点：{}", path.display()));
        }
        if let Some(group) = self.current_selected_group_name() {
            field_lines.push(format!(
                "当前已选组：{}（{} 个软件）",
                self.package_group_label(&group),
                self.current_selected_group_member_count()
            ));
            if let Some(description) = self.package_group_description(&group) {
                field_lines.push(format!("组说明：{description}"));
            }
        }

        let status = if let Some(user) = self.current_package_user()
            && self.package_dirty_users.contains(user)
        {
            "状态：当前用户有未保存修改".to_string()
        } else {
            "状态：当前用户没有未保存修改".to_string()
        };

        let guard_errors = self.current_package_managed_guard_errors();
        let managed_guard = if self.current_package_user().is_none() {
            EditCheckModel {
                summary: "受管保护：无可用目标".to_string(),
                details: Vec::new(),
            }
        } else if guard_errors.is_empty() {
            EditCheckModel {
                summary: "受管保护：通过".to_string(),
                details: Vec::new(),
            }
        } else {
            EditCheckModel {
                summary: "受管保护：存在问题".to_string(),
                details: guard_errors
                    .into_iter()
                    .map(|err| format!("- {err}"))
                    .collect(),
            }
        };

        EditSummaryModel {
            header_lines,
            focused_row: None,
            field_lines,
            detail: EditDetailModel {
                status,
                action_summary: None,
                validation: None,
                managed_guard,
                notes: Vec::new(),
            },
        }
    }

    pub(crate) fn package_list_model(&self) -> PackageListModel {
        let title = format!("Packages ({})", self.current_package_mode_label());
        let filtered = self.package_filtered_indices();
        if filtered.is_empty() {
            let empty_text = if self.current_package_mode() == PackageDataMode::Search {
                "当前搜索条件下没有结果。\n\n尝试：\n- 按 / 输入关键词\n- Enter 或 r 刷新 nixpkgs 搜索\n- 按 f 切回本地覆盖层"
            } else {
                "当前过滤条件下没有可选软件。\n\n尝试：\n- 切换分类\n- 清空搜索\n- 按 f 切到 nixpkgs 搜索"
            };
            return PackageListModel {
                title,
                empty_text: Some(empty_text.to_string()),
                items: Vec::new(),
                selected_index: None,
            };
        }

        let items = filtered
            .iter()
            .filter_map(|index| self.context.catalog_entries.get(*index))
            .map(|entry| {
                let selected = self
                    .current_package_user()
                    .and_then(|user| self.package_user_selections.get(user))
                    .is_some_and(|set| set.contains_key(&entry.id));
                let group = if selected {
                    self.effective_selected_group(entry)
                } else {
                    entry.group_key().to_string()
                };
                PackageListItemModel {
                    selected,
                    name: entry.name.clone(),
                    category: entry.category.clone(),
                    group_label: self.package_group_display(&group),
                }
            })
            .collect();

        PackageListModel {
            title,
            empty_text: None,
            items,
            selected_index: Some(self.package_cursor),
        }
    }

    pub(crate) fn package_selection_model(&self) -> PackageSelectionModel {
        let mut current_entry_fields = Vec::new();
        if let Some(entry) = self.current_package_entry() {
            current_entry_fields.push(EditRow {
                label: "当前条目".to_string(),
                value: entry.name.clone(),
            });
            current_entry_fields.push(EditRow {
                label: "id".to_string(),
                value: entry.id.clone(),
            });
            current_entry_fields.push(EditRow {
                label: "分类".to_string(),
                value: entry.category.clone(),
            });
            current_entry_fields.push(EditRow {
                label: "来源".to_string(),
                value: entry.source_label().to_string(),
            });
            let workflow_labels = self.package_entry_workflow_labels(entry);
            if !workflow_labels.is_empty() {
                current_entry_fields.push(EditRow {
                    label: "工作流".to_string(),
                    value: workflow_labels.join(", "),
                });
            }
            if let Some(lifecycle) = &entry.lifecycle {
                current_entry_fields.push(EditRow {
                    label: "生命周期".to_string(),
                    value: lifecycle.clone(),
                });
            }
            if let Some(group) = self.package_group_for_current_entry() {
                current_entry_fields.push(EditRow {
                    label: "目标组".to_string(),
                    value: self.package_group_display(&group),
                });
                if let Some(description) = self.package_group_description(&group) {
                    current_entry_fields.push(EditRow {
                        label: "组说明".to_string(),
                        value: description.to_string(),
                    });
                }
            }
            for workflow in &entry.workflow_tags {
                if let Some(description) = self.package_workflow_description(workflow) {
                    current_entry_fields.push(EditRow {
                        label: format!("{} 说明", self.package_workflow_label(workflow)),
                        value: description.to_string(),
                    });
                }
            }
            current_entry_fields.push(EditRow {
                label: "表达式".to_string(),
                value: entry.expr.clone(),
            });
            if let Some(description) = &entry.description {
                current_entry_fields.push(EditRow {
                    label: "说明".to_string(),
                    value: description.clone(),
                });
            }
            if !entry.platforms.is_empty() {
                current_entry_fields.push(EditRow {
                    label: "平台".to_string(),
                    value: entry.platforms.join(", "),
                });
            }
            if !entry.keywords.is_empty() {
                current_entry_fields.push(EditRow {
                    label: "关键词".to_string(),
                    value: entry.keywords.join(", "),
                });
            }
            if let Some(flag) = &entry.desktop_entry_flag {
                current_entry_fields.push(EditRow {
                    label: "桌面入口 flag".to_string(),
                    value: flag.clone(),
                });
            }
            if let Some(group) = self.current_selected_group_name() {
                current_entry_fields.push(EditRow {
                    label: "当前组成员数".to_string(),
                    value: self.current_selected_group_member_count().to_string(),
                });
                current_entry_fields.push(EditRow {
                    label: "当前整组操作对象".to_string(),
                    value: group,
                });
            }
        }

        let current_group = self.package_group_for_current_entry();
        let filter_group = self.current_package_group_filter();
        let group_rows = self
            .package_groups_overview()
            .into_iter()
            .map(|(group, count)| PackageGroupOverviewRow {
                group_label: self.package_group_display(&group),
                count,
                filter_selected: filter_group == Some(group.as_str()),
                current_selected: current_group.as_deref() == Some(group.as_str()),
            })
            .collect();

        let current_workflows = self
            .current_package_entry()
            .map(|entry| entry.workflow_tags.iter().cloned().collect::<BTreeSet<_>>())
            .unwrap_or_default();
        let filter_workflow = self.current_package_workflow_filter();
        let workflow_rows = self
            .package_workflows_overview()
            .into_iter()
            .map(
                |(workflow, total_count, selected_count)| PackageWorkflowOverviewRow {
                    workflow_label: self.package_workflow_display(&workflow),
                    total_count,
                    selected_count,
                    filter_selected: filter_workflow == Some(workflow.as_str()),
                    current_selected: current_workflows.contains(&workflow),
                },
            )
            .collect();

        let active_workflow = self.current_package_workflow_filter().map(|workflow| {
            let selected = self.current_user_selection();
            let mut selected_rows = Vec::new();
            let mut missing_rows = Vec::new();

            for entry in self.package_local_entries_for_workflow(workflow) {
                let group = if selected.is_some_and(|selection| selection.contains_key(&entry.id)) {
                    self.effective_selected_group(entry)
                } else {
                    self.default_group_for_entry(entry)
                };
                let row = PackageWorkflowEntryRow {
                    name: entry.name.clone(),
                    category: entry.category.clone(),
                    group_label: self.package_group_display(&group),
                };

                if selected.is_some_and(|selection| selection.contains_key(&entry.id)) {
                    selected_rows.push(row);
                } else {
                    missing_rows.push(row);
                }
            }

            let (total_count, selected_count) = self
                .current_package_workflow_overview()
                .map(|(_, total_count, selected_count)| (total_count, selected_count))
                .unwrap_or((
                    selected_rows.len() + missing_rows.len(),
                    selected_rows.len(),
                ));

            PackageActiveWorkflowModel {
                workflow_label: self.package_workflow_display(workflow),
                description: self
                    .package_workflow_description(workflow)
                    .map(str::to_string),
                total_count,
                selected_count,
                selected_rows,
                missing_rows,
            }
        });

        let selected_rows = self
            .package_selected_entries()
            .into_iter()
            .map(|entry| {
                let group = self.effective_selected_group(entry);
                PackageSelectedEntryRow {
                    name: entry.name.clone(),
                    category: entry.category.clone(),
                    group_label: self.package_group_display(&group),
                }
            })
            .collect();

        let action_summary = self.current_package_action_summary_model();

        PackageSelectionModel {
            current_entry_fields,
            group_rows,
            workflow_rows,
            active_workflow,
            action_summary,
            selected_rows,
            status: self.package_selection_status(),
        }
    }

    pub fn current_package_managed_guard_errors(&self) -> Vec<String> {
        let Some(user) = self.current_package_user() else {
            return Vec::new();
        };
        let managed_dir = self
            .context
            .repo_root
            .join("home/users")
            .join(user)
            .join("managed");
        let selected = self
            .package_user_selections
            .get(user)
            .cloned()
            .unwrap_or_default();
        managed_package_guard_errors(&managed_dir, &self.context.catalog_entries, &selected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn package_summary_lines_surface_managed_guard_errors() -> Result<()> {
        let root = create_temp_repo("mcbctl-packages-summary-guards")?;
        let grouped_dir = root.join("home/users/alice/managed/packages");
        std::fs::create_dir_all(&grouped_dir)?;
        std::fs::write(
            grouped_dir.join("manual.nix"),
            "{ pkgs, ... }: { home.packages = [ pkgs.hello ]; }\n",
        )?;

        let state = test_state(&root);
        let lines = state.package_summary_model().lines();

        assert!(lines.iter().any(|line| line == "受管保护：存在问题"));
        assert!(
            lines
                .iter()
                .any(|line| { line.contains("refusing to remove stale unmanaged package file") })
        );

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn package_page_model_assembles_summary_list_and_selection() {
        let mut state = test_state(Path::new("/tmp/demo-package-page"));
        state.package_user_selections.insert(
            "alice".to_string(),
            BTreeMap::from([("hello".to_string(), "misc".to_string())]),
        );

        let model = state.package_page_model();

        assert_eq!(model.summary.header_lines[0], "数据源：本地覆盖/已声明");
        assert_eq!(model.list.title, "Packages (本地覆盖/已声明)");
        assert!(
            model
                .selection
                .selected_rows
                .iter()
                .any(|row| row.name == "Hello")
        );
    }

    #[test]
    fn package_selection_model_tracks_current_group_and_selected_rows() {
        let mut state = test_state(Path::new("/tmp/demo-selection"));
        state.package_group_filter = Some("misc".to_string());
        state.package_workflow_filter = Some("containers".to_string());
        state.package_user_selections.insert(
            "alice".to_string(),
            BTreeMap::from([("hello".to_string(), "misc".to_string())]),
        );

        let model = state.package_selection_model();

        assert!(
            model
                .current_entry_fields
                .iter()
                .any(|row| row.label == "目标组" && row.value == "misc")
        );
        assert!(
            model.group_rows.iter().any(|row| row.group_label == "misc"
                && row.filter_selected
                && row.current_selected)
        );
        assert!(model.workflow_rows.iter().any(|row| row.workflow_label
            == "容器与集群 [containers]"
            && row.filter_selected
            && row.current_selected
            && row.selected_count == 1
            && row.total_count == 1));
        let active_workflow = model.active_workflow.as_ref().expect("active workflow");
        assert_eq!(active_workflow.workflow_label, "容器与集群 [containers]");
        assert_eq!(active_workflow.selected_count, 1);
        assert_eq!(active_workflow.total_count, 1);
        assert!(model.action_summary.is_none());
        assert!(
            model
                .selected_rows
                .iter()
                .any(|row| row.name == "Hello" && row.group_label == "misc")
        );
    }

    #[test]
    fn package_filtered_indices_respect_workflow_filter() {
        let mut state = test_state(Path::new("/tmp/demo-workflow-filter"));

        assert_eq!(state.package_filtered_count(), 1);
        state.package_workflow_filter = Some("containers".to_string());
        assert_eq!(state.package_filtered_count(), 1);
        state.package_workflow_filter = Some("security".to_string());
        assert_eq!(state.package_filtered_count(), 0);
        assert_eq!(state.current_package_workflow_filter_label(), "security");
    }

    #[test]
    fn package_summary_model_shows_active_workflow_overview() {
        let mut state = test_state(Path::new("/tmp/demo-workflow-summary"));
        state.package_workflow_filter = Some("containers".to_string());
        state.package_user_selections.insert(
            "alice".to_string(),
            BTreeMap::from([("hello".to_string(), "misc".to_string())]),
        );

        let model = state.package_summary_model();

        assert!(
            model
                .field_lines
                .iter()
                .any(|line| line == "当前工作流：容器与集群 [containers]")
        );
        assert!(model.field_lines.iter().any(|line| line == "工作流可选：1"));
        assert!(model.field_lines.iter().any(|line| line == "工作流已选：1"));
        assert!(
            model
                .field_lines
                .iter()
                .any(|line| line == "工作流说明：容器与集群工具")
        );
    }

    #[test]
    fn package_selection_model_tracks_selected_and_missing_entries_for_active_workflow() {
        let mut state = test_state(Path::new("/tmp/demo-active-workflow-diff"));
        state.context.catalog_entries.push(CatalogEntry {
            id: "podman".to_string(),
            name: "Podman".to_string(),
            category: "containers".to_string(),
            group: Some("cloud-containers".to_string()),
            expr: "pkgs.podman".to_string(),
            description: Some("容器运行时".to_string()),
            keywords: Vec::new(),
            workflow_tags: vec!["containers".to_string()],
            lifecycle: Some("stable".to_string()),
            source: Some("nixpkgs".to_string()),
            platforms: Vec::new(),
            desktop_entry_flag: None,
        });
        state.package_local_entry_ids.insert("podman".to_string());
        state.package_workflow_filter = Some("containers".to_string());
        state.package_user_selections.insert(
            "alice".to_string(),
            BTreeMap::from([("hello".to_string(), "misc".to_string())]),
        );

        let model = state.package_selection_model();
        let active_workflow = model.active_workflow.as_ref().expect("active workflow");

        assert_eq!(active_workflow.selected_count, 1);
        assert_eq!(active_workflow.total_count, 2);
        assert_eq!(active_workflow.selected_rows.len(), 1);
        assert_eq!(active_workflow.selected_rows[0].name, "Hello");
        assert_eq!(active_workflow.missing_rows.len(), 1);
        assert_eq!(active_workflow.missing_rows[0].name, "Podman");
        assert_eq!(
            active_workflow.missing_rows[0].group_label,
            "cloud-containers"
        );
    }

    #[test]
    fn package_selection_model_surfaces_packages_success_feedback() {
        let mut state = test_state(Path::new("/tmp/demo-package-action-summary"));
        state.feedback = UiFeedback::with_next_step(
            UiFeedbackLevel::Success,
            UiFeedbackScope::Packages,
            "Packages 已加入工作流 容器与集群 [containers] 下的 1 个未选软件。",
            "继续调整列表，完成后按 s 保存。",
        );
        state.status = state.feedback.legacy_status_text();

        let model = state.package_selection_model();
        let action_summary = model.action_summary.as_ref().expect("action summary");

        assert!(
            action_summary
                .latest_result
                .contains("加入工作流 容器与集群")
        );
        assert_eq!(action_summary.next_step, "继续调整列表，完成后按 s 保存。");
    }

    #[test]
    fn package_selection_model_uses_stable_status_instead_of_legacy_feedback_text() {
        let mut state = test_state(Path::new("/tmp/demo-package-stable-status"));
        state.package_dirty_users.insert("alice".to_string());
        state.feedback = UiFeedback::with_next_step(
            UiFeedbackLevel::Success,
            UiFeedbackScope::Packages,
            "Packages 已选中 alice：Hello",
            "继续调整列表，完成后按 s 保存。",
        );
        state.status = state.feedback.legacy_status_text();

        let model = state.package_selection_model();

        assert_eq!(model.status, "未保存");
        let action_summary = model.action_summary.as_ref().expect("action summary");
        assert_eq!(action_summary.latest_result, "Packages 已选中 alice：Hello");
    }

    #[test]
    fn add_current_workflow_missing_packages_adds_only_missing_entries() {
        let mut state = test_state(Path::new("/tmp/demo-active-workflow-apply"));
        state.context.catalog_entries.push(CatalogEntry {
            id: "podman".to_string(),
            name: "Podman".to_string(),
            category: "containers".to_string(),
            group: Some("cloud-containers".to_string()),
            expr: "pkgs.podman".to_string(),
            description: Some("容器运行时".to_string()),
            keywords: Vec::new(),
            workflow_tags: vec!["containers".to_string()],
            lifecycle: Some("stable".to_string()),
            source: Some("nixpkgs".to_string()),
            platforms: Vec::new(),
            desktop_entry_flag: None,
        });
        state.package_local_entry_ids.insert("podman".to_string());
        state.package_workflow_filter = Some("containers".to_string());
        state.package_user_selections.insert(
            "alice".to_string(),
            BTreeMap::from([("hello".to_string(), "misc".to_string())]),
        );

        state.add_current_workflow_missing_packages();

        let selection = state
            .package_user_selections
            .get("alice")
            .expect("alice selection");
        assert!(selection.contains_key("hello"));
        assert!(selection.contains_key("podman"));
        assert_eq!(
            selection.get("podman").map(String::as_str),
            Some("cloud-containers")
        );
        assert!(state.package_dirty_users.contains("alice"));
        assert!(
            state
                .status
                .contains("Packages 已加入工作流 容器与集群 [containers] 下的 1 个未选软件")
        );
    }

    #[test]
    fn open_current_workflow_missing_packages_confirm_enters_confirm_mode() {
        let mut state = test_state(Path::new("/tmp/demo-active-workflow-confirm"));
        state.context.catalog_entries.push(CatalogEntry {
            id: "podman".to_string(),
            name: "Podman".to_string(),
            category: "containers".to_string(),
            group: Some("cloud-containers".to_string()),
            expr: "pkgs.podman".to_string(),
            description: Some("容器运行时".to_string()),
            keywords: Vec::new(),
            workflow_tags: vec!["containers".to_string()],
            lifecycle: Some("stable".to_string()),
            source: Some("nixpkgs".to_string()),
            platforms: Vec::new(),
            desktop_entry_flag: None,
        });
        state.package_local_entry_ids.insert("podman".to_string());
        state.package_workflow_filter = Some("containers".to_string());
        state.package_user_selections.insert(
            "alice".to_string(),
            BTreeMap::from([("hello".to_string(), "misc".to_string())]),
        );

        state.open_current_workflow_missing_packages_confirm();

        assert!(state.package_workflow_add_confirm_mode);
        assert_eq!(
            state.active_package_text_mode(),
            Some(PackageTextMode::ConfirmWorkflowAdd)
        );
        assert!(
            state
                .status
                .contains("Packages 准备为 alice 批量加入工作流")
        );
    }

    #[test]
    fn confirm_current_workflow_missing_packages_applies_and_exits_confirm_mode() {
        let mut state = test_state(Path::new("/tmp/demo-active-workflow-confirm-apply"));
        state.context.catalog_entries.push(CatalogEntry {
            id: "podman".to_string(),
            name: "Podman".to_string(),
            category: "containers".to_string(),
            group: Some("cloud-containers".to_string()),
            expr: "pkgs.podman".to_string(),
            description: Some("容器运行时".to_string()),
            keywords: Vec::new(),
            workflow_tags: vec!["containers".to_string()],
            lifecycle: Some("stable".to_string()),
            source: Some("nixpkgs".to_string()),
            platforms: Vec::new(),
            desktop_entry_flag: None,
        });
        state.package_local_entry_ids.insert("podman".to_string());
        state.package_workflow_filter = Some("containers".to_string());
        state.package_user_selections.insert(
            "alice".to_string(),
            BTreeMap::from([("hello".to_string(), "misc".to_string())]),
        );
        state.open_current_workflow_missing_packages_confirm();

        state.confirm_current_workflow_missing_packages();

        assert!(!state.package_workflow_add_confirm_mode);
        let selection = state
            .package_user_selections
            .get("alice")
            .expect("alice selection");
        assert!(selection.contains_key("podman"));
        assert_eq!(state.feedback.scope, UiFeedbackScope::Packages);
    }

    #[test]
    fn package_list_model_marks_selected_entry_and_group() {
        let mut state = test_state(Path::new("/tmp/demo-package-list"));
        state.package_user_selections.insert(
            "alice".to_string(),
            BTreeMap::from([("hello".to_string(), "misc".to_string())]),
        );

        let model = state.package_page_model();

        assert_eq!(model.list.title, "Packages (本地覆盖/已声明)");
        assert_eq!(model.list.selected_index, Some(0));
        assert!(
            model
                .list
                .items
                .iter()
                .any(|item| item.selected && item.name == "Hello" && item.group_label == "misc")
        );
        assert!(
            model
                .selection
                .current_entry_fields
                .iter()
                .any(|row| row.label == "工作流" && row.value == "容器与集群 [containers]")
        );
    }

    fn test_state(root: &Path) -> AppState {
        AppState {
            context: AppContext {
                repo_root: root.to_path_buf(),
                etc_root: PathBuf::from("/etc/nixos"),
                current_host: "demo".to_string(),
                current_system: "x86_64-linux".to_string(),
                current_user: "alice".to_string(),
                privilege_mode: "sudo-available".to_string(),
                hosts: vec!["demo".to_string()],
                users: vec!["alice".to_string()],
                catalog_path: root.join("catalog/packages"),
                catalog_groups_path: root.join("catalog/groups.toml"),
                catalog_home_options_path: root.join("catalog/home-options.toml"),
                catalog_workflows_path: root.join("catalog/workflows.toml"),
                catalog_entries: vec![CatalogEntry {
                    id: "hello".to_string(),
                    name: "Hello".to_string(),
                    category: "cli".to_string(),
                    group: Some("misc".to_string()),
                    expr: "pkgs.hello".to_string(),
                    description: None,
                    keywords: Vec::new(),
                    workflow_tags: vec!["containers".to_string()],
                    lifecycle: Some("stable".to_string()),
                    source: Some("nixpkgs".to_string()),
                    platforms: Vec::new(),
                    desktop_entry_flag: None,
                }],
                catalog_groups: BTreeMap::new(),
                catalog_home_options: Vec::new(),
                catalog_workflows: BTreeMap::from([(
                    "containers".to_string(),
                    WorkflowMeta {
                        id: "containers".to_string(),
                        label: "容器与集群".to_string(),
                        description: Some("容器与集群工具".to_string()),
                        order: 20,
                    },
                )]),
                catalog_categories: Vec::new(),
                catalog_sources: Vec::new(),
            },
            active_page: 0,
            active_edit_page: 0,
            deploy_focus: 0,
            advanced_deploy_focus: 0,
            target_host: "demo".to_string(),
            deploy_task: DeployTask::DirectDeploy,
            deploy_source: DeploySource::CurrentRepo,
            deploy_source_ref: String::new(),
            deploy_action: DeployAction::Switch,
            flake_update: false,
            advanced_target_host: "demo".to_string(),
            advanced_deploy_task: DeployTask::DirectDeploy,
            advanced_deploy_source: DeploySource::CurrentRepo,
            advanced_deploy_source_ref: String::new(),
            advanced_deploy_action: DeployAction::Switch,
            advanced_flake_update: false,
            help_overlay_visible: false,
            deploy_text_mode: None,
            users_focus: 0,
            hosts_focus: 0,
            users_text_mode: None,
            hosts_text_mode: None,
            host_text_input: String::new(),
            host_settings_by_name: BTreeMap::new(),
            host_settings_errors_by_name: BTreeMap::new(),
            host_dirty_user_hosts: BTreeSet::new(),
            host_dirty_runtime_hosts: BTreeSet::new(),
            package_user_index: 0,
            package_mode: PackageDataMode::Local,
            package_cursor: 0,
            package_category_index: 0,
            package_group_filter: None,
            package_source_filter: None,
            package_workflow_filter: None,
            package_search: String::new(),
            package_search_result_indices: Vec::new(),
            package_local_entry_ids: BTreeSet::from(["hello".to_string()]),
            package_search_mode: false,
            package_group_create_mode: false,
            package_group_rename_mode: false,
            package_workflow_add_confirm_mode: false,
            package_group_rename_source: String::new(),
            package_group_input: String::new(),
            package_user_selections: BTreeMap::new(),
            package_dirty_users: BTreeSet::new(),
            home_user_index: 0,
            home_focus: 0,
            home_settings_by_user: BTreeMap::new(),
            home_dirty_users: BTreeSet::new(),
            inspect_action: crate::domain::tui::ActionItem::FlakeCheck,
            advanced_action: crate::domain::tui::ActionItem::FlakeUpdate,
            overview_repo_integrity: OverviewCheckState::NotRun,
            overview_doctor: OverviewCheckState::NotRun,
            feedback: UiFeedback::default(),
            status: String::new(),
        }
    }

    fn create_temp_repo(prefix: &str) -> Result<PathBuf> {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let root = std::env::temp_dir().join(format!("{prefix}-{}-{unique}", std::process::id()));
        std::fs::create_dir_all(&root)?;
        Ok(root)
    }
}
