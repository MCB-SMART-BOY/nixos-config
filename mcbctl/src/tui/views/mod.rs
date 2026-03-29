use crate::domain::tui::{HostsTextMode, PackageDataMode, Page, UsersTextMode};
use crate::tui::state::AppState;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs, Wrap};

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
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .select(state.active_page);
    frame.render_widget(tabs, area);
}

fn render_body(frame: &mut Frame, area: Rect, state: &AppState) {
    match state.page() {
        Page::Dashboard => render_dashboard(frame, area, state),
        Page::Deploy => render_deploy(frame, area, state),
        Page::Users => render_users(frame, area, state),
        Page::Hosts => render_hosts(frame, area, state),
        Page::Packages => render_packages(frame, area, state),
        Page::Home => render_home(frame, area, state),
        Page::Actions => render_actions(frame, area, state),
    }
}

fn render_dashboard(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(46), Constraint::Percentage(54)])
        .split(area);

    let left = Paragraph::new(format!(
        "仓库: {}\n/etc/nixos: {}\n检测 hostname: {}\n默认部署目标: {}\n当前用户: {}\n权限模式: {}\n可用 hosts: {}\n可用用户: {}",
        state.context.repo_root.display(),
        state.context.etc_root.display(),
        state.context.current_host,
        state.target_host,
        state.context.current_user,
        state.context.privilege_mode,
        state.context.hosts.join(", "),
        state.context.users.join(", ")
    ))
    .block(Block::default().borders(Borders::ALL).title("Context"))
    .wrap(Wrap { trim: false });
    frame.render_widget(left, chunks[0]);

    let right = Paragraph::new(
        "当前进度:\n\
         - Deploy 心智模型已经落地\n\
         - managed/ 机器写入边界已接通\n\
         - Packages 页默认走 nixpkgs 搜索，并按组写入 managed/packages/*.nix\n\
         - 本地 catalog 已降级为覆盖层 / 本地包元数据，不再充当主软件源\n\
         - Packages 页已经支持新建组、重命名组、整组移动、组过滤\n\n\
         - Home 页已经支持写入 managed/settings/desktop.nix（Noctalia / 桌面入口）\n\
         - Users 页现在写 users.nix，Hosts 页现在写 network/gpu/virtualization 分片\n\n\
         下一步:\n\
         - Packages 页继续往 channel / 搜索缓存方向扩展\n\
         - Deploy 执行流程继续从 mcb-deploy 向共享计划层收口\n\
         - Home 页继续把 session/mime 等结构化设置接入分片\n\
         - Users / Hosts 页继续细化分片级编辑与摘要\n\
         - Packages / Home 扩展更多 metadata 驱动字段",
    )
    .block(Block::default().borders(Borders::ALL).title("Roadmap"))
    .wrap(Wrap { trim: false });
    frame.render_widget(right, chunks[1]);
}

fn render_deploy(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(area);

    let rows = state
        .deploy_rows()
        .into_iter()
        .map(|(label, value)| ListItem::new(format!("{label:<14} {value}")))
        .collect::<Vec<_>>();

    let mut list_state = ListState::default();
    list_state.select(Some(state.deploy_focus));
    let list = List::new(rows)
        .block(Block::default().borders(Borders::ALL).title("Deploy Model"))
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    frame.render_stateful_widget(list, chunks[0], &mut list_state);

    let summary = Paragraph::new(state.deploy_summary().join("\n"))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Summary Preview"),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(summary, chunks[1]);
}

fn render_users(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(area);

    let rows = state
        .users_rows()
        .into_iter()
        .map(|(label, value)| ListItem::new(format!("{label:<14} {value}")))
        .collect::<Vec<_>>();

    let mut list_state = ListState::default();
    list_state.select(Some(state.users_focus));
    let list = List::new(rows)
        .block(Block::default().borders(Borders::ALL).title("Users Model"))
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    frame.render_stateful_widget(list, chunks[0], &mut list_state);

    let summary = Paragraph::new(state.users_summary_lines().join("\n"))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Users Summary"),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(summary, chunks[1]);

    if state.active_users_text_mode().is_some() {
        render_users_text_dialog(frame, area, state);
    }
}

