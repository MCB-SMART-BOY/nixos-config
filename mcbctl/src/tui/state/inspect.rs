use super::*;
use crate::domain::tui::ActionDestination;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InspectModel {
    pub(crate) repo_integrity: OverviewCheckState,
    pub(crate) doctor: OverviewCheckState,
    pub(crate) managed_guards: Vec<ManagedGuardSnapshot>,
    pub(crate) commands: Vec<InspectCommandModel>,
    pub(crate) latest_result: Option<UiFeedback>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InspectCommandModel {
    pub(crate) action: ActionItem,
    pub(crate) group: &'static str,
    pub(crate) label: &'static str,
    pub(crate) available: bool,
    pub(crate) preview: Option<String>,
}

impl AppState {
    pub(crate) fn inspect_model(&self) -> InspectModel {
        let commands = ActionItem::ALL
            .into_iter()
            .filter(|action| action.destination() == ActionDestination::Inspect)
            .map(|action| InspectCommandModel {
                action,
                group: action.group_label(),
                label: action.label(),
                available: self.action_available(action),
                preview: self.action_command_preview(action),
            })
            .collect();

        let latest_result =
            matches!(self.feedback.scope, UiFeedbackScope::Inspect).then(|| self.feedback.clone());

        InspectModel {
            repo_integrity: self.overview_repo_integrity.clone(),
            doctor: self.overview_doctor.clone(),
            managed_guards: self.managed_guard_snapshots(),
            commands,
            latest_result,
        }
    }

    pub(crate) fn ensure_inspect_action_focus(&mut self) {
        if self.current_action_item().destination() != ActionDestination::Inspect {
            self.actions_focus = inspect_action_index(0);
        }
    }

    pub(crate) fn next_inspect_action(&mut self) {
        let current = inspect_action_offset(self.current_action_item()).unwrap_or(0);
        self.actions_focus = inspect_action_index((current + 1) % inspect_action_count());
    }

    pub(crate) fn previous_inspect_action(&mut self) {
        let current = inspect_action_offset(self.current_action_item()).unwrap_or(0);
        let previous = if current == 0 {
            inspect_action_count() - 1
        } else {
            current - 1
        };
        self.actions_focus = inspect_action_index(previous);
    }

    pub(crate) fn current_inspect_action(&self) -> ActionItem {
        let action = self.current_action_item();
        if action.destination() == ActionDestination::Inspect {
            action
        } else {
            inspect_actions()[0]
        }
    }

    pub(crate) fn selected_inspect_row_index(&self) -> usize {
        inspect_action_offset(self.current_inspect_action()).unwrap_or(0)
    }
}

fn inspect_actions() -> &'static [ActionItem] {
    static ACTIONS: [ActionItem; 2] = [ActionItem::FlakeCheck, ActionItem::UpdateUpstreamCheck];
    &ACTIONS
}

fn inspect_action_count() -> usize {
    inspect_actions().len()
}

fn inspect_action_index(offset: usize) -> usize {
    let action = inspect_actions()[offset];
    ActionItem::ALL
        .iter()
        .position(|candidate| *candidate == action)
        .expect("inspect action must exist in ActionItem::ALL")
}

fn inspect_action_offset(action: ActionItem) -> Option<usize> {
    inspect_actions()
        .iter()
        .position(|candidate| *candidate == action)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::PathBuf;

    #[test]
    fn inspect_model_surfaces_health_snapshots_and_command_previews() {
        let mut state = test_state();
        state.overview_repo_integrity = OverviewCheckState::Error {
            summary: "failed (1 finding(s))".to_string(),
            details: vec!["- [rule] path: detail".to_string()],
        };
        state.overview_doctor = OverviewCheckState::Healthy {
            summary: "ok with 1 warning(s)".to_string(),
            details: vec!["缺少 cargo".to_string()],
        };

        let model = state.inspect_model();

        assert_eq!(model.repo_integrity, state.overview_repo_integrity);
        assert_eq!(model.doctor, state.overview_doctor);
        assert_eq!(model.managed_guards.len(), 4);
        assert_eq!(model.commands.len(), 2);
        assert_eq!(model.commands[0].action, ActionItem::FlakeCheck);
        assert_eq!(model.commands[0].group, "Repo Checks");
        assert!(
            model.commands[0]
                .preview
                .as_deref()
                .is_some_and(|preview| preview.contains("flake check"))
        );
        assert_eq!(model.commands[1].action, ActionItem::UpdateUpstreamCheck);
    }

    #[test]
    fn inspect_model_only_exposes_inspect_scoped_feedback_as_latest_result() {
        let mut state = test_state();
        state.set_feedback_message(
            UiFeedbackLevel::Success,
            UiFeedbackScope::Inspect,
            "flake check 已完成。",
        );
        let inspect = state.inspect_model();
        assert_eq!(
            inspect.latest_result,
            Some(UiFeedback {
                level: UiFeedbackLevel::Success,
                scope: UiFeedbackScope::Inspect,
                message: "flake check 已完成。".to_string(),
                next_step: None,
            })
        );

        state.set_feedback_message(
            UiFeedbackLevel::Info,
            UiFeedbackScope::Apply,
            "当前组合可直接 Apply。",
        );
        let inspect = state.inspect_model();
        assert!(inspect.latest_result.is_none());
    }

    #[test]
    fn inspect_focus_falls_back_to_first_inspect_action() {
        let mut state = test_state();
        state.actions_focus = 4;

        state.ensure_inspect_action_focus();

        assert_eq!(state.current_inspect_action(), ActionItem::FlakeCheck);
        assert_eq!(state.selected_inspect_row_index(), 0);
    }

    #[test]
    fn inspect_focus_cycles_only_between_inspect_actions() {
        let mut state = test_state();
        state.ensure_inspect_action_focus();
        assert_eq!(state.current_inspect_action(), ActionItem::FlakeCheck);

        state.next_inspect_action();
        assert_eq!(
            state.current_inspect_action(),
            ActionItem::UpdateUpstreamCheck
        );

        state.next_inspect_action();
        assert_eq!(state.current_inspect_action(), ActionItem::FlakeCheck);

        state.previous_inspect_action();
        assert_eq!(
            state.current_inspect_action(),
            ActionItem::UpdateUpstreamCheck
        );
    }

    fn test_state() -> AppState {
        AppState {
            context: AppContext {
                repo_root: PathBuf::from("/repo"),
                etc_root: PathBuf::from("/etc/nixos"),
                current_host: "demo".to_string(),
                current_system: "x86_64-linux".to_string(),
                current_user: "alice".to_string(),
                privilege_mode: "sudo-available".to_string(),
                hosts: vec!["demo".to_string()],
                users: vec!["alice".to_string()],
                catalog_path: PathBuf::from("catalog/packages"),
                catalog_groups_path: PathBuf::from("catalog/groups.toml"),
                catalog_home_options_path: PathBuf::from("catalog/home-options.toml"),
                catalog_entries: Vec::new(),
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
            package_mode: PackageDataMode::Search,
            package_cursor: 0,
            package_category_index: 0,
            package_group_filter: None,
            package_source_filter: None,
            package_search: String::new(),
            package_search_result_indices: Vec::new(),
            package_local_entry_ids: BTreeSet::new(),
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
}
