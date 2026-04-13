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
    NixosRebuildPlan, RepoSyncPlan, ensure_host_hardware_config, host_hardware_config_path,
    merged_nix_config, run_nixos_rebuild, run_repo_sync, run_root_command_ok,
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
mod model;
mod packages;

use helpers::*;
pub use model::{AppContext, AppState};
