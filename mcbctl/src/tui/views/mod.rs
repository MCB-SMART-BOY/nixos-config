mod actions;
mod dashboard;
mod deploy;
mod home;
mod hosts;
mod inspect;
mod packages;
mod users;

use crate::domain::tui::{
    DeployTextMode, HostsTextMode, PackageTextMode, Page, TopLevelPage, UsersTextMode,
};
use crate::tui::state::{
    AppState, EditPageModel, EditRow, EditSummaryModel, PackagePageModel, PackageSelectionModel,
};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs, Wrap};

pub(super) struct EditListModel {
    pub(super) rows: Vec<EditRow>,
    pub(super) selected: usize,
}

pub(super) struct EditSummaryPaneModel<'a> {
    pub(super) summary: &'a EditSummaryModel,
}

pub(super) struct EditPageConfig {
    pub(super) left_percentage: u16,
    pub(super) list_title: String,
    pub(super) summary_title: &'static str,
    pub(super) label_width: usize,
}

pub(super) struct PackagePageConfig {
    pub(super) summary_percentage: u16,
    pub(super) list_percentage: u16,
    pub(super) summary_title: &'static str,
    pub(super) selection_title: &'static str,
}

pub fn render(frame: &mut Frame, state: &AppState) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(frame.area());

    render_tabs(frame, layout[0], state);
    render_body(frame, layout[1], state);
    render_footer(frame, layout[2], state);
}

fn render_tabs(frame: &mut Frame, area: Rect, state: &AppState) {
    let titles = TopLevelPage::ALL
        .iter()
        .map(|page| page.title())
        .collect::<Vec<_>>();
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("MCBCTL"))
        .highlight_style(ratatui::prelude::Style::default().fg(ratatui::prelude::Color::Cyan))
        .select(state.active_page);
    frame.render_widget(tabs, area);
}

fn render_body(frame: &mut Frame, area: Rect, state: &AppState) {
    match state.page() {
        Page::Dashboard => dashboard::render(frame, area, state),
        Page::Packages | Page::Home | Page::Users | Page::Hosts => {
            render_edit_workspace(frame, area, state)
        }
        Page::Deploy | Page::Advanced => {
            deploy::render(frame, area, state);
            if state.active_deploy_text_mode().is_some() {
                render_deploy_text_dialog(frame, area, state);
            }
        }
        Page::Inspect => inspect::render(frame, area, state),
        Page::Actions => deploy::render(frame, area, state),
    }
}

fn render_edit_workspace(frame: &mut Frame, area: Rect, state: &AppState) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(4),
            Constraint::Min(10),
        ])
        .split(area);

    let titles = Page::EDIT_ALL
        .iter()
        .enumerate()
        .map(|(index, page)| format!("{} {}", index + 1, page.title()))
        .collect::<Vec<_>>();
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Edit Pages"))
        .highlight_style(ratatui::prelude::Style::default().fg(ratatui::prelude::Color::Cyan))
        .select(state.active_edit_page);
    frame.render_widget(tabs, layout[0]);

    let summary = Paragraph::new(state.edit_workspace_summary_model().lines().join("\n"))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Edit Workspace"),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(summary, layout[1]);

    match state.edit_page() {
        Page::Users => {
            users::render(frame, layout[2], state);
            if state.active_users_text_mode().is_some() {
                render_users_text_dialog(frame, layout[2], state);
            }
        }
        Page::Hosts => {
            hosts::render(frame, layout[2], state);
            if state.active_hosts_text_mode().is_some() {
                render_hosts_text_dialog(frame, layout[2], state);
            }
        }
        Page::Packages => {
            packages::render(frame, layout[2], state);
            if matches!(
                state.active_package_text_mode(),
                Some(PackageTextMode::CreateGroup | PackageTextMode::RenameGroup)
            ) {
                render_package_group_dialog(frame, layout[2], state);
            }
        }
        Page::Home => home::render(frame, layout[2], state),
        Page::Dashboard | Page::Deploy | Page::Advanced | Page::Inspect | Page::Actions => {}
    }
}

