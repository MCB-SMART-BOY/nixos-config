use super::state::AppState;
use super::views;
use crate::domain::tui::{DeployTextMode, PackageTextMode, Page, TopLevelPage};
use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use std::io::{self, Stdout};
use std::time::Duration;

type CrosstermTerminal = Terminal<ratatui::backend::CrosstermBackend<Stdout>>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DashboardKeyOutcome {
    NotHandled,
    Routed,
    RunApply,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AdvancedKeyOutcome {
    NotHandled,
    Routed,
    RunAction,
}

pub fn run(mut state: AppState) -> Result<()> {
    let mut terminal = setup_terminal()?;
    let result = run_loop(&mut terminal, &mut state);
    restore_terminal(&mut terminal)?;
    result
}

fn setup_terminal() -> Result<CrosstermTerminal> {
    enable_raw_mode().context("failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).context("failed to enter alternate screen")?;
    Terminal::new(ratatui::backend::CrosstermBackend::new(stdout))
        .context("failed to create terminal")
}

fn restore_terminal(terminal: &mut CrosstermTerminal) -> Result<()> {
    disable_raw_mode().context("failed to disable raw mode")?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .context("failed to leave alternate screen")?;
    terminal.show_cursor().context("failed to show cursor")
}

fn run_loop(terminal: &mut CrosstermTerminal, state: &mut AppState) -> Result<()> {
    loop {
        terminal.draw(|frame| views::render(frame, state))?;
        if !event::poll(Duration::from_millis(200))? {
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        if state.captures_text_input() {
            handle_text_input(state, key.code, key.modifiers)?;
            continue;
        }

        match key.code {
            KeyCode::Char('q') => break,
            _ if handle_shell_navigation_key(state, key.code, key.modifiers) => {}
            _ => handle_page_key(terminal, state, key.code, key.modifiers)?,
        }
    }
    Ok(())
}

fn handle_text_input(state: &mut AppState, code: KeyCode, modifiers: KeyModifiers) -> Result<()> {
    if modifiers.contains(KeyModifiers::CONTROL) {
        return Ok(());
    }

    match state.page() {
        Page::Packages => {
            match state.active_package_text_mode() {
                Some(PackageTextMode::Search) => state.handle_search_input(code),
                Some(PackageTextMode::CreateGroup) => state.handle_group_input(code),
                Some(PackageTextMode::RenameGroup) => state.handle_group_input(code),
                None => {}
            }
            Ok(())
        }
        Page::Deploy | Page::Advanced => {
            match state.active_deploy_text_mode() {
                Some(DeployTextMode::ApplyRemotePinnedRef) => state.handle_apply_text_input(code),
                Some(DeployTextMode::AdvancedWizardRemotePinnedRef) => {
                    state.handle_advanced_wizard_text_input(code)
                }
                None => {}
            }
            Ok(())
        }
        Page::Users => {
            if state.active_users_text_mode().is_some() {
                state.handle_users_text_input(code);
            }
            Ok(())
        }
        Page::Hosts => {
            if state.active_hosts_text_mode().is_some() {
                state.handle_hosts_text_input(code);
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn handle_page_key(
    terminal: &mut CrosstermTerminal,
    state: &mut AppState,
    code: KeyCode,
    modifiers: KeyModifiers,
) -> Result<()> {
    if modifiers.contains(KeyModifiers::CONTROL) {
        return Ok(());
    }

    if state.top_level_page() == TopLevelPage::Edit {
        match code {
            KeyCode::Char('1') => {
                state.open_edit_page(Page::Packages);
                return Ok(());
            }
            KeyCode::Char('2') => {
                state.open_edit_page(Page::Home);
                return Ok(());
            }
            KeyCode::Char('3') => {
                state.open_edit_page(Page::Users);
                return Ok(());
            }
            KeyCode::Char('4') => {
                state.open_edit_page(Page::Hosts);
                return Ok(());
            }
            _ => {}
        }
    }

    match state.page() {
        Page::Dashboard => match handle_dashboard_key(state, code) {
            DashboardKeyOutcome::NotHandled | DashboardKeyOutcome::Routed => {}
            DashboardKeyOutcome::RunApply => {
                run_foreground_task(terminal, state, "Apply", |state| state.execute_deploy())?
            }
        },
        Page::Deploy => match code {
            KeyCode::Down | KeyCode::Char('j') => state.next_apply_control(),
            KeyCode::Up | KeyCode::Char('k') => state.previous_apply_control(),
            KeyCode::Left | KeyCode::Char('h') => state.adjust_apply_control(-1),
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter | KeyCode::Char(' ') => {
                state.adjust_apply_control(1)
            }
            KeyCode::Char('x') => {
                run_foreground_task(terminal, state, "Apply", |state| state.execute_deploy())?
            }
            _ => {}
        },
        Page::Advanced => match handle_advanced_key(state, code) {
            AdvancedKeyOutcome::NotHandled | AdvancedKeyOutcome::Routed => {}
            AdvancedKeyOutcome::RunAction => {
                run_foreground_task(terminal, state, "Advanced", |state| {
                    state.execute_current_advanced_action_from_apply()
                })?
            }
        },
        Page::Inspect => match code {
            KeyCode::Down | KeyCode::Char('j') => state.next_inspect_action(),
            KeyCode::Up | KeyCode::Char('k') => state.previous_inspect_action(),
            KeyCode::Char('r') => state.refresh_overview_repo_integrity(),
            KeyCode::Char('d') => state.refresh_overview_doctor(),
            KeyCode::Char('R') => state.refresh_overview_health(),
            KeyCode::Char('x') => run_foreground_task(terminal, state, "Inspect", |state| {
                state.ensure_inspect_action_focus();
                state.execute_current_action()
            })?,
            _ => {}
        },
        Page::Users => match code {
            KeyCode::Down | KeyCode::Char('j') => state.next_users_field(),
            KeyCode::Up | KeyCode::Char('k') => state.previous_users_field(),
            KeyCode::Left => state.switch_target_host(-1),
            KeyCode::Right => state.switch_target_host(1),
            KeyCode::Char('h') => state.adjust_users_field(-1),
            KeyCode::Char('l') | KeyCode::Char(' ') => state.adjust_users_field(1),
            KeyCode::Enter => state.open_users_text_edit(),
            KeyCode::Char('s') => state.save_current_host_users()?,
            _ => {}
        },
        Page::Hosts => match code {
            KeyCode::Down | KeyCode::Char('j') => state.next_hosts_field(),
            KeyCode::Up | KeyCode::Char('k') => state.previous_hosts_field(),
            KeyCode::Left => state.switch_target_host(-1),
            KeyCode::Right => state.switch_target_host(1),
            KeyCode::Char('h') => state.adjust_hosts_field(-1),
            KeyCode::Char('l') | KeyCode::Char(' ') => state.adjust_hosts_field(1),
            KeyCode::Enter => state.open_hosts_text_edit(),
            KeyCode::Char('s') => state.save_current_host_runtime()?,
            _ => {}
        },
        Page::Packages => match code {
            KeyCode::Down | KeyCode::Char('j') => state.next_package_item(),
            KeyCode::Up | KeyCode::Char('k') => state.previous_package_item(),
            KeyCode::Left => state.previous_package_user(),
            KeyCode::Right => state.next_package_user(),
            KeyCode::Char('f') => state.toggle_package_mode(),
            KeyCode::Char('r') => state.refresh_package_search_results(),
            KeyCode::Char('[') | KeyCode::Char('h') => state.previous_package_category(),
            KeyCode::Char(']') | KeyCode::Char('l') => state.next_package_category(),
            KeyCode::Char('u') => state.adjust_package_source_filter(-1),
            KeyCode::Char('i') => state.adjust_package_source_filter(1),
            KeyCode::Char('g') => state.adjust_current_package_group(-1),
            KeyCode::Char('G') => state.adjust_current_package_group(1),
            KeyCode::Char('m') => state.move_current_selected_group(-1),
            KeyCode::Char('M') => state.move_current_selected_group(1),
            KeyCode::Char(',') => state.adjust_package_group_filter(-1),
            KeyCode::Char('.') => state.adjust_package_group_filter(1),
            KeyCode::Char('z') => state.focus_current_selected_group(),
            KeyCode::Char('Z') => state.clear_package_group_filter(),
            KeyCode::Char('n') => state.open_package_group_creation(),
            KeyCode::Char('R') => state.open_package_group_rename(),
            KeyCode::Char('/') => state.open_package_search(),
            KeyCode::Backspace => state.clear_package_search(),
            KeyCode::Enter | KeyCode::Char(' ') => state.toggle_current_package(),
            KeyCode::Char('s') => state.save_current_user_packages()?,
            _ => {}
        },
        Page::Home => match code {
            KeyCode::Down | KeyCode::Char('j') => state.next_home_field(),
            KeyCode::Up | KeyCode::Char('k') => state.previous_home_field(),
            KeyCode::Left => state.previous_home_user(),
            KeyCode::Right => state.next_home_user(),
            KeyCode::Char('h') => state.adjust_home_field(-1),
            KeyCode::Char('l') | KeyCode::Enter | KeyCode::Char(' ') => state.adjust_home_field(1),
            KeyCode::Char('s') => state.save_current_home_settings()?,
            _ => {}
        },
        Page::Actions => match code {
            KeyCode::Down | KeyCode::Char('j') => state.next_action_item(),
            KeyCode::Up | KeyCode::Char('k') => state.previous_action_item(),
            KeyCode::Enter | KeyCode::Char(' ') => state.open_current_action_destination(),
            KeyCode::Char('x') => run_foreground_task(terminal, state, "Actions", |state| {
                state.execute_current_action_from_actions()
            })?,
            _ => {}
        },
    }
    Ok(())
}

fn handle_shell_navigation_key(
    state: &mut AppState,
    code: KeyCode,
    modifiers: KeyModifiers,
) -> bool {
    match code {
        KeyCode::Tab if modifiers.is_empty() => {
            state.next_page();
            true
        }
        KeyCode::BackTab if modifiers.is_empty() => {
            state.previous_page();
            true
        }
        _ => false,
    }
}

fn handle_dashboard_key(state: &mut AppState, code: KeyCode) -> DashboardKeyOutcome {
    match code {
        KeyCode::Enter | KeyCode::Char(' ') => {
            state.open_overview_primary_action();
            DashboardKeyOutcome::Routed
        }
        KeyCode::Char('p') => {
            state.open_overview_apply();
            DashboardKeyOutcome::Routed
        }
        KeyCode::Char('i') => {
            state.open_overview_inspect();
            DashboardKeyOutcome::Routed
        }
        KeyCode::Char('a') => {
            if state.apply_model().can_apply_current_host {
                DashboardKeyOutcome::RunApply
            } else {
                state.open_overview_primary_action();
                DashboardKeyOutcome::Routed
            }
        }
        KeyCode::Char('r') => {
            state.refresh_overview_repo_integrity();
            DashboardKeyOutcome::Routed
        }
        KeyCode::Char('d') => {
            state.refresh_overview_doctor();
            DashboardKeyOutcome::Routed
        }
        KeyCode::Char('R') => {
            state.refresh_overview_health();
            DashboardKeyOutcome::Routed
        }
        _ => DashboardKeyOutcome::NotHandled,
    }
}

fn handle_advanced_key(state: &mut AppState, code: KeyCode) -> AdvancedKeyOutcome {
    match code {
        KeyCode::Char('J') => {
            state.next_advanced_action();
            AdvancedKeyOutcome::Routed
        }
        KeyCode::Char('K') => {
            state.previous_advanced_action();
            AdvancedKeyOutcome::Routed
        }
        KeyCode::Char('X') | KeyCode::Char('x') => AdvancedKeyOutcome::RunAction,
        KeyCode::Char('b') => {
            state.return_from_advanced_to_apply();
            AdvancedKeyOutcome::Routed
        }
        KeyCode::Down | KeyCode::Char('j') if !state.advanced_action_uses_deploy_parameters() => {
            state.next_advanced_action();
            AdvancedKeyOutcome::Routed
        }
        KeyCode::Up | KeyCode::Char('k') if !state.advanced_action_uses_deploy_parameters() => {
            state.previous_advanced_action();
            AdvancedKeyOutcome::Routed
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.next_advanced_wizard_field();
            AdvancedKeyOutcome::Routed
        }
        KeyCode::Up | KeyCode::Char('k') => {
            state.previous_advanced_wizard_field();
            AdvancedKeyOutcome::Routed
        }
        KeyCode::Left | KeyCode::Char('h') if !state.advanced_action_uses_deploy_parameters() => {
            AdvancedKeyOutcome::Routed
        }
        KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter | KeyCode::Char(' ')
            if !state.advanced_action_uses_deploy_parameters() =>
        {
            AdvancedKeyOutcome::Routed
        }
        KeyCode::Left | KeyCode::Char('h') => {
            state.adjust_advanced_wizard_field(-1);
            AdvancedKeyOutcome::Routed
        }
        KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter | KeyCode::Char(' ') => {
            state.adjust_advanced_wizard_field(1);
            AdvancedKeyOutcome::Routed
        }
        _ => AdvancedKeyOutcome::NotHandled,
    }
}

fn run_foreground_task<F>(
    terminal: &mut CrosstermTerminal,
    state: &mut AppState,
    title: &str,
    action: F,
) -> Result<()>
where
    F: FnOnce(&mut AppState) -> Result<()>,
{
    restore_terminal(terminal)?;

    println!("== {title} ==");
    let result = action(state);
    match &result {
        Ok(_) => {
            println!();
            println!("{}", state.status);
        }
        Err(err) => {
            eprintln!();
            eprintln!("执行失败：{err:#}");
            state.status = format!("执行失败：{err:#}");
        }
    }
    println!();
    println!("按 Enter 返回 mcbctl...");
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);

    *terminal = setup_terminal()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::tui::{
        ActionItem, DeployAction, DeploySource, DeployTask, HostManagedSettings, PackageDataMode,
    };
    use crate::tui::state::{
        AppContext, AppState, DeployPageModel, OverviewCheckState, UiFeedback,
    };
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::PathBuf;

    #[test]
    fn dashboard_enter_routes_to_advanced_when_primary_action_requires_handoff() {
        let mut state = test_state("sudo-available");
        state.deploy_source = DeploySource::RemoteHead;
        state.deploy_focus = 6;

        assert_eq!(
            handle_dashboard_key(&mut state, KeyCode::Enter),
            DashboardKeyOutcome::Routed
        );
        assert_eq!(state.page(), Page::Advanced);
        assert_eq!(
            state.feedback.scope,
            crate::tui::state::UiFeedbackScope::Advanced
        );
        assert_eq!(
            state.feedback.message,
            "Overview 已跳到 Advanced，并对准 launch deploy wizard。推荐原因：当前来源是远端最新版本；默认 Apply 不会直接执行，必须交给完整高级路径。"
        );
        assert_eq!(
            state.feedback.next_step.as_deref(),
            Some("在 Advanced 里先确认 Deploy Parameters，再执行 launch deploy wizard")
        );
        assert_eq!(
            state.current_advanced_action(),
            ActionItem::LaunchDeployWizard
        );
        assert_eq!(state.advanced_deploy_focus, 2);
        match state.deploy_page_model() {
            DeployPageModel::AdvancedWizard(model) => {
                assert_eq!(model.controls.focused_row.label, "来源");
                assert_eq!(model.controls.focused_row.value, "远端最新版本");
            }
            other => panic!("expected advanced wizard page model, got {other:?}"),
        }
    }

    #[test]
    fn dashboard_a_routes_instead_of_running_when_apply_is_blocked() {
        let mut state = test_state("sudo-available");
        state.deploy_source = DeploySource::RemotePinned;
        state.deploy_source_ref = "v5.0.0".to_string();
        state.deploy_focus = 6;

        assert_eq!(
            handle_dashboard_key(&mut state, KeyCode::Char('a')),
            DashboardKeyOutcome::Routed
        );
        assert_eq!(state.page(), Page::Advanced);
        assert_eq!(
            state.feedback.scope,
            crate::tui::state::UiFeedbackScope::Advanced
        );
        assert_eq!(
            state.feedback.message,
            "Overview 已跳到 Advanced，并对准 launch deploy wizard。推荐原因：当前来源是远端固定版本；默认 Apply 不会直接执行，必须交给完整高级路径。"
        );
        assert_eq!(
            state.feedback.next_step.as_deref(),
            Some("在 Advanced 里先确认 Deploy Parameters，再执行 launch deploy wizard")
        );
        assert_eq!(state.advanced_deploy_focus, 3);
        match state.deploy_page_model() {
            DeployPageModel::AdvancedWizard(model) => {
                assert_eq!(model.controls.focused_row.label, "固定 ref");
                assert_eq!(model.controls.focused_row.value, "v5.0.0");
            }
            other => panic!("expected advanced wizard page model, got {other:?}"),
        }
    }

    #[test]
    fn dashboard_enter_routes_to_inspect_with_repo_integrity_reason() {
        let mut state = test_state("sudo-available");
        state.overview_repo_integrity = OverviewCheckState::Error {
            summary: "failed (1 finding(s))".to_string(),
            details: vec!["- [rule] path: detail".to_string()],
        };
        state.actions_focus = 4;

        assert_eq!(
            handle_dashboard_key(&mut state, KeyCode::Enter),
            DashboardKeyOutcome::Routed
        );
        assert_eq!(state.page(), Page::Inspect);
        assert_eq!(
            state.feedback.scope,
            crate::tui::state::UiFeedbackScope::Inspect
        );
        assert_eq!(
            state.feedback.message,
            "Overview 推荐先进入 Inspect 处理 repo-integrity（failed (1 finding(s))）。"
        );
        assert_eq!(
            state.feedback.next_step.as_deref(),
            Some("在 Inspect 先看 repo-integrity，再决定是否执行 flake check")
        );
        let inspect = state.inspect_model();
        assert_eq!(inspect.detail.action, ActionItem::FlakeCheck);
        assert_eq!(inspect.repo_integrity, state.overview_repo_integrity);
        assert_eq!(inspect.detail.label, "flake check");
    }

    #[test]
    fn dashboard_a_requests_direct_apply_when_current_host_is_ready() {
        let mut state = test_state("sudo-available");

        assert_eq!(
            handle_dashboard_key(&mut state, KeyCode::Char('a')),
            DashboardKeyOutcome::RunApply
        );
        assert_eq!(state.page(), Page::Dashboard);
        assert_eq!(state.feedback, UiFeedback::default());
    }

    #[test]
    fn dashboard_enter_routes_to_apply_with_aligned_feedback_when_direct_apply_is_ready() {
        let mut state = test_state("sudo-available");

        assert_eq!(
            handle_dashboard_key(&mut state, KeyCode::Enter),
            DashboardKeyOutcome::Routed
        );
        assert_eq!(state.page(), Page::Deploy);
        assert_eq!(
            state.feedback.scope,
            crate::tui::state::UiFeedbackScope::Apply
        );
        assert_eq!(
            state.feedback.message,
            "Overview 已把你带到 Apply；当前组合可直接执行。"
        );
        assert_eq!(
            state.feedback.next_step.as_deref(),
            Some("在 Apply 查看预览，或按 a / x 直接运行")
        );
    }

    #[test]
    fn dashboard_shortcuts_p_and_i_route_with_aligned_feedback() {
        let mut state = test_state("sudo-available");

        assert_eq!(
            handle_dashboard_key(&mut state, KeyCode::Char('p')),
            DashboardKeyOutcome::Routed
        );
        assert_eq!(state.page(), Page::Deploy);
        assert_eq!(
            state.feedback.scope,
            crate::tui::state::UiFeedbackScope::Apply
        );
        assert_eq!(
            state.feedback.message,
            "Overview 已把你带到 Apply；当前组合可直接执行。"
        );
        assert_eq!(
            state.feedback.next_step.as_deref(),
            Some("在 Apply 查看预览，或按 a / x 直接运行")
        );

        state.open_overview();
        assert_eq!(
            handle_dashboard_key(&mut state, KeyCode::Char('i')),
            DashboardKeyOutcome::Routed
        );
        assert_eq!(state.page(), Page::Inspect);
        assert_eq!(
            state.feedback.scope,
            crate::tui::state::UiFeedbackScope::Inspect
        );
        assert_eq!(state.feedback.message, "Overview 已跳到 Inspect。");
        assert_eq!(
            state.feedback.next_step.as_deref(),
            Some("在 Inspect 查看健康详情和检查命令")
        );
    }

    #[test]
    fn dashboard_shortcut_p_uses_apply_handoff_feedback_when_advanced_is_required() {
        let mut state = test_state("sudo-available");
        state.deploy_source = DeploySource::RemotePinned;
        state.deploy_source_ref = "v5.0.0".to_string();

        assert_eq!(
            handle_dashboard_key(&mut state, KeyCode::Char('p')),
            DashboardKeyOutcome::Routed
        );
        assert_eq!(state.page(), Page::Deploy);
        assert_eq!(
            state.feedback.scope,
            crate::tui::state::UiFeedbackScope::Apply
        );
        assert_eq!(
            state.feedback.message,
            "Overview 已跳到 Apply；当前来源是远端固定版本；默认 Apply 不会直接执行，必须交给完整高级路径。"
        );
        assert_eq!(
            state.feedback.next_step.as_deref(),
            Some("在 Apply 先看 handoff 预览；如需继续，切到 Advanced 执行 launch deploy wizard")
        );
    }

    #[test]
    fn dashboard_shortcut_p_ignores_stale_apply_workspace_when_reentering_apply() {
        let mut state = test_state("sudo-available");
        state.show_advanced = true;

        assert_eq!(
            handle_dashboard_key(&mut state, KeyCode::Char('p')),
            DashboardKeyOutcome::Routed
        );
        assert_eq!(state.page(), Page::Deploy);
        assert!(!state.show_advanced);
        assert_eq!(
            state.feedback.message,
            "Overview 已把你带到 Apply；当前组合可直接执行。"
        );
        assert_eq!(
            state.feedback.next_step.as_deref(),
            Some("在 Apply 查看预览，或按 a / x 直接运行")
        );
    }

    #[test]
    fn dashboard_shortcut_i_uses_specific_inspect_feedback_when_doctor_fails() {
        let mut state = test_state("sudo-available");
        state.overview_doctor = OverviewCheckState::Error {
            summary: "failed (missing nixos-rebuild)".to_string(),
            details: vec!["- deployment environment: missing nixos-rebuild".to_string()],
        };

        assert_eq!(
            handle_dashboard_key(&mut state, KeyCode::Char('i')),
            DashboardKeyOutcome::Routed
        );
        assert_eq!(state.page(), Page::Inspect);
        assert_eq!(
            state.feedback.scope,
            crate::tui::state::UiFeedbackScope::Inspect
        );
        assert_eq!(
            state.feedback.message,
            "Overview 推荐先进入 Inspect 处理 doctor（failed (missing nixos-rebuild)）。"
        );
        assert_eq!(
            state.feedback.next_step.as_deref(),
            Some("在 Inspect 先看 doctor 详情；如需仓库校验，再执行 flake check")
        );
        let inspect = state.inspect_model();
        assert_eq!(inspect.detail.action, ActionItem::FlakeCheck);
        assert_eq!(inspect.doctor, state.overview_doctor);
        assert_eq!(inspect.detail.label, "flake check");
    }

    #[test]
    fn shell_navigation_cycles_top_level_areas_and_preserves_edit_leaf() {
        let mut state = test_state("sudo-available");

        assert!(handle_shell_navigation_key(
            &mut state,
            KeyCode::Tab,
            KeyModifiers::NONE
        ));
        assert_eq!(state.top_level_page(), TopLevelPage::Edit);
        assert_eq!(state.page(), Page::Packages);

        state.open_edit_page(Page::Hosts);
        assert_eq!(state.page(), Page::Hosts);

        assert!(handle_shell_navigation_key(
            &mut state,
            KeyCode::Tab,
            KeyModifiers::NONE
        ));
        assert_eq!(state.top_level_page(), TopLevelPage::Apply);
        assert_eq!(state.page(), Page::Deploy);

        assert!(handle_shell_navigation_key(
            &mut state,
            KeyCode::BackTab,
            KeyModifiers::NONE
        ));
        assert_eq!(state.top_level_page(), TopLevelPage::Edit);
        assert_eq!(state.page(), Page::Hosts);

        assert!(handle_shell_navigation_key(
            &mut state,
            KeyCode::BackTab,
            KeyModifiers::NONE
        ));
        assert_eq!(state.top_level_page(), TopLevelPage::Overview);
        assert_eq!(state.page(), Page::Dashboard);
    }

    #[test]
    fn advanced_b_returns_to_apply_with_aligned_feedback() {
        let mut state = test_state("sudo-available");
        state.open_advanced();
        state.show_advanced = true;

        assert_eq!(
            handle_advanced_key(&mut state, KeyCode::Char('b')),
            AdvancedKeyOutcome::Routed
        );
        assert_eq!(state.page(), Page::Deploy);
        assert!(!state.show_advanced);
        assert_eq!(
            state.feedback.scope,
            crate::tui::state::UiFeedbackScope::Apply
        );
        assert_eq!(
            state.feedback.message,
            "已从 Advanced 返回 Apply；当前组合可直接执行。"
        );
        assert_eq!(
            state.feedback.next_step.as_deref(),
            Some("在 Apply 查看预览，或按 a / x 直接运行")
        );
    }

    fn test_state(privilege_mode: &str) -> AppState {
        let mut host_settings_by_name = BTreeMap::new();
        host_settings_by_name.insert(
            "demo".to_string(),
            HostManagedSettings {
                primary_user: "alice".to_string(),
                users: vec!["alice".to_string()],
                admin_users: vec!["alice".to_string()],
                ..HostManagedSettings::default()
            },
        );

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
            deploy_action: if privilege_mode == "rootless" {
                DeployAction::Build
            } else {
                DeployAction::Switch
            },
            flake_update: false,
            advanced_target_host: "demo".to_string(),
            advanced_deploy_task: DeployTask::DirectDeploy,
            advanced_deploy_source: DeploySource::CurrentRepo,
            advanced_deploy_source_ref: String::new(),
            advanced_deploy_action: if privilege_mode == "rootless" {
                DeployAction::Build
            } else {
                DeployAction::Switch
            },
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
            overview_repo_integrity: OverviewCheckState::Healthy {
                summary: "ok".to_string(),
                details: Vec::new(),
            },
            overview_doctor: OverviewCheckState::NotRun,
            feedback: UiFeedback::default(),
            status: String::new(),
        }
    }
}
