#![cfg_attr(not(test), allow(dead_code))]

use super::*;
use crate::domain::tui::ActionDestination;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ActionDisplayRow {
    pub(crate) label: String,
    pub(crate) value: String,
    pub(crate) selectable: bool,
}

impl AppState {
    pub fn next_action_item(&mut self) {
        self.actions_focus = (self.actions_focus + 1) % ActionItem::ALL.len();
    }

    pub fn previous_action_item(&mut self) {
        self.actions_focus = if self.actions_focus == 0 {
            ActionItem::ALL.len() - 1
        } else {
            self.actions_focus - 1
        };
    }

    pub fn current_action_item(&self) -> ActionItem {
        ActionItem::ALL[self.actions_focus]
    }

    pub(crate) fn action_display_rows(&self) -> Vec<ActionDisplayRow> {
        let mut rows = Vec::new();
        let mut current_destination = None;

        for item in ActionItem::ALL {
            let destination = item.destination();
            if current_destination != Some(destination) {
                rows.push(ActionDisplayRow {
                    label: destination.label().to_string(),
                    value: String::new(),
                    selectable: false,
                });
                current_destination = Some(destination);
            }

            rows.push(ActionDisplayRow {
                label: format!("{} / {}", item.group_label(), item.label()),
                value: if self.action_available(item) {
                    "可执行".to_string()
                } else {
                    "需切换场景".to_string()
                },
                selectable: true,
            });
        }

        rows
    }

    pub(crate) fn selected_action_row_index(&self) -> usize {
        let mut row_index = 0;
        let mut current_destination = None;

        for (index, item) in ActionItem::ALL.iter().enumerate() {
            let destination = item.destination();
            if current_destination != Some(destination) {
                row_index += 1;
                current_destination = Some(destination);
            }
            if index == self.actions_focus {
                return row_index;
            }
            row_index += 1;
        }

        0
    }

    pub(crate) fn open_current_action_destination(&mut self) {
        let action = self.current_action_item();
        match action.destination() {
            ActionDestination::Inspect => {
                let feedback = self.actions_inspect_route_feedback(action);
                self.open_inspect();
                self.set_feedback_with_next_step(
                    UiFeedbackLevel::Info,
                    UiFeedbackScope::Inspect,
                    feedback.message,
                    feedback.next_step,
                );
            }
            ActionDestination::Apply => {
                let feedback = self.actions_apply_route_feedback(action);
                self.open_apply();
                self.set_feedback_with_next_step(
                    UiFeedbackLevel::Info,
                    UiFeedbackScope::Apply,
                    feedback.message,
                    feedback.next_step,
                );
            }
            ActionDestination::Advanced => {
                self.sync_advanced_deploy_parameters_from_apply();
                self.focus_advanced_action(action);
                self.open_advanced();
                let feedback = self.actions_advanced_route_feedback(action);
                self.set_feedback_with_next_step(
                    UiFeedbackLevel::Info,
                    UiFeedbackScope::Advanced,
                    feedback.message,
                    feedback.next_step,
                );
            }
        }
    }

    pub fn actions_summary_lines(&self) -> Vec<String> {
        let action = self.current_action_item();
        let mut lines = vec![
            format!("当前动作：{}", action.label()),
            format!("归宿：{}", action.destination().label()),
            format!("分组：{}", action.group_label()),
            format!("说明：{}", action.description()),
            format!("当前仓库：{}", self.context.repo_root.display()),
            format!("/etc/nixos：{}", self.context.etc_root.display()),
            format!("当前主机：{}", self.target_host),
            format!(
                "权限：{}",
                match self.context.privilege_mode.as_str() {
                    "root" => "root",
                    "sudo-session" => "sudo session",
                    "sudo-available" => "sudo available",
                    _ => "rootless",
                }
            ),
        ];

        if let Some(preview) = self.action_command_preview(action) {
            lines.push(format!("命令预览：{preview}"));
        }
        if self.action_available(action) {
            lines.push("状态：当前环境可以直接执行".to_string());
        } else {
            lines.push("状态：当前环境不适合直接执行；请改用 Apply 页或切换权限".to_string());
        }
        lines.push(action_transition_hint(action).to_string());

        lines.push(String::new());
        lines.push("当前页说明：".to_string());
        lines.push("- 当前页是过渡入口：先按 Inspect / Apply / Advanced 给动作分组。".to_string());
        lines.push("- 当前页现在只做跳转，不再直接执行动作。".to_string());
        lines.push("- Advanced 动作会进入独立的 Advanced 区，而不是继续停在 Apply。".to_string());
        lines
    }
}