pub(super) fn render_edit_list(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    rows: Vec<EditRow>,
    selected: usize,
    label_width: usize,
) {
    let rows = rows
        .into_iter()
        .map(|row| ListItem::new(format_edit_row(&row, label_width)))
        .collect::<Vec<_>>();

    let mut list_state = ListState::default();
    list_state.select(Some(selected));
    let list = List::new(rows)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    frame.render_stateful_widget(list, area, &mut list_state);
}

pub(super) fn render_edit_summary(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    summary_model: &EditSummaryModel,
) {
    let summary = Paragraph::new(format_edit_summary(summary_model))
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: false });
    frame.render_widget(summary, area);
}

pub(super) fn render_edit_page(
    frame: &mut Frame,
    area: Rect,
    config: EditPageConfig,
    list: EditListModel,
    summary: EditSummaryPaneModel<'_>,
) {
    let chunks = edit_page_chunks(area, config.left_percentage);
    render_edit_list(
        frame,
        chunks[0],
        &config.list_title,
        list.rows,
        list.selected,
        config.label_width,
    );
    render_edit_summary(frame, chunks[1], config.summary_title, summary.summary);
}

pub(super) fn render_edit_page_with_model(
    frame: &mut Frame,
    area: Rect,
    config: EditPageConfig,
    page_model: &EditPageModel,
) {
    render_edit_page(
        frame,
        area,
        config,
        EditListModel {
            rows: page_model.rows.clone(),
            selected: page_model.selected,
        },
        EditSummaryPaneModel {
            summary: &page_model.summary,
        },
    );
}

pub(super) fn render_package_page(
    frame: &mut Frame,
    area: Rect,
    config: PackagePageConfig,
    page: &PackagePageModel,
) {
    let chunks = package_page_chunks(area, config.summary_percentage, config.list_percentage);
    render_edit_summary(frame, chunks[0], config.summary_title, &page.summary);
    render_package_list(frame, chunks[1], &page.list);
    render_package_selection(frame, chunks[2], config.selection_title, &page.selection);
}

fn format_edit_row(row: &EditRow, label_width: usize) -> String {
    format!("{:<width$} {}", row.label, row.value, width = label_width)
}

fn format_edit_summary(model: &EditSummaryModel) -> String {
    model.lines().join("\n")
}

fn edit_page_chunks(area: Rect, left_percentage: u16) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(left_percentage),
            Constraint::Percentage(100 - left_percentage),
        ])
        .split(area)
        .to_vec()
}

fn package_page_chunks(area: Rect, summary_percentage: u16, list_percentage: u16) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(summary_percentage),
            Constraint::Percentage(list_percentage),
            Constraint::Percentage(100 - summary_percentage - list_percentage),
        ])
        .split(area)
        .to_vec()
}

fn render_package_list(
    frame: &mut Frame,
    area: Rect,
    list_model: &crate::tui::state::PackageListModel,
) {
    if let Some(empty_text) = &list_model.empty_text {
        let empty = Paragraph::new(empty_text.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(list_model.title.as_str()),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(empty, area);
        return;
    }

    let items = list_model
        .items
        .iter()
        .map(|item| item.display_line())
        .map(ListItem::new)
        .collect::<Vec<_>>();

    let mut list_state = ListState::default();
    list_state.select(list_model.selected_index);
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(list_model.title.as_str()),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_package_selection(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    selection: &PackageSelectionModel,
) {
    let pane = Paragraph::new(selection.lines().join("\n"))
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: false });
    frame.render_widget(pane, area);
}

