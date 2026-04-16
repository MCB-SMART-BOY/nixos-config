use super::*;
use crate::domain::tui::ActionDestination;
use crate::repo::ensure_repository_integrity;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ApplyModel {
    pub(crate) target_host: String,
    pub(crate) task: DeployTask,
    pub(crate) source: DeploySource,
    pub(crate) source_detail: Option<String>,
    pub(crate) action: DeployAction,
    pub(crate) flake_update: bool,
    pub(crate) advanced: bool,
    pub(crate) sync_preview: Option<String>,
    pub(crate) rebuild_preview: Option<String>,
    pub(crate) can_execute_directly: bool,
    pub(crate) can_apply_current_host: bool,
    pub(crate) blockers: Vec<String>,
    pub(crate) warnings: Vec<String>,
    pub(crate) handoffs: Vec<String>,
    pub(crate) infos: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AdvancedWizardModel {
    pub(crate) target_host: String,
    pub(crate) task: DeployTask,
    pub(crate) source: DeploySource,
    pub(crate) source_detail: Option<String>,
    pub(crate) action: DeployAction,
    pub(crate) flake_update: bool,
    pub(crate) sync_preview: Option<String>,
    pub(crate) rebuild_preview: Option<String>,
    pub(crate) blockers: Vec<String>,
    pub(crate) warnings: Vec<String>,
    pub(crate) handoffs: Vec<String>,
    pub(crate) infos: Vec<String>,
    pub(crate) command_preview: String,
    pub(crate) validation_error: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DeployParameterSnapshot {
    target_host: String,
    task: DeployTask,
    source: DeploySource,
    source_ref: String,
    action: DeployAction,
    flake_update: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AdvancedSummaryModel {
    pub(crate) current_action: ActionItem,
    pub(crate) recommended_action: ActionItem,
    pub(crate) reason: String,
    pub(crate) completion_hint: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AdvancedContextModel {
    pub(crate) focused_row: DeployControlRow,
    pub(crate) default_target: String,
    pub(crate) recommendation: String,
    pub(crate) execution_hint: String,
    pub(crate) operation_hint: String,
    pub(crate) advanced_action_hint: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AdvancedMaintenanceModel {
    pub(crate) summary: AdvancedSummaryModel,
    pub(crate) write_target: &'static str,
    pub(crate) impact: &'static str,
    pub(crate) return_hint: String,
    pub(crate) command_preview: Option<String>,
    pub(crate) latest_result: String,
    pub(crate) repo_root: PathBuf,
    pub(crate) available: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AdvancedWizardDetailModel {
    pub(crate) action: ActionItem,
    pub(crate) recommended_action: ActionItem,
    pub(crate) reason: String,
    pub(crate) status: String,
    pub(crate) command_preview: String,
    pub(crate) completion_hint: String,
    pub(crate) latest_result: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ApplyAdvancedWorkspaceModel {
    pub(crate) top_level_label: String,
    pub(crate) action: ActionItem,
    pub(crate) status: String,
    pub(crate) command_preview: String,
    pub(crate) latest_result: String,
    pub(crate) operation_hint: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ApplySelectionModel {
    pub(crate) focused_row: DeployControlRow,
    pub(crate) default_target: String,
    pub(crate) recommendation: String,
    pub(crate) execution_hint: String,
    pub(crate) operation_hint: String,
    pub(crate) advanced_action_hint: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ApplyExecutionGateModel {
    pub(crate) status: String,
    pub(crate) latest_result: String,
    pub(crate) primary_action: String,
    pub(crate) blockers: Vec<String>,
    pub(crate) warnings: Vec<String>,
    pub(crate) handoffs: Vec<String>,
    pub(crate) infos: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ApplyGuidanceState {
    Direct,
    WorkspaceOpen,
    Handoff,
    Blocked,
    Review,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ApplyGuidanceCopyKind {
    Direct,
    WorkspaceOpen,
    Handoff(ActionItem),
    Blocked,
    Review,
}

enum ApplyRouteOrigin {
    Overview,
    Actions(ActionItem),
    AdvancedReturn,
}

enum AdvancedRouteOrigin {
    Overview,
    Apply,
    Actions(ActionItem),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RebuildCompletionOrigin {
    Apply,
    CurrentHostAction,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ApplyGuidanceModel {
    state: ApplyGuidanceState,
    feedback_detail: String,
    next_step: String,
    gate_status: String,
    gate_primary_action: String,
    recommendation: String,
    execution_hint: String,
    advanced_action_hint: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ApplyGuidanceCopy {
    gate_status: String,
    next_step: String,
    gate_primary_action: String,
    recommendation: String,
    execution_hint: String,
    advanced_action_hint: String,
    preview_command_fallback: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ApplyFeedbackState {
    recent_feedback: String,
    next_step: String,
    latest_result: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AdvancedFeedbackState {
    active: bool,
    recent_feedback: String,
    completion_hint: String,
    latest_result: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RouteFeedback {
    pub(crate) message: String,
    pub(crate) next_step: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CompletionFeedback {
    message: String,
    next_step: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AdvancedActionDisplayRow {
    pub(crate) label: String,
    pub(crate) value: String,
    pub(crate) selectable: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AdvancedActionsListModel {
    pub(crate) rows: Vec<AdvancedActionDisplayRow>,
    pub(crate) selected_index: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DeployShellMode {
    Apply,
    AdvancedMaintenance,
    AdvancedWizard,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DeployShellModel {
    pub(crate) mode: DeployShellMode,
    pub(crate) workspace_visible: bool,
    pub(crate) summary_title: &'static str,
    pub(crate) preview_title: &'static str,
    pub(crate) context_title: &'static str,
    pub(crate) controls_title: &'static str,
    pub(crate) detail_title: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ApplyPageModel {
    pub(crate) shell: DeployShellModel,
    pub(crate) apply: ApplyModel,
    pub(crate) gate: ApplyExecutionGateModel,
    pub(crate) preview_command_fallback: String,
    pub(crate) selection: ApplySelectionModel,
    pub(crate) controls: DeployControlsModel,
    pub(crate) advanced_actions: Option<AdvancedActionsListModel>,
    pub(crate) workspace: Option<ApplyAdvancedWorkspaceModel>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AdvancedMaintenancePageModel {
    pub(crate) shell: DeployShellModel,
    pub(crate) maintenance: AdvancedMaintenanceModel,
    pub(crate) context: AdvancedContextModel,
    pub(crate) advanced_actions: Option<AdvancedActionsListModel>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AdvancedWizardPageModel {
    pub(crate) shell: DeployShellModel,
    pub(crate) summary: AdvancedSummaryModel,
    pub(crate) wizard: AdvancedWizardModel,
    pub(crate) context: AdvancedContextModel,
    pub(crate) controls: DeployControlsModel,
    pub(crate) detail: AdvancedWizardDetailModel,
    pub(crate) advanced_actions: Option<AdvancedActionsListModel>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum DeployPageModel {
    Apply(Box<ApplyPageModel>),
    AdvancedMaintenance(Box<AdvancedMaintenancePageModel>),
    AdvancedWizard(Box<AdvancedWizardPageModel>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DeployControlRow {
    pub(crate) label: String,
    pub(crate) value: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DeployControlsModel {
    pub(crate) rows: Vec<DeployControlRow>,
    pub(crate) selected_focus: usize,
    pub(crate) focused_row: DeployControlRow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DeployControlScope {
    Apply,
    AdvancedWizard,
}

impl DeployControlScope {
    fn text_mode(self) -> DeployTextMode {
        match self {
            DeployControlScope::Apply => DeployTextMode::ApplyRemotePinnedRef,
            DeployControlScope::AdvancedWizard => DeployTextMode::AdvancedWizardRemotePinnedRef,
        }
    }

    fn label(self) -> &'static str {
        match self {
            DeployControlScope::Apply => "Apply",
            DeployControlScope::AdvancedWizard => "Advanced Wizard",
        }
    }

    fn from_text_mode(mode: DeployTextMode) -> Self {
        match mode {
            DeployTextMode::ApplyRemotePinnedRef => DeployControlScope::Apply,
            DeployTextMode::AdvancedWizardRemotePinnedRef => DeployControlScope::AdvancedWizard,
        }
    }
}

impl AppState {
    fn apply_parameter_snapshot(&self) -> DeployParameterSnapshot {
        DeployParameterSnapshot {
            target_host: self.target_host.clone(),
            task: self.deploy_task,
            source: self.deploy_source,
            source_ref: self.deploy_source_ref.clone(),
            action: self.deploy_action,
            flake_update: self.flake_update,
        }
    }

    fn advanced_parameter_snapshot(&self) -> DeployParameterSnapshot {
        DeployParameterSnapshot {
            target_host: self.advanced_target_host.clone(),
            task: self.advanced_deploy_task,
            source: self.advanced_deploy_source,
            source_ref: self.advanced_deploy_source_ref.clone(),
            action: self.advanced_deploy_action,
            flake_update: self.advanced_flake_update,
        }
    }

    fn parameter_snapshot_for_scope(&self, scope: DeployControlScope) -> DeployParameterSnapshot {
        match scope {
            DeployControlScope::Apply => self.apply_parameter_snapshot(),
            DeployControlScope::AdvancedWizard => self.advanced_parameter_snapshot(),
        }
    }

    fn current_parameter_snapshot(&self) -> DeployParameterSnapshot {
        self.parameter_snapshot_for_scope(self.current_deploy_control_scope())
    }

    fn advanced_recommendation_snapshot(&self) -> DeployParameterSnapshot {
        if self.page() == Page::Advanced {
            self.advanced_parameter_snapshot()
        } else {
            self.apply_parameter_snapshot()
        }
    }

    pub(crate) fn sync_advanced_deploy_parameters_from_apply(&mut self) {
        self.advanced_target_host = self.target_host.clone();
        self.advanced_deploy_task = self.deploy_task;
        self.advanced_deploy_source = self.deploy_source;
        self.advanced_deploy_source_ref = self.deploy_source_ref.clone();
        self.advanced_deploy_action = self.deploy_action;
        self.advanced_flake_update = self.flake_update;
        self.advanced_deploy_focus = advanced_handoff_focus_for_snapshot(
            &self.apply_parameter_snapshot(),
            self.deploy_focus,
        );
    }

    fn current_deploy_control_scope(&self) -> DeployControlScope {
        match self.page() {
            Page::Advanced => DeployControlScope::AdvancedWizard,
            _ => DeployControlScope::Apply,
        }
    }

    fn deploy_focus_for_scope(&self, scope: DeployControlScope) -> usize {
        match scope {
            DeployControlScope::Apply => self.deploy_focus,
            DeployControlScope::AdvancedWizard => self.advanced_deploy_focus,
        }
    }

    fn deploy_focus_mut_for_scope(&mut self, scope: DeployControlScope) -> &mut usize {
        match scope {
            DeployControlScope::Apply => &mut self.deploy_focus,
            DeployControlScope::AdvancedWizard => &mut self.advanced_deploy_focus,
        }
    }

    fn active_deploy_focus(&self) -> usize {
        self.deploy_focus_for_scope(self.current_deploy_control_scope())
    }

    fn next_deploy_field_for_scope(&mut self, scope: DeployControlScope) {
        let focus = self.deploy_focus_mut_for_scope(scope);
        *focus = (*focus + 1) % 7;
    }

    fn previous_deploy_field_for_scope(&mut self, scope: DeployControlScope) {
        let focus = self.deploy_focus_mut_for_scope(scope);
        *focus = if *focus == 0 { 6 } else { *focus - 1 };
    }

    fn adjust_deploy_field_for_scope(&mut self, scope: DeployControlScope, delta: i8) {
        match self.deploy_focus_for_scope(scope) {
            0 => {
                let hosts = self.context.hosts.clone();
                match scope {
                    DeployControlScope::Apply => cycle_string(&mut self.target_host, &hosts, delta),
                    DeployControlScope::AdvancedWizard => {
                        cycle_string(&mut self.advanced_target_host, &hosts, delta)
                    }
                }
            }
            1 => match scope {
                DeployControlScope::Apply => {
                    cycle_enum(&mut self.deploy_task, &DeployTask::ALL, delta)
                }
                DeployControlScope::AdvancedWizard => {
                    cycle_enum(&mut self.advanced_deploy_task, &DeployTask::ALL, delta)
                }
            },
            2 => match scope {
                DeployControlScope::Apply => {
                    cycle_enum(&mut self.deploy_source, &DeploySource::ALL, delta)
                }
                DeployControlScope::AdvancedWizard => {
                    cycle_enum(&mut self.advanced_deploy_source, &DeploySource::ALL, delta)
                }
            },
            3 => self.open_deploy_text_edit_for_scope(scope),
            4 => match scope {
                DeployControlScope::Apply => {
                    cycle_enum(&mut self.deploy_action, &DeployAction::ALL, delta)
                }
                DeployControlScope::AdvancedWizard => {
                    cycle_enum(&mut self.advanced_deploy_action, &DeployAction::ALL, delta)
                }
            },
            5 => match scope {
                DeployControlScope::Apply => self.flake_update = !self.flake_update,
                DeployControlScope::AdvancedWizard => {
                    self.advanced_flake_update = !self.advanced_flake_update
                }
            },
            6 => match scope {
                DeployControlScope::Apply => {
                    self.sync_advanced_deploy_parameters_from_apply();
                    self.focus_recommended_advanced_action();
                    self.open_advanced();
                    let feedback = self.apply_advanced_route_feedback();
                    self.set_feedback_with_next_step(
                        UiFeedbackLevel::Info,
                        UiFeedbackScope::Advanced,
                        feedback.message,
                        feedback.next_step,
                    );
                }
                DeployControlScope::AdvancedWizard => self.open_apply(),
            },
            _ => {}
        }
    }

    fn deploy_source_label_for_scope(&self, scope: DeployControlScope) -> &'static str {
        self.parameter_snapshot_for_scope(scope).source.label()
    }

    pub fn next_apply_control(&mut self) {
        self.next_deploy_field_for_scope(DeployControlScope::Apply);
    }

    pub fn previous_apply_control(&mut self) {
        self.previous_deploy_field_for_scope(DeployControlScope::Apply);
    }

    pub fn adjust_apply_control(&mut self, delta: i8) {
        self.adjust_deploy_field_for_scope(DeployControlScope::Apply, delta);
    }

    pub fn next_advanced_wizard_field(&mut self) {
        self.next_deploy_field_for_scope(DeployControlScope::AdvancedWizard);
    }

    pub fn previous_advanced_wizard_field(&mut self) {
        self.previous_deploy_field_for_scope(DeployControlScope::AdvancedWizard);
    }

    pub fn adjust_advanced_wizard_field(&mut self, delta: i8) {
        self.adjust_deploy_field_for_scope(DeployControlScope::AdvancedWizard, delta);
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn deploy_rows(&self) -> Vec<DeployControlRow> {
        if self.page() == Page::Advanced {
            self.advanced_wizard_controls_model().rows
        } else {
            self.apply_controls_model().rows
        }
    }

    pub fn can_execute_deploy_directly(&self) -> bool {
        can_execute_deploy_directly_for(self.deploy_source, self.show_advanced)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn selected_deploy_focus(&self) -> usize {
        self.active_deploy_focus()
    }

    pub(crate) fn apply_controls_model(&self) -> DeployControlsModel {
        self.build_deploy_controls_model(
            &self.apply_parameter_snapshot(),
            self.deploy_focus,
            deploy_area_row_for(false, self.show_advanced),
        )
    }

    pub(crate) fn advanced_wizard_controls_model(&self) -> DeployControlsModel {
        self.build_deploy_controls_model(
            &self.advanced_parameter_snapshot(),
            self.advanced_deploy_focus,
            deploy_area_row_for(true, false),
        )
    }

    pub(crate) fn apply_deploy_source_label(&self) -> &'static str {
        self.deploy_source_label_for_scope(DeployControlScope::Apply)
    }

    pub(crate) fn advanced_deploy_source_label(&self) -> &'static str {
        self.deploy_source_label_for_scope(DeployControlScope::AdvancedWizard)
    }

    pub(crate) fn apply_deploy_wizard_args(&self) -> Vec<String> {
        deploy_wizard_args_for_snapshot(&self.apply_parameter_snapshot())
    }

    pub(crate) fn current_deploy_wizard_args(&self) -> Vec<String> {
        if self.page() == Page::Advanced {
            deploy_wizard_args_for_snapshot(&self.advanced_parameter_snapshot())
        } else {
            self.apply_deploy_wizard_args()
        }
    }

    pub(crate) fn current_deploy_wizard_command_preview(&self) -> String {
        let preview = command_preview_for_program("mcb-deploy", &self.current_deploy_wizard_args());
        match self.current_deploy_wizard_validation_error() {
            Some(error) => format!("{preview}  [blocked: {error}]"),
            None => preview,
        }
    }

    pub(crate) fn current_deploy_wizard_validation_error(&self) -> Option<String> {
        deploy_wizard_validation_error_for_snapshot(&self.current_parameter_snapshot())
    }

    fn apply_deploy_wizard_validation_error(&self) -> Option<String> {
        deploy_wizard_validation_error_for_snapshot(&self.apply_parameter_snapshot())
    }

    pub(crate) fn advanced_action_items(&self) -> Vec<ActionItem> {
        ordered_advanced_actions(self.recommended_advanced_action()).to_vec()
    }

    pub(crate) fn advanced_action_display_rows(&self) -> Vec<AdvancedActionDisplayRow> {
        let recommended = self.recommended_advanced_action();
        let mut rows = Vec::new();
        let mut current_group = None;

        for action in self.advanced_action_items() {
            let group = action.group_label();
            if current_group != Some(group) {
                rows.push(AdvancedActionDisplayRow {
                    label: group.to_string(),
                    value: String::new(),
                    selectable: false,
                });
                current_group = Some(group);
            }

            let mut tags = Vec::new();
            if action == recommended {
                tags.push("推荐");
            }
            tags.push(if self.action_available(action) {
                "可执行"
            } else {
                "需切换场景"
            });

            rows.push(AdvancedActionDisplayRow {
                label: action.label().to_string(),
                value: tags.join(" / "),
                selectable: true,
            });
        }

        rows
    }

    pub(crate) fn advanced_actions_list_model(&self) -> AdvancedActionsListModel {
        AdvancedActionsListModel {
            rows: self.advanced_action_display_rows(),
            selected_index: self.selected_advanced_row_index(),
        }
    }

    pub(crate) fn ensure_advanced_action_focus(&mut self) {
        if self.current_action_item().destination() != ActionDestination::Advanced {
            self.set_advanced_action_focus(self.advanced_action_items()[0]);
        }
    }

    pub(crate) fn next_advanced_action(&mut self) {
        let actions = self.advanced_action_items();
        let current = actions
            .iter()
            .position(|candidate| *candidate == self.current_advanced_action())
            .unwrap_or(0);
        self.set_advanced_action_focus(actions[(current + 1) % actions.len()]);
    }

    pub(crate) fn previous_advanced_action(&mut self) {
        let actions = self.advanced_action_items();
        let current = actions
            .iter()
            .position(|candidate| *candidate == self.current_advanced_action())
            .unwrap_or(0);
        let previous = if current == 0 {
            actions.len() - 1
        } else {
            current - 1
        };
        self.set_advanced_action_focus(actions[previous]);
    }

    pub(crate) fn current_advanced_action(&self) -> ActionItem {
        let action = self.current_advanced_action_or_default();
        if action.destination() == ActionDestination::Advanced {
            action
        } else {
            self.recommended_advanced_action()
        }
    }

    pub(crate) fn selected_advanced_row_index(&self) -> usize {
        let mut row_index = 0;
        let mut current_group = None;

        for action in self.advanced_action_items() {
            let group = action.group_label();
            if current_group != Some(group) {
                row_index += 1;
                current_group = Some(group);
            }
            if action == self.current_advanced_action() {
                return row_index;
            }
            row_index += 1;
        }

        0
    }

    pub(crate) fn advanced_action_uses_deploy_parameters(&self) -> bool {
        matches!(
            self.current_advanced_action(),
            ActionItem::LaunchDeployWizard
        )
    }

    pub(crate) fn deploy_shell_model(&self) -> DeployShellModel {
        let advanced_entry = self.page() == Page::Advanced;
        let workspace_visible = self.advanced_workspace_visible();

        if !advanced_entry {
            return DeployShellModel {
                mode: DeployShellMode::Apply,
                workspace_visible,
                summary_title: "Execution Gate",
                preview_title: "Apply Preview",
                context_title: "Current Selection",
                controls_title: "Apply Controls",
                detail_title: "Advanced Detail",
            };
        }

        if self.advanced_action_uses_deploy_parameters() {
            DeployShellModel {
                mode: DeployShellMode::AdvancedWizard,
                workspace_visible,
                summary_title: "Advanced Summary",
                preview_title: "Deploy Preview",
                context_title: "Advanced Context",
                controls_title: "Deploy Parameters",
                detail_title: "Deploy Wizard Detail",
            }
        } else {
            DeployShellModel {
                mode: DeployShellMode::AdvancedMaintenance,
                workspace_visible,
                summary_title: "Advanced Summary",
                preview_title: "Maintenance Preview",
                context_title: "Advanced Context",
                controls_title: "Repository Context",
                detail_title: "Maintenance Detail",
            }
        }
    }

    pub(crate) fn deploy_page_model(&self) -> DeployPageModel {
        let shell = self.deploy_shell_model();
        let workspace_visible = shell.workspace_visible;
        let advanced_actions = shell
            .workspace_visible
            .then(|| self.advanced_actions_list_model());

        match shell.mode {
            DeployShellMode::Apply => DeployPageModel::Apply(Box::new(ApplyPageModel {
                shell,
                apply: self.apply_model(),
                gate: self.apply_execution_gate_model(),
                preview_command_fallback: self.apply_preview_command_fallback(),
                selection: self.apply_selection_model(),
                controls: self.apply_controls_model(),
                advanced_actions,
                workspace: workspace_visible.then(|| self.apply_advanced_workspace_model()),
            })),
            DeployShellMode::AdvancedMaintenance => {
                DeployPageModel::AdvancedMaintenance(Box::new(AdvancedMaintenancePageModel {
                    shell,
                    maintenance: self.advanced_maintenance_model(),
                    context: self.advanced_context_model(),
                    advanced_actions,
                }))
            }
            DeployShellMode::AdvancedWizard => {
                DeployPageModel::AdvancedWizard(Box::new(AdvancedWizardPageModel {
                    shell,
                    summary: self.advanced_summary_model(),
                    wizard: self.advanced_wizard_model(),
                    context: self.advanced_context_model(),
                    controls: self.advanced_wizard_controls_model(),
                    detail: self.advanced_wizard_detail_model(),
                    advanced_actions,
                }))
            }
        }
    }

    pub(crate) fn execute_current_advanced_action_from_apply(&mut self) -> Result<()> {
        self.ensure_advanced_action_focus();
        self.execute_current_action()
    }

    pub(crate) fn recommended_advanced_action(&self) -> ActionItem {
        if matches!(
            self.advanced_recommendation_snapshot().source,
            DeploySource::RemotePinned | DeploySource::RemoteHead
        ) {
            ActionItem::LaunchDeployWizard
        } else {
            self.current_advanced_action_or_default()
        }
    }

    pub(crate) fn focus_recommended_advanced_action(&mut self) {
        self.set_advanced_action_focus(self.recommended_advanced_action());
    }

    pub(crate) fn focus_advanced_action(&mut self, action: ActionItem) {
        self.set_advanced_action_focus(action);
    }

    fn advanced_route_next_step(
        &self,
        current_action: ActionItem,
        recommended_action: ActionItem,
    ) -> String {
        if current_action == recommended_action {
            return match current_action {
                ActionItem::LaunchDeployWizard => {
                    "在 Advanced 里先确认 Deploy Parameters，再执行 launch deploy wizard"
                        .to_string()
                }
                action => format!(
                    "在 Advanced 里先确认当前高级动作，再执行 {}",
                    action.label()
                ),
            };
        }

        format!(
            "在 Advanced 里先看摘要里的“推荐动作：{}”；如无特殊目的，先切过去。",
            recommended_action.label()
        )
    }

    fn advanced_route_feedback_for(&self, origin: AdvancedRouteOrigin) -> RouteFeedback {
        let summary = self.advanced_summary_model();

        let message = match origin {
            AdvancedRouteOrigin::Overview => format!(
                "Overview 已跳到 Advanced，并对准 {}。推荐原因：{}",
                summary.current_action.label(),
                summary.reason
            ),
            AdvancedRouteOrigin::Apply => format!(
                "Apply 已跳到 Advanced，并对准 {}。推荐原因：{}",
                summary.current_action.label(),
                summary.reason
            ),
            AdvancedRouteOrigin::Actions(action) => {
                if summary.current_action == summary.recommended_action {
                    format!(
                        "{} 归属 Advanced；已跳到 Advanced，并对准 {}。推荐原因：{}",
                        action.label(),
                        summary.current_action.label(),
                        summary.reason
                    )
                } else {
                    format!(
                        "{} 归属 Advanced；已跳到 Advanced，并定位到 {}；默认推荐是 {}。推荐原因：{}",
                        action.label(),
                        summary.current_action.label(),
                        summary.recommended_action.label(),
                        summary.reason
                    )
                }
            }
        };

        let next_step =
            self.advanced_route_next_step(summary.current_action, summary.recommended_action);

        RouteFeedback { message, next_step }
    }

    pub(crate) fn actions_advanced_route_feedback(&self, action: ActionItem) -> RouteFeedback {
        self.advanced_route_feedback_for(AdvancedRouteOrigin::Actions(action))
    }

    pub(crate) fn overview_advanced_route_feedback(&self) -> RouteFeedback {
        self.advanced_route_feedback_for(AdvancedRouteOrigin::Overview)
    }

    pub(crate) fn apply_advanced_route_feedback(&self) -> RouteFeedback {
        self.advanced_route_feedback_for(AdvancedRouteOrigin::Apply)
    }

    pub(crate) fn advanced_summary_model(&self) -> AdvancedSummaryModel {
        let current_action = self.current_advanced_action();
        let recommended_action = self.recommended_advanced_action();
        let reason = advanced_summary_reason(
            self.advanced_recommendation_snapshot().source,
            self.advanced_path_active(),
            current_action,
            recommended_action,
        );
        let feedback =
            self.current_advanced_feedback_state(advanced_completion_hint(recommended_action));

        AdvancedSummaryModel {
            current_action,
            recommended_action,
            reason,
            completion_hint: feedback.completion_hint,
        }
    }

    pub(crate) fn advanced_maintenance_model(&self) -> AdvancedMaintenanceModel {
        let current_action = self.current_advanced_action();
        let summary = self.advanced_summary_model();
        let feedback = self.current_advanced_feedback_state(summary.completion_hint.clone());

        AdvancedMaintenanceModel {
            return_hint: self.advanced_maintenance_return_hint(&summary, feedback.active),
            summary,
            write_target: advanced_action_write_target(current_action),
            impact: advanced_maintenance_impact(current_action),
            command_preview: self.action_command_preview(current_action),
            latest_result: feedback.latest_result,
            repo_root: self.context.repo_root.clone(),
            available: self.action_available(current_action),
        }
    }

    pub(crate) fn advanced_context_model(&self) -> AdvancedContextModel {
        let uses_deploy_parameters = self.advanced_action_uses_deploy_parameters();
        let focused_row = if uses_deploy_parameters {
            self.advanced_wizard_controls_model().focused_row
        } else {
            DeployControlRow {
                label: "动作".to_string(),
                value: self.current_advanced_action().label().to_string(),
            }
        };
        let default_target = if uses_deploy_parameters {
            "默认目标：先选右侧高级动作，再决定是否调整左侧向导参数".to_string()
        } else {
            "默认目标：当前动作是仓库维护；重点看仓库上下文和右下角动作详情".to_string()
        };
        let recommendation = {
            let recommended = self.recommended_advanced_action();
            if self.current_advanced_action() == recommended {
                "建议：当前动作就是默认推荐，可直接按 x/X。".to_string()
            } else {
                format!(
                    "建议：先切到 {}，再决定是否调整左侧向导参数。",
                    recommended.label()
                )
            }
        };
        let execution_hint = if uses_deploy_parameters {
            "高级动作优先".to_string()
        } else {
            "当前动作不使用 deploy 参数".to_string()
        };
        let operation_hint = if uses_deploy_parameters {
            "操作：j/k 选 deploy 参数  h/l 或 Enter 调整".to_string()
        } else {
            "操作：j/k 或 J/K 切高级动作；当前动作不显示 deploy 参数".to_string()
        };
        let advanced_action_hint = if uses_deploy_parameters {
            "高级动作：J/K 选择  x/X 执行当前高级动作  b 返回 Apply".to_string()
        } else {
            "高级动作：j/k 或 J/K 选择  x/X 执行当前高级动作  b 返回 Apply".to_string()
        };

        AdvancedContextModel {
            focused_row,
            default_target,
            recommendation,
            execution_hint,
            operation_hint,
            advanced_action_hint,
        }
    }

    pub(crate) fn advanced_wizard_model(&self) -> AdvancedWizardModel {
        let snapshot = self.advanced_parameter_snapshot();
        let apply = self.build_deploy_model(&snapshot, false, true);
        AdvancedWizardModel {
            target_host: apply.target_host,
            task: apply.task,
            source: apply.source,
            source_detail: apply.source_detail,
            action: apply.action,
            flake_update: apply.flake_update,
            sync_preview: apply.sync_preview,
            rebuild_preview: apply.rebuild_preview,
            blockers: apply.blockers,
            warnings: apply.warnings,
            handoffs: apply.handoffs,
            infos: apply.infos,
            command_preview: command_preview_for_program(
                "mcb-deploy",
                &deploy_wizard_args_for_snapshot(&snapshot),
            ),
            validation_error: deploy_wizard_validation_error_for_snapshot(&snapshot),
        }
    }

    pub(crate) fn advanced_wizard_detail_model(&self) -> AdvancedWizardDetailModel {
        let action = self.current_advanced_action();
        let summary = self.advanced_summary_model();
        let feedback = self.current_advanced_feedback_state(advanced_completion_hint(action));

        AdvancedWizardDetailModel {
            action,
            recommended_action: summary.recommended_action,
            reason: summary.reason,
            status: if self.action_available(action) {
                "当前环境可直接执行".to_string()
            } else {
                "当前环境需切换场景或权限".to_string()
            },
            command_preview: self
                .action_command_preview(action)
                .unwrap_or_else(|| "无".to_string()),
            completion_hint: feedback.completion_hint,
            latest_result: feedback.latest_result,
        }
    }

    pub(crate) fn apply_advanced_workspace_model(&self) -> ApplyAdvancedWorkspaceModel {
        let action = self.current_advanced_action();
        let feedback = self.current_advanced_feedback_state(advanced_completion_hint(action));

        ApplyAdvancedWorkspaceModel {
            top_level_label: if self.page() == Page::Advanced {
                "Advanced".to_string()
            } else {
                "Apply / 高级工作区".to_string()
            },
            action,
            status: if self.action_available(action) {
                "当前环境可直接执行".to_string()
            } else {
                "当前环境需切换场景或权限".to_string()
            },
            command_preview: self
                .action_command_preview(action)
                .unwrap_or_else(|| "无".to_string()),
            latest_result: feedback.latest_result,
            operation_hint: if self.page() == Page::Advanced {
                "操作：J/K 选择高级动作  x/X 执行当前高级动作  b 返回 Apply".to_string()
            } else {
                "操作：J/K 选择高级动作  X 执行当前高级动作".to_string()
            },
        }
    }

    fn current_advanced_feedback_state(
        &self,
        fallback_completion_hint: String,
    ) -> AdvancedFeedbackState {
        let snapshot = self.scoped_feedback_snapshot(
            UiFeedbackScope::Advanced,
            fallback_completion_hint,
            "暂无",
        );
        AdvancedFeedbackState {
            active: snapshot.active,
            recent_feedback: snapshot.message,
            completion_hint: snapshot.next_step,
            latest_result: snapshot.latest_result_text,
        }
    }

    fn advanced_maintenance_return_hint(
        &self,
        summary: &AdvancedSummaryModel,
        advanced_feedback_active: bool,
    ) -> String {
        if advanced_feedback_active {
            summary.completion_hint.clone()
        } else {
            advanced_maintenance_return_hint(summary.recommended_action)
        }
    }

    fn apply_route_model(&self) -> ApplyModel {
        self.build_deploy_model(&self.apply_parameter_snapshot(), false, false)
    }

    fn apply_guidance_for(&self, apply: &ApplyModel) -> ApplyGuidanceModel {
        let recommended_action = self.recommended_advanced_action();

        if apply.can_apply_current_host {
            let copy = apply_guidance_copy(ApplyGuidanceCopyKind::Direct);
            return ApplyGuidanceModel {
                state: ApplyGuidanceState::Direct,
                feedback_detail: "当前组合可直接执行。".to_string(),
                next_step: copy.next_step,
                gate_status: copy.gate_status,
                gate_primary_action: copy.gate_primary_action,
                recommendation: copy.recommendation,
                execution_hint: copy.execution_hint,
                advanced_action_hint: copy.advanced_action_hint,
            };
        }

        if apply.advanced {
            let copy = apply_guidance_copy(ApplyGuidanceCopyKind::WorkspaceOpen);
            return ApplyGuidanceModel {
                state: ApplyGuidanceState::WorkspaceOpen,
                feedback_detail: "当前已打开高级工作区。".to_string(),
                next_step: copy.next_step,
                gate_status: copy.gate_status,
                gate_primary_action: copy.gate_primary_action,
                recommendation: copy.recommendation,
                execution_hint: copy.execution_hint,
                advanced_action_hint: copy.advanced_action_hint,
            };
        }

        if !apply.handoffs.is_empty() {
            let copy = apply_guidance_copy(ApplyGuidanceCopyKind::Handoff(recommended_action));
            return ApplyGuidanceModel {
                state: ApplyGuidanceState::Handoff,
                feedback_detail: apply.handoffs.join(" | "),
                next_step: copy.next_step,
                gate_status: copy.gate_status,
                gate_primary_action: copy.gate_primary_action,
                recommendation: copy.recommendation,
                execution_hint: copy.execution_hint,
                advanced_action_hint: copy.advanced_action_hint,
            };
        }

        if !apply.blockers.is_empty() {
            let copy = apply_guidance_copy(ApplyGuidanceCopyKind::Blocked);
            return ApplyGuidanceModel {
                state: ApplyGuidanceState::Blocked,
                feedback_detail: format!("当前组合仍有 blocker：{}。", apply.blockers.join(" | ")),
                next_step: copy.next_step,
                gate_status: copy.gate_status,
                gate_primary_action: copy.gate_primary_action,
                recommendation: copy.recommendation,
                execution_hint: copy.execution_hint,
                advanced_action_hint: copy.advanced_action_hint,
            };
        }

        let copy = apply_guidance_copy(ApplyGuidanceCopyKind::Review);
        ApplyGuidanceModel {
            state: ApplyGuidanceState::Review,
            feedback_detail: "已跳到 Apply 页查看当前主机预览和执行门槛。".to_string(),
            next_step: copy.next_step,
            gate_status: copy.gate_status,
            gate_primary_action: copy.gate_primary_action,
            recommendation: copy.recommendation,
            execution_hint: copy.execution_hint,
            advanced_action_hint: copy.advanced_action_hint,
        }
    }

    fn current_apply_guidance(&self) -> ApplyGuidanceModel {
        self.apply_guidance_for(&self.apply_model())
    }

    pub(crate) fn current_apply_next_step(&self) -> String {
        self.current_apply_guidance().next_step
    }

    pub(crate) fn current_apply_feedback_detail(&self) -> String {
        self.current_apply_guidance().feedback_detail
    }

    pub(crate) fn current_apply_feedback_summary(&self) -> RouteFeedback {
        let feedback = self.current_apply_feedback_state();
        RouteFeedback {
            message: feedback.recent_feedback,
            next_step: feedback.next_step,
        }
    }

    pub(crate) fn current_advanced_feedback_summary(
        &self,
        fallback_feedback: &str,
        fallback_next_step: &str,
    ) -> RouteFeedback {
        let feedback = self.current_advanced_feedback_state(fallback_next_step.to_string());

        if feedback.active {
            RouteFeedback {
                message: feedback.recent_feedback,
                next_step: feedback.completion_hint,
            }
        } else {
            RouteFeedback {
                message: fallback_feedback.to_string(),
                next_step: feedback.completion_hint,
            }
        }
    }

    fn apply_route_guidance(&self) -> ApplyGuidanceModel {
        self.apply_guidance_for(&self.apply_route_model())
    }

    pub(crate) fn apply_execution_gate_model(&self) -> ApplyExecutionGateModel {
        let apply = self.apply_model();
        let guidance = self.current_apply_guidance();
        let feedback = self.current_apply_feedback_state();

        ApplyExecutionGateModel {
            status: guidance.gate_status,
            latest_result: feedback.latest_result,
            primary_action: guidance.gate_primary_action,
            blockers: apply.blockers,
            warnings: apply.warnings,
            handoffs: apply.handoffs,
            infos: apply.infos,
        }
    }

    fn current_apply_feedback_state(&self) -> ApplyFeedbackState {
        let guidance = self.current_apply_guidance();
        let snapshot =
            self.scoped_feedback_snapshot(UiFeedbackScope::Apply, guidance.next_step, "暂无");

        ApplyFeedbackState {
            recent_feedback: if snapshot.message.is_empty() {
                guidance.feedback_detail
            } else {
                snapshot.message
            },
            next_step: snapshot.next_step,
            latest_result: snapshot.latest_result_text,
        }
    }

    pub(crate) fn current_apply_latest_result(&self) -> String {
        self.current_apply_feedback_state().latest_result
    }

    pub(crate) fn apply_preview_command_fallback(&self) -> String {
        let guidance = self.current_apply_guidance();

        apply_guidance_copy(match guidance.state {
            ApplyGuidanceState::Direct => ApplyGuidanceCopyKind::Direct,
            ApplyGuidanceState::WorkspaceOpen => ApplyGuidanceCopyKind::WorkspaceOpen,
            ApplyGuidanceState::Handoff => {
                ApplyGuidanceCopyKind::Handoff(self.recommended_advanced_action())
            }
            ApplyGuidanceState::Blocked => ApplyGuidanceCopyKind::Blocked,
            ApplyGuidanceState::Review => ApplyGuidanceCopyKind::Review,
        })
        .preview_command_fallback
    }

    fn apply_route_feedback_for(&self, origin: ApplyRouteOrigin) -> RouteFeedback {
        let guidance = self.apply_route_guidance();
        let message = match origin {
            ApplyRouteOrigin::Overview => match guidance.state {
                ApplyGuidanceState::Direct => {
                    format!("Overview 已把你带到 Apply；{}", guidance.feedback_detail)
                }
                ApplyGuidanceState::WorkspaceOpen
                | ApplyGuidanceState::Handoff
                | ApplyGuidanceState::Blocked
                | ApplyGuidanceState::Review => {
                    format!("Overview 已跳到 Apply；{}", guidance.feedback_detail)
                }
            },
            ApplyRouteOrigin::Actions(action) => {
                format!(
                    "{} 归属 Apply；{}",
                    action.label(),
                    guidance.feedback_detail
                )
            }
            ApplyRouteOrigin::AdvancedReturn => {
                format!("已从 Advanced 返回 Apply；{}", guidance.feedback_detail)
            }
        };
        RouteFeedback {
            message,
            next_step: guidance.next_step,
        }
    }

    pub(crate) fn overview_apply_route_feedback(&self) -> RouteFeedback {
        self.apply_route_feedback_for(ApplyRouteOrigin::Overview)
    }

    pub(crate) fn actions_apply_route_feedback(&self, action: ActionItem) -> RouteFeedback {
        self.apply_route_feedback_for(ApplyRouteOrigin::Actions(action))
    }

    pub(crate) fn return_from_advanced_to_apply(&mut self) {
        self.open_apply();
        let feedback = self.apply_route_feedback_for(ApplyRouteOrigin::AdvancedReturn);
        self.set_feedback_with_next_step(
            UiFeedbackLevel::Info,
            UiFeedbackScope::Apply,
            feedback.message,
            feedback.next_step,
        );
    }

    pub(crate) fn apply_selection_model(&self) -> ApplySelectionModel {
        let controls = self.apply_controls_model();
        let guidance = self.current_apply_guidance();

        ApplySelectionModel {
            focused_row: controls.focused_row,
            default_target: "默认目标：先看左侧预览，再决定是否调整右侧 Apply 项".to_string(),
            recommendation: guidance.recommendation,
            execution_hint: guidance.execution_hint,
            operation_hint: "操作：j/k 选 Apply 项  h/l 或 Enter 调整".to_string(),
            advanced_action_hint: guidance.advanced_action_hint,
        }
    }

    pub(crate) fn apply_model(&self) -> ApplyModel {
        self.build_deploy_model(&self.apply_parameter_snapshot(), self.show_advanced, false)
    }

    fn build_deploy_model(
        &self,
        snapshot: &DeployParameterSnapshot,
        apply_advanced_handoff: bool,
        advanced_workspace_active: bool,
    ) -> ApplyModel {
        let can_execute_directly =
            can_execute_deploy_directly_for(snapshot.source, apply_advanced_handoff);
        let sync_preview =
            deploy_sync_plan_for_snapshot(self, snapshot).map(|plan| plan.command_preview());
        let rebuild_preview = if can_execute_directly {
            deploy_rebuild_plan_for_snapshot(self, snapshot)
                .map(|plan| plan.command_preview(self.should_use_sudo()))
        } else {
            None
        };
        let mut blockers = Vec::new();
        if let Err(err) = self.ensure_no_unsaved_changes_for_execution() {
            blockers.push(err.to_string());
        }
        if let Some(error) = deploy_wizard_validation_error_for_snapshot(snapshot) {
            blockers.push(error);
        }
        let host_errors = self.host_configuration_validation_errors(&snapshot.target_host);
        blockers.extend(host_errors.into_iter().map(|error| {
            format!(
                "主机 {} 的 TUI 配置未通过校验：{error}",
                snapshot.target_host
            )
        }));
        if self.context.privilege_mode == "rootless" && snapshot.action != DeployAction::Build {
            blockers.push(
                "rootless 模式下当前页只能直接执行 build；如需 switch/test/boot，请使用 sudo/root 或退回 deploy wizard。"
                    .to_string(),
            );
        }

        let mut warnings = Vec::new();
        if let Some(preview) = &sync_preview {
            warnings.push(format!("当前组合会先把仓库同步到 /etc/nixos：{preview}"));
        }
        if snapshot.flake_update {
            warnings.push("当前组合会以 --upgrade 执行重建。".to_string());
        }
        if self.should_use_sudo() {
            warnings.push("当前组合会使用 sudo -E 执行受权命令。".to_string());
        }
        let needs_real_hardware =
            !(self.context.privilege_mode == "rootless" && snapshot.action == DeployAction::Build);
        if needs_real_hardware {
            warnings.push(format!(
                "当前组合要求 {} 存在真实 hardware-configuration.nix。",
                host_hardware_config_path(&self.context.etc_root, &snapshot.target_host).display()
            ));
        }

        let mut handoffs = Vec::new();
        match snapshot.source {
            DeploySource::RemotePinned | DeploySource::RemoteHead => {
                handoffs.push(advanced_source_handoff_reason(snapshot.source).to_string())
            }
            DeploySource::CurrentRepo | DeploySource::EtcNixos => {}
        }
        if apply_advanced_handoff {
            handoffs.push("当前已打开 Apply 内高级工作区，应交给 Advanced 区处理。".to_string());
        }

        let mut infos = Vec::new();
        if !can_execute_directly {
            infos.push("当前组合不会直接执行，而是回退到完整 deploy wizard。".to_string());
        }
        infos.push(format!("检测 hostname：{}", self.context.current_host));

        ApplyModel {
            target_host: snapshot.target_host.clone(),
            task: snapshot.task,
            source: snapshot.source,
            source_detail: deploy_source_detail_for_snapshot(snapshot),
            action: snapshot.action,
            flake_update: snapshot.flake_update,
            advanced: advanced_workspace_active || apply_advanced_handoff,
            sync_preview,
            rebuild_preview,
            can_execute_directly,
            can_apply_current_host: can_execute_directly && blockers.is_empty(),
            blockers,
            warnings,
            handoffs,
            infos,
        }
    }

    fn build_deploy_controls_model(
        &self,
        snapshot: &DeployParameterSnapshot,
        selected_focus: usize,
        area_row: DeployControlRow,
    ) -> DeployControlsModel {
        let rows = deploy_rows_for_snapshot(snapshot, area_row);
        let selected_focus = selected_focus.min(rows.len().saturating_sub(1));
        let focused_row = rows
            .get(selected_focus)
            .cloned()
            .unwrap_or_else(|| DeployControlRow {
                label: "<无>".to_string(),
                value: String::new(),
            });

        DeployControlsModel {
            rows,
            selected_focus,
            focused_row,
        }
    }

    pub(crate) fn set_deploy_wizard_return_feedback(&mut self) {
        self.set_feedback_with_next_step(
            UiFeedbackLevel::Info,
            UiFeedbackScope::Advanced,
            "已返回完整部署向导。",
            "继续在 Advanced 完成复杂部署",
        );
    }

    pub(crate) fn set_sync_repo_completion_feedback(&mut self) {
        let feedback = sync_repo_completion_feedback();
        self.set_feedback_with_next_step(
            UiFeedbackLevel::Success,
            UiFeedbackScope::Apply,
            feedback.message,
            feedback.next_step,
        );
    }

    pub(crate) fn set_advanced_maintenance_completion_feedback(&mut self, action: ActionItem) {
        let feedback = advanced_maintenance_completion_feedback(action);
        self.set_feedback_with_next_step(
            UiFeedbackLevel::Success,
            UiFeedbackScope::Advanced,
            feedback.message,
            feedback.next_step,
        );
    }

    pub(crate) fn set_apply_rebuild_completion_feedback(&mut self, plan: &NixosRebuildPlan) {
        self.set_rebuild_completion_feedback(RebuildCompletionOrigin::Apply, plan);
    }

    pub(crate) fn set_current_host_rebuild_completion_feedback(&mut self, plan: &NixosRebuildPlan) {
        self.set_rebuild_completion_feedback(RebuildCompletionOrigin::CurrentHostAction, plan);
    }

    fn set_rebuild_completion_feedback(
        &mut self,
        origin: RebuildCompletionOrigin,
        plan: &NixosRebuildPlan,
    ) {
        let feedback = rebuild_completion_feedback(origin, plan);
        self.set_feedback_with_next_step(
            UiFeedbackLevel::Success,
            UiFeedbackScope::Apply,
            feedback.message,
            feedback.next_step,
        );
    }

    pub fn execute_deploy(&mut self) -> Result<()> {
        self.ensure_no_unsaved_changes_for_execution()?;
        ensure_repository_integrity(&self.context.repo_root)?;
        self.ensure_host_configuration_is_valid(&self.target_host)?;

        if !self.can_execute_deploy_directly() {
            if let Some(error) = self.apply_deploy_wizard_validation_error() {
                anyhow::bail!("{error}");
            }
            let status =
                self.run_sibling_in_repo("mcb-deploy", &self.apply_deploy_wizard_args())?;
            if status.success() {
                self.set_deploy_wizard_return_feedback();
                return Ok(());
            }
            anyhow::bail!("mcb-deploy exited with {}", status.code().unwrap_or(1));
        }

        if self.context.privilege_mode == "rootless" && self.deploy_action != DeployAction::Build {
            anyhow::bail!(
                "rootless 模式下当前页只能直接执行 build；如需 switch/test/boot，请使用 sudo/root 或退回 deploy wizard。"
            );
        }

        let use_sudo = self.should_use_sudo();
        let needs_root_hw = !(self.context.privilege_mode == "rootless"
            && self.deploy_action == DeployAction::Build);
        if needs_root_hw {
            ensure_host_hardware_config(&self.context.etc_root, &self.target_host, use_sudo)?;
        }

        let sync_plan = self.deploy_sync_plan_for_execution();
        let rebuild_plan = self
            .deploy_rebuild_plan_for_execution()
            .context("当前 Apply 组合还没有可执行的重建计划")?;

        if let Some(plan) = sync_plan {
            run_repo_sync(
                &plan,
                |cmd, args| {
                    let status = std::process::Command::new(cmd)
                        .args(args)
                        .stdin(std::process::Stdio::inherit())
                        .stdout(std::process::Stdio::inherit())
                        .stderr(std::process::Stdio::inherit())
                        .status()
                        .with_context(|| format!("failed to run {cmd}"))?;
                    if status.success() {
                        Ok(())
                    } else {
                        anyhow::bail!("{cmd} failed with {}", status.code().unwrap_or(1));
                    }
                },
                |cmd, args| run_root_command_ok(cmd, args, use_sudo),
                || self.clean_etc_dir_keep_hardware(),
            )?;
        }

        let status = run_nixos_rebuild(&rebuild_plan, use_sudo)?;
        if !status.success() {
            anyhow::bail!("nixos-rebuild exited with {}", status.code().unwrap_or(1));
        }

        self.set_apply_rebuild_completion_feedback(&rebuild_plan);
        Ok(())
    }

    pub(crate) fn deploy_rebuild_plan_for_execution(&self) -> Option<NixosRebuildPlan> {
        deploy_rebuild_plan_for_snapshot(self, &self.apply_parameter_snapshot())
    }

    pub(crate) fn deploy_sync_plan_for_execution(&self) -> Option<RepoSyncPlan> {
        deploy_sync_plan_for_snapshot(self, &self.apply_parameter_snapshot())
    }

    fn open_deploy_text_edit_for_scope(&mut self, scope: DeployControlScope) {
        let snapshot = self.parameter_snapshot_for_scope(scope);
        if snapshot.source != DeploySource::RemotePinned {
            self.status = format!(
                "{} 当前来源不是远端固定版本；无需填写固定 ref。",
                scope.label()
            );
            return;
        }

        self.deploy_text_mode = Some(scope.text_mode());
        self.host_text_input = snapshot.source_ref;
        self.status = format!("开始编辑 {} 远端固定版本 ref/pin。", scope.label());
    }

    pub fn open_apply_text_edit(&mut self) {
        self.open_deploy_text_edit_for_scope(DeployControlScope::Apply);
    }

    pub fn open_advanced_wizard_text_edit(&mut self) {
        self.open_deploy_text_edit_for_scope(DeployControlScope::AdvancedWizard);
    }

    fn handle_deploy_text_input_for_mode(
        &mut self,
        mode: DeployTextMode,
        code: crossterm::event::KeyCode,
    ) {
        match code {
            crossterm::event::KeyCode::Enter => self.confirm_deploy_text_edit(mode),
            crossterm::event::KeyCode::Esc => {
                self.deploy_text_mode = None;
                self.host_text_input.clear();
                let scope = DeployControlScope::from_text_mode(mode);
                self.status = format!("已取消 {} 参数编辑。", scope.label());
            }
            crossterm::event::KeyCode::Backspace => {
                self.host_text_input.pop();
            }
            crossterm::event::KeyCode::Char(ch) => {
                self.host_text_input.push(ch);
            }
            _ => {}
        }
    }

    pub fn handle_apply_text_input(&mut self, code: crossterm::event::KeyCode) {
        if self.deploy_text_mode == Some(DeployTextMode::ApplyRemotePinnedRef) {
            self.handle_deploy_text_input_for_mode(DeployTextMode::ApplyRemotePinnedRef, code);
        }
    }

    pub fn handle_advanced_wizard_text_input(&mut self, code: crossterm::event::KeyCode) {
        if self.deploy_text_mode == Some(DeployTextMode::AdvancedWizardRemotePinnedRef) {
            self.handle_deploy_text_input_for_mode(
                DeployTextMode::AdvancedWizardRemotePinnedRef,
                code,
            );
        }
    }

    fn confirm_deploy_text_edit(&mut self, mode: DeployTextMode) {
        let scope = DeployControlScope::from_text_mode(mode);
        let raw = self.host_text_input.trim().to_string();
        match scope {
            DeployControlScope::Apply => self.deploy_source_ref = raw.clone(),
            DeployControlScope::AdvancedWizard => self.advanced_deploy_source_ref = raw.clone(),
        }
        self.deploy_text_mode = None;
        self.host_text_input.clear();
        if raw.is_empty() {
            self.status = format!(
                "已清空 {} 远端固定版本 ref；当前组合仍需补全 ref/pin 才能启动 deploy wizard。",
                scope.label()
            );
        } else {
            self.status = format!("{} 远端固定版本 ref 已更新为：{raw}", scope.label());
        }
    }
}

fn can_execute_deploy_directly_for(source: DeploySource, apply_advanced_handoff: bool) -> bool {
    !matches!(
        source,
        DeploySource::RemotePinned | DeploySource::RemoteHead
    ) && !apply_advanced_handoff
}

fn deploy_area_row_for(advanced_entry: bool, show_advanced: bool) -> DeployControlRow {
    if advanced_entry {
        DeployControlRow {
            label: "区域切换".to_string(),
            value: "Enter 返回 Apply".to_string(),
        }
    } else if show_advanced {
        DeployControlRow {
            label: "高级工作区".to_string(),
            value: "Apply 内兼容模式".to_string(),
        }
    } else {
        DeployControlRow {
            label: "区域切换".to_string(),
            value: "Enter 进入 Advanced".to_string(),
        }
    }
}

fn advanced_handoff_focus_for_snapshot(
    snapshot: &DeployParameterSnapshot,
    fallback_focus: usize,
) -> usize {
    match snapshot.source {
        DeploySource::RemoteHead => 2,
        DeploySource::RemotePinned => 3,
        DeploySource::CurrentRepo | DeploySource::EtcNixos => fallback_focus,
    }
}

fn deploy_rows_for_snapshot(
    snapshot: &DeployParameterSnapshot,
    area_row: DeployControlRow,
) -> Vec<DeployControlRow> {
    vec![
        DeployControlRow {
            label: "目标主机".to_string(),
            value: snapshot.target_host.clone(),
        },
        DeployControlRow {
            label: "任务".to_string(),
            value: snapshot.task.label().to_string(),
        },
        DeployControlRow {
            label: "来源".to_string(),
            value: snapshot.source.label().to_string(),
        },
        DeployControlRow {
            label: "固定 ref".to_string(),
            value: deploy_source_ref_label(snapshot),
        },
        DeployControlRow {
            label: "动作".to_string(),
            value: snapshot.action.label().to_string(),
        },
        DeployControlRow {
            label: "flake update".to_string(),
            value: bool_label(snapshot.flake_update).to_string(),
        },
        area_row,
    ]
}

fn deploy_wizard_args_for_snapshot(snapshot: &DeployParameterSnapshot) -> Vec<String> {
    let mut args = vec![
        "--mode".to_string(),
        deploy_mode_arg_for_task(snapshot.task).to_string(),
        "--action".to_string(),
        snapshot.action.rebuild_mode().to_string(),
    ];

    if !snapshot.target_host.trim().is_empty() {
        args.push("--host".to_string());
        args.push(snapshot.target_host.clone());
    }

    match snapshot.source {
        DeploySource::CurrentRepo => {
            args.push("--source".to_string());
            args.push("current-repo".to_string());
        }
        DeploySource::EtcNixos => {
            args.push("--source".to_string());
            args.push("etc-nixos".to_string());
        }
        DeploySource::RemoteHead => {
            args.push("--source".to_string());
            args.push("remote-head".to_string());
        }
        DeploySource::RemotePinned => {
            args.push("--source".to_string());
            args.push("remote-pinned".to_string());
            if !snapshot.source_ref.trim().is_empty() {
                args.push("--ref".to_string());
                args.push(snapshot.source_ref.trim().to_string());
            }
        }
    }

    if snapshot.flake_update {
        args.push("--upgrade".to_string());
    }

    args
}

fn deploy_source_ref_label(snapshot: &DeployParameterSnapshot) -> String {
    if snapshot.source != DeploySource::RemotePinned {
        return "仅远端固定版本".to_string();
    }

    let source_ref = snapshot.source_ref.trim();
    if source_ref.is_empty() {
        "未设置".to_string()
    } else {
        source_ref.to_string()
    }
}

fn deploy_source_detail_for_snapshot(snapshot: &DeployParameterSnapshot) -> Option<String> {
    match snapshot.source {
        DeploySource::RemotePinned => {
            let source_ref = snapshot.source_ref.trim();
            (!source_ref.is_empty()).then(|| source_ref.to_string())
        }
        DeploySource::CurrentRepo | DeploySource::EtcNixos | DeploySource::RemoteHead => None,
    }
}

fn deploy_wizard_validation_error_for_snapshot(
    snapshot: &DeployParameterSnapshot,
) -> Option<String> {
    if snapshot.source == DeploySource::RemotePinned && snapshot.source_ref.trim().is_empty() {
        Some("远端固定版本缺少 ref/pin；先在 Deploy Parameters 里填写固定版本。".to_string())
    } else {
        None
    }
}

fn deploy_mode_arg_for_task(task: DeployTask) -> &'static str {
    match task {
        DeployTask::AdjustStructure => "manage-users",
        DeployTask::DirectDeploy | DeployTask::Maintenance => "update-existing",
    }
}

fn command_preview_for_program(program: &str, args: &[String]) -> String {
    if args.is_empty() {
        program.to_string()
    } else {
        format!("{program} {}", args.join(" "))
    }
}

fn sync_repo_completion_feedback() -> CompletionFeedback {
    CompletionFeedback {
        message: "仓库已同步到 /etc/nixos。".to_string(),
        next_step: "回到 Apply 或 Overview 继续后续重建".to_string(),
    }
}

fn rebuild_completion_feedback(
    origin: RebuildCompletionOrigin,
    plan: &NixosRebuildPlan,
) -> CompletionFeedback {
    let message = match origin {
        RebuildCompletionOrigin::Apply => format!(
            "Apply 已执行完成：{} {}",
            plan.action.label(),
            plan.target_host
        ),
        RebuildCompletionOrigin::CurrentHostAction => format!(
            "当前主机 {} 已完成一次 {}。",
            plan.target_host,
            plan.action.label()
        ),
    };

    CompletionFeedback {
        message,
        next_step: "回到 Overview 检查健康和下一步".to_string(),
    }
}

fn advanced_maintenance_completion_feedback(action: ActionItem) -> CompletionFeedback {
    let message = match action {
        ActionItem::FlakeUpdate => "flake update 已完成。".to_string(),
        ActionItem::UpdateUpstreamPins => "上游 pin 刷新已完成。".to_string(),
        ActionItem::LaunchDeployWizard => "已返回完整部署向导。".to_string(),
        ActionItem::FlakeCheck
        | ActionItem::UpdateUpstreamCheck
        | ActionItem::SyncRepoToEtc
        | ActionItem::RebuildCurrentHost => format!("{} 已完成。", action.label()),
    };

    CompletionFeedback {
        message,
        next_step: advanced_completion_hint(action),
    }
}

fn should_sync_current_repo_before_rebuild_for(
    state: &AppState,
    snapshot: &DeployParameterSnapshot,
) -> bool {
    snapshot.source == DeploySource::CurrentRepo
        && state.context.repo_root != state.context.etc_root
        && snapshot.action != DeployAction::Build
        && state.context.privilege_mode != "rootless"
}

fn deploy_rebuild_plan_for_snapshot(
    state: &AppState,
    snapshot: &DeployParameterSnapshot,
) -> Option<NixosRebuildPlan> {
    let flake_root = match snapshot.source {
        DeploySource::CurrentRepo
            if should_sync_current_repo_before_rebuild_for(state, snapshot) =>
        {
            state.context.etc_root.clone()
        }
        DeploySource::CurrentRepo => state.context.repo_root.clone(),
        DeploySource::EtcNixos => state.context.etc_root.clone(),
        DeploySource::RemotePinned | DeploySource::RemoteHead => return None,
    };

    Some(NixosRebuildPlan {
        action: snapshot.action,
        upgrade: snapshot.flake_update,
        flake_root,
        target_host: snapshot.target_host.clone(),
    })
}

fn deploy_sync_plan_for_snapshot(
    state: &AppState,
    snapshot: &DeployParameterSnapshot,
) -> Option<RepoSyncPlan> {
    match snapshot.source {
        DeploySource::CurrentRepo
            if should_sync_current_repo_before_rebuild_for(state, snapshot) =>
        {
            Some(RepoSyncPlan {
                source_dir: state.context.repo_root.clone(),
                destination_dir: state.context.etc_root.clone(),
                delete_extra: true,
            })
        }
        _ => None,
    }
}

fn ordered_advanced_actions(recommended: ActionItem) -> [ActionItem; 3] {
    if matches!(recommended, ActionItem::LaunchDeployWizard) {
        [
            ActionItem::LaunchDeployWizard,
            ActionItem::FlakeUpdate,
            ActionItem::UpdateUpstreamPins,
        ]
    } else {
        [
            ActionItem::FlakeUpdate,
            ActionItem::UpdateUpstreamPins,
            ActionItem::LaunchDeployWizard,
        ]
    }
}

impl AppState {
    fn set_advanced_action_focus(&mut self, action: ActionItem) {
        self.actions_focus = ActionItem::ALL
            .iter()
            .position(|candidate| *candidate == action)
            .expect("advanced action must exist in ActionItem::ALL");
    }

    fn current_advanced_action_or_default(&self) -> ActionItem {
        let action = self.current_action_item();
        if action.destination() == ActionDestination::Advanced {
            action
        } else {
            ActionItem::FlakeUpdate
        }
    }
}

fn advanced_summary_reason(
    source: DeploySource,
    advanced_enabled: bool,
    current_action: ActionItem,
    recommended_action: ActionItem,
) -> String {
    if let Some(reason) = advanced_source_handoff_reason_opt(source) {
        return reason.to_string();
    }
    if advanced_enabled && recommended_action == ActionItem::LaunchDeployWizard {
        return "当前已显式打开高级路径；复杂部署、初始化和专家交互应在这里完成。".to_string();
    }

    match current_action {
        ActionItem::FlakeUpdate => {
            "当前没有必须 handoff 的部署阻塞；适合先更新 flake.lock 再复查仓库状态。".to_string()
        }
        ActionItem::UpdateUpstreamPins => {
            "当前没有必须 handoff 的部署阻塞；适合先刷新上游 pin 再回看仓库健康。".to_string()
        }
        ActionItem::LaunchDeployWizard => {
            "当前没有 direct apply 的唯一主路径；如需远端来源、初始化或复杂交互，请走完整向导。"
                .to_string()
        }
        ActionItem::FlakeCheck
        | ActionItem::UpdateUpstreamCheck
        | ActionItem::SyncRepoToEtc
        | ActionItem::RebuildCurrentHost => "当前动作不属于 Advanced。".to_string(),
    }
}

fn advanced_source_handoff_reason_opt(source: DeploySource) -> Option<&'static str> {
    match source {
        DeploySource::RemotePinned => {
            Some(advanced_source_handoff_reason(DeploySource::RemotePinned))
        }
        DeploySource::RemoteHead => Some(advanced_source_handoff_reason(DeploySource::RemoteHead)),
        DeploySource::CurrentRepo | DeploySource::EtcNixos => None,
    }
}

fn advanced_source_handoff_reason(source: DeploySource) -> &'static str {
    match source {
        DeploySource::RemotePinned => {
            "当前来源是远端固定版本；默认 Apply 不会直接执行，必须交给完整高级路径。"
        }
        DeploySource::RemoteHead => {
            "当前来源是远端最新版本；默认 Apply 不会直接执行，必须交给完整高级路径。"
        }
        DeploySource::CurrentRepo | DeploySource::EtcNixos => "当前来源不需要交给完整高级路径。",
    }
}

fn advanced_completion_hint(action: ActionItem) -> String {
    match action {
        ActionItem::FlakeUpdate | ActionItem::UpdateUpstreamPins => {
            "做完后回 Inspect 或 Overview 复查仓库状态，必要时再回 Apply。".to_string()
        }
        ActionItem::LaunchDeployWizard => {
            "做完后回 Apply 或 Overview 检查默认路径、健康和下一步。".to_string()
        }
        ActionItem::FlakeCheck
        | ActionItem::UpdateUpstreamCheck
        | ActionItem::SyncRepoToEtc
        | ActionItem::RebuildCurrentHost => "完成后回到对应归宿页继续主线。".to_string(),
    }
}

fn apply_guidance_copy(kind: ApplyGuidanceCopyKind) -> ApplyGuidanceCopy {
    match kind {
        ApplyGuidanceCopyKind::Direct => ApplyGuidanceCopy {
            gate_status: "当前可直接 Apply".to_string(),
            next_step: "在 Apply 查看预览，或按 a / x 直接运行".to_string(),
            gate_primary_action: "主动作：按 x 立即执行当前 Apply".to_string(),
            recommendation: "建议：左侧预览已可直接执行；确认无误后可按 x Apply。".to_string(),
            execution_hint: "可执行：当前组合可直接 Apply".to_string(),
            advanced_action_hint: "高级动作：打开高级模式后可在右下角执行 Advanced 动作"
                .to_string(),
            preview_command_fallback: "当前组合可直接执行 Apply".to_string(),
        },
        ApplyGuidanceCopyKind::WorkspaceOpen => ApplyGuidanceCopy {
            gate_status: "当前已打开高级工作区".to_string(),
            next_step: "在 Apply 先看右下角高级工作区；x 仍按当前 Apply 路径处理".to_string(),
            gate_primary_action: "主动作：在右侧高级工作区选择动作并按 X 执行".to_string(),
            recommendation:
                "建议：当前已打开高级工作区；先确认右下角动作，再决定是否回默认 Apply。".to_string(),
            execution_hint: "当前已打开高级工作区".to_string(),
            advanced_action_hint: "高级动作：J/K 选择  X 执行  x 仍按当前 Apply 路径处理"
                .to_string(),
            preview_command_fallback: "当前组合会在右下角高级工作区继续执行".to_string(),
        },
        ApplyGuidanceCopyKind::Handoff(action) => ApplyGuidanceCopy {
            gate_status: "当前组合应转交给 Advanced".to_string(),
            next_step: format!(
                "在 Apply 先看 handoff 预览；如需继续，切到 Advanced 执行 {}",
                action.label()
            ),
            gate_primary_action: format!("主动作：切到 Advanced 执行 {}", action.label()),
            recommendation: format!(
                "建议：当前默认应切到 Advanced 执行 {}；先看 handoff 预览。",
                action.label()
            ),
            execution_hint: format!("需交接：默认应切到 Advanced 执行 {}", action.label()),
            advanced_action_hint: format!(
                "高级动作：当前默认应切到 Advanced 执行 {}",
                action.label()
            ),
            preview_command_fallback: format!("当前组合会转交给 Advanced 执行 {}", action.label()),
        },
        ApplyGuidanceCopyKind::Blocked => ApplyGuidanceCopy {
            gate_status: "当前不能直接 Apply".to_string(),
            next_step: "在 Apply 先看 blocker / warning，再决定是否调整 Apply 项".to_string(),
            gate_primary_action: "主动作：先修复阻塞项，再回到 Apply".to_string(),
            recommendation: "建议：先看 blocker / warning，再决定是否直接 Apply。".to_string(),
            execution_hint: "不可执行：当前仍有 blocker".to_string(),
            advanced_action_hint: "高级动作：修复 blocker 后，仍可切到 Advanced".to_string(),
            preview_command_fallback: "当前组合暂不生成直接命令预览；请先处理 blocker / warning"
                .to_string(),
        },
        ApplyGuidanceCopyKind::Review => ApplyGuidanceCopy {
            gate_status: "当前待确认 Apply 门槛".to_string(),
            next_step: "在 Apply 查看预览并决定下一步".to_string(),
            gate_primary_action: "主动作：先确认预览和执行门槛".to_string(),
            recommendation: "建议：先看左侧预览和执行门槛，再决定是否直接 Apply。".to_string(),
            execution_hint: "待确认：先看预览和执行门槛".to_string(),
            advanced_action_hint: "高级动作：打开高级模式后可在右下角执行 Advanced 动作"
                .to_string(),
            preview_command_fallback: "当前组合暂不生成直接命令预览；请先确认预览和执行门槛"
                .to_string(),
        },
    }
}

fn advanced_action_write_target(action: ActionItem) -> &'static str {
    match action {
        ActionItem::FlakeUpdate => "flake.lock",
        ActionItem::UpdateUpstreamPins => "source.nix / upstream pins",
        ActionItem::LaunchDeployWizard => "完整部署向导参数",
        ActionItem::FlakeCheck
        | ActionItem::UpdateUpstreamCheck
        | ActionItem::SyncRepoToEtc
        | ActionItem::RebuildCurrentHost => "当前动作不属于 Advanced",
    }
}

fn advanced_maintenance_impact(action: ActionItem) -> &'static str {
    match action {
        ActionItem::FlakeUpdate => "会刷新 flake.lock；执行后建议先复查仓库健康。",
        ActionItem::UpdateUpstreamPins => "会刷新上游 pin；执行后建议回 Inspect 或 Overview 复查。",
        ActionItem::LaunchDeployWizard => "会切回完整部署向导，继续处理复杂交互。",
        ActionItem::FlakeCheck
        | ActionItem::UpdateUpstreamCheck
        | ActionItem::SyncRepoToEtc
        | ActionItem::RebuildCurrentHost => "当前动作不属于 Advanced。",
    }
}

fn advanced_maintenance_return_hint(recommended_action: ActionItem) -> String {
    if recommended_action == ActionItem::LaunchDeployWizard {
        format!(
            "完成后回 Inspect / Overview 复查；如果要继续复杂部署，切到 {}。",
            recommended_action.label()
        )
    } else {
        "完成后回 Inspect 或 Overview 复查；如果只想回默认应用路径，按 b 返回 Apply。".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn current_repo_switch_uses_sync_plan_and_etc_rebuild_target() {
        let state = test_state("/repo", "/etc/nixos", "sudo-available");

        assert!(state.can_execute_deploy_directly());
        let sync = state
            .deploy_sync_plan_for_execution()
            .expect("current repo switch should sync into /etc/nixos");
        assert_eq!(sync.source_dir, PathBuf::from("/repo"));
        assert_eq!(sync.destination_dir, PathBuf::from("/etc/nixos"));
        assert!(sync.delete_extra);

        let rebuild = state
            .deploy_rebuild_plan_for_execution()
            .expect("current repo switch should produce rebuild plan");
        assert_eq!(rebuild.action, DeployAction::Switch);
        assert_eq!(rebuild.flake_root, PathBuf::from("/etc/nixos"));
        assert_eq!(rebuild.target_host, "demo");
    }

    #[test]
    fn rootless_build_stays_on_repo_and_skips_sync() {
        let mut state = test_state("/repo", "/etc/nixos", "rootless");
        state.deploy_action = DeployAction::Build;

        assert!(state.can_execute_deploy_directly());
        assert!(state.deploy_sync_plan_for_execution().is_none());

        let rebuild = state
            .deploy_rebuild_plan_for_execution()
            .expect("rootless build should still produce rebuild plan");
        assert_eq!(rebuild.action, DeployAction::Build);
        assert_eq!(rebuild.flake_root, PathBuf::from("/repo"));
    }

    #[test]
    fn remote_sources_fall_back_to_wizard() {
        let mut state = test_state("/repo", "/etc/nixos", "sudo-available");
        state.deploy_source = DeploySource::RemoteHead;

        assert!(!state.can_execute_deploy_directly());
        let model = state.apply_model();
        assert!(!model.can_execute_directly);
        assert!(
            model
                .handoffs
                .iter()
                .any(|item| item.contains("远端最新版本"))
        );
        assert!(state.deploy_sync_plan_for_execution().is_none());
        assert!(state.deploy_rebuild_plan_for_execution().is_none());
    }

    #[test]
    fn advanced_focus_falls_back_to_first_advanced_action() {
        let mut state = test_state("/repo", "/etc/nixos", "sudo-available");
        state.actions_focus = 0;

        state.ensure_advanced_action_focus();

        assert_eq!(state.current_advanced_action(), ActionItem::FlakeUpdate);
        assert_eq!(state.selected_advanced_row_index(), 1);
    }

    #[test]
    fn advanced_focus_cycles_only_between_advanced_actions() {
        let mut state = test_state("/repo", "/etc/nixos", "sudo-available");
        state.ensure_advanced_action_focus();
        assert_eq!(state.current_advanced_action(), ActionItem::FlakeUpdate);

        state.next_advanced_action();
        assert_eq!(
            state.current_advanced_action(),
            ActionItem::UpdateUpstreamPins
        );

        state.next_advanced_action();
        assert_eq!(
            state.current_advanced_action(),
            ActionItem::LaunchDeployWizard
        );

        state.next_advanced_action();
        assert_eq!(state.current_advanced_action(), ActionItem::FlakeUpdate);

        state.previous_advanced_action();
        assert_eq!(
            state.current_advanced_action(),
            ActionItem::LaunchDeployWizard
        );
    }

    #[test]
    fn advanced_action_uses_deploy_parameters_only_for_wizard() {
        let mut state = test_state("/repo", "/etc/nixos", "sudo-available");
        state.ensure_advanced_action_focus();
        assert!(!state.advanced_action_uses_deploy_parameters());

        state.next_advanced_action();
        assert!(!state.advanced_action_uses_deploy_parameters());

        state.next_advanced_action();
        assert!(state.advanced_action_uses_deploy_parameters());
    }

    #[test]
    fn recommended_advanced_action_prefers_wizard_when_apply_requires_handoff() {
        let mut state = test_state("/repo", "/etc/nixos", "sudo-available");
        state.deploy_source = DeploySource::RemoteHead;

        assert_eq!(
            state.recommended_advanced_action(),
            ActionItem::LaunchDeployWizard
        );

        state.focus_recommended_advanced_action();

        assert_eq!(
            state.current_advanced_action(),
            ActionItem::LaunchDeployWizard
        );
        assert_eq!(state.selected_advanced_row_index(), 1);
    }

    #[test]
    fn advanced_action_display_rows_group_and_reorder_items_by_recommended_group() {
        let state = test_state("/repo", "/etc/nixos", "sudo-available");
        let rows = state.advanced_action_display_rows();

        assert_eq!(rows[0].label, "Repository Maintenance");
        assert!(!rows[0].selectable);
        assert_eq!(rows[1].label, "flake update");
        assert_eq!(rows[1].value, "推荐 / 可执行");
        assert!(rows[1].selectable);
        assert_eq!(rows[3].label, "Deploy");
        assert_eq!(rows[4].label, "launch deploy wizard");
    }

    #[test]
    fn advanced_action_display_rows_put_deploy_group_first_when_wizard_is_recommended() {
        let mut state = test_state("/repo", "/etc/nixos", "sudo-available");
        state.deploy_source = DeploySource::RemoteHead;

        let rows = state.advanced_action_display_rows();

        assert_eq!(rows[0].label, "Deploy");
        assert_eq!(rows[1].label, "launch deploy wizard");
        assert_eq!(rows[1].value, "推荐 / 可执行");
        assert_eq!(rows[2].label, "Repository Maintenance");
    }

    #[test]
    fn advanced_actions_list_model_keeps_rows_and_selection_in_sync() {
        let mut state = test_state("/repo", "/etc/nixos", "sudo-available");
        state.deploy_source = DeploySource::RemoteHead;
        state.focus_recommended_advanced_action();

        let model = state.advanced_actions_list_model();

        assert_eq!(model.selected_index, 1);
        assert_eq!(model.rows[0].label, "Deploy");
        assert_eq!(model.rows[1].label, "launch deploy wizard");
        assert_eq!(model.rows[1].value, "推荐 / 可执行");
    }

    #[test]
    fn advanced_summary_model_prefers_feedback_next_step_for_advanced_scope() {
        let mut state = test_state("/repo", "/etc/nixos", "sudo-available");
        state.set_feedback_with_next_step(
            UiFeedbackLevel::Success,
            UiFeedbackScope::Advanced,
            "flake update 已完成。",
            "继续在 Advanced 处理后续仓库维护",
        );

        let summary = state.advanced_summary_model();

        assert_eq!(summary.current_action, ActionItem::FlakeUpdate);
        assert_eq!(summary.recommended_action, ActionItem::FlakeUpdate);
        assert_eq!(summary.completion_hint, "继续在 Advanced 处理后续仓库维护");
        assert!(summary.reason.contains("更新 flake.lock"));
    }

    #[test]
    fn advanced_summary_model_uses_top_level_advanced_without_apply_flag() {
        let mut state = test_state("/repo", "/etc/nixos", "sudo-available");
        state.open_advanced();
        state.ensure_advanced_action_focus();

        let summary = state.advanced_summary_model();

        assert_eq!(summary.current_action, ActionItem::FlakeUpdate);
        assert_eq!(summary.recommended_action, ActionItem::FlakeUpdate);
        assert!(!state.show_advanced);
        assert!(summary.reason.contains("更新 flake.lock"));
    }

    #[test]
    fn deploy_shell_model_switches_titles_and_modes_by_current_path() {
        let mut state = test_state("/repo", "/etc/nixos", "sudo-available");

        let apply_shell = state.deploy_shell_model();
        assert_eq!(apply_shell.mode, DeployShellMode::Apply);
        assert_eq!(apply_shell.summary_title, "Execution Gate");
        assert_eq!(apply_shell.controls_title, "Apply Controls");
        assert!(!apply_shell.workspace_visible);

        state.show_advanced = true;
        let apply_workspace_shell = state.deploy_shell_model();
        assert_eq!(apply_workspace_shell.mode, DeployShellMode::Apply);
        assert!(apply_workspace_shell.workspace_visible);
        assert_eq!(apply_workspace_shell.detail_title, "Advanced Detail");

        state.open_advanced();
        state.ensure_advanced_action_focus();
        let maintenance_shell = state.deploy_shell_model();
        assert_eq!(maintenance_shell.mode, DeployShellMode::AdvancedMaintenance);
        assert_eq!(maintenance_shell.preview_title, "Maintenance Preview");
        assert_eq!(maintenance_shell.controls_title, "Repository Context");
        assert_eq!(maintenance_shell.detail_title, "Maintenance Detail");

        state.set_advanced_action_focus(ActionItem::LaunchDeployWizard);
        let wizard_shell = state.deploy_shell_model();
        assert_eq!(wizard_shell.mode, DeployShellMode::AdvancedWizard);
        assert_eq!(wizard_shell.preview_title, "Deploy Preview");
        assert_eq!(wizard_shell.controls_title, "Deploy Parameters");
        assert_eq!(wizard_shell.detail_title, "Deploy Wizard Detail");
    }

    #[test]
    fn deploy_page_model_collects_mode_specific_state_in_state_layer() {
        let mut state = test_state("/repo", "/etc/nixos", "sudo-available");

        match state.deploy_page_model() {
            DeployPageModel::Apply(model) => {
                assert_eq!(model.shell.mode, DeployShellMode::Apply);
                assert_eq!(model.selection.focused_row.label, "目标主机");
                assert_eq!(model.selection.focused_row.value, "demo");
                assert_eq!(
                    model.selection.execution_hint,
                    "可执行：当前组合可直接 Apply"
                );
                assert_eq!(model.controls.focused_row.label, "目标主机");
                assert_eq!(model.controls.focused_row.value, "demo");
                assert!(model.advanced_actions.is_none());
                assert!(model.workspace.is_none());
            }
            other => panic!("expected apply page model, got {other:?}"),
        }

        state.show_advanced = true;
        match state.deploy_page_model() {
            DeployPageModel::Apply(model) => {
                assert!(model.shell.workspace_visible);
                assert_eq!(model.selection.execution_hint, "当前已打开高级工作区");
                assert!(model.advanced_actions.is_some());
                assert!(model.workspace.is_some());
            }
            other => panic!("expected apply page model with workspace, got {other:?}"),
        }

        state.open_advanced();
        state.ensure_advanced_action_focus();
        match state.deploy_page_model() {
            DeployPageModel::AdvancedMaintenance(model) => {
                assert_eq!(model.shell.mode, DeployShellMode::AdvancedMaintenance);
                assert_eq!(
                    model.maintenance.summary.current_action,
                    ActionItem::FlakeUpdate
                );
                assert!(model.advanced_actions.is_some());
            }
            other => panic!("expected advanced maintenance page model, got {other:?}"),
        }

        state.set_advanced_action_focus(ActionItem::LaunchDeployWizard);
        match state.deploy_page_model() {
            DeployPageModel::AdvancedWizard(model) => {
                assert_eq!(model.shell.mode, DeployShellMode::AdvancedWizard);
                assert_eq!(model.wizard.action, DeployAction::Switch);
                assert_eq!(model.controls.focused_row.label, "目标主机");
                assert_eq!(model.controls.focused_row.value, "demo");
                assert!(model.advanced_actions.is_some());
            }
            other => panic!("expected advanced wizard page model, got {other:?}"),
        }
    }

    #[test]
    fn overview_apply_route_feedback_ignores_stale_apply_workspace_state() {
        let mut state = test_state("/repo", "/etc/nixos", "sudo-available");
        state.show_advanced = true;

        let feedback = state.overview_apply_route_feedback();

        assert_eq!(
            feedback.message,
            "Overview 已把你带到 Apply；当前组合可直接执行。"
        );
        assert_eq!(feedback.next_step, "在 Apply 查看预览，或按 a / x 直接运行");
    }

    #[test]
    fn actions_apply_route_feedback_keeps_direct_apply_copy_aligned() {
        let state = test_state("/repo", "/etc/nixos", "sudo-available");

        let feedback = state.actions_apply_route_feedback(ActionItem::SyncRepoToEtc);

        assert_eq!(
            feedback.message,
            "sync to /etc/nixos 归属 Apply；当前组合可直接执行。"
        );
        assert_eq!(feedback.next_step, "在 Apply 查看预览，或按 a / x 直接运行");
    }

    #[test]
    fn return_from_advanced_to_apply_restores_direct_feedback() {
        let mut state = test_state("/repo", "/etc/nixos", "sudo-available");
        state.open_advanced();

        state.return_from_advanced_to_apply();

        assert_eq!(state.page(), Page::Deploy);
        assert!(!state.show_advanced);
        assert_eq!(state.feedback.scope, UiFeedbackScope::Apply);
        assert_eq!(
            state.feedback.message,
            "已从 Advanced 返回 Apply；当前组合可直接执行。"
        );
        assert_eq!(
            state.feedback.next_step.as_deref(),
            Some("在 Apply 查看预览，或按 a / x 直接运行")
        );
    }

    #[test]
    fn return_from_advanced_to_apply_restores_handoff_feedback() {
        let mut state = test_state("/repo", "/etc/nixos", "sudo-available");
        state.deploy_source = DeploySource::RemotePinned;
        state.deploy_source_ref = "v5.0.0".to_string();
        state.open_advanced();

        state.return_from_advanced_to_apply();

        assert_eq!(state.page(), Page::Deploy);
        assert!(!state.show_advanced);
        assert_eq!(state.feedback.scope, UiFeedbackScope::Apply);
        assert_eq!(
            state.feedback.message,
            "已从 Advanced 返回 Apply；当前来源是远端固定版本；默认 Apply 不会直接执行，必须交给完整高级路径。"
        );
        assert_eq!(
            state.feedback.next_step.as_deref(),
            Some("在 Apply 先看 handoff 预览；如需继续，切到 Advanced 执行 launch deploy wizard")
        );
    }

    #[test]
    fn apply_selection_model_surfaces_handoff_specific_guidance() {
        let mut state = test_state("/repo", "/etc/nixos", "sudo-available");
        state.deploy_source = DeploySource::RemotePinned;
        state.deploy_source_ref = "v5.0.0".to_string();

        let selection = state.apply_selection_model();

        assert_eq!(
            selection.recommendation,
            "建议：当前默认应切到 Advanced 执行 launch deploy wizard；先看 handoff 预览。"
        );
        assert_eq!(
            selection.execution_hint,
            "需交接：默认应切到 Advanced 执行 launch deploy wizard"
        );
        assert_eq!(
            selection.advanced_action_hint,
            "高级动作：当前默认应切到 Advanced 执行 launch deploy wizard"
        );
    }

    #[test]
    fn apply_handoff_preview_fallback_stays_aligned_with_selection_guidance() {
        let mut state = test_state("/repo", "/etc/nixos", "sudo-available");
        state.deploy_source = DeploySource::RemotePinned;
        state.deploy_source_ref = "v5.0.0".to_string();

        let selection = state.apply_selection_model();
        let preview_fallback = state.apply_preview_command_fallback();

        assert_eq!(
            selection.recommendation,
            "建议：当前默认应切到 Advanced 执行 launch deploy wizard；先看 handoff 预览。"
        );
        assert_eq!(
            preview_fallback,
            "当前组合会转交给 Advanced 执行 launch deploy wizard"
        );
    }

    #[test]
    fn apply_model_handoff_omits_direct_rebuild_preview() {
        let mut state = test_state("/repo", "/etc/nixos", "sudo-available");
        state.deploy_source = DeploySource::RemotePinned;
        state.deploy_source_ref = "v5.0.0".to_string();

        let model = state.apply_model();

        assert!(!model.can_execute_directly);
        assert!(!model.can_apply_current_host);
        assert!(model.rebuild_preview.is_none());
        assert_eq!(
            model.handoffs,
            vec![
                "当前来源是远端固定版本；默认 Apply 不会直接执行，必须交给完整高级路径。"
                    .to_string()
            ]
        );
    }

    #[test]
    fn deploy_rows_switch_last_row_to_area_action_inside_advanced() {
        let mut state = test_state("/repo", "/etc/nixos", "sudo-available");
        state.open_advanced();

        let rows = state.deploy_rows();

        assert_eq!(rows[6].label, "区域切换");
        assert_eq!(rows[6].value, "Enter 返回 Apply");
    }

    #[test]
    fn advanced_wizard_field_navigation_stays_independent_from_apply_focus() {
        let mut state = test_state("/repo", "/etc/nixos", "sudo-available");
        state.deploy_focus = 3;
        state.open_advanced();
        state.set_advanced_action_focus(ActionItem::LaunchDeployWizard);

        assert_eq!(state.selected_deploy_focus(), 0);

        state.next_advanced_wizard_field();

        assert_eq!(state.advanced_deploy_focus, 1);
        assert_eq!(state.deploy_focus, 3);

        state.open_apply();

        assert_eq!(state.selected_deploy_focus(), 3);
    }

    #[test]
    fn advanced_wizard_preview_and_command_use_advanced_parameter_snapshot() {
        let mut state = test_state("/repo", "/etc/nixos", "sudo-available");
        state.target_host = "apply-host".to_string();
        state.deploy_task = DeployTask::DirectDeploy;
        state.deploy_source = DeploySource::CurrentRepo;
        state.deploy_action = DeployAction::Switch;
        state.flake_update = false;

        state.open_advanced();
        state.set_advanced_action_focus(ActionItem::LaunchDeployWizard);
        state.advanced_target_host = "wizard-host".to_string();
        state.advanced_deploy_task = DeployTask::AdjustStructure;
        state.advanced_deploy_source = DeploySource::EtcNixos;
        state.advanced_deploy_action = DeployAction::Boot;
        state.advanced_flake_update = true;

        let model = state.advanced_wizard_model();

        assert_eq!(model.target_host, "wizard-host");
        assert_eq!(model.task, DeployTask::AdjustStructure);
        assert_eq!(model.source, DeploySource::EtcNixos);
        assert_eq!(model.action, DeployAction::Boot);
        assert!(model.flake_update);
        assert_eq!(
            model.command_preview,
            "mcb-deploy --mode manage-users --action boot --host wizard-host --source etc-nixos --upgrade"
        );
        assert_eq!(model.validation_error, None);
        assert_eq!(
            state.current_deploy_wizard_command_preview(),
            "mcb-deploy --mode manage-users --action boot --host wizard-host --source etc-nixos --upgrade"
        );
    }

    #[test]
    fn apply_and_advanced_controls_models_keep_independent_rows_and_focus() {
        let mut state = test_state("/repo", "/etc/nixos", "sudo-available");
        state.target_host = "apply-host".to_string();
        state.deploy_focus = 4;
        state.open_advanced();
        state.set_advanced_action_focus(ActionItem::LaunchDeployWizard);
        state.advanced_target_host = "wizard-host".to_string();
        state.advanced_deploy_source = DeploySource::RemotePinned;
        state.advanced_deploy_source_ref = "v5.0.0".to_string();
        state.advanced_deploy_focus = 3;

        let apply_controls = state.apply_controls_model();
        let wizard_controls = state.advanced_wizard_controls_model();

        assert_eq!(apply_controls.rows[0].value, "apply-host");
        assert_eq!(apply_controls.focused_row.label, "动作");
        assert_eq!(apply_controls.focused_row.value, "switch");
        assert_eq!(wizard_controls.rows[0].value, "wizard-host");
        assert_eq!(wizard_controls.rows[3].value, "v5.0.0");
        assert_eq!(wizard_controls.focused_row.label, "固定 ref");
        assert_eq!(wizard_controls.focused_row.value, "v5.0.0");
    }

    #[test]
    fn apply_area_switch_handoff_aligns_wizard_focus_and_feedback() {
        let mut state = test_state("/repo", "/etc/nixos", "sudo-available");
        state.deploy_source = DeploySource::RemotePinned;
        state.deploy_source_ref = "v5.0.0".to_string();
        state.deploy_focus = 6;

        state.adjust_apply_control(1);

        assert_eq!(state.page(), Page::Advanced);
        assert_eq!(
            state.current_advanced_action(),
            ActionItem::LaunchDeployWizard
        );
        assert_eq!(state.advanced_deploy_focus, 3);
        assert_eq!(state.feedback.scope, UiFeedbackScope::Advanced);
        assert_eq!(
            state.feedback.message,
            "Apply 已跳到 Advanced，并对准 launch deploy wizard。推荐原因：当前来源是远端固定版本；默认 Apply 不会直接执行，必须交给完整高级路径。"
        );
        assert_eq!(
            state.feedback.next_step.as_deref(),
            Some("在 Advanced 里先确认 Deploy Parameters，再执行 launch deploy wizard")
        );
        match state.deploy_page_model() {
            DeployPageModel::AdvancedWizard(model) => {
                assert_eq!(model.controls.focused_row.label, "固定 ref");
                assert_eq!(model.controls.focused_row.value, "v5.0.0");
            }
            other => panic!("expected advanced wizard page model, got {other:?}"),
        }
    }

    #[test]
    fn remote_pinned_rows_preview_and_command_include_ref() {
        let mut state = test_state("/repo", "/etc/nixos", "sudo-available");
        state.open_advanced();
        state.set_advanced_action_focus(ActionItem::LaunchDeployWizard);
        state.advanced_deploy_source = DeploySource::RemotePinned;
        state.advanced_deploy_source_ref = "v5.0.0".to_string();

        let rows = state.deploy_rows();

        assert_eq!(rows[3].label, "固定 ref");
        assert_eq!(rows[3].value, "v5.0.0");
        assert_eq!(
            state.current_deploy_wizard_command_preview(),
            "mcb-deploy --mode update-existing --action switch --host demo --source remote-pinned --ref v5.0.0"
        );
    }

    #[test]
    fn remote_pinned_without_ref_surfaces_validation_before_handoff() {
        let mut state = test_state("/repo", "/etc/nixos", "sudo-available");
        state.open_advanced();
        state.set_advanced_action_focus(ActionItem::LaunchDeployWizard);
        state.advanced_deploy_source = DeploySource::RemotePinned;

        let model = state.advanced_wizard_model();

        assert!(
            model
                .blockers
                .iter()
                .any(|item| item.contains("缺少 ref/pin"))
        );
        assert!(
            model
                .validation_error
                .as_deref()
                .is_some_and(|error| error.contains("缺少 ref/pin"))
        );
        assert!(model.command_preview.contains("--source remote-pinned"));
        assert!(
            state
                .current_deploy_wizard_validation_error()
                .is_some_and(|error| error.contains("缺少 ref/pin"))
        );
        assert!(
            state
                .current_deploy_wizard_command_preview()
                .contains("[blocked: 远端固定版本缺少 ref/pin")
        );
    }

    #[test]
    fn apply_and_advanced_wizard_text_edits_keep_refs_independent() {
        let mut state = test_state("/repo", "/etc/nixos", "sudo-available");
        state.deploy_source = DeploySource::RemotePinned;
        state.deploy_source_ref = "apply-ref".to_string();
        state.advanced_deploy_source = DeploySource::RemotePinned;
        state.advanced_deploy_source_ref = "wizard-ref".to_string();

        state.open_apply_text_edit();
        assert_eq!(
            state.active_deploy_text_mode(),
            Some(DeployTextMode::ApplyRemotePinnedRef)
        );
        state.host_text_input = "apply-release".to_string();
        state.handle_apply_text_input(crossterm::event::KeyCode::Enter);

        assert_eq!(state.deploy_source_ref, "apply-release");
        assert_eq!(state.advanced_deploy_source_ref, "wizard-ref");
        assert!(state.active_deploy_text_mode().is_none());

        state.open_advanced();
        state.set_advanced_action_focus(ActionItem::LaunchDeployWizard);
        state.advanced_deploy_source_ref.clear();

        state.open_advanced_wizard_text_edit();
        assert_eq!(
            state.active_deploy_text_mode(),
            Some(DeployTextMode::AdvancedWizardRemotePinnedRef)
        );
        state.host_text_input = "release-2026".to_string();
        state.handle_advanced_wizard_text_input(crossterm::event::KeyCode::Enter);

        assert_eq!(state.advanced_deploy_source_ref, "release-2026");
        assert_eq!(state.deploy_source_ref, "apply-release");
        assert!(state.active_deploy_text_mode().is_none());
    }

    #[test]
    fn advanced_maintenance_model_avoids_apply_handoff_logic_for_local_repo_actions() {
        let mut state = test_state("/repo", "/etc/nixos", "sudo-available");
        state.open_advanced();
        state.ensure_advanced_action_focus();

        let model = state.advanced_maintenance_model();

        assert_eq!(model.summary.current_action, ActionItem::FlakeUpdate);
        assert_eq!(model.summary.recommended_action, ActionItem::FlakeUpdate);
        assert_eq!(model.write_target, "flake.lock");
        assert!(!state.show_advanced);
        assert!(model.impact.contains("flake.lock"));
        assert!(model.return_hint.contains("按 b 返回 Apply"));
    }

    #[test]
    fn advanced_maintenance_model_points_back_to_wizard_when_remote_source_is_selected() {
        let mut state = test_state("/repo", "/etc/nixos", "sudo-available");
        state.show_advanced = true;
        state.deploy_source = DeploySource::RemoteHead;
        state.set_advanced_action_focus(ActionItem::FlakeUpdate);

        let model = state.advanced_maintenance_model();

        assert_eq!(model.summary.current_action, ActionItem::FlakeUpdate);
        assert_eq!(
            model.summary.recommended_action,
            ActionItem::LaunchDeployWizard
        );
        assert!(model.return_hint.contains("launch deploy wizard"));
        assert_eq!(
            model.command_preview.as_deref(),
            Some(
                "nix --extra-experimental-features 'nix-command flakes' flake update --flake /repo"
            )
        );
    }

    #[test]
    fn advanced_maintenance_model_surfaces_latest_result_for_advanced_feedback() {
        let mut state = test_state("/repo", "/etc/nixos", "sudo-available");
        state.open_advanced();
        state.ensure_advanced_action_focus();
        state.set_advanced_maintenance_completion_feedback(ActionItem::FlakeUpdate);

        let model = state.advanced_maintenance_model();

        assert_eq!(
            model.summary.completion_hint,
            "做完后回 Inspect 或 Overview 复查仓库状态，必要时再回 Apply。"
        );
        assert_eq!(
            model.latest_result,
            "flake update 已完成。 下一步：做完后回 Inspect 或 Overview 复查仓库状态，必要时再回 Apply。"
        );
        assert_eq!(
            model.return_hint,
            "做完后回 Inspect 或 Overview 复查仓库状态，必要时再回 Apply。"
        );
    }

    #[test]
    fn advanced_wizard_detail_model_prefers_active_advanced_feedback_completion_hint() {
        let mut state = test_state("/repo", "/etc/nixos", "sudo-available");
        state.open_advanced();
        state.ensure_advanced_action_focus();
        state.set_advanced_action_focus(ActionItem::LaunchDeployWizard);
        state.set_deploy_wizard_return_feedback();

        let model = state.advanced_wizard_detail_model();

        assert_eq!(model.action, ActionItem::LaunchDeployWizard);
        assert_eq!(model.completion_hint, "继续在 Advanced 完成复杂部署");
        assert_eq!(
            model.latest_result,
            "已返回完整部署向导。 下一步：继续在 Advanced 完成复杂部署"
        );
    }

    #[test]
    fn apply_model_surfaces_blockers_warnings_and_previews() {
        let mut state = test_state("/repo", "/etc/nixos", "sudo-available");
        state.package_dirty_users.insert("alice".to_string());
        state.flake_update = true;

        let model = state.apply_model();

        assert_eq!(model.target_host, "demo");
        assert!(model.can_execute_directly);
        assert!(!model.can_apply_current_host);
        assert!(
            model
                .blockers
                .iter()
                .any(|item| item.contains("Packages: alice"))
        );
        assert!(
            model
                .warnings
                .iter()
                .any(|item| item.contains("同步到 /etc/nixos"))
        );
        assert!(model.warnings.iter().any(|item| item.contains("--upgrade")));
        assert!(model.sync_preview.is_some());
        assert!(
            model
                .rebuild_preview
                .as_deref()
                .is_some_and(|preview| preview.contains("/etc/nixos#demo"))
        );
    }

    #[test]
    fn apply_model_reports_rootless_direct_build_without_sync() {
        let mut state = test_state("/repo", "/etc/nixos", "rootless");
        state.deploy_action = DeployAction::Build;

        let model = state.apply_model();

        assert!(model.can_execute_directly);
        assert!(model.can_apply_current_host);
        assert!(model.sync_preview.is_none());
        assert!(model.blockers.is_empty());
        assert!(model.warnings.iter().all(|item| !item.contains("sudo -E")));
    }

    #[test]
    fn execute_deploy_rejects_unsaved_changes_before_other_checks() {
        let mut state = test_state("/definitely/missing/repo", "/etc/nixos", "sudo-available");
        state.home_dirty_users.insert("alice".to_string());
        state.package_dirty_users.insert("alice".to_string());

        let err = state
            .execute_deploy()
            .expect_err("unsaved changes should block execution immediately");
        let text = err.to_string();
        assert!(text.contains("仍有未保存修改"));
        assert!(text.contains("Packages: alice"));
        assert!(text.contains("Home: alice"));
    }

    #[test]
    fn execute_deploy_rejects_rootless_non_build_before_external_commands() -> Result<()> {
        let repo_root = create_temp_dir("mcbctl-deploy-state")?;
        let mut state = test_state(
            repo_root.to_string_lossy().as_ref(),
            "/etc/nixos",
            "rootless",
        );
        state.deploy_action = DeployAction::Switch;

        let err = state
            .execute_deploy()
            .expect_err("rootless direct switch should be rejected");
        assert!(
            err.to_string()
                .contains("rootless 模式下当前页只能直接执行 build")
        );

        std::fs::remove_dir_all(repo_root)?;
        Ok(())
    }

    #[test]
    fn deploy_wizard_return_feedback_uses_shared_advanced_copy() {
        let mut state = test_state("/repo", "/etc/nixos", "sudo-available");

        state.set_deploy_wizard_return_feedback();

        assert_eq!(state.feedback.scope, UiFeedbackScope::Advanced);
        assert_eq!(state.feedback.level, UiFeedbackLevel::Info);
        assert_eq!(state.feedback.message, "已返回完整部署向导。");
        assert_eq!(
            state.feedback.next_step.as_deref(),
            Some("继续在 Advanced 完成复杂部署")
        );
    }

    #[test]
    fn sync_repo_completion_feedback_uses_shared_apply_copy() {
        let mut state = test_state("/repo", "/etc/nixos", "sudo-available");

        state.set_sync_repo_completion_feedback();

        assert_eq!(state.feedback.scope, UiFeedbackScope::Apply);
        assert_eq!(state.feedback.level, UiFeedbackLevel::Success);
        assert_eq!(state.feedback.message, "仓库已同步到 /etc/nixos。");
        assert_eq!(
            state.feedback.next_step.as_deref(),
            Some("回到 Apply 或 Overview 继续后续重建")
        );
    }

    #[test]
    fn rebuild_completion_feedback_keeps_apply_and_action_copy_aligned() {
        let mut state = test_state("/repo", "/etc/nixos", "sudo-available");
        let plan = NixosRebuildPlan {
            action: DeployAction::Switch,
            upgrade: false,
            flake_root: PathBuf::from("/etc/nixos"),
            target_host: "demo".to_string(),
        };

        state.set_apply_rebuild_completion_feedback(&plan);
        assert_eq!(state.feedback.scope, UiFeedbackScope::Apply);
        assert_eq!(state.feedback.level, UiFeedbackLevel::Success);
        assert_eq!(state.feedback.message, "Apply 已执行完成：switch demo");
        assert_eq!(
            state.feedback.next_step.as_deref(),
            Some("回到 Overview 检查健康和下一步")
        );

        state.set_current_host_rebuild_completion_feedback(&plan);
        assert_eq!(state.feedback.scope, UiFeedbackScope::Apply);
        assert_eq!(state.feedback.level, UiFeedbackLevel::Success);
        assert_eq!(state.feedback.message, "当前主机 demo 已完成一次 switch。");
        assert_eq!(
            state.feedback.next_step.as_deref(),
            Some("回到 Overview 检查健康和下一步")
        );
    }

    #[test]
    fn advanced_maintenance_completion_feedback_uses_action_specific_copy() {
        let mut state = test_state("/repo", "/etc/nixos", "sudo-available");

        state.set_advanced_maintenance_completion_feedback(ActionItem::FlakeUpdate);
        assert_eq!(state.feedback.scope, UiFeedbackScope::Advanced);
        assert_eq!(state.feedback.level, UiFeedbackLevel::Success);
        assert_eq!(state.feedback.message, "flake update 已完成。");
        assert_eq!(
            state.feedback.next_step.as_deref(),
            Some("做完后回 Inspect 或 Overview 复查仓库状态，必要时再回 Apply。")
        );

        state.set_advanced_maintenance_completion_feedback(ActionItem::UpdateUpstreamPins);
        assert_eq!(state.feedback.scope, UiFeedbackScope::Advanced);
        assert_eq!(state.feedback.level, UiFeedbackLevel::Success);
        assert_eq!(state.feedback.message, "上游 pin 刷新已完成。");
        assert_eq!(
            state.feedback.next_step.as_deref(),
            Some("做完后回 Inspect 或 Overview 复查仓库状态，必要时再回 Apply。")
        );
    }

    fn test_state(repo_root: &str, etc_root: &str, privilege_mode: &str) -> AppState {
        let context = AppContext {
            repo_root: PathBuf::from(repo_root),
            etc_root: PathBuf::from(etc_root),
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
        };

        let mut host_settings_by_name = BTreeMap::new();
        host_settings_by_name.insert("demo".to_string(), valid_host_settings());

        AppState {
            context,
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
            host_settings_by_name,
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

    fn valid_host_settings() -> HostManagedSettings {
        HostManagedSettings {
            primary_user: "alice".to_string(),
            users: vec!["alice".to_string()],
            admin_users: vec!["alice".to_string()],
            ..HostManagedSettings::default()
        }
    }

    fn create_temp_dir(prefix: &str) -> Result<PathBuf> {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let root = std::env::temp_dir().join(format!("{prefix}-{}-{unique}", std::process::id()));
        std::fs::create_dir_all(&root)?;
        Ok(root)
    }
}
