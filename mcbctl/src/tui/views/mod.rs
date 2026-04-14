mod actions;
mod dashboard;
mod deploy;
mod home;
mod hosts;
mod inspect;
mod packages;
mod users;

use crate::domain::tui::{HostsTextMode, Page, UsersTextMode};
use crate::tui::state::AppState;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Tabs, Wrap};

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
    let titles = Page::ALL
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
        Page::Deploy => deploy::render(frame, area, state),
        Page::Inspect => inspect::render(frame, area, state),
        Page::Users => {
            users::render(frame, area, state);
            if state.active_users_text_mode().is_some() {
                render_users_text_dialog(frame, area, state);
            }
        }
        Page::Hosts => {
            hosts::render(frame, area, state);
            if state.active_hosts_text_mode().is_some() {
                render_hosts_text_dialog(frame, area, state);
            }
        }
        Page::Packages => {
            packages::render(frame, area, state);
            if state.package_group_create_mode || state.package_group_rename_mode {
                render_package_group_dialog(frame, area, state);
            }
        }
        Page::Home => home::render(frame, area, state),
        Page::Actions => actions::render(frame, area, state),
    }
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

    let (title, body) = if state.package_group_rename_mode {
        (
            "Rename Group",
            format!(
                "重命名当前用户的一个软件组\n\n当前软件: {current_entry}\n原组名: {}\n输入: {raw}\n规范化预览: {normalized}\n\nEnter 确认  Esc 取消",
                state.package_group_rename_source
            ),
        )
    } else {
        (
            "New Group",
            format!(
                "为当前软件创建或指定一个组\n\n当前软件: {current_entry}\n输入: {raw}\n规范化预览: {normalized}\n\nEnter 确认  Esc 取消"
            ),
        )
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
    let footer = match state.page() {
        Page::Dashboard => {
            "Overview: Enter 打开推荐动作  a 直接执行当前 Apply  p 打开 Apply  i 打开 Inspect  r/d/R 刷新健康项  Tab/Shift-Tab 切页  q 退出"
        }
        Page::Deploy => {
            if state.show_advanced {
                "Apply: 左侧先看执行门槛和预览  j/k 选高级项  h/l 或 Enter 调整  J/K 选高级动作  X 执行高级动作  x 按当前 Apply 路径处理  Tab/Shift-Tab 切页  q 退出"
            } else {
                "Apply: 左侧先看执行门槛和预览  j/k 选高级项  h/l 或 Enter 调整  x 执行当前 Apply  Tab/Shift-Tab 切页  q 退出"
            }
        }
        Page::Inspect => {
            "Inspect: j/k 选命令  r 刷新 repo-integrity  d 刷新 doctor  R 刷新全部健康项  x 执行当前 inspect 命令  Tab/Shift-Tab 切页  q 退出"
        }
        Page::Users if state.active_users_text_mode().is_some() => {
            "Users 输入中: 直接键入  Backspace 删除  Enter 确认  Esc 取消"
        }
        Page::Hosts if state.active_hosts_text_mode().is_some() => {
            "Hosts 输入中: 直接键入  Backspace 删除  Enter 确认  Esc 取消"
        }
        Page::Packages if state.package_group_create_mode => {
            "Packages 新组输入中: 直接键入  Backspace 删除  Enter 确认  Esc 取消"
        }
        Page::Packages if state.package_group_rename_mode => {
            "Packages 组重命名输入中: 直接键入  Backspace 删除  Enter 确认  Esc 取消"
        }
        Page::Packages if state.package_search_mode => {
            "Packages 搜索输入中: 直接键入  Backspace 删除  Enter/Esc 结束"
        }
        Page::Packages => {
            "Packages: ←/→ 切用户  f 切换本地/搜索  r 刷新搜索  j/k 列表  [/]/h/l 分类  u/i 来源过滤  ,/. 组过滤  z 聚焦当前组  Z 清空组过滤  g/G 改条目组  m/M 整组移动  n 新建组  R 重命名组  / 搜索  Space 勾选  s 保存  q 退出"
        }
        Page::Users => "Users: ←/→ 切主机  j/k 字段  h/l 调整枚举  Enter 编辑列表  s 保存  q 退出",
        Page::Hosts => {
            "Hosts: ←/→ 切主机  j/k 字段  h/l 调整枚举  Enter 编辑文本/映射  s 保存  q 退出"
        }
        Page::Home => "Home: ←/→ 切用户  j/k 选项  h/l 或 Enter 调整  s 保存  q 退出",
        Page::Actions => {
            "Actions: j/k 选动作  Enter/Space/x 打开归宿页  当前页只做分组和跳转  Tab/Shift-Tab 切页  q 退出"
        }
    };
    let help = Paragraph::new(footer).block(Block::default().borders(Borders::ALL).title("Help"));
    frame.render_widget(Clear, area);
    frame.render_widget(help, area);
}
