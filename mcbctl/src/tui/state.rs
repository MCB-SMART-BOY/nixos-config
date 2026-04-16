use crate::domain::tui::{
    ActionItem, CatalogEntry, DeployAction, DeploySource, DeployTask, DeployTextMode, GroupMeta,
    HomeManagedSettings, HomeOptionMeta, HostManagedSettings, HostsTextMode, ManagedBarProfile,
    ManagedToggle, PackageDataMode, PackageTextMode, Page, TopLevelPage, UsersTextMode,
};
use crate::store::catalog::{load_catalog, load_group_catalog, load_home_options_catalog};
use crate::store::context::{
    detect_hostname, detect_nix_system, detect_privilege_mode, detect_repo_root, list_hosts,
    list_users,
};
use crate::store::deploy::{
    NixosRebuildPlan, RepoSyncPlan, ensure_host_hardware_config, host_hardware_config_path,
    merged_nix_config, run_nixos_rebuild, run_repo_sync, run_root_command_ok,
};
use crate::store::home::{
    ensure_managed_settings_layout, load_home_user_settings, managed_home_desktop_path,
    render_managed_desktop_file, user_has_custom_noctalia_layout, user_noctalia_override_path,
};
use crate::store::hosts::{
    ensure_managed_host_layout, load_host_settings, managed_host_gpu_path,
    managed_host_network_path, managed_host_users_path, managed_host_virtualization_path,
    write_host_runtime_fragments, write_host_users_fragment,
};
use crate::store::packages::{
    ensure_managed_packages_layout, load_managed_package_entries, load_package_user_selections,
    managed_package_group_path, managed_package_guard_errors, write_grouped_managed_packages,
};
use crate::store::search::search_catalog_entries;
use crate::{resolve_sibling_binary, write_managed_file};
use anyhow::{Context, Result};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::PathBuf;

