use super::state::AppState;
use super::views;
use crate::domain::tui::{PackageTextMode, Page};
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
            KeyCode::Tab if key.modifiers.is_empty() => state.next_page(),
            KeyCode::BackTab if key.modifiers.is_empty() => state.previous_page(),
            _ => handle_page_key(state, key.code, key.modifiers)?,
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

fn handle_page_key(state: &mut AppState, code: KeyCode, modifiers: KeyModifiers) -> Result<()> {
    if modifiers.contains(KeyModifiers::CONTROL) {
        return Ok(());
    }

    match state.page() {
        Page::Deploy => match code {
            KeyCode::Down | KeyCode::Char('j') => state.next_deploy_field(),
            KeyCode::Up | KeyCode::Char('k') => state.previous_deploy_field(),
            KeyCode::Left | KeyCode::Char('h') => state.adjust_deploy_field(-1),
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter | KeyCode::Char(' ') => {
                state.adjust_deploy_field(1)
            }
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
        _ => {}
    }
    Ok(())
}
