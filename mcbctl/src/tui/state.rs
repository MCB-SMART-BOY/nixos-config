use crate::domain::deploy::DeployPlan;
use crate::domain::tui::{
    ActionItem, CatalogEntry, DeployAction, DeploySource, DeployTask, GroupMeta,
    HomeManagedSettings, HomeOptionMeta, HostManagedSettings, HostsTextMode, ManagedBarProfile,
    ManagedToggle, PackageDataMode, PackageTextMode, Page, UsersTextMode,
};
use crate::store::catalog::{load_catalog, load_group_catalog, load_home_options_catalog};
use crate::store::context::{
    detect_hostname, detect_nix_system, detect_privilege_mode, detect_repo_root, list_hosts,
    list_users,
};
use crate::store::deploy::{
    NixosRebuildPlan, RepoSyncPlan, ensure_root_hardware_config, merged_nix_config,
    run_nixos_rebuild, run_repo_sync, run_root_command_ok,
};
use crate::store::home::{
    ensure_managed_settings_layout, load_home_user_settings, managed_home_desktop_path,
    render_managed_desktop_file,
};
use crate::store::hosts::{
    ensure_managed_host_layout, load_host_settings, managed_host_gpu_path,
    managed_host_network_path, managed_host_users_path, managed_host_virtualization_path,
    write_host_runtime_fragments, write_host_users_fragment,
};
use crate::store::packages::{
    ensure_managed_packages_layout, load_managed_package_entries, load_package_user_selections,
    managed_package_group_path, write_grouped_managed_packages,
};
use crate::store::search::search_catalog_entries;
use crate::{resolve_sibling_binary, write_file_atomic};
use anyhow::{Context, Result};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::PathBuf;

mod actions;
mod deploy;
mod home;
mod hosts;
mod packages;

#[derive(Clone, Debug)]
pub struct AppContext {
    pub repo_root: PathBuf,
    pub etc_root: PathBuf,
    pub current_host: String,
    pub current_system: String,
    pub current_user: String,
    pub privilege_mode: String,
    pub hosts: Vec<String>,
    pub users: Vec<String>,
    pub catalog_path: PathBuf,
    pub catalog_groups_path: PathBuf,
    pub catalog_home_options_path: PathBuf,
    pub catalog_entries: Vec<CatalogEntry>,
    pub catalog_groups: BTreeMap<String, GroupMeta>,
    pub catalog_home_options: Vec<HomeOptionMeta>,
    pub catalog_categories: Vec<String>,
    pub catalog_sources: Vec<String>,
}

impl AppContext {
    pub fn detect() -> Result<Self> {
        let repo_root = detect_repo_root().context("failed to detect repo root")?;
        let etc_root = PathBuf::from("/etc/nixos");
        let current_host = detect_hostname();
        let current_system = detect_nix_system();
        let current_user = env::var("USER").unwrap_or_else(|_| "unknown".to_string());
        let privilege_mode = detect_privilege_mode();
        let hosts = list_hosts(&repo_root);
        let users = list_users(&repo_root);

        let catalog_path = repo_root.join("catalog/packages");
        let catalog_groups_path = repo_root.join("catalog/groups.toml");
        let catalog_home_options_path = repo_root.join("catalog/home-options.toml");
        let (catalog_entries, catalog_categories, catalog_sources) = load_catalog(&catalog_path);
        let catalog_groups = load_group_catalog(&catalog_groups_path);
        let catalog_home_options = load_home_options_catalog(&catalog_home_options_path);

        Ok(Self {
            repo_root,
            etc_root,
            current_host,
            current_system,
            current_user,
            privilege_mode,
            hosts,
            users,
            catalog_path,
            catalog_groups_path,
            catalog_home_options_path,
            catalog_entries,
            catalog_groups,
            catalog_home_options,
            catalog_categories,
            catalog_sources,
        })
    }
}

