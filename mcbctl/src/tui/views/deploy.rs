use self::advanced::{render_advanced_maintenance_page, render_advanced_wizard_page};
use self::apply::render_apply_page;
use crate::tui::state::{AppState, DeployPageModel};
use ratatui::Frame;
use ratatui::layout::Rect;

#[path = "deploy/advanced.rs"]
mod advanced;
#[path = "deploy/apply.rs"]
mod apply;
#[path = "deploy/chrome.rs"]
mod chrome;
#[path = "deploy/shared.rs"]
mod shared;
#[cfg(test)]
#[path = "deploy/test_support.rs"]
mod test_support;

pub(super) fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let page = state.deploy_page_model();
    match &page {
        DeployPageModel::Apply(model) => render_apply_page(frame, area, model.as_ref()),
        DeployPageModel::AdvancedMaintenance(model) => {
            render_advanced_maintenance_page(frame, area, model.as_ref())
        }
        DeployPageModel::AdvancedWizard(model) => {
            render_advanced_wizard_page(frame, area, model.as_ref())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::{render_view_text, test_app_state};
    use super::*;

    #[test]
    fn top_level_render_routes_default_state_to_apply_page() {
        let state = test_app_state();

        let text = render_view_text(120, 40, |frame| {
            render(frame, Rect::new(0, 0, 120, 40), &state)
        });

        assert!(text.contains("Execution Gate"));
        assert!(text.contains("Apply Preview"));
        assert!(text.contains("Apply Controls"));
        assert!(!text.contains("Advanced Actions"));
    }

    #[test]
    fn top_level_render_routes_apply_workspace_state_to_apply_page_with_workspace() {
        let mut state = test_app_state();
        state.show_advanced = true;

        let text = render_view_text(120, 40, |frame| {
            render(frame, Rect::new(0, 0, 120, 40), &state)
        });

        assert!(text.contains("Execution Gate"));
        assert!(text.contains("Advanced Actions"));
        assert!(text.contains("Advanced Detail"));
    }

    #[test]
    fn top_level_render_routes_advanced_state_to_maintenance_page() {
        let mut state = test_app_state();
        state.open_advanced();
        state.ensure_advanced_action_focus();

        let text = render_view_text(120, 40, |frame| {
            render(frame, Rect::new(0, 0, 120, 40), &state)
        });

        assert!(text.contains("Advanced Summary"));
        assert!(text.contains("Maintenance Preview"));
        assert!(text.contains("Repository Context"));
        assert!(text.contains("Maintenance Detail"));
    }

    #[test]
    fn top_level_render_routes_advanced_wizard_state_to_wizard_page() {
        let mut state = test_app_state();
        state.open_advanced();
        state.actions_focus = 6;

        let text = render_view_text(120, 40, |frame| {
            render(frame, Rect::new(0, 0, 120, 40), &state)
        });

        assert!(text.contains("Advanced Summary"));
        assert!(text.contains("Deploy Preview"));
        assert!(text.contains("Deploy Parameters"));
        assert!(text.contains("Deploy Wizard Detail"));
    }
}
