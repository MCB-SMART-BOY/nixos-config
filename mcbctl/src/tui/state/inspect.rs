use super::*;
use crate::domain::tui::ActionDestination;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InspectModel {
    pub(crate) health_focus: InspectHealthFocus,
    pub(crate) repo_integrity: OverviewCheckState,
    pub(crate) doctor: OverviewCheckState,
    pub(crate) managed_guards: Vec<ManagedGuardSnapshot>,
    pub(crate) commands: Vec<InspectCommandModel>,
    pub(crate) selected_index: usize,
    pub(crate) detail: InspectCommandDetailModel,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InspectCommandDetailModel {
    pub(crate) action: ActionItem,
    pub(crate) group: &'static str,
    pub(crate) label: &'static str,
    pub(crate) available: bool,
    pub(crate) preview: Option<String>,
    pub(crate) latest_result: String,
    pub(crate) page_title: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InspectHealthFocus {
    RepoIntegrity,
    Doctor,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct InspectFeedbackState {
    latest_result: Option<UiFeedback>,
    latest_result_text: String,
    recent_feedback: String,
    next_step: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RouteFeedback {
    pub(crate) message: String,
    pub(crate) next_step: String,
}

impl AppState {
    pub(crate) fn inspect_model(&self) -> InspectModel {
        let commands: Vec<InspectCommandModel> = ActionItem::ALL
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
        let selected_index = self.selected_inspect_row_index();

        let feedback = self.current_inspect_feedback_state("无".to_string());
        let command = &commands[selected_index];
        let detail = InspectCommandDetailModel {
            action: command.action,
            group: command.group,
            label: command.label,
            available: command.available,
            preview: command.preview.clone(),
            latest_result: feedback.latest_result_text.clone(),
            page_title: self.page().title(),
        };

        InspectModel {
            health_focus: preferred_inspect_health_focus(
                &self.overview_repo_integrity,
                &self.overview_doctor,
            ),
            repo_integrity: self.overview_repo_integrity.clone(),
            doctor: self.overview_doctor.clone(),
            managed_guards: self.managed_guard_snapshots(),
            commands,
            selected_index,
            detail,
            latest_result: feedback.latest_result,
        }
    }

    pub(crate) fn current_inspect_feedback_summary(
        &self,
        fallback_message: &str,
        fallback_next_step: &str,
    ) -> RouteFeedback {
        let feedback = self.current_inspect_feedback_state(fallback_next_step.to_string());
        let message = if feedback.recent_feedback.is_empty() {
            fallback_message.to_string()
        } else {
            feedback.recent_feedback
        };
        RouteFeedback {
            message,
            next_step: feedback.next_step,
        }
    }

    fn current_inspect_feedback_state(&self, fallback_next_step: String) -> InspectFeedbackState {
        let snapshot =
            self.scoped_feedback_snapshot(UiFeedbackScope::Inspect, fallback_next_step, "无");
        InspectFeedbackState {
            latest_result: snapshot.feedback,
            latest_result_text: snapshot.latest_result_text,
            recent_feedback: snapshot.message,
            next_step: snapshot.next_step,
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

    pub(crate) fn set_inspect_completion_feedback(&mut self, action: ActionItem) {
        let feedback = inspect_completion_feedback(action);
        self.set_feedback_with_next_step(
            UiFeedbackLevel::Success,
            UiFeedbackScope::Inspect,
            feedback.message,
            feedback.next_step,
        );
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

fn preferred_inspect_health_focus(
    repo_integrity: &OverviewCheckState,
    doctor: &OverviewCheckState,
) -> InspectHealthFocus {
    if matches!(repo_integrity, OverviewCheckState::Error { .. }) {
        InspectHealthFocus::RepoIntegrity
    } else if matches!(doctor, OverviewCheckState::Error { .. }) {
        InspectHealthFocus::Doctor
    } else {
        InspectHealthFocus::RepoIntegrity
    }
}

fn inspect_completion_feedback(action: ActionItem) -> RouteFeedback {
    match action {
        ActionItem::FlakeCheck => RouteFeedback {
            message: "flake check 已完成。".to_string(),
            next_step: "切到 Inspect 查看检查结果".to_string(),
        },
        ActionItem::UpdateUpstreamCheck => RouteFeedback {
            message: "上游 pin 检查已完成。".to_string(),
            next_step: "切到 Inspect 查看 pin 状态".to_string(),
        },
        ActionItem::FlakeUpdate
        | ActionItem::UpdateUpstreamPins
        | ActionItem::SyncRepoToEtc
        | ActionItem::RebuildCurrentHost
        | ActionItem::LaunchDeployWizard => RouteFeedback {
            message: "Inspect 命令已完成。".to_string(),
            next_step: "切到 Inspect 查看结果".to_string(),
        },
    }
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

        assert_eq!(model.health_focus, InspectHealthFocus::RepoIntegrity);
        assert_eq!(model.repo_integrity, state.overview_repo_integrity);
        assert_eq!(model.doctor, state.overview_doctor);
        assert_eq!(model.managed_guards.len(), 4);
        assert_eq!(model.commands.len(), 2);
        assert_eq!(model.commands[0].action, ActionItem::FlakeCheck);
        assert_eq!(model.commands[0].group, "Repo Checks");
        assert_eq!(model.selected_index, 0);
        assert_eq!(model.detail.action, ActionItem::FlakeCheck);
        assert_eq!(model.detail.label, "flake check");
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
        assert_eq!(inspect.detail.latest_result, "flake check 已完成。");

        state.set_feedback_message(
            UiFeedbackLevel::Info,
            UiFeedbackScope::Apply,
            "当前组合可直接 Apply。",
        );
        let inspect = state.inspect_model();
        assert!(inspect.latest_result.is_none());
        assert_eq!(inspect.detail.latest_result, "无");
    }

    #[test]
    fn current_inspect_feedback_summary_prefers_inspect_scoped_feedback() {
        let mut state = test_state();
        state.set_feedback_with_next_step(
            UiFeedbackLevel::Success,
            UiFeedbackScope::Inspect,
            "flake check 已完成。",
            "留在 Inspect 复查健康详情",
        );

        let summary = state.current_inspect_feedback_summary(
            "Overview 推荐先进入 Inspect 处理 repo-integrity。",
            "在 Inspect 先看 repo-integrity",
        );

        assert_eq!(
            summary,
            RouteFeedback {
                message: "flake check 已完成。".to_string(),
                next_step: "留在 Inspect 复查健康详情".to_string(),
            }
        );
    }

    #[test]
    fn current_inspect_feedback_summary_falls_back_for_other_scopes() {
        let mut state = test_state();
        state.set_feedback_with_next_step(
            UiFeedbackLevel::Info,
            UiFeedbackScope::Packages,
            "Packages 已写入",
            "回到 Packages 查看结果",
        );

        let summary = state.current_inspect_feedback_summary(
            "Overview 推荐先进入 Inspect 处理 repo-integrity。",
            "在 Inspect 先看 repo-integrity",
        );

        assert_eq!(
            summary,
            RouteFeedback {
                message: "Overview 推荐先进入 Inspect 处理 repo-integrity。".to_string(),
                next_step: "在 Inspect 先看 repo-integrity".to_string(),
            }
        );
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

    #[test]
    fn inspect_model_detail_tracks_selected_command_focus() {
        let mut state = test_state();
        state.open_inspect();
        state.ensure_inspect_action_focus();
        state.next_inspect_action();
        state.set_feedback_message(
            UiFeedbackLevel::Success,
            UiFeedbackScope::Inspect,
            "check upstream pins 已完成。",
        );

        let model = state.inspect_model();

        assert_eq!(model.selected_index, 1);
        assert_eq!(model.detail.action, ActionItem::UpdateUpstreamCheck);
        assert_eq!(model.detail.group, "Upstream Pins");
        assert_eq!(model.detail.label, "check upstream pins");
        assert!(
            model
                .detail
                .preview
                .as_deref()
                .is_some_and(|preview| preview.contains("update-upstream-apps --check"))
        );
        assert_eq!(model.detail.latest_result, "check upstream pins 已完成。");
        assert_eq!(model.detail.page_title, "Inspect");
    }

    #[test]
    fn inspect_model_prefers_doctor_health_focus_when_repo_integrity_is_clean() {
        let mut state = test_state();
        state.overview_doctor = OverviewCheckState::Error {
            summary: "failed (1 check(s))".to_string(),
            details: vec!["缺少 nixos-rebuild".to_string()],
        };

        let model = state.inspect_model();

        assert_eq!(model.health_focus, InspectHealthFocus::Doctor);
        assert_eq!(model.detail.action, ActionItem::FlakeCheck);
    }

    #[test]
    fn inspect_completion_feedback_uses_action_specific_copy() {
        let mut state = test_state();

        state.set_inspect_completion_feedback(ActionItem::FlakeCheck);
        assert_eq!(state.feedback.scope, UiFeedbackScope::Inspect);
        assert_eq!(state.feedback.level, UiFeedbackLevel::Success);
        assert_eq!(state.feedback.message, "flake check 已完成。");
        assert_eq!(
            state.feedback.next_step.as_deref(),
            Some("切到 Inspect 查看检查结果")
        );

        state.set_inspect_completion_feedback(ActionItem::UpdateUpstreamCheck);
        assert_eq!(state.feedback.scope, UiFeedbackScope::Inspect);
        assert_eq!(state.feedback.level, UiFeedbackLevel::Success);
        assert_eq!(state.feedback.message, "上游 pin 检查已完成。");
        assert_eq!(
            state.feedback.next_step.as_deref(),
            Some("切到 Inspect 查看 pin 状态")
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
            show_advanced: false,
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
