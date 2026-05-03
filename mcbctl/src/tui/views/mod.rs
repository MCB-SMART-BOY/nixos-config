mod dashboard;
mod deploy;
mod health;
mod home;
mod hosts;
mod inspect;
mod packages;
mod summary;
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

const EDIT_STACK_WIDTH: u16 = 96;
const EDIT_COMPACT_SUMMARY_WIDTH: u16 = 56;
const EDIT_COMPACT_SUMMARY_HEIGHT: u16 = 12;
const PACKAGE_STACK_WIDTH: u16 = 116;
const PACKAGE_FULL_STACK_WIDTH: u16 = 96;
const PACKAGE_LIST_COMPACT_WIDTH: u16 = 60;
const PACKAGE_LIST_TIGHT_WIDTH: u16 = 38;
const PACKAGE_SELECTION_COMPACT_WIDTH: u16 = 56;
const PACKAGE_SELECTION_TIGHT_WIDTH: u16 = 38;
const PACKAGE_SELECTION_COMPACT_HEIGHT: u16 = 14;
const PACKAGE_SELECTION_TIGHT_HEIGHT: u16 = 9;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PackageListDensity {
    Standard,
    Compact,
    Tight,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PackageSelectionDensity {
    Standard,
    Compact,
    Tight,
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
    if state.help_overlay_visible() {
        render_help_overlay(frame, layout[1], state);
    }
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

    let titles = edit_page_tabs(state);
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
        Page::Dashboard | Page::Deploy | Page::Advanced | Page::Inspect => {}
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
    let summary = Paragraph::new(format_edit_summary_for_area(summary_model, area))
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
    render_package_summary(frame, chunks[0], config.summary_title, &page.summary);
    render_package_list(frame, chunks[1], &page.list);
    render_package_selection(frame, chunks[2], config.selection_title, &page.selection);
}

fn format_edit_row(row: &EditRow, label_width: usize) -> String {
    format!("{:<width$} {}", row.label, row.value, width = label_width)
}

fn format_edit_summary(model: &EditSummaryModel) -> String {
    model.lines().join("\n")
}

fn format_edit_summary_for_area(model: &EditSummaryModel, area: Rect) -> String {
    if area.width > EDIT_COMPACT_SUMMARY_WIDTH && area.height > EDIT_COMPACT_SUMMARY_HEIGHT {
        return format_edit_summary(model);
    }

    compact_edit_summary_lines(model, area.width <= 40).join("\n")
}

fn compact_edit_summary_lines(model: &EditSummaryModel, tight: bool) -> Vec<String> {
    let mut lines = model
        .header_lines
        .iter()
        .map(|line| compact_edit_header_line(line))
        .collect::<Vec<_>>();

    if let Some(row) = &model.focused_row {
        lines.push(format!(
            "聚焦：{}={}",
            compact_edit_label(&row.label),
            row.value
        ));
    } else {
        lines.push("聚焦：无".to_string());
    }

    lines.extend(
        model
            .field_lines
            .iter()
            .map(|line| compact_edit_field_line(line)),
    );
    lines.push(compact_edit_status_line(&model.detail.status));
    if let Some(action_summary) = &model.detail.action_summary {
        lines.push(format!(
            "最近结果：{}",
            compact_edit_feedback_text(&action_summary.latest_result, tight)
        ));
        lines.push(format!(
            "下一步：{}",
            compact_edit_feedback_text(&action_summary.next_step, tight)
        ));
    }

    if let Some(validation) = &model.detail.validation {
        lines.push(compact_edit_check_summary(&validation.summary));
        lines.extend(validation.details.iter().cloned());
    }

    lines.push(compact_edit_check_summary(
        &model.detail.managed_guard.summary,
    ));
    lines.extend(model.detail.managed_guard.details.iter().cloned());

    lines.extend(
        model
            .detail
            .notes
            .iter()
            .filter_map(|line| compact_edit_note_line(line, tight)),
    );

    lines
}

fn compact_edit_header_line(line: &str) -> String {
    if let Some(rest) = line.strip_prefix("当前用户：") {
        return format!("用户：{rest}");
    }
    if let Some(rest) = line.strip_prefix("当前主机：") {
        return format!("主机：{rest}");
    }
    if let Some(rest) = line.strip_prefix("目标文件：") {
        return format!("目标：{}", compact_path_tail(rest, 4));
    }
    line.to_string()
}

fn compact_edit_field_line(line: &str) -> String {
    if let Some((label, value)) = line.split_once('：') {
        return format!("{}：{}", compact_edit_label(label), value);
    }
    line.to_string()
}

fn compact_edit_status_line(line: &str) -> String {
    if line.contains("没有未保存") {
        return "状态：已保存".to_string();
    }
    if line.contains("有未保存") {
        return "状态：未保存".to_string();
    }
    line.to_string()
}

fn compact_edit_feedback_text(text: &str, tight: bool) -> String {
    if text == "继续调整当前字段，完成后按 s 保存。" {
        return "继续调整，按 s 保存".to_string();
    }
    if text == "复查 Users Summary，确认后按 s 保存。"
        || text == "复查 Hosts Summary，确认后按 s 保存。"
    {
        return "复查摘要，按 s 保存".to_string();
    }
    if text == "继续编辑 Home，或切到 Apply / Overview 复查。"
        || text == "继续编辑 Users，或切到 Apply / Overview 复查。"
        || text == "继续编辑 Hosts，或切到 Apply / Overview 复查。"
    {
        return "继续编辑，或去 Apply/Overview".to_string();
    }
    if tight && text.starts_with("Home 已更新用户 ") {
        return "Home 已更新字段".to_string();
    }
    if tight && text.starts_with("Users 已") {
        return "Users 已更新".to_string();
    }
    if tight && text.starts_with("Hosts 已") {
        return "Hosts 已更新".to_string();
    }
    text.trim_end_matches('。').to_string()
}

fn compact_edit_check_summary(line: &str) -> String {
    line.to_string()
}

fn compact_edit_note_line(line: &str, tight: bool) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.starts_with("Noctalia：当前用户由 ") {
        return Some("Noctalia：自定义布局接管".to_string());
    }
    if trimmed == "当前阶段已接入的结构化设置：" {
        return Some("已接入：".to_string());
    }
    if trimmed.starts_with("这些内容只会写入 managed/settings/desktop.nix") {
        return Some("写回：managed/settings/desktop.nix".to_string());
    }
    if let Some(item) = trimmed.strip_prefix("- ") {
        if let Some((label, _)) = item.split_once('：') {
            return Some(format!("- {}", compact_edit_label(label)));
        }
        return Some(trimmed.to_string());
    }
    if tight && trimmed.starts_with("说明：") {
        return None;
    }
    Some(trimmed.to_string())
}

fn compact_edit_label(label: &str) -> &str {
    match label {
        "当前条目" => "条目",
        "目标组" => "组",
        "工作流" => "流程",
        "桌面入口 flag" => "桌面flag",
        "当前组成员数" => "组成员",
        "当前整组操作对象" => "整组对象",
        other => other,
    }
}

fn compact_path_tail(path: &str, segments: usize) -> String {
    let normalized = path.replace('\\', "/");
    let parts = normalized
        .split('/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.len() <= segments {
        return normalized;
    }
    format!(".../{}", parts[parts.len() - segments..].join("/"))
}

fn edit_page_tabs(state: &AppState) -> Vec<String> {
    Page::EDIT_ALL
        .iter()
        .enumerate()
        .map(|(index, page)| {
            let dirty = if state.edit_page_is_dirty(*page) {
                "*"
            } else {
                ""
            };
            format!("{} {}{}", index + 1, page.title(), dirty)
        })
        .collect()
}

