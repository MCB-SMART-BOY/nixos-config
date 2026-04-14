use super::*;

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
    pub host_settings_errors_by_name: BTreeMap<String, String>,
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
        let loaded_host_settings = load_host_settings(&context.repo_root, &context.hosts);
        let host_settings_by_name = loaded_host_settings.settings_by_name;
        let host_settings_errors_by_name = loaded_host_settings.errors_by_name;
        let package_user_index =
            default_package_user_index(&context, &target_host, &host_settings_by_name);
        let package_user_selections = load_package_user_selections(
            &context.repo_root,
            &context.users,
            &context.catalog_entries,
        );
        let home_user_index =
            default_package_user_index(&context, &target_host, &host_settings_by_name);
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
            host_settings_errors_by_name,
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
