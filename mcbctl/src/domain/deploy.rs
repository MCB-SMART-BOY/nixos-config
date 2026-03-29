use super::tui::{DeployAction, DeploySource, DeployTask};

#[derive(Clone, Debug)]
pub struct DeployPlan {
    pub task: DeployTask,
    pub detected_host: Option<String>,
    pub target_host: String,
    pub source: DeploySource,
    pub source_detail: Option<String>,
    pub action: DeployAction,
    pub notes: Vec<String>,
}

impl DeployPlan {
    pub fn summary_lines(&self) -> Vec<String> {
        let mut lines = vec![format!("任务：{}", self.task.label())];

        if let Some(host) = self.detected_host.as_deref()
            && !host.trim().is_empty()
        {
            lines.push(format!("检测 hostname：{host}"));
        }

        if !self.target_host.trim().is_empty() {
            lines.push(format!("部署目标：{}", self.target_host));
        }

        let source_label = if let Some(detail) = self.source_detail.as_deref() {
            if detail.trim().is_empty() {
                self.source.label().to_string()
            } else {
                format!("{} ({detail})", self.source.label())
            }
        } else {
            self.source.label().to_string()
        };
        lines.push(format!("来源：{source_label}"));
        lines.push(format!("动作：{}", self.action.label()));

        lines.extend(self.notes.iter().cloned());
        lines
    }
}