fn edit_page_chunks(area: Rect, left_percentage: u16) -> Vec<Rect> {
    if area.width < EDIT_STACK_WIDTH {
        return Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area)
            .to_vec();
    }

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
    if area.width < PACKAGE_FULL_STACK_WIDTH {
        return Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(28),
                Constraint::Percentage(40),
                Constraint::Percentage(32),
            ])
            .split(area)
            .to_vec();
    }

    if area.width < PACKAGE_STACK_WIDTH {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(area);
        let lower = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[1]);
        return vec![rows[0], lower[0], lower[1]];
    }

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
    let density = package_list_density(area);
    let title = compact_package_list_title(list_model.title.as_str(), density);
    if let Some(empty_text) = &list_model.empty_text {
        let empty = Paragraph::new(empty_text.as_str())
            .block(Block::default().borders(Borders::ALL).title(title.as_str()))
            .wrap(Wrap { trim: false });
        frame.render_widget(empty, area);
        return;
    }

    let items = list_model
        .items
        .iter()
        .map(|item| format_package_list_item(item, density))
        .map(ListItem::new)
        .collect::<Vec<_>>();

    let mut list_state = ListState::default();
    list_state.select(list_model.selected_index);
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title.as_str()))
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(if density == PackageListDensity::Tight {
            "> "
        } else {
            ">> "
        });
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_package_selection(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    selection: &PackageSelectionModel,
) {
    let pane = Paragraph::new(render_package_selection_lines(
        selection,
        package_selection_density(area),
    ))
    .block(Block::default().borders(Borders::ALL).title(title))
    .wrap(Wrap { trim: false });
    frame.render_widget(pane, area);
}

fn package_list_density(area: Rect) -> PackageListDensity {
    if area.width <= PACKAGE_LIST_TIGHT_WIDTH {
        PackageListDensity::Tight
    } else if area.width <= PACKAGE_LIST_COMPACT_WIDTH {
        PackageListDensity::Compact
    } else {
        PackageListDensity::Standard
    }
}

fn render_package_summary(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    summary_model: &EditSummaryModel,
) {
    let summary = Paragraph::new(format_package_summary_for_area(summary_model, area))
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: false });
    frame.render_widget(summary, area);
}

fn format_package_summary_for_area(model: &EditSummaryModel, area: Rect) -> String {
    if area.width > EDIT_COMPACT_SUMMARY_WIDTH && area.height > EDIT_COMPACT_SUMMARY_HEIGHT {
        return format_edit_summary(model);
    }

    compact_package_summary_lines(model, area.width <= 40).join("\n")
}

fn compact_package_summary_lines(model: &EditSummaryModel, tight: bool) -> Vec<String> {
    let mut lines = Vec::new();

    let source = package_summary_header_value(model, "数据源").unwrap_or("未知");
    let user = package_summary_header_value(model, "当前用户").unwrap_or("无可用用户");
    lines.push(format!(
        "源/用户：{} / {}",
        compact_package_mode_label(source, tight),
        user
    ));

    if !tight && let Some(target) = package_summary_header_value(model, "目标目录") {
        lines.push(format!("目标：{}", compact_path_tail(target, 4)));
    }

    let category = package_summary_field_value(model, "分类过滤").unwrap_or("全部");
    let group = package_summary_field_value(model, "组过滤").unwrap_or("全部");
    let source_filter = package_summary_field_value(model, "来源过滤").unwrap_or("全部");
    lines.push(format!(
        "过滤：类={} 组={} 源={}",
        compact_package_text(category, if tight { 8 } else { 12 }),
        compact_package_group_label(group, tight),
        compact_package_text(source_filter, if tight { 8 } else { 12 })
    ));

    let workflow_filter = package_summary_field_value(model, "工作流过滤").unwrap_or("全部");
    let search = package_summary_field_value(model, "搜索").unwrap_or("无");
    if source.contains("搜索") || workflow_filter != "全部" || search != "无" {
        lines.push(format!(
            "流程/搜：{} / {}",
            compact_package_workflow_label(workflow_filter, tight),
            compact_package_text(search, if tight { 12 } else { 18 })
        ));
    }

    let total = package_summary_field_value(model, "目录总数").unwrap_or("0");
    let filtered = package_summary_field_value(model, "过滤后数量").unwrap_or("0");
    let selected = package_summary_field_value(model, "当前用户已选").unwrap_or("0");
    let dirty = package_summary_field_value(model, "未保存用户").unwrap_or("0");
    lines.push(if tight {
        format!("数量：总{total} / 选{selected} / dirty{dirty}")
    } else {
        format!("数量：总{total} / 过滤{filtered} / 已选{selected} / dirty{dirty}")
    });

    if let Some(current_workflow) = package_summary_field_value(model, "当前工作流") {
        let total = package_summary_field_value(model, "工作流可选").unwrap_or("0");
        let selected = package_summary_field_value(model, "工作流已选").unwrap_or("0");
        lines.push(format!(
            "当前流程：{} {selected}/{total}",
            compact_package_workflow_label(current_workflow, tight)
        ));
    }

    if let Some(current_group) = package_summary_field_value(model, "当前已选组") {
        lines.push(format!(
            "当前组：{}",
            compact_package_text(current_group, if tight { 24 } else { 32 })
        ));
    }

    if !tight && let Some(path) = package_summary_field_value(model, "当前组落点") {
        lines.push(format!("落点：{}", compact_path_tail(path, 4)));
    }

    lines.push(compact_edit_status_line(&model.detail.status));

    if let Some(validation) = &model.detail.validation {
        lines.push(compact_edit_check_summary(&validation.summary));
        lines.extend(validation.details.iter().cloned());
    }

    lines.push(compact_edit_check_summary(
        &model.detail.managed_guard.summary,
    ));
    lines.extend(model.detail.managed_guard.details.iter().cloned());

    lines
}

fn compact_package_list_title(title: &str, density: PackageListDensity) -> String {
    if density == PackageListDensity::Standard {
        return title.to_string();
    }
    if title.starts_with("Packages") {
        return "Packages".to_string();
    }
    title.to_string()
}

fn format_package_list_item(
    item: &crate::tui::state::PackageListItemModel,
    density: PackageListDensity,
) -> String {
    match density {
        PackageListDensity::Standard => item.display_line(),
        PackageListDensity::Compact => {
            let marker = if item.selected { "[x]" } else { "[ ]" };
            format!(
                "{marker} {} [{}/{}]",
                item.name, item.category, item.group_label
            )
        }
        PackageListDensity::Tight => {
            let marker = if item.selected { "[x]" } else { "[ ]" };
            format!("{marker} {}", item.name)
        }
    }
}

fn package_selection_density(area: Rect) -> PackageSelectionDensity {
    if area.width <= PACKAGE_SELECTION_TIGHT_WIDTH || area.height <= PACKAGE_SELECTION_TIGHT_HEIGHT
    {
        PackageSelectionDensity::Tight
    } else if area.width <= PACKAGE_SELECTION_COMPACT_WIDTH
        || area.height <= PACKAGE_SELECTION_COMPACT_HEIGHT
    {
        PackageSelectionDensity::Compact
    } else {
        PackageSelectionDensity::Standard
    }
}

fn render_package_selection_lines(
    selection: &PackageSelectionModel,
    density: PackageSelectionDensity,
) -> String {
    match density {
        PackageSelectionDensity::Standard => selection.lines(),
        PackageSelectionDensity::Compact => compact_package_selection_lines(selection, false),
        PackageSelectionDensity::Tight => compact_package_selection_lines(selection, true),
    }
    .join("\n")
}

