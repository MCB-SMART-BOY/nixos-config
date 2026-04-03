#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Page {
    Dashboard,
    Deploy,
    Users,
    Hosts,
    Packages,
    Home,
    Actions,
}

impl Page {
    pub const ALL: [Page; 7] = [
        Page::Dashboard,
        Page::Deploy,
        Page::Users,
        Page::Hosts,
        Page::Packages,
        Page::Home,
        Page::Actions,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Page::Dashboard => "Dashboard",
            Page::Deploy => "Deploy",
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
    FlakeUpdate,
    UpdateUpstreamCheck,
    UpdateUpstreamPins,
    SyncRepoToEtc,
    RebuildCurrentHost,
    LaunchDeployWizard,
}

impl ActionItem {
    pub const ALL: [ActionItem; 7] = [
        ActionItem::FlakeCheck,
        ActionItem::FlakeUpdate,
        ActionItem::UpdateUpstreamCheck,
        ActionItem::UpdateUpstreamPins,
        ActionItem::SyncRepoToEtc,
        ActionItem::RebuildCurrentHost,
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
            ActionItem::FlakeUpdate => "更新当前仓库的 flake.lock。",
            ActionItem::UpdateUpstreamCheck => "检查 Zed / YesPlayMusic 等上游 pin 是否已落后。",
            ActionItem::UpdateUpstreamPins => "刷新上游 pin 并回写仓库里的 source.nix。",
            ActionItem::SyncRepoToEtc => "把当前仓库同步到 /etc/nixos，同时保留根目录硬件配置。",
            ActionItem::RebuildCurrentHost => {
                "对当前主机执行一次标准重建；rootless 下自动退化为 build。"
            }
            ActionItem::LaunchDeployWizard => {
                "退回到完整部署向导，处理远端来源、初始化与复杂交互。"
            }
        }
    }
}