fn render_package_group_dialog(frame: &mut Frame, area: Rect, state: &AppState) {
    let popup = centered_rect(62, 30, area);
    let current_entry = state
        .current_package_entry()
        .map(|entry| entry.name.as_str())
        .unwrap_or("无");
    let raw = if state.package_group_input.is_empty() {
        "<空>"
    } else {
        &state.package_group_input
    };
    let normalized = {
        let preview = state.package_group_input_preview();
        if preview.is_empty() {
            "<空>".to_string()
        } else {
            preview
        }
    };

    let (title, body) = match state.active_package_text_mode() {
        Some(PackageTextMode::RenameGroup) => (
            "Rename Group",
            format!(
                "重命名当前用户的一个软件组\n\n当前软件: {current_entry}\n原组名: {}\n输入: {raw}\n规范化预览: {normalized}\n\nEnter 确认  Esc 取消",
                state.package_group_rename_source
            ),
        ),
        Some(PackageTextMode::CreateGroup) => (
            "New Group",
            format!(
                "为当前软件创建或指定一个组\n\n当前软件: {current_entry}\n输入: {raw}\n规范化预览: {normalized}\n\nEnter 确认  Esc 取消"
            ),
        ),
        Some(PackageTextMode::Search) | None => return,
    };

    let widget = Paragraph::new(body)
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: false });
    frame.render_widget(Clear, popup);
    frame.render_widget(widget, popup);
}

fn render_users_text_dialog(frame: &mut Frame, area: Rect, state: &AppState) {
    let popup = centered_rect(62, 30, area);
    let (title, hint) = match state.active_users_text_mode() {
        Some(UsersTextMode::ManagedUsers) => ("Edit Managed Users", "使用逗号分隔用户名"),
        Some(UsersTextMode::AdminUsers) => ("Edit Admin Users", "使用逗号分隔用户名"),
        None => return,
    };

    let raw = if state.host_text_input.is_empty() {
        "<空>"
    } else {
        &state.host_text_input
    };
    let body = format!(
        "{hint}\n\n当前主机: {}\n输入: {raw}\n\nEnter 确认  Esc 取消",
        state.target_host
    );
    let widget = Paragraph::new(body)
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: false });
    frame.render_widget(Clear, popup);
    frame.render_widget(widget, popup);
}

fn render_deploy_text_dialog(frame: &mut Frame, area: Rect, state: &AppState) {
    let popup = centered_rect(62, 28, area);
    let (title, hint, path_label, source_label) = match state.active_deploy_text_mode() {
        Some(DeployTextMode::ApplyRemotePinnedRef) => (
            "Edit Apply Remote Pinned Ref",
            "输入远端固定版本 ref / tag / commit，例如 v6.0.0 或 a1b2c3d",
            "Apply",
            state.apply_deploy_source_label(),
        ),
        Some(DeployTextMode::AdvancedWizardRemotePinnedRef) => (
            "Edit Advanced Wizard Remote Pinned Ref",
            "输入远端固定版本 ref / tag / commit，例如 v6.0.0 或 a1b2c3d",
            "Advanced Wizard",
            state.advanced_deploy_source_label(),
        ),
        None => return,
    };

    let raw = if state.host_text_input.is_empty() {
        "<空>"
    } else {
        &state.host_text_input
    };
    let body = format!(
        "{hint}\n\n当前路径: {path_label}\n当前来源: {source_label}\n输入: {raw}\n\nEnter 确认  Esc 取消",
    );
    let widget = Paragraph::new(body)
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: false });
    frame.render_widget(Clear, popup);
    frame.render_widget(widget, popup);
}