fn compact_package_selection_lines(selection: &PackageSelectionModel, tight: bool) -> Vec<String> {
    let mut lines = Vec::new();

    if let Some(line) = compact_package_entry_line(selection, tight) {
        lines.push(line);
    }
    if let Some(line) = compact_package_entry_meta_line(selection, tight) {
        lines.push(line);
    }
    if let Some(line) = compact_package_group_line(selection, tight) {
        lines.push(line);
    }
    if let Some(line) = compact_package_workflow_line(selection, tight) {
        lines.push(line);
    }
    if let Some(line) = compact_package_action_line(selection, tight) {
        lines.push(line);
    }
    lines.push(compact_package_selected_line(selection, tight));
    lines.push(format!(
        "状态：{}",
        compact_package_status(&selection.status)
    ));

    lines
}

fn compact_package_entry_line(selection: &PackageSelectionModel, tight: bool) -> Option<String> {
    let name = package_selection_field_value(selection, "当前条目")?;
    let category = package_selection_field_value(selection, "分类");
    let group = package_selection_field_value(selection, "目标组");

    if tight {
        return Some(format!(
            "条目：{}{}",
            name,
            compact_package_bracket_suffix(category, group)
        ));
    }

    Some(format!("条目：{name}"))
}

fn compact_package_entry_meta_line(
    selection: &PackageSelectionModel,
    tight: bool,
) -> Option<String> {
    let category = package_selection_field_value(selection, "分类");
    let group = package_selection_field_value(selection, "目标组");
    let source = package_selection_field_value(selection, "来源");
    let workflow = package_selection_field_value(selection, "工作流")
        .map(|label| compact_package_workflow_label(label, tight));

    if tight {
        match (source, workflow) {
            (Some(source), Some(workflow)) => Some(format!("来源/流：{source} / {workflow}")),
            (Some(source), None) => Some(format!("来源：{source}")),
            (None, Some(workflow)) => Some(format!("流程：{workflow}")),
            (None, None) => None,
        }
    } else {
        match (category, group, source, workflow) {
            (Some(category), Some(group), Some(source), Some(workflow)) => Some(format!(
                "类/组/源：{category} / {group} / {source} | 流程：{workflow}"
            )),
            (Some(category), Some(group), Some(source), None) => {
                Some(format!("类/组/源：{category} / {group} / {source}"))
            }
            (Some(category), Some(group), None, Some(workflow)) => {
                Some(format!("类/组：{category} / {group} | 流程：{workflow}"))
            }
            (Some(category), Some(group), None, None) => {
                Some(format!("类/组：{category} / {group}"))
            }
            _ => None,
        }
    }
}

fn compact_package_group_line(selection: &PackageSelectionModel, tight: bool) -> Option<String> {
    let current = selection
        .group_rows
        .iter()
        .find(|row| row.current_selected)
        .map(|row| row.group_label.as_str());
    let filter = selection
        .group_rows
        .iter()
        .find(|row| row.filter_selected)
        .map(|row| row.group_label.as_str());

    match (current, filter) {
        (None, None) => None,
        (Some(current), Some(filter)) if current == filter => Some(format!(
            "组：{}（过滤）",
            compact_package_group_label(current, tight)
        )),
        (Some(current), Some(filter)) => Some(format!(
            "组：{} | 过滤：{}",
            compact_package_group_label(current, tight),
            compact_package_group_label(filter, tight)
        )),
        (Some(current), None) => Some(format!(
            "组：{}",
            compact_package_group_label(current, tight)
        )),
        (None, Some(filter)) => Some(format!(
            "过滤组：{}",
            compact_package_group_label(filter, tight)
        )),
    }
}

fn compact_package_workflow_line(selection: &PackageSelectionModel, tight: bool) -> Option<String> {
    if let Some(active) = &selection.active_workflow {
        let label = compact_package_workflow_label(&active.workflow_label, tight);
        let missing = active.missing_rows.len();
        return Some(if tight {
            format!(
                "流程：{} {}/{} 缺{}",
                label, active.selected_count, active.total_count, missing
            )
        } else {
            format!(
                "流程：{}（已选 {}/{}, 缺 {}）",
                label, active.selected_count, active.total_count, missing
            )
        });
    }

    let filter = selection
        .workflow_rows
        .iter()
        .find(|row| row.filter_selected)
        .map(|row| compact_package_workflow_label(&row.workflow_label, tight));
    let current = selection
        .workflow_rows
        .iter()
        .find(|row| row.current_selected)
        .map(|row| compact_package_workflow_label(&row.workflow_label, tight));

    match (current, filter) {
        (None, None) => None,
        (Some(current), Some(filter)) if current == filter => Some(format!("流程：{current}")),
        (Some(current), Some(filter)) => Some(format!("流程：{current} | 过滤：{filter}")),
        (Some(current), None) => Some(format!("流程：{current}")),
        (None, Some(filter)) => Some(format!("过滤流：{filter}")),
    }
}

fn compact_package_action_line(selection: &PackageSelectionModel, tight: bool) -> Option<String> {
    let action = selection.action_summary.as_ref()?;
    let result = compact_package_feedback(&action.latest_result);
    let next_step = compact_package_feedback(&action.next_step);

    Some(if tight {
        format!("结果：{result}")
    } else {
        format!("结果/下一步：{result} | {next_step}")
    })
}

fn compact_package_selected_line(selection: &PackageSelectionModel, tight: bool) -> String {
    if selection.selected_rows.is_empty() {
        return "已选：无".to_string();
    }

    let preview_limit = if tight { 1 } else { 2 };
    let preview = selection
        .selected_rows
        .iter()
        .take(preview_limit)
        .map(|row| row.name.as_str())
        .collect::<Vec<_>>();

    let mut preview_text = preview.join(", ");
    if selection.selected_rows.len() > preview.len() {
        preview_text.push_str(" 等");
    }

    format!(
        "已选：{} 项（{}）",
        selection.selected_rows.len(),
        preview_text
    )
}

fn compact_package_status(status: &str) -> String {
    if status == "ready" {
        return "就绪".to_string();
    }
    if status.starts_with("已写入 ") {
        return "已写入".to_string();
    }
    if status.starts_with("Packages 未写入：") {
        return "未写入".to_string();
    }
    if let Some(rest) = status.strip_prefix("当前软件来源过滤：") {
        return format!("来源过滤：{rest}");
    }
    if let Some(rest) = status.strip_prefix("当前软件组过滤：") {
        return format!("组过滤：{rest}");
    }
    if let Some(rest) = status.strip_prefix("当前项目工作流过滤：") {
        return format!("流程过滤：{rest}");
    }
    status.to_string()
}

fn compact_package_feedback(feedback: &str) -> String {
    if feedback.starts_with("Packages 已写入 ") || feedback.starts_with("已写入 ") {
        return "已写入".to_string();
    }
    if feedback.starts_with("Packages 未写入：") {
        return "未写入".to_string();
    }
    if feedback.starts_with("Packages 已刷新 nixpkgs 搜索：") {
        return "搜索已刷新".to_string();
    }
    if feedback.starts_with("Packages nixpkgs 搜索失败：") {
        return "搜索失败".to_string();
    }
    if feedback.starts_with("Packages 已选中 ") {
        return "已选中".to_string();
    }
    if feedback.starts_with("Packages 已取消 ") {
        return "已取消".to_string();
    }
    if feedback.starts_with("Packages 已把 ") && feedback.contains("调整到组") {
        return "已调组".to_string();
    }
    if feedback.starts_with("Packages 已将组 ") {
        return "已改组".to_string();
    }
    if feedback.starts_with("Packages 已新建组 ") {
        return "已建组".to_string();
    }
    if feedback.starts_with("Packages 已加入工作流 ") {
        return "已批量加入".to_string();
    }
    if feedback == "Packages 已清空来源过滤。" {
        return "来源=全部".to_string();
    }
    if let Some(rest) = feedback.strip_prefix("Packages 来源过滤：") {
        return format!("来源={rest}");
    }
    if feedback == "Packages 已清空组过滤。" {
        return "组=全部".to_string();
    }
    if let Some(rest) = feedback.strip_prefix("Packages 组过滤：") {
        return format!("组={rest}");
    }
    if feedback == "Packages 已清空工作流过滤。" {
        return "流程=全部".to_string();
    }
    if let Some(rest) = feedback.strip_prefix("Packages 流程过滤：") {
        return format!("流程={rest}");
    }
    if feedback == "Packages 已切到 nixpkgs 搜索模式。" {
        return "搜索模式".to_string();
    }
    if feedback == "Packages 已切回本地覆盖/已声明视图。" {
        return "本地模式".to_string();
    }
    if feedback == "Packages 搜索输入已打开。" || feedback == "Packages 搜索输入结束。"
    {
        return "搜索输入".to_string();
    }
    if feedback == "暂无" {
        return "暂无".to_string();
    }
    feedback.to_string()
}