mod actions;
mod deploy;
mod helpers;
mod home;
mod hosts;
mod inspect;
mod model;
mod overview;
mod packages;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EditRow {
    pub(crate) label: String,
    pub(crate) value: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EditSummaryModel {
    pub(crate) header_lines: Vec<String>,
    pub(crate) focused_row: Option<EditRow>,
    pub(crate) field_lines: Vec<String>,
    pub(crate) detail: EditDetailModel,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EditPageModel {
    pub(crate) rows: Vec<EditRow>,
    pub(crate) selected: usize,
    pub(crate) summary: EditSummaryModel,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EditWorkspaceSummaryModel {
    pub(crate) current_page: String,
    pub(crate) dirty: String,
    pub(crate) recommendation: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EditCheckModel {
    pub(crate) summary: String,
    pub(crate) details: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EditDetailModel {
    pub(crate) status: String,
    pub(crate) validation: Option<EditCheckModel>,
    pub(crate) managed_guard: EditCheckModel,
    pub(crate) notes: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PackageGroupOverviewRow {
    pub(crate) group_label: String,
    pub(crate) count: usize,
    pub(crate) filter_selected: bool,
    pub(crate) current_selected: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PackageSelectedEntryRow {
    pub(crate) name: String,
    pub(crate) category: String,
    pub(crate) group_label: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PackageSelectionModel {
    pub(crate) current_entry_fields: Vec<EditRow>,
    pub(crate) group_rows: Vec<PackageGroupOverviewRow>,
    pub(crate) selected_rows: Vec<PackageSelectedEntryRow>,
    pub(crate) status: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PackageListItemModel {
    pub(crate) selected: bool,
    pub(crate) name: String,
    pub(crate) category: String,
    pub(crate) group_label: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PackageListModel {
    pub(crate) title: String,
    pub(crate) empty_text: Option<String>,
    pub(crate) items: Vec<PackageListItemModel>,
    pub(crate) selected_index: Option<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PackagePageModel {
    pub(crate) summary: EditSummaryModel,
    pub(crate) list: PackageListModel,
    pub(crate) selection: PackageSelectionModel,
}

impl EditSummaryModel {
    pub(crate) fn lines(&self) -> Vec<String> {
        let mut lines = self.header_lines.clone();
        if let Some(row) = &self.focused_row {
            lines.push(format!("当前聚焦：{} = {}", row.label, row.value));
        } else {
            lines.push("当前聚焦：无可用项".to_string());
        }
        lines.extend(self.field_lines.iter().cloned());
        lines.push(self.detail.status.clone());
        if let Some(validation) = &self.detail.validation {
            lines.push(validation.summary.clone());
            lines.extend(validation.details.iter().cloned());
        }
        lines.push(self.detail.managed_guard.summary.clone());
        lines.extend(self.detail.managed_guard.details.iter().cloned());
        lines.extend(self.detail.notes.iter().cloned());
        lines
    }
}

impl EditWorkspaceSummaryModel {
    pub(crate) fn lines(&self) -> Vec<String> {
        vec![
            self.current_page.clone(),
            self.dirty.clone(),
            self.recommendation.clone(),
        ]
    }
}

impl PackageGroupOverviewRow {
    pub(crate) fn display_line(&self) -> String {
        let filter_marker = if self.filter_selected { ">" } else { " " };
        let current_marker = if self.current_selected { "*" } else { " " };
        format!(
            "{filter_marker}{current_marker} {} ({})",
            self.group_label, self.count
        )
    }
}

impl PackageSelectedEntryRow {
    pub(crate) fn display_line(&self) -> String {
        format!(
            "- {} ({}, 组: {})",
            self.name, self.category, self.group_label
        )
    }
}

impl PackageSelectionModel {
    pub(crate) fn lines(&self) -> Vec<String> {
        let mut lines = Vec::new();

        if !self.current_entry_fields.is_empty() {
            lines.extend(
                self.current_entry_fields
                    .iter()
                    .map(|row| format!("{}：{}", row.label, row.value)),
            );
            lines.push(String::new());
        }

        if self.group_rows.is_empty() {
            lines.push("当前用户还没有可用软件组。".to_string());
        } else {
            lines.push("当前用户分组（> 过滤，* 当前条目）：".to_string());
            lines.extend(
                self.group_rows
                    .iter()
                    .map(PackageGroupOverviewRow::display_line),
            );
            lines.push(String::new());
        }

        if self.selected_rows.is_empty() {
            lines.push("当前用户尚未选中任何软件。".to_string());
        } else {
            lines.push("当前用户已选：".to_string());
            lines.extend(
                self.selected_rows
                    .iter()
                    .map(PackageSelectedEntryRow::display_line),
            );
        }

        lines.push(String::new());
        lines.push(format!("状态：{}", self.status));
        lines
    }
}

impl PackageListItemModel {
    pub(crate) fn display_line(&self) -> String {
        let marker = if self.selected { "[x]" } else { "[ ]" };
        format!(
            "{marker} {} ({}, -> {})",
            self.name, self.category, self.group_label
        )
    }
}

impl AppState {
    pub(crate) fn edit_workspace_summary_model(&self) -> EditWorkspaceSummaryModel {
        EditWorkspaceSummaryModel {
            current_page: format!(
                "当前子页：{}  目标：{}  切换：1/2/3/4",
                self.edit_page().title(),
                self.edit_workspace_target_label()
            ),
            dirty: format!("Dirty：{}", self.edit_workspace_dirty_summary()),
            recommendation: self.edit_workspace_recommendation_line(),
        }
    }

    fn edit_workspace_target_label(&self) -> String {
        match self.edit_page() {
            Page::Packages => self
                .current_package_user()
                .map(|user| format!("user {user}"))
                .unwrap_or_else(|| "无可用用户".to_string()),
            Page::Home => self
                .current_home_user()
                .map(|user| format!("user {user}"))
                .unwrap_or_else(|| "无可用用户".to_string()),
            Page::Users | Page::Hosts => format!("host {}", self.target_host),
            Page::Dashboard | Page::Deploy | Page::Advanced | Page::Inspect | Page::Actions => {
                "<无>".to_string()
            }
        }
    }

    fn edit_workspace_target_available(&self) -> bool {
        match self.edit_page() {
            Page::Packages => self.current_package_user().is_some(),
            Page::Home => self.current_home_user().is_some(),
            Page::Users | Page::Hosts => !self.target_host.trim().is_empty(),
            Page::Dashboard | Page::Deploy | Page::Advanced | Page::Inspect | Page::Actions => {
                false
            }
        }
    }

    fn edit_workspace_recommendation_line(&self) -> String {
        let page = self.edit_page();
        let page_label = page.title();
        let target = self.edit_workspace_target_label();

        if !self.edit_workspace_target_available() {
            return format!(
                "建议：当前页 {page_label}[{target}] 没有可用目标；先补用户或切到其他编辑页。"
            );
        }

        if self.edit_workspace_current_page_dirty() {
            return format!("建议：当前页 {page_label}[{target}] 还有未保存修改；先按 s 保存。");
        }

        if let Some(reason) = self.edit_workspace_current_page_guard_reason() {
            return format!("建议：当前页 {page_label}[{target}] 先处理受管保护：{reason}");
        }

        if let Some((section, item)) = self.preferred_edit_dirty_section() {
            return format!("建议：先切到 {section}[{item}] 保存未保存修改。");
        }

        if let Some((section, target, reason)) = self.preferred_edit_managed_guard() {
            return format!("建议：先切到 {section}[{target}] 处理受管保护：{reason}");
        }

        format!("建议：当前页 {page_label}[{target}] 已就绪，可继续编辑或按 s 保存。")
    }

    fn edit_workspace_dirty_summary(&self) -> String {
        [
            edit_workspace_dirty_section(
                "Packages",
                self.current_package_user(),
                self.current_package_user()
                    .is_some_and(|user| self.package_dirty_users.contains(user)),
            ),
            edit_workspace_dirty_section(
                "Home",
                self.current_home_user(),
                self.current_home_user()
                    .is_some_and(|user| self.home_dirty_users.contains(user)),
            ),
            edit_workspace_dirty_section(
                "Users",
                Some(self.target_host.as_str()),
                self.host_dirty_user_hosts.contains(&self.target_host),
            ),
            edit_workspace_dirty_section(
                "Hosts",
                Some(self.target_host.as_str()),
                self.host_dirty_runtime_hosts.contains(&self.target_host),
            ),
        ]
        .join(" | ")
    }

    fn edit_workspace_current_page_dirty(&self) -> bool {
        match self.edit_page() {
            Page::Packages => self
                .current_package_user()
                .is_some_and(|user| self.package_dirty_users.contains(user)),
            Page::Home => self
                .current_home_user()
                .is_some_and(|user| self.home_dirty_users.contains(user)),
            Page::Users => self.host_dirty_user_hosts.contains(&self.target_host),
            Page::Hosts => self.host_dirty_runtime_hosts.contains(&self.target_host),
            Page::Dashboard | Page::Deploy | Page::Advanced | Page::Inspect | Page::Actions => {
                false
            }
        }
    }

    fn edit_workspace_current_page_guard_reason(&self) -> Option<String> {
        let mut errors = match self.edit_page() {
            Page::Packages => self.current_package_managed_guard_errors(),
            Page::Home => self.current_home_managed_guard_errors(),
            Page::Users => self.current_host_users_managed_guard_errors(),
            Page::Hosts => self.current_host_runtime_managed_guard_errors(),
            Page::Dashboard | Page::Deploy | Page::Advanced | Page::Inspect | Page::Actions => {
                Vec::new()
            }
        };
        errors.drain(..).next()
    }
}

fn edit_workspace_dirty_section(label: &str, target: Option<&str>, dirty: bool) -> String {
    match (target, dirty) {
        (Some(target), true) => format!("{label}({target})"),
        (Some(_), false) => format!("{label}(clean)"),
        (None, _) => format!("{label}(n/a)"),
    }
}

pub(crate) use deploy::{
    AdvancedActionsListModel, AdvancedContextModel, AdvancedMaintenanceModel,
    AdvancedMaintenancePageModel, AdvancedSummaryModel, AdvancedWizardDetailModel,
    AdvancedWizardModel, AdvancedWizardPageModel, ApplyAdvancedWorkspaceModel,
    ApplyExecutionGateModel, ApplyModel, ApplyPageModel, ApplySelectionModel, DeployControlsModel,
    DeployPageModel,
};
use helpers::*;
#[cfg(test)]
pub(crate) use inspect::InspectCommandModel;
pub(crate) use inspect::{InspectCommandDetailModel, InspectHealthFocus, InspectModel};
pub use model::{AppContext, AppState, UiFeedback, UiFeedbackLevel, UiFeedbackScope};
pub(crate) use overview::{
    ManagedGuardSnapshot, OverviewCheckState, OverviewHealthFocus, OverviewHostStatus,
    OverviewModel, OverviewPrimaryActionKind,
};
#[cfg(test)]
pub(crate) use overview::{
    OverviewApplySummaryModel, OverviewContext, OverviewDirtySection, OverviewPrimaryAction,
};