#[derive(Debug)]
pub struct AppState {
    pub context: AppContext,
    pub active_page: usize,
    pub deploy_focus: usize,
    pub target_host: String,
    pub deploy_task: DeployTask,
    pub deploy_source: DeploySource,
    pub deploy_action: DeployAction,
    pub flake_update: bool,
    pub show_advanced: bool,
    pub users_focus: usize,
    pub hosts_focus: usize,
    pub users_text_mode: Option<UsersTextMode>,
    pub hosts_text_mode: Option<HostsTextMode>,
    pub host_text_input: String,
    pub host_settings_by_name: BTreeMap<String, HostManagedSettings>,
    pub host_dirty_user_hosts: BTreeSet<String>,
    pub host_dirty_runtime_hosts: BTreeSet<String>,
    pub package_user_index: usize,
    pub package_mode: PackageDataMode,
    pub package_cursor: usize,
    pub package_category_index: usize,
    pub package_group_filter: Option<String>,
    pub package_source_filter: Option<String>,
    pub package_search: String,
    pub package_search_result_indices: Vec<usize>,
    pub package_local_entry_ids: BTreeSet<String>,
    pub package_search_mode: bool,
    pub package_group_create_mode: bool,
    pub package_group_rename_mode: bool,
    pub package_group_rename_source: String,
    pub package_group_input: String,
    pub package_user_selections: BTreeMap<String, BTreeMap<String, String>>,
    pub package_dirty_users: BTreeSet<String>,
    pub home_user_index: usize,
    pub home_focus: usize,
    pub home_settings_by_user: BTreeMap<String, HomeManagedSettings>,
    pub home_dirty_users: BTreeSet<String>,
    pub actions_focus: usize,
    pub status: String,
}

impl AppState {
    pub fn new(mut context: AppContext) -> Self {
        let deploy_source = if context.repo_root == context.etc_root {
            DeploySource::EtcNixos
        } else {
            DeploySource::CurrentRepo
        };
        let deploy_action = if context.privilege_mode == "rootless" {
            DeployAction::Build
        } else {
            DeployAction::Switch
        };
        let target_host = default_target_host(&context);
        let mut package_local_entry_ids = context
            .catalog_entries
            .iter()
            .filter(|entry| is_local_overlay_entry(entry))
            .map(|entry| entry.id.clone())
            .collect::<BTreeSet<_>>();
        let managed_entries = load_managed_package_entries(
            &context.repo_root,
            &context.users,
            &context.catalog_entries,
        );
        for entry in managed_entries {
            if !context
                .catalog_entries
                .iter()
                .any(|existing| existing.id == entry.id)
            {
                package_local_entry_ids.insert(entry.id.clone());
                context.catalog_entries.push(entry);
            }
        }
        refresh_local_catalog_indexes(&mut context, &package_local_entry_ids);
        let host_settings_by_name = load_host_settings(&context.repo_root, &context.hosts);
        let package_user_index = default_package_user_index(&context);
        let package_user_selections = load_package_user_selections(
            &context.repo_root,
            &context.users,
            &context.catalog_entries,
        );
        let home_user_index = default_package_user_index(&context);
        let home_settings_by_user = load_home_user_settings(&context.repo_root, &context.users);

        Self {
            context,
            active_page: 0,
            deploy_focus: 0,
            target_host,
            deploy_task: DeployTask::DirectDeploy,
            deploy_source,
            deploy_action,
            flake_update: false,
            show_advanced: false,
            users_focus: 0,
            hosts_focus: 0,
            users_text_mode: None,
            hosts_text_mode: None,
            host_text_input: String::new(),
            host_settings_by_name,
            host_dirty_user_hosts: BTreeSet::new(),
            host_dirty_runtime_hosts: BTreeSet::new(),
            package_user_index,
            package_mode: PackageDataMode::Search,
            package_cursor: 0,
            package_category_index: 0,
            package_group_filter: None,
            package_source_filter: None,
            package_search: String::new(),
            package_search_result_indices: Vec::new(),
            package_local_entry_ids,
            package_search_mode: false,
            package_group_create_mode: false,
            package_group_rename_mode: false,
            package_group_rename_source: String::new(),
            package_group_input: String::new(),
            package_user_selections,
            package_dirty_users: BTreeSet::new(),
            home_user_index,
            home_focus: 0,
            home_settings_by_user,
            home_dirty_users: BTreeSet::new(),
            actions_focus: 0,
            status: "Packages 现在默认使用 nixpkgs 搜索；本地覆盖与已声明软件可按 f 切回查看。"
                .to_string(),
        }
    }

