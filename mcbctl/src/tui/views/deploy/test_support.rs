use crate::domain::tui::{DeployAction, DeploySource, DeployTask, HostManagedSettings};
use crate::tui::state::{AdvancedWizardModel, AppContext, AppState, ApplyModel};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

pub(super) fn test_apply_model() -> ApplyModel {
    ApplyModel {
        target_host: "nixos".to_string(),
        task: DeployTask::DirectDeploy,
        source: DeploySource::CurrentRepo,
        source_detail: None,
        action: DeployAction::Switch,
        flake_update: false,
        sync_preview: Some("sudo rsync /repo /etc/nixos".to_string()),
        rebuild_preview: Some(
            "sudo -E env nixos-rebuild switch --flake /etc/nixos#nixos".to_string(),
        ),
        can_execute_directly: true,
        can_apply_current_host: true,
        blockers: Vec::new(),
        warnings: vec!["当前组合会使用 sudo -E 执行受权命令。".to_string()],
        handoffs: Vec::new(),
        infos: vec!["检测 hostname：nixos".to_string()],
    }
}

pub(super) fn test_advanced_wizard_model() -> AdvancedWizardModel {
    AdvancedWizardModel {
        target_host: "nixos".to_string(),
        task: DeployTask::DirectDeploy,
        source: DeploySource::CurrentRepo,
        source_detail: None,
        action: DeployAction::Switch,
        flake_update: false,
        sync_preview: Some("sudo rsync /repo /etc/nixos".to_string()),
        rebuild_preview: Some(
            "sudo -E env nixos-rebuild switch --flake /etc/nixos#nixos".to_string(),
        ),
        blockers: Vec::new(),
        warnings: vec!["当前组合会使用 sudo -E 执行受权命令。".to_string()],
        handoffs: Vec::new(),
        infos: vec!["检测 hostname：nixos".to_string()],
        command_preview:
            "mcb-deploy --mode update-existing --action switch --host nixos --source current-repo"
                .to_string(),
        validation_error: None,
    }
}

pub(super) fn test_app_state() -> AppState {
    let mut host_settings_by_name = BTreeMap::new();
    host_settings_by_name.insert("nixos".to_string(), valid_host_settings());

    AppState {
        context: AppContext {
            repo_root: PathBuf::from("/repo"),
            etc_root: PathBuf::from("/etc/nixos"),
            current_host: "nixos".to_string(),
            current_system: "x86_64-linux".to_string(),
            current_user: "alice".to_string(),
            privilege_mode: "sudo-available".to_string(),
            hosts: vec!["nixos".to_string()],
            users: vec!["alice".to_string()],
            catalog_path: PathBuf::from("catalog/packages"),
            catalog_groups_path: PathBuf::from("catalog/groups.toml"),
            catalog_home_options_path: PathBuf::from("catalog/home-options.toml"),
            catalog_workflows_path: PathBuf::from("catalog/workflows.toml"),
            catalog_entries: Vec::new(),
            catalog_groups: BTreeMap::new(),
            catalog_home_options: Vec::new(),
            catalog_workflows: BTreeMap::new(),
            catalog_categories: Vec::new(),
            catalog_sources: Vec::new(),
        },
        active_page: 1,
        active_edit_page: 0,
        deploy_focus: 0,
        advanced_deploy_focus: 0,
        target_host: "nixos".to_string(),
        deploy_task: DeployTask::DirectDeploy,
        deploy_source: DeploySource::CurrentRepo,
        deploy_source_ref: String::new(),
        deploy_action: DeployAction::Switch,
        flake_update: false,
        advanced_target_host: "nixos".to_string(),
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
        host_settings_by_name,
        host_settings_errors_by_name: BTreeMap::new(),
        host_dirty_user_hosts: BTreeSet::new(),
        host_dirty_runtime_hosts: BTreeSet::new(),
        package_user_index: 0,
        package_mode: crate::domain::tui::PackageDataMode::Search,
        package_cursor: 0,
        package_category_index: 0,
        package_group_filter: None,
        package_source_filter: None,
        package_workflow_filter: None,
        package_search: String::new(),
        package_search_result_indices: Vec::new(),
        package_local_entry_ids: BTreeSet::new(),
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
        overview_repo_integrity: crate::tui::state::OverviewCheckState::NotRun,
        overview_doctor: crate::tui::state::OverviewCheckState::NotRun,
        feedback: crate::tui::state::UiFeedback::default(),
        status: String::new(),
    }
}

fn valid_host_settings() -> HostManagedSettings {
    HostManagedSettings {
        primary_user: "alice".to_string(),
        users: vec!["alice".to_string()],
        admin_users: vec!["alice".to_string()],
        ..HostManagedSettings::default()
    }
}

pub(super) fn render_view_text(
    width: u16,
    height: u16,
    render: impl FnOnce(&mut ratatui::Frame<'_>),
) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    terminal
        .draw(|frame| render(frame))
        .expect("test terminal draw should succeed");
    buffer_to_string(terminal.backend().buffer())
}

fn buffer_to_string(buffer: &Buffer) -> String {
    (0..buffer.area.height)
        .map(|y| {
            (0..buffer.area.width)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}