fn render_hosts(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(44), Constraint::Percentage(56)])
        .split(area);

    let rows = state
        .hosts_rows()
        .into_iter()
        .map(|(label, value)| ListItem::new(format!("{label:<16} {value}")))
        .collect::<Vec<_>>();

    let mut list_state = ListState::default();
    list_state.select(Some(state.hosts_focus));
    let list = List::new(rows)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Host Override"),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    frame.render_stateful_widget(list, chunks[0], &mut list_state);

    let summary = Paragraph::new(state.hosts_summary_lines().join("\n"))
        .block(Block::default().borders(Borders::ALL).title("Host Summary"))
        .wrap(Wrap { trim: false });
    frame.render_widget(summary, chunks[1]);

    if state.active_hosts_text_mode().is_some() {
        render_hosts_text_dialog(frame, area, state);
    }
}

fn render_packages(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(28),
            Constraint::Percentage(39),
            Constraint::Percentage(33),
        ])
        .split(area);

    let left = Paragraph::new(state.package_summary_lines().join("\n"))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Package Context"),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(left, chunks[0]);

    let filtered = state.package_filtered_indices();
    if filtered.is_empty() {
        let empty_text = if state.current_package_mode() == PackageDataMode::Search {
            "当前搜索条件下没有结果。\n\n尝试：\n- 按 / 输入关键词\n- Enter 或 r 刷新 nixpkgs 搜索\n- 按 f 切回本地覆盖层"
        } else {
            "当前过滤条件下没有可选软件。\n\n尝试：\n- 切换分类\n- 清空搜索\n- 按 f 切到 nixpkgs 搜索"
        };
        let empty = Paragraph::new(empty_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Packages ({})", state.current_package_mode_label())),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(empty, chunks[1]);
    } else {
        let items = filtered
            .iter()
            .filter_map(|index| state.context.catalog_entries.get(*index))
            .map(|entry| {
                let selected = state
                    .current_package_user()
                    .and_then(|user| state.package_user_selections.get(user))
                    .is_some_and(|set| set.contains_key(&entry.id));
                let marker = if selected { "[x]" } else { "[ ]" };
                let group = if selected {
                    state.effective_selected_group(entry)
                } else {
                    entry.group_key().to_string()
                };
                ListItem::new(format!(
                    "{marker} {} ({}, -> {})",
                    entry.name,
                    entry.category,
                    state.package_group_display(&group)
                ))
            })
            .collect::<Vec<_>>();

        let mut list_state = ListState::default();
        list_state.select(Some(state.package_cursor));
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Packages ({})", state.current_package_mode_label())),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");
        frame.render_stateful_widget(list, chunks[1], &mut list_state);
    }

    let mut lines = Vec::new();
    if let Some(entry) = state.current_package_entry() {
        lines.push(format!("当前条目：{}", entry.name));
        lines.push(format!("id：{}", entry.id));
        lines.push(format!("分类：{}", entry.category));
        lines.push(format!("来源：{}", entry.source_label()));
        if let Some(group) = state.package_group_for_current_entry() {
            lines.push(format!("目标组：{}", state.package_group_display(&group)));
            if let Some(description) = state.package_group_description(&group) {
                lines.push(format!("组说明：{description}"));
            }
        }
        lines.push(format!("表达式：{}", entry.expr));
        if let Some(description) = &entry.description {
            lines.push(format!("说明：{description}"));
        }
        if !entry.platforms.is_empty() {
            lines.push(format!("平台：{}", entry.platforms.join(", ")));
        }
        if !entry.keywords.is_empty() {
            lines.push(format!("关键词：{}", entry.keywords.join(", ")));
        }
        if let Some(flag) = &entry.desktop_entry_flag {
            lines.push(format!("桌面入口 flag：{flag}"));
        }
        if let Some(group) = state.current_selected_group_name() {
            lines.push(format!(
                "当前组成员数：{}",
                state.current_selected_group_member_count()
            ));
            lines.push(format!("当前整组操作对象：{group}"));
        }
        lines.push(String::new());
    }

    let group_counts = state.package_groups_overview();
    if group_counts.is_empty() {
        lines.push("当前用户还没有可用软件组。".to_string());
    } else {
        lines.push("当前用户分组（> 过滤，* 当前条目）：".to_string());
        let current_group = state.package_group_for_current_entry();
        let filter_group = state.current_package_group_filter();
        for (group, count) in group_counts {
            let filter_marker = if filter_group == Some(group.as_str()) {
                ">"
            } else {
                " "
            };
            let current_marker = if current_group.as_deref() == Some(group.as_str()) {
                "*"
            } else {
                " "
            };
            lines.push(format!(
                "{filter_marker}{current_marker} {} ({count})",
                state.package_group_display(&group)
            ));
        }
        lines.push(String::new());
    }

    let selected_entries = state.package_selected_entries();
    if selected_entries.is_empty() {
        lines.push("当前用户尚未选中任何软件。".to_string());
    } else {
        lines.push("当前用户已选：".to_string());
        for entry in selected_entries {
            let group = state.effective_selected_group(entry);
            lines.push(format!(
                "- {} ({}, 组: {})",
                entry.name,
                entry.category,
                state.package_group_display(&group)
            ));
        }
    }
    lines.push(String::new());
    lines.push(format!("状态：{}", state.status));

    let right = Paragraph::new(lines.join("\n"))
        .block(Block::default().borders(Borders::ALL).title("Selection"))
        .wrap(Wrap { trim: false });
    frame.render_widget(right, chunks[2]);

    if state.package_group_create_mode || state.package_group_rename_mode {
        render_package_group_dialog(frame, area, state);
    }
}

