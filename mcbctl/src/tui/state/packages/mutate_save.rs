use super::*;

impl AppState {
    pub fn save_current_user_packages(&mut self) -> Result<()> {
        let Some(user) = self.current_package_user().map(ToOwned::to_owned) else {
            self.status = "没有可保存的用户。".to_string();
            return Ok(());
        };

        let selected = self
            .package_user_selections
            .get(&user)
            .cloned()
            .unwrap_or_default();
        let managed_dir = self
            .context
            .repo_root
            .join("home/users")
            .join(&user)
            .join("managed");
        let guard_errors = managed_package_guard_errors(
            &managed_dir,
            &self.context.catalog_entries,
            &selected,
        );
        if !guard_errors.is_empty() {
            self.status = format!("Packages 未写入：{}", guard_errors.join("；"));
            return Ok(());
        }
        if let Err(err) = ensure_managed_packages_layout(&managed_dir) {
            self.status = format!("Packages 未写入：{err:#}");
            return Ok(());
        }
        if let Err(err) =
            write_grouped_managed_packages(&managed_dir, &self.context.catalog_entries, &selected)
        {
            self.status = format!("Packages 未写入：{err:#}");
            return Ok(());
        }
        self.package_dirty_users.remove(&user);
        self.status = format!("已写入 {}", managed_dir.join("packages").display());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{managed_file_is_valid, managed_file_kind};
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn save_current_user_packages_rejects_unknown_selected_ids_and_keeps_dirty() -> Result<()> {
        let root = create_temp_repo("mcbctl-packages-save-invalid")?;
        let mut state = test_state(&root);
        state.package_user_selections.insert(
            "alice".to_string(),
            BTreeMap::from([("missing".to_string(), "misc".to_string())]),
        );
        state.package_dirty_users.insert("alice".to_string());

        state.save_current_user_packages()?;
        assert!(
            state
                .status
                .contains("refusing to write package selections with unknown catalog ids")
        );
        assert!(state.package_dirty_users.contains("alice"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn save_current_user_packages_writes_managed_group_and_clears_dirty() -> Result<()> {
        let root = create_temp_repo("mcbctl-packages-save-valid")?;
        let mut state = test_state(&root);
        state.package_user_selections.insert(
            "alice".to_string(),
            BTreeMap::from([("hello".to_string(), "misc".to_string())]),
        );
        state.package_dirty_users.insert("alice".to_string());

        state.save_current_user_packages()?;

        let group_path = managed_package_group_path(&root, "alice", "misc");
        let content = std::fs::read_to_string(&group_path)?;
        assert_eq!(managed_file_kind(&content), Some("package-group:misc"));
        assert!(managed_file_is_valid(&content));
        assert!(content.contains("# managed-id: hello"));
        assert!(content.contains("pkgs.hello"));
        assert!(!state.package_dirty_users.contains("alice"));
        assert!(state.status.contains("managed/packages"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn save_current_user_packages_rejects_guard_errors_and_keeps_dirty() -> Result<()> {
        let root = create_temp_repo("mcbctl-packages-save-guard")?;
        let grouped_dir = root.join("home/users/alice/managed/packages");
        std::fs::create_dir_all(&grouped_dir)?;
        std::fs::write(
            grouped_dir.join("manual.nix"),
            "{ pkgs, ... }: { home.packages = [ pkgs.hello ]; }\n",
        )?;

        let mut state = test_state(&root);
        state.package_dirty_users.insert("alice".to_string());

        state.save_current_user_packages()?;

        assert!(
            state
                .status
                .contains("refusing to remove stale unmanaged package file")
        );
        assert!(state.package_dirty_users.contains("alice"));

        std::fs::remove_dir_all(root)?;
        Ok(())
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
                catalog_entries: vec![CatalogEntry {
                    id: "hello".to_string(),
                    name: "Hello".to_string(),
                    category: "cli".to_string(),
                    group: Some("misc".to_string()),
                    expr: "pkgs.hello".to_string(),
                    description: None,
                    keywords: Vec::new(),
                    source: Some("nixpkgs".to_string()),
                    platforms: Vec::new(),
                    desktop_entry_flag: None,
                }],
                catalog_groups: BTreeMap::new(),
                catalog_home_options: Vec::new(),
                catalog_categories: Vec::new(),
                catalog_sources: Vec::new(),
            },
            active_page: 0,
            deploy_focus: 0,
            target_host: "demo".to_string(),
            deploy_task: DeployTask::DirectDeploy,
            deploy_source: DeploySource::CurrentRepo,
            deploy_action: DeployAction::Switch,
            flake_update: false,
            show_advanced: false,
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
            package_search: String::new(),
            package_search_result_indices: Vec::new(),
            package_local_entry_ids: BTreeSet::from(["hello".to_string()]),
            package_search_mode: false,
            package_group_create_mode: false,
            package_group_rename_mode: false,
            package_group_rename_source: String::new(),
            package_group_input: String::new(),
            package_user_selections: BTreeMap::new(),
            package_dirty_users: BTreeSet::new(),
            home_user_index: 0,
            home_focus: 0,
            home_settings_by_user: BTreeMap::new(),
            home_dirty_users: BTreeSet::new(),
            actions_focus: 0,
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
