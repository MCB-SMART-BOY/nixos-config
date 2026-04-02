use crate::tui::state::AppState;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

pub(super) fn render(frame: &mut Frame, area: Rect, state: &AppState) {
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
         - Users 页现在写 users.nix，Hosts 页现在写 network/gpu/virtualization 分片\n\
         - Deploy / Actions 已经接入真实执行链\n\n\
         下一步:\n\
         - 继续拆分超长状态文件与 deploy 巨石文件\n\
         - 继续把远端来源准备从 mcb-deploy 抽到共享执行层\n\
         - Home 页继续把 session/mime 等结构化设置接入分片\n\
         - Packages / Home 扩展更多 metadata 驱动字段",
    )
    .block(Block::default().borders(Borders::ALL).title("Roadmap"))
    .wrap(Wrap { trim: false });
    frame.render_widget(right, chunks[1]);
}
