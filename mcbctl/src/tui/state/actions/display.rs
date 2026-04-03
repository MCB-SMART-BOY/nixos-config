use super::*;

impl AppState {
    pub fn next_action_item(&mut self) {
        self.actions_focus = (self.actions_focus + 1) % ActionItem::ALL.len();
    }

    pub fn previous_action_item(&mut self) {
        self.actions_focus = if self.actions_focus == 0 {
            ActionItem::ALL.len() - 1
        } else {
            self.actions_focus - 1
        };
    }

    pub fn current_action_item(&self) -> ActionItem {
        ActionItem::ALL[self.actions_focus]
    }

    pub fn actions_rows(&self) -> Vec<(String, String)> {
        ActionItem::ALL
            .iter()
            .map(|item| {
                (
                    item.label().to_string(),
                    if self.action_available(*item) {
                        "可执行".to_string()
                    } else {
                        "需切换场景".to_string()
                    },
                )
            })
            .collect()
    }

    pub fn actions_summary_lines(&self) -> Vec<String> {
        let action = self.current_action_item();
        let mut lines = vec![
            format!("当前动作：{}", action.label()),
            format!("说明：{}", action.description()),
            format!("当前仓库：{}", self.context.repo_root.display()),
            format!("/etc/nixos：{}", self.context.etc_root.display()),
            format!("当前主机：{}", self.target_host),
            format!(
                "权限：{}",
                match self.context.privilege_mode.as_str() {
                    "root" => "root",
                    "sudo-session" => "sudo session",
                    "sudo-available" => "sudo available",
                    _ => "rootless",
                }
            ),
        ];

        if let Some(preview) = self.action_command_preview(action) {
            lines.push(format!("命令预览：{preview}"));
        }
        if self.action_available(action) {
            lines.push("状态：当前环境可以直接执行".to_string());
        } else {
            lines.push("状态：当前环境不适合直接执行；请改用 Deploy 页或切换权限".to_string());
        }

        lines.push(String::new());
        lines.push("当前页说明：".to_string());
        lines.push("- 这里只放高频维护动作，不处理复杂初始化向导".to_string());
        lines.push("- 直接执行外部命令前，会临时退出 TUI，执行完成后再返回".to_string());
        lines.push("- 如需远端来源、模板生成、复杂交互，请使用 deploy wizard".to_string());
        lines
    }
}