    pub fn page(&self) -> Page {
        Page::ALL[self.active_page]
    }

    pub fn next_page(&mut self) {
        self.active_page = (self.active_page + 1) % Page::ALL.len();
    }

    pub fn previous_page(&mut self) {
        self.active_page = if self.active_page == 0 {
            Page::ALL.len() - 1
        } else {
            self.active_page - 1
        };
    }

    pub fn captures_text_input(&self) -> bool {
        self.package_search_mode
            || self.package_group_create_mode
            || self.package_group_rename_mode
            || self.users_text_mode.is_some()
            || self.hosts_text_mode.is_some()
    }

    pub fn active_package_text_mode(&self) -> Option<PackageTextMode> {
        if self.package_group_rename_mode {
            Some(PackageTextMode::RenameGroup)
        } else if self.package_group_create_mode {
            Some(PackageTextMode::CreateGroup)
        } else if self.package_search_mode {
            Some(PackageTextMode::Search)
        } else {
            None
        }
    }

    pub fn active_users_text_mode(&self) -> Option<UsersTextMode> {
        self.users_text_mode
    }

    pub fn active_hosts_text_mode(&self) -> Option<HostsTextMode> {
        self.hosts_text_mode
    }
}

fn cycle_enum<T: Copy + Eq>(current: &mut T, all: &[T], delta: i8) {
    let Some(index) = all.iter().position(|item| item == current) else {
        return;
    };
    let len = all.len() as isize;
    let next = (index as isize + delta as isize).rem_euclid(len) as usize;
    *current = all[next];
}

fn bool_label(value: bool) -> &'static str {
    if value { "开启" } else { "关闭" }
}

fn default_target_host(context: &AppContext) -> String {
    if context
        .hosts
        .iter()
        .any(|host| host == &context.current_host)
    {
        return context.current_host.clone();
    }
    if context.hosts.iter().any(|host| host == "nixos") {
        return "nixos".to_string();
    }
    context
        .hosts
        .first()
        .cloned()
        .unwrap_or_else(|| context.current_host.clone())
}

fn default_package_user_index(context: &AppContext) -> usize {
    if let Some(index) = context
        .users
        .iter()
        .position(|user| user == &context.current_user)
    {
        return index;
    }
    if let Some(index) = context.users.iter().position(|user| user == "mcbnixos") {
        return index;
    }
    0
}

fn format_string_list(items: &[String]) -> String {
    if items.is_empty() {
        "无".to_string()
    } else {
        items.join(", ")
    }
}

fn serialize_string_list(items: &[String]) -> String {
    items.join(", ")
}