fn compact_package_mode_label(label: &str, tight: bool) -> String {
    if tight {
        if label == "本地覆盖/已声明" {
            return "本地".to_string();
        }
        if label == "nixpkgs 搜索" {
            return "搜索".to_string();
        }
    }
    label.to_string()
}

fn compact_package_group_label(label: &str, tight: bool) -> String {
    compact_package_text(label, if tight { 18 } else { 24 })
}

fn compact_package_workflow_label(label: &str, tight: bool) -> String {
    compact_package_text(&label.replace(" [", "["), if tight { 20 } else { 28 })
}

fn compact_package_text(text: &str, max_chars: usize) -> String {
    let count = text.chars().count();
    if count <= max_chars {
        return text.to_string();
    }
    let truncated = text
        .chars()
        .take(max_chars.saturating_sub(3))
        .collect::<String>();
    format!("{truncated}...")
}

fn compact_package_bracket_suffix(left: Option<&str>, right: Option<&str>) -> String {
    match (left, right) {
        (Some(left), Some(right)) => format!(" [{left}/{right}]"),
        (Some(left), None) => format!(" [{left}]"),
        (None, Some(right)) => format!(" [{right}]"),
        (None, None) => String::new(),
    }
}

fn package_selection_field_value<'a>(
    selection: &'a PackageSelectionModel,
    label: &str,
) -> Option<&'a str> {
    selection
        .current_entry_fields
        .iter()
        .find(|row| row.label == label)
        .map(|row| row.value.as_str())
}

fn package_summary_header_value<'a>(model: &'a EditSummaryModel, label: &str) -> Option<&'a str> {
    model
        .header_lines
        .iter()
        .find_map(|line| line.strip_prefix(&format!("{label}：")))
}

fn package_summary_field_value<'a>(model: &'a EditSummaryModel, label: &str) -> Option<&'a str> {
    model
        .field_lines
        .iter()
        .find_map(|line| line.strip_prefix(&format!("{label}：")))
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
        Some(PackageTextMode::ConfirmWorkflowAdd) => {
            let workflow = state
                .current_package_workflow_filter()
                .map(|workflow| state.package_workflow_display(workflow))
                .unwrap_or_else(|| "无".to_string());
            let user = state.current_package_user().unwrap_or("无可用用户");
            let rows = state
                .current_workflow_missing_package_rows()
                .unwrap_or_default()
                .into_iter()
                .map(|row| row.display_line())
                .collect::<Vec<_>>();
            let preview = if rows.is_empty() {
                "当前 workflow 没有未选软件。".to_string()
            } else {
                rows.join("\n")
            };
            (
                "Add Workflow Packages",
                format!(
                    "批量加入当前 workflow 下尚未选中的软件\n\n当前用户: {user}\n当前工作流: {workflow}\n将加入:\n{preview}\n\nEnter 确认  Esc 取消"
                ),
            )
        }
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
        .block(Block::default().borders(Borders::ALL).title("Keys"));
    frame.render_widget(Clear, area);
    frame.render_widget(help, area);
}

fn footer_text(state: &AppState) -> &'static str {
    match state.top_level_page() {
        TopLevelPage::Overview => {
            "Overview | Enter/Space 预览 Apply | i Inspect | ? 帮助 | Tab 区域"
        }
        TopLevelPage::Edit => edit_footer_text(state),
        TopLevelPage::Apply => {
            if state.active_deploy_text_mode() == Some(DeployTextMode::ApplyRemotePinnedRef) {
                "Apply ref | 输入 ref | Enter 确认 | Esc 取消"
            } else {
                "Apply | j/k 聚焦 | h/l/Enter 调整 | x 应用 | ? 帮助"
            }
        }
        TopLevelPage::Advanced => {
            if state.active_deploy_text_mode()
                == Some(DeployTextMode::AdvancedWizardRemotePinnedRef)
            {
                "Advanced ref | 输入 ref | Enter 确认 | Esc 取消"
            } else if state.advanced_action_uses_deploy_parameters() {
                "Advanced | j/k 参数 | h/l/Enter 调整 | J/K 动作 | x/X 执行 | b Apply | ? 帮助"
            } else {
                "Advanced | j/k/J/K 动作 | x/X 执行 | b Apply | ? 帮助"
            }
        }
        TopLevelPage::Inspect => "Inspect | j/k 命令 | x 执行 | r/d/R 刷新 | ? 帮助",
    }
}

fn edit_footer_text(state: &AppState) -> &'static str {
    match state.edit_page() {
        Page::Users if state.active_users_text_mode().is_some() => {
            "Edit/Users 输入 | 直接输入 | Enter 确认 | Esc 取消"
        }
        Page::Hosts if state.active_hosts_text_mode().is_some() => {
            "Edit/Hosts 输入 | 直接输入 | Enter 确认 | Esc 取消"
        }
        Page::Packages => match state.active_package_text_mode() {
            Some(PackageTextMode::CreateGroup) => {
                "Edit/Packages 新组 | 直接输入 | Enter 确认 | Esc 取消"
            }
            Some(PackageTextMode::RenameGroup) => {
                "Edit/Packages 重命名 | 直接输入 | Enter 确认 | Esc 取消"
            }
            Some(PackageTextMode::Search) => "Edit/Packages 搜索 | 直接输入 | Enter/Esc 完成",
            Some(PackageTextMode::ConfirmWorkflowAdd) => {
                "Edit/Packages 确认 | Enter 确认 | Esc 取消"
            }
            None => {
                "Edit/Packages | 1-4 子页 | ←/→ 目标 | j/k 移动 | Enter/Space 动作 | s 保存 | ? 帮助"
            }
        },
        Page::Home => {
            "Edit/Home | 1-4 子页 | ←/→ 目标 | j/k 移动 | h/l/Enter 调整 | s 保存 | ? 帮助"
        }
        Page::Users => {
            "Edit/Users | 1-4 子页 | ←/→ 目标 | j/k 移动 | h/l/Enter 动作 | s 保存 | ? 帮助"
        }
        Page::Hosts => {
            "Edit/Hosts | 1-4 子页 | ←/→ 目标 | j/k 移动 | h/l/Enter 动作 | s 保存 | ? 帮助"
        }
        Page::Dashboard | Page::Deploy | Page::Advanced | Page::Inspect => {
            "Edit | 1-4 子页 | ? 帮助 | Tab 区域"
        }
    }
}

fn render_help_overlay(frame: &mut Frame, area: Rect, state: &AppState) {
    let popup = centered_rect(76, 86, area);
    let panel = Paragraph::new(help_panel_text(state))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(help_panel_title(state)),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(Clear, popup);
    frame.render_widget(panel, popup);
}

