use super::*;
use crate::repo::ensure_repository_integrity;
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InspectModel {
    pub(crate) health_focus: InspectHealthFocus,
    pub(crate) repo_integrity: OverviewCheckState,
    pub(crate) doctor: OverviewCheckState,
    pub(crate) managed_guards: Vec<ManagedGuardSnapshot>,
    pub(crate) commands: Vec<InspectCommandModel>,
    pub(crate) selected_index: usize,
    pub(crate) summary: InspectSummaryModel,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InspectSummaryModel {
    pub(crate) status: String,
    pub(crate) latest_result: String,
    pub(crate) next_step: String,
    pub(crate) primary_action: String,
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
        let commands: Vec<InspectCommandModel> = inspect_actions()
            .iter()
            .copied()
            .map(|action| InspectCommandModel {
                action,
                group: action.group_label(),
                label: action.label(),
                available: self.inspect_action_available(action),
                preview: self.inspect_action_command_preview(action),
            })
            .collect();
        let selected_index = self.selected_inspect_row_index();

        let health_focus =
            preferred_inspect_health_focus(&self.overview_repo_integrity, &self.overview_doctor);
        let feedback = self.current_inspect_feedback_state("无".to_string());
        let command = &commands[selected_index];
        let summary = self.inspect_summary_model(command, health_focus);
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
            health_focus,
            repo_integrity: self.overview_repo_integrity.clone(),
            doctor: self.overview_doctor.clone(),
            managed_guards: self.managed_guard_snapshots(),
            commands,
            selected_index,
            summary,
            detail,
            latest_result: feedback.latest_result,
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
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

    fn inspect_summary_model(
        &self,
        command: &InspectCommandModel,
        health_focus: InspectHealthFocus,
    ) -> InspectSummaryModel {
        let fallback_next_step = self.inspect_summary_next_step(command, health_focus);
        let feedback = self.current_inspect_feedback_state(fallback_next_step);

        InspectSummaryModel {
            status: self.inspect_summary_status(command, health_focus),
            latest_result: feedback.latest_result_text,
            next_step: feedback.next_step,
            primary_action: self.inspect_primary_action(command, health_focus),
        }
    }

    fn inspect_summary_status(
        &self,
        command: &InspectCommandModel,
        health_focus: InspectHealthFocus,
    ) -> String {
        if !command.available {
            return "当前命令需切换场景".to_string();
        }

        match health_focus {
            InspectHealthFocus::RepoIntegrity
                if matches!(
                    self.overview_repo_integrity,
                    OverviewCheckState::Error { .. }
                ) =>
            {
                "当前应先复查 repo-integrity".to_string()
            }
            InspectHealthFocus::Doctor
                if matches!(self.overview_doctor, OverviewCheckState::Error { .. }) =>
            {
                "当前应先复查 doctor".to_string()
            }
            _ => "当前可直接执行当前 Inspect 命令".to_string(),
        }
    }

    fn inspect_primary_action(
        &self,
        command: &InspectCommandModel,
        health_focus: InspectHealthFocus,
    ) -> String {
        if !command.available {
            return "主动作：先切换到适合当前命令的场景".to_string();
        }

        match health_focus {
            InspectHealthFocus::RepoIntegrity
                if matches!(
                    self.overview_repo_integrity,
                    OverviewCheckState::Error { .. }
                ) =>
            {
                "主动作：先看健康详情，再决定是否执行当前检查".to_string()
            }
            InspectHealthFocus::Doctor
                if matches!(self.overview_doctor, OverviewCheckState::Error { .. }) =>
            {
                "主动作：先看健康详情，再决定是否执行当前检查".to_string()
            }
            _ => format!("主动作：按 x 执行 {}", command.label),
        }
    }

    fn inspect_summary_next_step(
        &self,
        command: &InspectCommandModel,
        health_focus: InspectHealthFocus,
    ) -> String {
        if !command.available {
            return format!("先切换到适合 {} 的场景，再回到 Inspect。", command.label);
        }

        match health_focus {
            InspectHealthFocus::RepoIntegrity
                if matches!(
                    self.overview_repo_integrity,
                    OverviewCheckState::Error { .. }
                ) =>
            {
                format!(
                    "先看 repo-integrity 详情；如需复查，再按 x 执行 {}。",
                    command.label
                )
            }
            InspectHealthFocus::Doctor
                if matches!(self.overview_doctor, OverviewCheckState::Error { .. }) =>
            {
                format!(
                    "先看 doctor 详情；如需复查，再按 x 执行 {}。",
                    command.label
                )
            }
            _ => format!("先看健康摘要；如需继续，按 x 执行 {}。", command.label),
        }
    }

    pub(crate) fn ensure_inspect_action_focus(&mut self) {
        if !is_inspect_action(self.inspect_action) {
            self.inspect_action = inspect_actions()[0];
        }
    }

    pub(crate) fn next_inspect_action(&mut self) {
        let current = inspect_action_offset(self.current_inspect_action()).unwrap_or(0);
        self.inspect_action = inspect_actions()[(current + 1) % inspect_action_count()];
    }

    pub(crate) fn previous_inspect_action(&mut self) {
        let current = inspect_action_offset(self.current_inspect_action()).unwrap_or(0);
        let previous = if current == 0 {
            inspect_action_count() - 1
        } else {
            current - 1
        };
        self.inspect_action = inspect_actions()[previous];
    }

    pub(crate) fn current_inspect_action(&self) -> ActionItem {
        let action = self.inspect_action;
        if is_inspect_action(action) {
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

    pub(crate) fn inspect_action_available(&self, action: ActionItem) -> bool {
        is_inspect_action(action)
    }

    pub(crate) fn inspect_action_command_preview(&self, action: ActionItem) -> Option<String> {
        match action {
            ActionItem::FlakeCheck => Some(format!(
                "nix --extra-experimental-features 'nix-command flakes' flake check path:{}",
                self.context.repo_root.display()
            )),
            ActionItem::UpdateUpstreamCheck => Some("update-upstream-apps --check".to_string()),
            _ => None,
        }
    }

    pub(crate) fn execute_current_inspect_action(&mut self) -> Result<()> {
        self.ensure_inspect_action_focus();
        self.ensure_no_unsaved_changes_for_execution()?;
        ensure_repository_integrity(&self.context.repo_root)?;

        let action = self.current_inspect_action();
        if !self.inspect_action_available(action) {
            anyhow::bail!("当前环境暂不适合直接执行动作：{}", action.label());
        }

        match action {
            ActionItem::FlakeCheck => {
                let mut cmd = std::process::Command::new("nix");
                cmd.arg("--extra-experimental-features")
                    .arg("nix-command flakes")
                    .arg("flake")
                    .arg("check")
                    .arg(format!("path:{}", self.context.repo_root.display()))
                    .env("NIX_CONFIG", merged_nix_config())
                    .stdin(std::process::Stdio::inherit())
                    .stdout(std::process::Stdio::inherit())
                    .stderr(std::process::Stdio::inherit());
                let status = cmd.status().context("failed to run nix flake check")?;
                if !status.success() {
                    anyhow::bail!("flake check exited with {}", status.code().unwrap_or(1));
                }
                self.set_inspect_completion_feedback(ActionItem::FlakeCheck);
            }
            ActionItem::UpdateUpstreamCheck => {
                let status =
                    self.run_sibling_in_repo("update-upstream-apps", &["--check".to_string()])?;
                if !status.success() {
                    anyhow::bail!(
                        "update-upstream-apps --check exited with {}",
                        status.code().unwrap_or(1)
                    );
                }
                self.set_inspect_completion_feedback(ActionItem::UpdateUpstreamCheck);
            }
            ActionItem::FlakeUpdate
            | ActionItem::UpdateUpstreamPins
            | ActionItem::LaunchDeployWizard => {
                anyhow::bail!("当前动作不属于 Inspect：{}", action.label())
            }
        }

        Ok(())
    }
}

fn inspect_actions() -> &'static [ActionItem] {
    static ACTIONS: [ActionItem; 2] = [ActionItem::FlakeCheck, ActionItem::UpdateUpstreamCheck];
    &ACTIONS
}

fn is_inspect_action(action: ActionItem) -> bool {
    inspect_action_offset(action).is_some()
}

fn inspect_action_count() -> usize {
    inspect_actions().len()
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
        assert_eq!(model.summary.status, "当前应先复查 repo-integrity");
        assert_eq!(
            model.summary.primary_action,
            "主动作：先看健康详情，再决定是否执行当前检查"
        );
        assert_eq!(
            model.summary.next_step,
            "先看 repo-integrity 详情；如需复查，再按 x 执行 flake check。"
        );
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
        assert_eq!(inspect.summary.latest_result, "flake check 已完成。");
        assert_eq!(inspect.detail.latest_result, "flake check 已完成。");

        state.set_feedback_message(
            UiFeedbackLevel::Info,
            UiFeedbackScope::Apply,
            "当前组合可直接 Apply。",
        );
        let inspect = state.inspect_model();
        assert!(inspect.latest_result.is_none());
        assert_eq!(inspect.summary.latest_result, "无");
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
        state.inspect_action = ActionItem::FlakeUpdate;

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
        assert_eq!(model.summary.status, "当前应先复查 doctor");
        assert_eq!(
            model.summary.next_step,
            "先看 doctor 详情；如需复查，再按 x 执行 flake check。"
        );
        assert_eq!(model.detail.action, ActionItem::FlakeCheck);
    }

    #[test]
    fn inspect_model_uses_direct_summary_when_health_is_clean() {
        let mut state = test_state();
        state.overview_repo_integrity = OverviewCheckState::Healthy {
            summary: "ok".to_string(),
            details: Vec::new(),
        };
        state.overview_doctor = OverviewCheckState::Healthy {
            summary: "ok".to_string(),
            details: Vec::new(),
        };

        let model = state.inspect_model();

        assert_eq!(model.summary.status, "当前可直接执行当前 Inspect 命令");
        assert_eq!(
            model.summary.primary_action,
            "主动作：按 x 执行 flake check"
        );
        assert_eq!(
            model.summary.next_step,
            "先看健康摘要；如需继续，按 x 执行 flake check。"
        );
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
                catalog_workflows_path: PathBuf::from("catalog/workflows.toml"),
                catalog_entries: Vec::new(),
                catalog_groups: BTreeMap::new(),
                catalog_home_options: Vec::new(),
                catalog_workflows: BTreeMap::new(),
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
            package_mode: PackageDataMode::Search,
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
            overview_repo_integrity: OverviewCheckState::NotRun,
            overview_doctor: OverviewCheckState::NotRun,
            feedback: UiFeedback::default(),
            status: String::new(),
        }
    }
}
