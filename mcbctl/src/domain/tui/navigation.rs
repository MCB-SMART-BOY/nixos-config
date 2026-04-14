#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Page {
    Dashboard,
    Deploy,
    Inspect,
    Users,
    Hosts,
    Packages,
    Home,
    Actions,
}

impl Page {
    pub const ALL: [Page; 8] = [
        Page::Dashboard,
        Page::Deploy,
        Page::Inspect,
        Page::Users,
        Page::Hosts,
        Page::Packages,
        Page::Home,
        Page::Actions,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Page::Dashboard => "Overview",
            Page::Deploy => "Apply",
            Page::Inspect => "Inspect",
            Page::Users => "Users",
            Page::Hosts => "Hosts",
            Page::Packages => "Packages",
            Page::Home => "Home",
            Page::Actions => "Actions",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActionItem {
    FlakeCheck,
    UpdateUpstreamCheck,
    SyncRepoToEtc,
    RebuildCurrentHost,
    FlakeUpdate,
    UpdateUpstreamPins,
    LaunchDeployWizard,
}

impl ActionItem {
    pub const ALL: [ActionItem; 7] = [
        ActionItem::FlakeCheck,
        ActionItem::UpdateUpstreamCheck,
        ActionItem::SyncRepoToEtc,
        ActionItem::RebuildCurrentHost,
        ActionItem::FlakeUpdate,
        ActionItem::UpdateUpstreamPins,
        ActionItem::LaunchDeployWizard,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ActionItem::FlakeCheck => "flake check",
            ActionItem::FlakeUpdate => "flake update",
            ActionItem::UpdateUpstreamCheck => "check upstream pins",
            ActionItem::UpdateUpstreamPins => "update upstream pins",
            ActionItem::SyncRepoToEtc => "sync to /etc/nixos",
            ActionItem::RebuildCurrentHost => "rebuild current host",
            ActionItem::LaunchDeployWizard => "launch deploy wizard",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            ActionItem::FlakeCheck => "运行 flake 自检，确认当前仓库仍可评估与构建。",
            ActionItem::UpdateUpstreamCheck => "检查 Zed / YesPlayMusic 等上游 pin 是否已落后。",
            ActionItem::SyncRepoToEtc => "把当前仓库同步到 /etc/nixos，同时保留根目录硬件配置。",
            ActionItem::RebuildCurrentHost => {
                "对当前主机执行一次标准重建；rootless 下自动退化为 build。"
            }
            ActionItem::FlakeUpdate => "更新当前仓库的 flake.lock。",
            ActionItem::UpdateUpstreamPins => "刷新上游 pin 并回写仓库里的 source.nix。",
            ActionItem::LaunchDeployWizard => {
                "退回到完整部署向导，处理远端来源、初始化与复杂交互。"
            }
        }
    }

    pub fn destination(self) -> ActionDestination {
        match self {
            ActionItem::FlakeCheck | ActionItem::UpdateUpstreamCheck => ActionDestination::Inspect,
            ActionItem::SyncRepoToEtc | ActionItem::RebuildCurrentHost => ActionDestination::Apply,
            ActionItem::FlakeUpdate
            | ActionItem::UpdateUpstreamPins
            | ActionItem::LaunchDeployWizard => ActionDestination::Advanced,
        }
    }

    pub fn group_label(self) -> &'static str {
        match self {
            ActionItem::FlakeCheck => "Repo Checks",
            ActionItem::UpdateUpstreamCheck => "Upstream Pins",
            ActionItem::SyncRepoToEtc | ActionItem::RebuildCurrentHost => "Manual Apply Helpers",
            ActionItem::FlakeUpdate | ActionItem::UpdateUpstreamPins => "Repository Maintenance",
            ActionItem::LaunchDeployWizard => "Deploy",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActionDestination {
    Inspect,
    Apply,
    Advanced,
}

impl ActionDestination {
    pub fn label(self) -> &'static str {
        match self {
            ActionDestination::Inspect => "Inspect",
            ActionDestination::Apply => "Apply",
            ActionDestination::Advanced => "Advanced",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_titles_expose_overview_and_apply_labels() {
        assert_eq!(Page::Dashboard.title(), "Overview");
        assert_eq!(Page::Deploy.title(), "Apply");
        assert_eq!(Page::Inspect.title(), "Inspect");
        assert_eq!(Page::Users.title(), "Users");
        assert_eq!(Page::Actions.title(), "Actions");
    }
}