fn action_transition_hint(action: ActionItem) -> &'static str {
    match action.destination() {
        ActionDestination::Inspect => "默认行为：Enter/Space/x 打开 Inspect，不在当前页直接执行。",
        ActionDestination::Apply => "默认行为：Enter/Space/x 打开 Apply，不在当前页直接执行。",
        ActionDestination::Advanced => "默认行为：Enter/Space/x 打开 Advanced 区。",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};

    #[test]
    fn action_items_have_stable_destinations_and_groups() {
        assert_eq!(
            ActionItem::FlakeCheck.destination(),
            ActionDestination::Inspect
        );
        assert_eq!(
            ActionItem::UpdateUpstreamCheck.destination(),
            ActionDestination::Inspect
        );
        assert_eq!(
            ActionItem::SyncRepoToEtc.destination(),
            ActionDestination::Apply
        );
        assert_eq!(
            ActionItem::RebuildCurrentHost.destination(),
            ActionDestination::Apply
        );
        assert_eq!(
            ActionItem::FlakeUpdate.destination(),
            ActionDestination::Advanced
        );
        assert_eq!(
            ActionItem::UpdateUpstreamPins.destination(),
            ActionDestination::Advanced
        );
        assert_eq!(
            ActionItem::LaunchDeployWizard.destination(),
            ActionDestination::Advanced
        );
        assert_eq!(ActionItem::FlakeCheck.group_label(), "Repo Checks");
        assert_eq!(
            ActionItem::SyncRepoToEtc.group_label(),
            "Manual Apply Helpers"
        );
    }

    #[test]
    fn action_display_rows_insert_destination_headers() {
        let state = test_state("sudo-available");
        let rows = state.action_display_rows();

        assert_eq!(rows[0].label, "Inspect");
        assert!(!rows[0].selectable);
        assert_eq!(rows[1].label, "Repo Checks / flake check");
        assert!(rows[1].selectable);
        assert_eq!(rows[3].label, "Apply");
        assert_eq!(rows[6].label, "Advanced");
        assert_eq!(state.selected_action_row_index(), 1);
    }

    #[test]
    fn action_summary_lines_include_destination_and_group() {
        let mut state = test_state("sudo-available");
        state.actions_focus = 4;

        let lines = state.actions_summary_lines();

        assert!(lines.iter().any(|line| line.contains("归宿：Advanced")));
        assert!(
            lines
                .iter()
                .any(|line| line.contains("分组：Repository Maintenance"))
        );
        assert!(lines.iter().any(|line| line.contains("打开 Advanced 区")));
    }

    #[test]
    fn action_summary_lines_explain_route_only_behavior_for_inspect_and_apply() {
        let inspect_lines = test_state("sudo-available").actions_summary_lines();
        assert!(
            inspect_lines
                .iter()
                .any(|line| line.contains("Enter/Space/x 打开 Inspect"))
        );

        let mut apply = test_state("sudo-available");
        apply.actions_focus = 2;
        let apply_lines = apply.actions_summary_lines();
        assert!(
            apply_lines
                .iter()
                .any(|line| line.contains("Enter/Space/x 打开 Apply"))
        );
    }

    #[test]
    fn open_current_action_destination_routes_to_transition_pages() {
        let mut inspect = test_state("sudo-available");
        inspect.open_current_action_destination();
        assert_eq!(inspect.page(), Page::Inspect);
        assert!(inspect.status.contains("Inspect"));

        let mut apply = test_state("sudo-available");
        apply.actions_focus = 2;
        apply.open_current_action_destination();
        assert_eq!(apply.page(), Page::Deploy);
        assert!(!apply.show_advanced);
        assert_eq!(apply.feedback.scope, UiFeedbackScope::Apply);
        assert!(apply.feedback.message.contains("Apply"));
        assert!(apply.feedback.message.contains("sync to /etc/nixos"));

        let mut advanced = test_state("sudo-available");
        advanced.actions_focus = 4;
        advanced.open_current_action_destination();
        assert_eq!(advanced.page(), Page::Advanced);
        assert!(advanced.advanced_workspace_visible());
        assert!(!advanced.show_advanced);
        assert_eq!(advanced.current_advanced_action(), ActionItem::FlakeUpdate);
        assert!(advanced.status.contains("对准 flake update"));
    }

    #[test]
    fn open_current_action_destination_keeps_selected_maintenance_action_with_wizard_recommendation()
     {
        let mut state = test_state("sudo-available");
        state.actions_focus = 4;
        state.deploy_source = DeploySource::RemoteHead;

        state.open_current_action_destination();

        assert_eq!(state.page(), Page::Advanced);
        assert_eq!(state.current_advanced_action(), ActionItem::FlakeUpdate);
        assert_eq!(state.feedback.scope, UiFeedbackScope::Advanced);
        assert_eq!(
            state.feedback.message,
            "flake update 归属 Advanced；已跳到 Advanced，并定位到 flake update；默认推荐是 launch deploy wizard。推荐原因：当前来源是远端最新版本；默认 Apply 不会直接执行，必须交给完整高级路径。"
        );
        assert_eq!(
            state.feedback.next_step.as_deref(),
            Some(
                "在 Advanced 里先看摘要里的“推荐动作：launch deploy wizard”；如无特殊目的，先切过去。"
            )
        );
    }

    #[test]
    fn open_current_action_destination_routes_wizard_with_aligned_focus_and_next_step() {
        let mut state = test_state("sudo-available");
        state.actions_focus = 6;
        state.deploy_source = DeploySource::RemotePinned;
        state.deploy_source_ref = "v5.0.0".to_string();
        state.deploy_focus = 6;

        state.open_current_action_destination();

        assert_eq!(state.page(), Page::Advanced);
        assert_eq!(
            state.current_advanced_action(),
            ActionItem::LaunchDeployWizard
        );
        assert_eq!(state.advanced_deploy_focus, 3);
        assert_eq!(state.feedback.scope, UiFeedbackScope::Advanced);
        assert_eq!(
            state.feedback.message,
            "launch deploy wizard 归属 Advanced；已跳到 Advanced，并对准 launch deploy wizard。推荐原因：当前来源是远端固定版本；默认 Apply 不会直接执行，必须交给完整高级路径。"
        );
        assert_eq!(
            state.feedback.next_step.as_deref(),
            Some("在 Advanced 里先确认 Deploy Parameters，再执行 launch deploy wizard")
        );
    }

    #[test]
    fn open_current_action_destination_routes_inspect_with_active_health_reason() {
        let mut state = test_state("sudo-available");
        state.overview_doctor = OverviewCheckState::Error {
            summary: "failed (missing nixos-rebuild)".to_string(),
            details: vec!["- deployment environment: missing nixos-rebuild".to_string()],
        };

        state.open_current_action_destination();

        assert_eq!(state.page(), Page::Inspect);
        assert_eq!(state.feedback.scope, UiFeedbackScope::Inspect);
        assert_eq!(
            state.feedback.message,
            "flake check 归属 Inspect；当前应先处理 doctor（failed (missing nixos-rebuild)）。"
        );
        assert_eq!(
            state.feedback.next_step.as_deref(),
            Some("在 Inspect 先看 doctor 详情；如需仓库校验，再执行 flake check")
        );
    }

    #[test]
    fn open_current_action_destination_routes_inspect_without_active_health_reason() {
        let mut state = test_state("sudo-available");

        state.open_current_action_destination();

        assert_eq!(state.page(), Page::Inspect);
        assert_eq!(state.feedback.scope, UiFeedbackScope::Inspect);
        assert_eq!(
            state.feedback.message,
            "flake check 归属 Inspect；已跳到 Inspect 页查看健康详情和命令预览。"
        );
        assert_eq!(
            state.feedback.next_step.as_deref(),
            Some("在 Inspect 查看健康详情和检查命令")
        );
    }

    #[test]
    fn open_current_action_destination_routes_apply_with_handoff_feedback() {
        let mut state = test_state("sudo-available");
        state.actions_focus = 2;
        state.deploy_source = DeploySource::RemotePinned;
        state.deploy_source_ref = "v5.0.0".to_string();

        state.open_current_action_destination();

        assert_eq!(state.page(), Page::Deploy);
        assert_eq!(state.feedback.scope, UiFeedbackScope::Apply);
        assert_eq!(
            state.feedback.message,
            "sync to /etc/nixos 归属 Apply；当前来源是远端固定版本；默认 Apply 不会直接执行，必须交给完整高级路径。"
        );
        assert_eq!(
            state.feedback.next_step.as_deref(),
            Some("在 Apply 先看 handoff 预览；如需继续，切到 Advanced 执行 launch deploy wizard")
        );
    }

    fn test_state(privilege_mode: &str) -> AppState {
        AppState {
            context: AppContext {
                repo_root: PathBuf::from("/repo"),
                etc_root: PathBuf::from("/etc/nixos"),
                current_host: "demo".to_string(),
                current_system: "x86_64-linux".to_string(),
                current_user: "alice".to_string(),
                privilege_mode: privilege_mode.to_string(),
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