fn help_panel_title(state: &AppState) -> &'static str {
    match state.top_level_page() {
        TopLevelPage::Overview => "Overview Help",
        TopLevelPage::Edit => match state.edit_page() {
            Page::Packages => "Packages Help",
            Page::Home => "Home Help",
            Page::Users => "Users Help",
            Page::Hosts => "Hosts Help",
            Page::Dashboard | Page::Deploy | Page::Advanced | Page::Inspect => "Edit Help",
        },
        TopLevelPage::Apply => "Apply Help",
        TopLevelPage::Advanced => "Advanced Help",
        TopLevelPage::Inspect => "Inspect Help",
    }
}

fn help_panel_text(state: &AppState) -> String {
    match state.top_level_page() {
        TopLevelPage::Overview => [
            "当前页：Overview",
            "页脚只保留主路径和最常用键；完整说明看这里。",
            "",
            "阅读顺序",
            "先看 Overview Summary 和 Current Context",
            "再看 Health、Dirty State、Apply Snapshot（来源 / 动作 / 同步）",
            "",
            "主路径",
            "Enter / Space  先进入 Apply 预览，不会直接执行",
            "a / p          也会进入 Apply 预览",
            "i              打开 Inspect 检查闭环",
            "",
            "健康刷新",
            "r              刷新 repo-integrity",
            "d              刷新 doctor",
            "R              顺序刷新两者",
            "",
            "统一键位",
            "Tab / Shift-Tab  切换顶层区域",
            "?                打开或关闭帮助",
            "Esc              关闭帮助面板",
            "q                退出 mcbctl",
        ]
        .join("\n"),
        TopLevelPage::Edit => edit_help_panel_text(state),
        TopLevelPage::Apply => {
            if state.active_deploy_text_mode() == Some(DeployTextMode::ApplyRemotePinnedRef) {
                [
                    "当前页：Apply / 固定 ref 输入",
                    "",
                    "输入模式",
                    "直接输入远端 ref / tag / commit",
                    "Backspace  删除字符",
                    "Enter      确认 ref",
                    "Esc        取消输入",
                ]
                .join("\n")
            } else {
                [
                    "当前页：Apply",
                    "页脚只保留默认路径常用键；完整说明看这里。",
                    "",
                    "统一语义",
                    "j / k           在 Apply 控件中移动",
                    "h / l / Enter   只调整当前控件，不会直接执行",
                    "x               执行 Apply Current Host",
                    "",
                    "阅读顺序",
                    "先看左侧 Apply Summary 和 Apply Preview",
                    "再看右侧 Current Selection 与 Apply Controls",
                    "若出现 handoff，请切到 Advanced 完成高级路径",
                    "",
                    "统一键位",
                    "Tab / Shift-Tab  切换顶层区域",
                    "?                打开或关闭帮助",
                    "Esc              关闭帮助面板",
                    "q                退出 mcbctl",
                ]
                .join("\n")
            }
        }
        TopLevelPage::Advanced => {
            if state.active_deploy_text_mode()
                == Some(DeployTextMode::AdvancedWizardRemotePinnedRef)
            {
                [
                    "当前页：Advanced / 固定 ref 输入",
                    "",
                    "输入模式",
                    "直接输入远端 ref / tag / commit",
                    "Backspace  删除字符",
                    "Enter      确认 ref",
                    "Esc        取消输入",
                ]
                .join("\n")
            } else if state.advanced_action_uses_deploy_parameters() {
                [
                    "当前页：Advanced / Deploy Wizard",
                    "页脚只保留当前路径常用键；完整说明看这里。",
                    "",
                    "统一语义",
                    "j / k           在 deploy 参数间移动",
                    "h / l / Enter   调整当前参数",
                    "J / K           在高级动作间切换",
                    "x / X           执行当前高级动作",
                    "b               返回 Apply",
                    "",
                    "统一键位",
                    "Tab / Shift-Tab  切换顶层区域",
                    "?                打开或关闭帮助",
                    "Esc              关闭帮助面板",
                    "q                退出 mcbctl",
                ]
                .join("\n")
            } else {
                [
                    "当前页：Advanced / Repository Maintenance",
                    "页脚只保留当前路径常用键；完整说明看这里。",
                    "",
                    "统一语义",
                    "j / k 或 J / K  在高级动作间切换",
                    "x / X           执行当前高级动作",
                    "b               返回 Apply",
                    "Enter / Space   当前模式不执行高级动作",
                    "",
                    "统一键位",
                    "Tab / Shift-Tab  切换顶层区域",
                    "?                打开或关闭帮助",
                    "Esc              关闭帮助面板",
                    "q                退出 mcbctl",
                ]
                .join("\n")
            }
        }
        TopLevelPage::Inspect => [
            "当前页：Inspect",
            "页脚只保留当前页最常用键；完整说明看这里。",
            "",
            "统一语义",
            "j / k           在检查命令间移动",
            "x               执行当前 Inspect 命令",
            "Enter / Space   当前页不执行检查命令",
            "r               刷新 repo-integrity",
            "d               刷新 doctor",
            "R               顺序刷新两者",
            "",
            "阅读顺序",
            "先看右侧 Inspect Summary",
            "再看 Health Details 和 Command Detail",
            "确认后按 x 执行当前 Inspect 命令",
            "",
            "统一键位",
            "Tab / Shift-Tab  切换顶层区域",
            "?                打开或关闭帮助",
            "Esc              关闭帮助面板",
            "q                退出 mcbctl",
        ]
        .join("\n"),
    }
}

fn edit_help_panel_text(state: &AppState) -> String {
    match state.edit_page() {
        Page::Packages
            if state.active_package_text_mode() == Some(PackageTextMode::CreateGroup) =>
        {
            edit_input_help_text(
                "Packages / 新建组输入",
                "直接输入组名",
                "Enter      确认创建",
            )
        }
        Page::Packages
            if state.active_package_text_mode() == Some(PackageTextMode::RenameGroup) =>
        {
            edit_input_help_text(
                "Packages / 重命名组输入",
                "直接输入组名",
                "Enter      确认重命名",
            )
        }
        Page::Packages if state.active_package_text_mode() == Some(PackageTextMode::Search) => {
            edit_input_help_text(
                "Packages / 搜索输入",
                "直接输入关键词",
                "Enter      结束搜索输入",
            )
        }
        Page::Packages
            if state.active_package_text_mode() == Some(PackageTextMode::ConfirmWorkflowAdd) =>
        {
            [
                "当前页：Edit / Packages / workflow 批量加入确认",
                "",
                "确认模式",
                "Enter      把当前 workflow 的缺失软件加入选中",
                "Esc        取消本次批量加入",
            ]
            .join("\n")
        }
        Page::Packages => edit_help_with_actions(
            "Packages",
            &[
                "Enter / Space   对当前软件执行主动作",
                "s               保存当前用户的 packages 分片",
                "",
                "扩展键",
                "/               打开搜索输入",
                "f               切换搜索 / 本地视图",
                "r               刷新搜索结果",
                "[ / ] 或 h / l  切换分类",
                "u / i           切换来源过滤",
                "o / p           切换 workflow 过滤",
                "A               批量加入当前 workflow 的缺失软件",
                ", / .           切换组过滤",
                "z / Z           聚焦当前组 / 清空组过滤",
                "g / G           调整当前软件的组",
                "m / M           移动当前选中组",
                "n               新建组",
                "R               重命名组",
            ],
        ),
        Page::Home => edit_help_with_actions(
            "Home",
            &[
                "h / l / Enter   调整当前字段",
                "s               保存当前用户的 desktop 分片",
            ],
        ),
        Page::Users if state.active_users_text_mode().is_some() => edit_input_help_text(
            "Users / 文本输入",
            "直接输入用户名列表",
            "Enter      确认写回当前字段",
        ),
        Page::Users => edit_help_with_actions(
            "Users",
            &[
                "h / l           调整枚举字段",
                "Enter           编辑用户列表",
                "s               保存当前主机的 users 分片",
            ],
        ),
        Page::Hosts if state.active_hosts_text_mode().is_some() => edit_input_help_text(
            "Hosts / 文本输入",
            "直接输入文本、列表或映射值",
            "Enter      确认写回当前字段",
        ),
        Page::Hosts => edit_help_with_actions(
            "Hosts",
            &[
                "h / l           调整枚举字段",
                "Enter           编辑文本或映射字段",
                "s               保存当前主机的 runtime 分片",
            ],
        ),
        Page::Dashboard | Page::Deploy | Page::Advanced | Page::Inspect => {
            edit_help_with_actions("Edit", &["1 / 2 / 3 / 4   切换 Edit 子页"])
        }
    }
}