fn render_hosts_text_dialog(frame: &mut Frame, area: Rect, state: &AppState) {
    let popup = centered_rect(66, 34, area);
    let (title, hint) = match state.active_hosts_text_mode() {
        Some(HostsTextMode::CustomSubstituters) => (
            "Edit Custom Substituters",
            "使用逗号分隔，例如 https://cache.example.org",
        ),
        Some(HostsTextMode::CustomTrustedPublicKeys) => (
            "Edit Custom Trusted Keys",
            "使用逗号分隔，每项为 trusted-public-keys 条目",
        ),
        Some(HostsTextMode::ProxyUrl) => ("Edit Proxy URL", "直接输入代理 URL，可留空"),
        Some(HostsTextMode::TunInterface) => ("Edit Tun Interface", "输入主 TUN 接口名，可留空"),
        Some(HostsTextMode::TunInterfaces) => (
            "Edit Extra Tun Interfaces",
            "使用逗号分隔，例如 Meta, Mihomo, clash0",
        ),
        Some(HostsTextMode::ProxyDnsAddr) => ("Edit Proxy DNS Address", "例如 127.0.0.1"),
        Some(HostsTextMode::ProxyDnsPort) => ("Edit Proxy DNS Port", "输入 1-65535"),
        Some(HostsTextMode::PerUserTunInterfaces) => (
            "Edit Per-user Tun Interfaces",
            "格式: user=iface, other=iface2",
        ),
        Some(HostsTextMode::PerUserTunDnsPorts) => {
            ("Edit Per-user DNS Ports", "格式: user=1053, other=2053")
        }
        Some(HostsTextMode::PerUserTunTableBase) => {
            ("Edit Per-user Table Base", "输入正整数，例如 1000")
        }
        Some(HostsTextMode::PerUserTunPriorityBase) => {
            ("Edit Per-user Priority Base", "输入正整数，例如 10000")
        }
        Some(HostsTextMode::IntelBusId) => ("Edit Intel Bus ID", "例如 PCI:0:2:0，可留空"),
        Some(HostsTextMode::AmdBusId) => ("Edit AMD Bus ID", "例如 PCI:4:0:0，可留空"),
        Some(HostsTextMode::NvidiaBusId) => ("Edit NVIDIA Bus ID", "例如 PCI:1:0:0，可留空"),
        Some(HostsTextMode::SpecialisationModes) => (
            "Edit GPU Specialisation Modes",
            "使用逗号分隔：igpu, hybrid, dgpu",
        ),
        None => return,
    };

    let raw = if state.host_text_input.is_empty() {
        "<空>"
    } else {
        &state.host_text_input
    };
    let body = format!(
        "{hint}\n\n当前主机: {}\n输入: {raw}\n\nEnter 确认  Esc 取消",
        state.target_host
    );
    let widget = Paragraph::new(body)
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: false });
    frame.render_widget(Clear, popup);
    frame.render_widget(widget, popup);
}

fn centered_rect(width_percent: u16, height_percent: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - height_percent) / 2),
            Constraint::Percentage(height_percent),
            Constraint::Percentage((100 - height_percent) / 2),
        ])
        .split(area);
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width_percent) / 2),
            Constraint::Percentage(width_percent),
            Constraint::Percentage((100 - width_percent) / 2),
        ])
        .split(vertical[1]);
    horizontal[1]
}

fn render_footer(frame: &mut Frame, area: Rect, state: &AppState) {
    let help = Paragraph::new(footer_text(state))
        .block(Block::default().borders(Borders::ALL).title("Help"));
    frame.render_widget(Clear, area);
    frame.render_widget(help, area);
}