fn format_string_map(items: &BTreeMap<String, String>) -> String {
    if items.is_empty() {
        "无".to_string()
    } else {
        items
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn serialize_string_map(items: &BTreeMap<String, String>) -> String {
    items
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_u16_map(items: &BTreeMap<String, u16>) -> String {
    if items.is_empty() {
        "无".to_string()
    } else {
        items
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn serialize_u16_map(items: &BTreeMap<String, u16>) -> String {
    items
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn parse_string_list(raw: &str) -> Vec<String> {
    dedup_string_list(
        raw.split(',')
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(ToOwned::to_owned)
            .collect(),
    )
}

fn dedup_string_list(items: Vec<String>) -> Vec<String> {
    let mut output = Vec::new();
    for item in items {
        if !output.contains(&item) {
            output.push(item);
        }
    }
    output
}

fn has_duplicates(items: &[String]) -> bool {
    let mut seen = BTreeSet::new();
    for item in items {
        if !seen.insert(item) {
            return true;
        }
    }
    false
}

fn parse_string_map(raw: &str) -> Result<BTreeMap<String, String>> {
    let mut output = BTreeMap::new();
    if raw.trim().is_empty() {
        return Ok(output);
    }

    for part in raw.split(',') {
        let piece = part.trim();
        if piece.is_empty() {
            continue;
        }
        let Some((key, value)) = piece.split_once('=') else {
            anyhow::bail!("映射项必须是 user=value 形式：{piece}");
        };
        let key = key.trim();
        let value = value.trim();
        if key.is_empty() || value.is_empty() {
            anyhow::bail!("映射项不能为空：{piece}");
        }
        output.insert(key.to_string(), value.to_string());
    }

    Ok(output)
}

fn parse_u16_map(raw: &str) -> Result<BTreeMap<String, u16>> {
    let mut output = BTreeMap::new();
    if raw.trim().is_empty() {
        return Ok(output);
    }

    for part in raw.split(',') {
        let piece = part.trim();
        if piece.is_empty() {
            continue;
        }
        let Some((key, value)) = piece.split_once('=') else {
            anyhow::bail!("端口映射必须是 user=1053 形式：{piece}");
        };
        let key = key.trim();
        let value = value.trim();
        if key.is_empty() || value.is_empty() {
            anyhow::bail!("端口映射项不能为空：{piece}");
        }
        let port = value
            .parse::<u16>()
            .with_context(|| format!("无效端口：{value}"))?;
        output.insert(key.to_string(), port);
    }

    Ok(output)
}

fn parse_gpu_modes(raw: &str) -> Result<Vec<String>> {
    let modes = parse_string_list(raw);
    for mode in &modes {
        if !matches!(mode.as_str(), "igpu" | "hybrid" | "dgpu") {
            anyhow::bail!("无效 GPU 特化模式：{mode}");
        }
    }
    Ok(modes)
}

fn empty_to_none(value: &str) -> Option<String> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value.trim().to_string())
    }
}

fn nonempty_label(value: &str) -> String {
    if value.trim().is_empty() {
        "无".to_string()
    } else {
        value.to_string()
    }
}

fn nonempty_opt_label(value: Option<&str>) -> String {
    value
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| "无".to_string())
}

fn normalize_package_group_name(input: &str) -> String {
    let mut output = String::new();
    let mut last_was_dash = false;

    for ch in input.chars().flat_map(char::to_lowercase) {
        let mapped = match ch {
            'a'..='z' | '0'..='9' => Some(ch),
            '-' | '_' | ' ' | '/' | '.' => Some('-'),
            _ => None,
        };

        let Some(ch) = mapped else {
            continue;
        };

        if ch == '-' {
            if output.is_empty() || last_was_dash {
                continue;
            }
            last_was_dash = true;
            output.push(ch);
        } else {
            last_was_dash = false;
            output.push(ch);
        }
    }

    while output.ends_with('-') {
        output.pop();
    }

    output
}

fn display_path(path: Option<PathBuf>) -> String {
    path.map(|path| path.display().to_string())
        .unwrap_or_else(|| "无".to_string())
}

fn is_local_overlay_entry(entry: &CatalogEntry) -> bool {
    let source = entry.source_label();
    source.starts_with("local/") || source.starts_with("overlay/") || source.starts_with("managed/")
}

fn refresh_local_catalog_indexes(context: &mut AppContext, local_entry_ids: &BTreeSet<String>) {
    let mut categories = BTreeSet::new();
    let mut sources = BTreeSet::new();

    for entry in &context.catalog_entries {
        if !local_entry_ids.contains(&entry.id) {
            continue;
        }
        categories.insert(entry.category.clone());
        sources.insert(entry.source_label().to_string());
    }

    context.catalog_categories = categories.into_iter().collect();
    context.catalog_sources = sources.into_iter().collect();
}

fn cycle_string_value(current: &str, all: &[String], delta: i8) -> Option<String> {
    if all.is_empty() {
        return None;
    }
    let index = all.iter().position(|item| item == current).unwrap_or(0);
    let len = all.len() as isize;
    let next = (index as isize + delta as isize).rem_euclid(len) as usize;
    Some(all[next].clone())
}

fn cycle_string(current: &mut String, all: &[String], delta: i8) {
    if all.is_empty() {
        return;
    }
    let index = all.iter().position(|item| item == current).unwrap_or(0);
    let len = all.len() as isize;
    let next = (index as isize + delta as isize).rem_euclid(len) as usize;
    *current = all[next].clone();
}
