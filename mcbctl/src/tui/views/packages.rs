use crate::domain::tui::PackageDataMode;
use crate::tui::state::AppState;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};

pub(super) fn render(frame: &mut Frame, area: Rect, state: &AppState) {
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
}