fn footer_text(state: &AppState) -> &'static str {
    match state.top_level_page() {
        TopLevelPage::Overview => {
            "Overview: Enter 打开推荐动作  a 直接执行当前 Apply  p 打开 Apply  i 打开 Inspect  r/d/R 刷新健康项  Tab/Shift-Tab 切区域  q 退出"
        }
        TopLevelPage::Edit => match state.edit_page() {
            Page::Users if state.active_users_text_mode().is_some() => {
                "Users 输入中: 直接键入  Backspace 删除  Enter 确认  Esc 取消"
            }
            Page::Hosts if state.active_hosts_text_mode().is_some() => {
                "Hosts 输入中: 直接键入  Backspace 删除  Enter 确认  Esc 取消"
            }
            Page::Packages => match state.active_package_text_mode() {
                Some(PackageTextMode::CreateGroup) => {
                    "Packages 新组输入中: 直接键入  Backspace 删除  Enter 确认  Esc 取消"
                }
                Some(PackageTextMode::RenameGroup) => {
                    "Packages 组重命名输入中: 直接键入  Backspace 删除  Enter 确认  Esc 取消"
                }
                Some(PackageTextMode::Search) => {
                    "Packages 搜索输入中: 直接键入  Backspace 删除  Enter/Esc 结束"
                }
                None => {
                    "Edit/Packages: 1/2/3/4 切子页  ←/→ 切用户  f 切换本地/搜索  r 刷新搜索  j/k 列表  [/]/h/l 分类  u/i 来源过滤  ,/. 组过滤  z 聚焦当前组  Z 清空组过滤  g/G 改条目组  m/M 整组移动  n 新建组  R 重命名组  / 搜索  Space 勾选  s 保存  q 退出"
                }
            },
            Page::Home => {
                "Edit/Home: 1/2/3/4 切子页  ←/→ 切用户  j/k 选项  h/l 或 Enter 调整  s 保存  q 退出"
            }
            Page::Users => {
                "Edit/Users: 1/2/3/4 切子页  ←/→ 切主机  j/k 字段  h/l 调整枚举  Enter 编辑列表  s 保存  q 退出"
            }
            Page::Hosts => {
                "Edit/Hosts: 1/2/3/4 切子页  ←/→ 切主机  j/k 字段  h/l 调整枚举  Enter 编辑文本/映射  s 保存  q 退出"
            }
            Page::Dashboard | Page::Deploy | Page::Advanced | Page::Inspect | Page::Actions => {
                "Edit: 1/2/3/4 切子页  Tab/Shift-Tab 切区域  q 退出"
            }
        },
        TopLevelPage::Apply => {
            if state.active_deploy_text_mode() == Some(DeployTextMode::ApplyRemotePinnedRef) {
                "Apply ref 输入中: 直接键入  Backspace 删除  Enter 确认  Esc 取消"
            } else {
                "Apply: 左侧先看执行门槛和预览  j/k 选 Apply 项  h/l 或 Enter 调整  x 执行当前 Apply  Tab/Shift-Tab 切区域  q 退出"
            }
        }
        TopLevelPage::Advanced => {
            if state.active_deploy_text_mode()
                == Some(DeployTextMode::AdvancedWizardRemotePinnedRef)
            {
                "Advanced Wizard ref 输入中: 直接键入  Backspace 删除  Enter 确认  Esc 取消"
            } else if state.advanced_action_uses_deploy_parameters() {
                "Advanced: 当前是 deploy 向导路径  j/k 选 deploy 参数  h/l 或 Enter 调整  J/K 选高级动作  x/X 执行高级动作  b 返回 Apply  Tab/Shift-Tab 切区域  q 退出"
            } else {
                "Advanced: 当前是仓库维护路径  j/k 或 J/K 切高级动作  x/X 执行高级动作  b 返回 Apply  Tab/Shift-Tab 切区域  q 退出"
            }
        }
        TopLevelPage::Inspect => {
            "Inspect: j/k 选命令  r 刷新 repo-integrity  d 刷新 doctor  R 刷新全部健康项  x 执行当前 inspect 命令  Tab/Shift-Tab 切区域  q 退出"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::tui::{
        CatalogEntry, DeployAction, DeploySource, DeployTask, GroupMeta, HomeOptionMeta,
        HostManagedSettings, PackageDataMode,
    };
    use crate::tui::state::{AppContext, OverviewCheckState, UiFeedback};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::buffer::Buffer;
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::PathBuf;

    #[test]
    fn format_edit_row_aligns_label_and_value() {
        let row = EditRow {
            label: "GPU 模式".to_string(),
            value: "igpu".to_string(),
        };

        assert_eq!(format_edit_row(&row, 10), "GPU 模式     igpu");
    }

    #[test]
    fn format_edit_summary_keeps_focus_status_and_notes_order() {
        let summary = EditSummaryModel {
            header_lines: vec!["当前用户：alice".to_string()],
            focused_row: Some(EditRow {
                label: "GPU 模式".to_string(),
                value: "igpu".to_string(),
            }),
            field_lines: vec!["GPU 模式：igpu".to_string()],
            detail: crate::tui::state::EditDetailModel {
                status: "状态：已保存".to_string(),
                validation: Some(crate::tui::state::EditCheckModel {
                    summary: "校验：通过".to_string(),
                    details: Vec::new(),
                }),
                managed_guard: crate::tui::state::EditCheckModel {
                    summary: "受管保护：通过".to_string(),
                    details: Vec::new(),
                },
                notes: vec!["说明：demo".to_string()],
            },
        };

        assert_eq!(
            format_edit_summary(&summary),
            [
                "当前用户：alice",
                "当前聚焦：GPU 模式 = igpu",
                "GPU 模式：igpu",
                "状态：已保存",
                "校验：通过",
                "受管保护：通过",
                "说明：demo",
            ]
            .join("\n")
        );
    }

    #[test]
    fn edit_page_chunks_respect_requested_percentages() {
        let area = Rect::new(0, 0, 100, 20);

        let home_like = edit_page_chunks(area, 42);
        let hosts_like = edit_page_chunks(area, 44);

        assert_eq!(home_like[0].width, 42);
        assert_eq!(home_like[1].width, 58);
        assert_eq!(hosts_like[0].width, 44);
        assert_eq!(hosts_like[1].width, 56);
    }

    #[test]
    fn edit_workspace_summary_lines_show_current_target_and_dirty_sections() {
        let mut state = test_state();
        state.open_edit_page(Page::Packages);
        state.package_dirty_users.insert("alice".to_string());
        state.host_dirty_runtime_hosts.insert("demo".to_string());

        let lines = state.edit_workspace_summary_model().lines();

        assert!(lines[0].contains("当前子页：Packages"));
        assert!(lines[0].contains("目标：user alice"));
        assert!(lines[1].contains("Packages(alice)"));
        assert!(lines[1].contains("Home(clean)"));
        assert!(lines[1].contains("Users(clean)"));
        assert!(lines[1].contains("Hosts(demo)"));
        assert!(lines[2].contains("当前页 Packages[user alice] 还有未保存修改"));
    }

    #[test]
    fn edit_workspace_summary_lines_show_na_for_missing_user_targets() {
        let mut state = test_state();
        state.context.users.clear();
        state.open_edit_page(Page::Home);

        let lines = state.edit_workspace_summary_model().lines();

        assert!(lines[0].contains("目标：无可用用户"));
        assert!(lines[1].contains("Packages(n/a)"));
        assert!(lines[1].contains("Home(n/a)"));
        assert!(lines[2].contains("没有可用目标"));
    }

    #[test]
    fn edit_workspace_summary_lines_recommend_switching_to_first_dirty_page() {
        let mut state = test_state();
        state.open_edit_page(Page::Home);
        state.host_dirty_user_hosts.insert("demo".to_string());

        let lines = state.edit_workspace_summary_model().lines();

        assert!(lines[2].contains("先切到 Users[demo] 保存未保存修改"));
    }

    #[test]
    fn footer_text_overview_mentions_primary_actions_and_shell_navigation() {
        let mut state = test_state();
        state.open_overview();

        let footer = footer_text(&state);

        assert!(footer.contains("Enter 打开推荐动作"));
        assert!(footer.contains("a 直接执行当前 Apply"));
        assert!(footer.contains("Tab/Shift-Tab 切区域"));
    }

    #[test]
    fn footer_text_edit_tracks_active_leaf_and_text_mode() {
        let mut state = test_state();
        state.open_edit_page(Page::Users);
        state.users_text_mode = Some(UsersTextMode::ManagedUsers);

        assert_eq!(
            footer_text(&state),
            "Users 输入中: 直接键入  Backspace 删除  Enter 确认  Esc 取消"
        );

        state.users_text_mode = None;
        assert_eq!(
            footer_text(&state),
            "Edit/Users: 1/2/3/4 切子页  ←/→ 切主机  j/k 字段  h/l 调整枚举  Enter 编辑列表  s 保存  q 退出"
        );

        state.open_edit_page(Page::Packages);
        state.package_group_create_mode = true;
        assert_eq!(
            footer_text(&state),
            "Packages 新组输入中: 直接键入  Backspace 删除  Enter 确认  Esc 取消"
        );

        state.package_group_create_mode = false;
        state.package_group_rename_mode = true;
        assert_eq!(
            footer_text(&state),
            "Packages 组重命名输入中: 直接键入  Backspace 删除  Enter 确认  Esc 取消"
        );

        state.package_group_rename_mode = false;
        state.package_search_mode = true;
        assert_eq!(
            footer_text(&state),
            "Packages 搜索输入中: 直接键入  Backspace 删除  Enter/Esc 结束"
        );
    }

    #[test]
    fn footer_text_apply_switches_to_ref_input_when_editing_pinned_ref() {
        let mut state = test_state();
        state.open_apply();
        state.deploy_source = DeploySource::RemotePinned;
        state.open_apply_text_edit();

        assert_eq!(
            footer_text(&state),
            "Apply ref 输入中: 直接键入  Backspace 删除  Enter 确认  Esc 取消"
        );
    }

    #[test]
    fn footer_text_advanced_switches_between_maintenance_and_wizard_paths() {
        let mut state = test_state();
        state.open_advanced();

        assert_eq!(
            footer_text(&state),
            "Advanced: 当前是仓库维护路径  j/k 或 J/K 切高级动作  x/X 执行高级动作  b 返回 Apply  Tab/Shift-Tab 切区域  q 退出"
        );

        state.advanced_deploy_source = DeploySource::RemoteHead;
        state.actions_focus = crate::domain::tui::ActionItem::ALL
            .iter()
            .position(|action| *action == crate::domain::tui::ActionItem::LaunchDeployWizard)
            .expect("launch deploy wizard should exist");

        assert_eq!(
            footer_text(&state),
            "Advanced: 当前是 deploy 向导路径  j/k 选 deploy 参数  h/l 或 Enter 调整  J/K 选高级动作  x/X 执行高级动作  b 返回 Apply  Tab/Shift-Tab 切区域  q 退出"
        );
    }

    #[test]
    fn render_edit_workspace_keeps_shared_shell_across_all_edit_pages() {
        let expectations = [
            (Page::Packages, "Package Context"),
            (Page::Home, "Home Settings (alice)"),
            (Page::Users, "Users Model"),
            (Page::Hosts, "Host Override"),
        ];

        for (page, page_title) in expectations {
            let mut state = test_state();
            state.open_edit_page(page);

            let text = render_view_text(140, 40, |frame| {
                render_edit_workspace(frame, Rect::new(0, 0, 140, 40), &state)
            });

            assert!(
                text.contains("Edit Pages"),
                "workspace shell should show Edit Pages for {:?}",
                page
            );
            assert!(
                text.contains("Edit Workspace"),
                "workspace shell should show Edit Workspace for {:?}",
                page
            );
            assert!(
                text.contains(page_title),
                "workspace should render page-specific title {page_title} for {:?}",
                page
            );
        }
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
                catalog_entries: Vec::<CatalogEntry>::new(),
                catalog_groups: BTreeMap::<String, GroupMeta>::new(),
                catalog_home_options: Vec::<HomeOptionMeta>::new(),
                catalog_categories: Vec::new(),
                catalog_sources: Vec::new(),
            },
            active_page: TopLevelPage::ALL
                .iter()
                .position(|page| *page == TopLevelPage::Edit)
                .expect("edit page index"),
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
            host_settings_by_name: BTreeMap::<String, HostManagedSettings>::new(),
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

    fn render_view_text(
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
}