fn render_home(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(area);

    let rows = state
        .home_rows()
        .into_iter()
        .map(|(label, value)| ListItem::new(format!("{label:<20} {value}")))
        .collect::<Vec<_>>();

    let mut list_state = ListState::default();
    list_state.select(Some(state.home_focus));
    let list = List::new(rows)
        .block(Block::default().borders(Borders::ALL).title(format!(
            "Home Settings ({})",
            state.current_home_user().unwrap_or("无可用用户")
        )))
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    frame.render_stateful_widget(list, chunks[0], &mut list_state);

    let summary = Paragraph::new(state.home_summary_lines().join("\n"))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Settings Preview"),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(summary, chunks[1]);
}

fn render_actions(frame: &mut Frame, area: Rect, state: &AppState) {
    render_placeholder(
        frame,
        area,
        "Actions",
        &[
            "这里会收口维护动作。",
            "计划纳入:",
            "  - flake check",
            "  - nix flake update",
            "  - update-upstream-apps",
            "  - sync to /etc/nixos",
            "  - rebuild current host",
        ],
        &state.status,
    );
}

fn render_placeholder(frame: &mut Frame, area: Rect, title: &str, lines: &[&str], status: &str) {
    let text = format!("{}\n\n状态: {}", lines.join("\n"), status);
    let widget = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: false });
    frame.render_widget(widget, area);
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
        Some(HostsTextMode::ProxyUrl) => ("Edit Proxy URL", "直接输入代理 URL，可留空"),
        Some(HostsTextMode::TunInterface) => ("Edit Tun Interface", "输入主 TUN 接口名，可留空"),
        Some(HostsTextMode::PerUserTunInterfaces) => (
            "Edit Per-user Tun Interfaces",
            "格式: user=iface, other=iface2",
        ),
        Some(HostsTextMode::PerUserTunDnsPorts) => {
            ("Edit Per-user DNS Ports", "格式: user=1053, other=2053")
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
        Page::Deploy => "Tab/Shift-Tab: 切页  j/k: 选字段  h/l 或 Enter: 调整  q: 退出",
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
        _ => "Tab/Shift-Tab: 切页  q: 退出",
    };
    let help = Paragraph::new(footer).block(Block::default().borders(Borders::ALL).title("Help"));
    frame.render_widget(Clear, area);
    frame.render_widget(help, area);
}