fn edit_input_help_text(title: &str, input_hint: &str, enter_hint: &str) -> String {
    [
        format!("当前页：Edit / {title}"),
        String::new(),
        "输入模式".to_string(),
        input_hint.to_string(),
        "Backspace  删除字符".to_string(),
        enter_hint.to_string(),
        "Esc        取消输入".to_string(),
    ]
    .join("\n")
}

fn edit_help_with_actions(page: &str, page_actions: &[&str]) -> String {
    let mut lines = vec![
        format!("当前页：Edit / {page}"),
        "页脚只保留当前页主动作；共同骨架和扩展键放在这里。".to_string(),
        "Edit Pages 顶栏中的 * 表示该子页有未保存修改。".to_string(),
        String::new(),
        "先看".to_string(),
        "先看 Edit Workspace，再看当前页主列表和右侧摘要".to_string(),
        String::new(),
        "共同骨架".to_string(),
        "← / →          切换当前页目标".to_string(),
        "j / k           在当前页主列表中移动".to_string(),
        "1 / 2 / 3 / 4   切换 Edit 子页".to_string(),
        String::new(),
        "当前页主动作".to_string(),
    ];
    lines.extend(page_actions.iter().map(|line| line.to_string()));
    lines.extend([
        String::new(),
        "统一键位".to_string(),
        "Tab / Shift-Tab  切换顶层区域".to_string(),
        "?                打开或关闭帮助".to_string(),
        "q                退出 mcbctl".to_string(),
    ]);
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::tui::{
        CatalogEntry, DeployAction, DeploySource, DeployTask, GroupMeta, HomeOptionMeta,
        HostManagedSettings, PackageDataMode, WorkflowMeta,
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
                action_summary: None,
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
    fn edit_page_chunks_keep_horizontal_split_on_wide_width() {
        let area = Rect::new(0, 0, 100, 20);

        let home_like = edit_page_chunks(area, 42);
        let hosts_like = edit_page_chunks(area, 44);

        assert_eq!(home_like[0].width, 42);
        assert_eq!(home_like[1].width, 58);
        assert_eq!(hosts_like[0].width, 44);
        assert_eq!(hosts_like[1].width, 56);
    }

    #[test]
    fn edit_page_chunks_stack_on_narrow_width() {
        let area = Rect::new(0, 0, 80, 20);

        let chunks = edit_page_chunks(area, 42);

        assert_eq!(chunks[0].width, 80);
        assert_eq!(chunks[1].width, 80);
        assert_eq!(chunks[0].y, 0);
        assert!(chunks[1].y > chunks[0].y);
    }

    #[test]
    fn package_page_chunks_keep_three_columns_on_wide_width() {
        let area = Rect::new(0, 0, 140, 24);

        let chunks = package_page_chunks(area, 28, 39);

        assert_eq!(chunks[0].width, 39);
        assert_eq!(chunks[1].width, 55);
        assert_eq!(chunks[2].width, 46);
        assert_eq!(chunks[0].y, 0);
        assert_eq!(chunks[1].y, 0);
        assert_eq!(chunks[2].y, 0);
    }

    #[test]
    fn package_page_chunks_move_summary_above_split_detail_on_medium_width() {
        let area = Rect::new(0, 0, 104, 24);

        let chunks = package_page_chunks(area, 28, 39);

        assert_eq!(chunks[0].width, 104);
        assert_eq!(chunks[0].y, 0);
        assert_eq!(chunks[1].y, chunks[2].y);
        assert!(chunks[1].y > chunks[0].y);
        assert_eq!(chunks[1].width + chunks[2].width, 104);
    }

    #[test]
    fn package_page_chunks_stack_all_sections_on_very_narrow_width() {
        let area = Rect::new(0, 0, 88, 24);

        let chunks = package_page_chunks(area, 28, 39);

        assert_eq!(chunks[0].width, 88);
        assert_eq!(chunks[1].width, 88);
        assert_eq!(chunks[2].width, 88);
        assert!(chunks[1].y > chunks[0].y);
        assert!(chunks[2].y > chunks[1].y);
    }

    #[test]
    fn compact_edit_summary_lines_shorten_home_copy_for_narrow_area() {
        let summary = EditSummaryModel {
            header_lines: vec![
                "当前用户：alice".to_string(),
                "目标文件：/repo/home/users/alice/managed/settings/desktop.nix".to_string(),
            ],
            focused_row: Some(EditRow {
                label: "Noctalia".to_string(),
                value: "default".to_string(),
            }),
            field_lines: vec!["Noctalia：default".to_string()],
            detail: crate::tui::state::EditDetailModel {
                status: "状态：当前用户没有未保存的 Home 设置修改".to_string(),
                action_summary: Some(crate::tui::state::EditActionSummaryModel {
                    latest_result: "Home 已更新用户 alice 的 Noctalia 顶栏。".to_string(),
                    next_step: "继续调整当前字段，完成后按 s 保存。".to_string(),
                }),
                validation: None,
                managed_guard: crate::tui::state::EditCheckModel {
                    summary: "受管保护：通过".to_string(),
                    details: Vec::new(),
                },
                notes: vec![
                    "当前阶段已接入的结构化设置：".to_string(),
                    "- Noctalia：顶栏配置".to_string(),
                    "这些内容只会写入 managed/settings/desktop.nix，不会直接改你的手写 config/。"
                        .to_string(),
                ],
            },
        };

        let text = format_edit_summary_for_area(&summary, Rect::new(0, 0, 40, 10));

        assert!(text.contains("用户：alice"));
        assert!(text.contains("目标：.../alice/managed/settings/desktop.nix"));
        assert!(text.contains("状态：已保存"));
        assert!(text.contains("最近结果：Home 已更新"));
        assert!(text.contains("下一步：继续调整，按 s 保存"));
        assert!(text.contains("已接入："));
        assert!(text.contains("写回：managed/settings/desktop.nix"));
        assert!(!text.contains("当前用户没有未保存的 Home 设置修改"));
    }

    #[test]
    fn compact_package_summary_lines_shorten_filters_counts_and_focus_for_narrow_area() {
        let summary = EditSummaryModel {
            header_lines: vec![
                "数据源：本地覆盖/已声明".to_string(),
                "当前用户：alice".to_string(),
                "目标目录：/repo/home/users/alice/managed/packages".to_string(),
            ],
            focused_row: None,
            field_lines: vec![
                "分类过滤：全部".to_string(),
                "组过滤：misc".to_string(),
                "来源过滤：全部".to_string(),
                "工作流过滤：容器与集群 [containers]".to_string(),
                "搜索：podman".to_string(),
                "目录总数：3".to_string(),
                "过滤后数量：2".to_string(),
                "当前用户已选：1".to_string(),
                "未保存用户：1".to_string(),
                "当前工作流：容器与集群 [containers]".to_string(),
                "工作流可选：2".to_string(),
                "工作流已选：1".to_string(),
                "工作流说明：容器工具链".to_string(),
                "当前组落点：/repo/home/users/alice/managed/packages/misc.nix".to_string(),
                "当前已选组：misc（1 个软件）".to_string(),
                "组说明：常用杂项".to_string(),
            ],
            detail: crate::tui::state::EditDetailModel {
                status: "状态：当前用户有未保存修改".to_string(),
                action_summary: None,
                validation: None,
                managed_guard: crate::tui::state::EditCheckModel {
                    summary: "受管保护：通过".to_string(),
                    details: Vec::new(),
                },
                notes: Vec::new(),
            },
        };

        let text = format_package_summary_for_area(&summary, Rect::new(0, 0, 44, 12));

        assert!(text.contains("源/用户：本地覆盖/已声明 / alice"));
        assert!(text.contains("目标：.../users/alice/managed/packages"));
        assert!(text.contains("过滤：类=全部 组=misc 源=全部"));
        assert!(text.contains("流程/搜：容器与集群[containers] / podman"));
        assert!(text.contains("数量：总3 / 过滤2 / 已选1 / dirty1"));
        assert!(text.contains("当前流程：容器与集群[containers] 1/2"));
        assert!(text.contains("当前组：misc（1 个软件）"));
        assert!(text.contains("落点：.../alice/managed/packages/misc.nix"));
        assert!(text.contains("状态：未保存"));
        assert!(!text.contains("工作流说明：容器工具链"));
        assert!(!text.contains("组说明：常用杂项"));
    }

    #[test]
    fn compact_package_summary_lines_drop_target_path_in_tight_mode() {
        let summary = EditSummaryModel {
            header_lines: vec![
                "数据源：本地覆盖/已声明".to_string(),
                "当前用户：alice".to_string(),
                "目标目录：/repo/home/users/alice/managed/packages".to_string(),
            ],
            focused_row: None,
            field_lines: vec![
                "分类过滤：全部".to_string(),
                "组过滤：misc".to_string(),
                "来源过滤：全部".to_string(),
                "工作流过滤：全部".to_string(),
                "搜索：无".to_string(),
                "目录总数：3".to_string(),
                "过滤后数量：2".to_string(),
                "当前用户已选：1".to_string(),
                "未保存用户：1".to_string(),
            ],
            detail: crate::tui::state::EditDetailModel {
                status: "状态：当前用户有未保存修改".to_string(),
                action_summary: None,
                validation: None,
                managed_guard: crate::tui::state::EditCheckModel {
                    summary: "受管保护：通过".to_string(),
                    details: Vec::new(),
                },
                notes: Vec::new(),
            },
        };

        let text = format_package_summary_for_area(&summary, Rect::new(0, 0, 36, 9));

        assert!(text.contains("源/用户：本地 / alice"));
        assert!(text.contains("过滤：类=全部 组=misc 源=全部"));
        assert!(text.contains("数量：总3 / 选1 / dirty1"));
        assert!(text.contains("状态：未保存"));
        assert!(!text.contains("目标："));
    }

    #[test]
    fn package_list_item_compacts_to_shorter_shape_on_narrow_width() {
        let item = crate::tui::state::PackageListItemModel {
            selected: true,
            name: "Hello".to_string(),
            category: "cli".to_string(),
            group_label: "misc".to_string(),
        };

        assert_eq!(
            format_package_list_item(&item, PackageListDensity::Compact),
            "[x] Hello [cli/misc]"
        );
        assert_eq!(
            format_package_list_item(&item, PackageListDensity::Tight),
            "[x] Hello"
        );
    }

    #[test]
    fn compact_package_selection_lines_keep_entry_and_selected_count_visible() {
        let selection = PackageSelectionModel {
            current_entry_fields: vec![
                EditRow {
                    label: "当前条目".to_string(),
                    value: "Hello".to_string(),
                },
                EditRow {
                    label: "分类".to_string(),
                    value: "cli".to_string(),
                },
                EditRow {
                    label: "来源".to_string(),
                    value: "nixpkgs".to_string(),
                },
                EditRow {
                    label: "工作流".to_string(),
                    value: "容器与集群 [containers]".to_string(),
                },
                EditRow {
                    label: "目标组".to_string(),
                    value: "misc".to_string(),
                },
            ],
            group_rows: vec![crate::tui::state::PackageGroupOverviewRow {
                group_label: "misc".to_string(),
                count: 1,
                filter_selected: true,
                current_selected: true,
            }],
            workflow_rows: vec![crate::tui::state::PackageWorkflowOverviewRow {
                workflow_label: "容器与集群 [containers]".to_string(),
                total_count: 1,
                selected_count: 1,
                filter_selected: true,
                current_selected: true,
            }],
            active_workflow: Some(crate::tui::state::PackageActiveWorkflowModel {
                workflow_label: "容器与集群 [containers]".to_string(),
                description: Some("容器相关工具".to_string()),
                total_count: 1,
                selected_count: 1,
                selected_rows: vec![crate::tui::state::PackageWorkflowEntryRow {
                    name: "Hello".to_string(),
                    category: "cli".to_string(),
                    group_label: "misc".to_string(),
                }],
                missing_rows: Vec::new(),
            }),
            action_summary: Some(crate::tui::state::PackageActionSummaryModel {
                latest_result: "Packages 已刷新 nixpkgs 搜索：hello（1 条结果）。".to_string(),
                next_step: "继续浏览结果，或修改关键词后再按 Enter / r 刷新。".to_string(),
            }),
            selected_rows: vec![crate::tui::state::PackageSelectedEntryRow {
                name: "Hello".to_string(),
                category: "cli".to_string(),
                group_label: "misc".to_string(),
            }],
            status: "未保存".to_string(),
        };

        let text = render_package_selection_lines(&selection, PackageSelectionDensity::Compact);

        assert!(text.contains("条目：Hello"));
        assert!(text.contains("结果/下一步：搜索已刷新"));
        assert!(text.contains("已选：1 项"));
        assert!(text.contains("状态：未保存"));
        assert!(!text.contains("当前用户分组（> 过滤，* 当前条目）"));
    }

    #[test]
    fn edit_workspace_summary_lines_show_current_target_and_dirty_sections() {
        let mut state = test_state();
        state.open_edit_page(Page::Packages);
        state.package_dirty_users.insert("alice".to_string());
        state.host_dirty_runtime_hosts.insert("demo".to_string());

        let lines = state.edit_workspace_summary_model().lines();

        assert_eq!(lines[0], "当前页/目标：Packages / user alice");
        assert!(lines[1].contains("Packages(alice)"));
        assert!(lines[1].contains("Home(clean)"));
        assert!(lines[1].contains("Users(clean)"));
        assert!(lines[1].contains("Hosts(demo)"));
        assert_eq!(lines[2], "建议：当前页有未保存修改；先按 s 保存。");
    }

    #[test]
    fn edit_workspace_summary_lines_show_na_for_missing_user_targets() {
        let mut state = test_state();
        state.context.users.clear();
        state.open_edit_page(Page::Home);

        let lines = state.edit_workspace_summary_model().lines();

        assert_eq!(lines[0], "当前页/目标：Home / 无可用用户");
        assert!(lines[1].contains("Packages(n/a)"));
        assert!(lines[1].contains("Home(n/a)"));
        assert_eq!(lines[2], "建议：先补可用目标，或切到其他编辑页。");
    }

    #[test]
    fn edit_workspace_summary_lines_recommend_switching_to_first_dirty_page() {
        let mut state = test_state();
        state.open_edit_page(Page::Home);
        state.host_dirty_user_hosts.insert("demo".to_string());

        let lines = state.edit_workspace_summary_model().lines();

        assert_eq!(lines[2], "建议：先去 Users[demo] 保存未保存修改。");
    }

    #[test]
    fn edit_page_tabs_mark_dirty_subpages_without_switching_views() {
        let mut state = test_state();
        state.package_dirty_users.insert("alice".to_string());
        state.host_dirty_user_hosts.insert("demo".to_string());

        let tabs = edit_page_tabs(&state);

        assert_eq!(tabs, vec!["1 Packages*", "2 Home", "3 Users*", "4 Hosts"]);
    }

    #[test]
    fn footer_text_overview_mentions_primary_actions_and_shell_navigation() {
        let mut state = test_state();
        state.open_overview();

        let footer = footer_text(&state);

        assert!(footer.contains("Enter/Space 预览 Apply"));
        assert!(footer.contains("i Inspect"));
        assert!(footer.contains("Tab 区域"));
        assert!(footer.contains("? 帮助"));
    }

    #[test]
    fn footer_text_edit_tracks_active_leaf_and_text_mode() {
        let mut state = test_state();
        state.open_edit_page(Page::Users);
        state.users_text_mode = Some(UsersTextMode::ManagedUsers);

        assert_eq!(
            footer_text(&state),
            "Edit/Users 输入 | 直接输入 | Enter 确认 | Esc 取消"
        );

        state.users_text_mode = None;
        assert_eq!(
            footer_text(&state),
            "Edit/Users | 1-4 子页 | ←/→ 目标 | j/k 移动 | h/l/Enter 动作 | s 保存 | ? 帮助"
        );

        state.open_edit_page(Page::Packages);
        state.package_group_create_mode = true;
        assert_eq!(
            footer_text(&state),
            "Edit/Packages 新组 | 直接输入 | Enter 确认 | Esc 取消"
        );

        state.package_group_create_mode = false;
        state.package_group_rename_mode = true;
        assert_eq!(
            footer_text(&state),
            "Edit/Packages 重命名 | 直接输入 | Enter 确认 | Esc 取消"
        );

        state.package_group_rename_mode = false;
        state.package_search_mode = true;
        assert_eq!(
            footer_text(&state),
            "Edit/Packages 搜索 | 直接输入 | Enter/Esc 完成"
        );

        state.package_search_mode = false;
        assert_eq!(
            footer_text(&state),
            "Edit/Packages | 1-4 子页 | ←/→ 目标 | j/k 移动 | Enter/Space 动作 | s 保存 | ? 帮助"
        );

        state.package_workflow_add_confirm_mode = true;
        assert_eq!(
            footer_text(&state),
            "Edit/Packages 确认 | Enter 确认 | Esc 取消"
        );
    }

    #[test]
    fn help_panel_edit_pages_share_common_shell_and_page_specific_actions() {
        let mut state = test_state();

        state.open_edit_page(Page::Packages);
        let packages_help = help_panel_text(&state);
        assert!(packages_help.contains("当前页：Edit / Packages"));
        assert!(packages_help.contains("先看 Edit Workspace，再看当前页主列表和右侧摘要"));
        assert!(packages_help.contains("← / →          切换当前页目标"));
        assert!(packages_help.contains("Enter / Space   对当前软件执行主动作"));
        assert!(packages_help.contains("扩展键"));

        state.open_edit_page(Page::Home);
        let home_help = help_panel_text(&state);
        assert!(home_help.contains("当前页：Edit / Home"));
        assert!(home_help.contains("h / l / Enter   调整当前字段"));
        assert!(home_help.contains("s               保存当前用户的 desktop 分片"));

        state.open_edit_page(Page::Users);
        let users_help = help_panel_text(&state);
        assert!(users_help.contains("当前页：Edit / Users"));
        assert!(users_help.contains("Enter           编辑用户列表"));

        state.open_edit_page(Page::Hosts);
        let hosts_help = help_panel_text(&state);
        assert!(hosts_help.contains("当前页：Edit / Hosts"));
        assert!(hosts_help.contains("Enter           编辑文本或映射字段"));
    }

    #[test]
    fn footer_text_apply_switches_to_ref_input_when_editing_pinned_ref() {
        let mut state = test_state();
        state.open_apply();
        state.deploy_source = DeploySource::RemotePinned;
        state.open_apply_text_edit();

        assert_eq!(
            footer_text(&state),
            "Apply ref | 输入 ref | Enter 确认 | Esc 取消"
        );
    }

    #[test]
    fn footer_text_advanced_switches_between_maintenance_and_wizard_paths() {
        let mut state = test_state();
        state.open_advanced();

        assert_eq!(
            footer_text(&state),
            "Advanced | j/k/J/K 动作 | x/X 执行 | b Apply | ? 帮助"
        );

        state.advanced_deploy_source = DeploySource::RemoteHead;
        state.advanced_action = crate::domain::tui::ActionItem::LaunchDeployWizard;

        assert_eq!(
            footer_text(&state),
            "Advanced | j/k 参数 | h/l/Enter 调整 | J/K 动作 | x/X 执行 | b Apply | ? 帮助"
        );
    }

    #[test]
    fn footer_text_inspect_stays_short_and_action_first() {
        let mut state = test_state();
        state.open_inspect();

        assert_eq!(
            footer_text(&state),
            "Inspect | j/k 命令 | x 执行 | r/d/R 刷新 | ? 帮助"
        );
    }

    #[test]
    fn render_help_overlay_shows_contextual_panel_without_hiding_footer_shell() {
        let mut state = test_state();
        state.open_overview();
        state.help_overlay_visible = true;

        let help = help_panel_text(&state);
        let text = render_view_text(140, 40, |frame| render(frame, &state));

        assert!(help.contains("先看 Overview Summary 和 Current Context"));
        assert!(help.contains("先进入 Apply 预览，不会直接执行"));
        assert!(help.contains("统一键位"));
        assert!(text.contains("Overview Help"));
        assert!(text.contains("Keys"));
    }

    #[test]
    fn help_panel_apply_clarifies_enter_adjusts_and_x_runs() {
        let mut state = test_state();
        state.open_apply();

        let help = help_panel_text(&state);

        assert!(help.contains("先看左侧 Apply Summary 和 Apply Preview"));
        assert!(help.contains("h / l / Enter   只调整当前控件，不会直接执行"));
        assert!(help.contains("x               执行 Apply Current Host"));
        assert!(help.contains("Esc              关闭帮助面板"));
    }

    #[test]
    fn help_panel_inspect_clarifies_enter_space_do_not_run_commands() {
        let mut state = test_state();
        state.open_inspect();

        let help = help_panel_text(&state);

        assert!(help.contains("先看右侧 Inspect Summary"));
        assert!(help.contains("Enter / Space   当前页不执行检查命令"));
        assert!(help.contains("x               执行当前 Inspect 命令"));
    }

    #[test]
    fn help_panel_advanced_maintenance_clarifies_non_executing_enter_space() {
        let mut state = test_state();
        state.open_advanced();

        let help = help_panel_text(&state);

        assert!(help.contains("Enter / Space   当前模式不执行高级动作"));
        assert!(help.contains("x / X           执行当前高级动作"));
    }

    #[test]
    fn help_panel_advanced_wizard_clarifies_enter_adjusts_parameters() {
        let mut state = test_state();
        state.open_advanced();
        state.advanced_action = crate::domain::tui::ActionItem::LaunchDeployWizard;

        let help = help_panel_text(&state);

        assert!(help.contains("h / l / Enter   调整当前参数"));
        assert!(help.contains("x / X           执行当前高级动作"));
    }

    #[test]
    fn render_edit_workspace_keeps_shared_shell_across_all_edit_pages() {
        let expectations = [
            (Page::Packages, "Packages Summary"),
            (Page::Home, "Home (alice)"),
            (Page::Users, "Users (demo)"),
            (Page::Hosts, "Hosts (demo)"),
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
                catalog_workflows_path: PathBuf::from("catalog/workflows.toml"),
                catalog_entries: Vec::<CatalogEntry>::new(),
                catalog_groups: BTreeMap::<String, GroupMeta>::new(),
                catalog_home_options: Vec::<HomeOptionMeta>::new(),
                catalog_workflows: BTreeMap::<String, WorkflowMeta>::new(),
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
            help_overlay_visible: false,
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
