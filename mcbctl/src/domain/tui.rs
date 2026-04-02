use serde::Deserialize;
use std::collections::BTreeMap;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeployTask {
    DirectDeploy,
    AdjustStructure,
    Maintenance,
}

impl DeployTask {
    pub const ALL: [DeployTask; 3] = [
        DeployTask::DirectDeploy,
        DeployTask::AdjustStructure,
        DeployTask::Maintenance,
    ];

    pub fn label(self) -> &'static str {
        match self {
            DeployTask::DirectDeploy => "直接部署这台机器",
            DeployTask::AdjustStructure => "调整主机结构与用户",
            DeployTask::Maintenance => "维护与诊断",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeploySource {
    CurrentRepo,
    EtcNixos,
    RemotePinned,
    RemoteHead,
}

impl DeploySource {
    pub const ALL: [DeploySource; 4] = [
        DeploySource::CurrentRepo,
        DeploySource::EtcNixos,
        DeploySource::RemotePinned,
        DeploySource::RemoteHead,
    ];

    pub fn label(self) -> &'static str {
        match self {
            DeploySource::CurrentRepo => "当前仓库",
            DeploySource::EtcNixos => "/etc/nixos",
            DeploySource::RemotePinned => "远端固定版本",
            DeploySource::RemoteHead => "远端最新版本",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeployAction {
    Switch,
    Test,
    Boot,
    Build,
}

impl DeployAction {
    pub const ALL: [DeployAction; 4] = [
        DeployAction::Switch,
        DeployAction::Test,
        DeployAction::Boot,
        DeployAction::Build,
    ];

    pub fn label(self) -> &'static str {
        match self {
            DeployAction::Switch => "switch",
            DeployAction::Test => "test",
            DeployAction::Boot => "boot",
            DeployAction::Build => "build",
        }
    }

    pub fn rebuild_mode(self) -> &'static str {
        match self {
            DeployAction::Switch => "switch",
            DeployAction::Test => "test",
            DeployAction::Boot => "boot",
            DeployAction::Build => "build",
        }
    }

    pub fn from_rebuild_mode(mode: &str) -> Self {
        match mode {
            "test" => DeployAction::Test,
            "boot" => DeployAction::Boot,
            "build" => DeployAction::Build,
            _ => DeployAction::Switch,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PackageTextMode {
    Search,
    CreateGroup,
    RenameGroup,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PackageDataMode {
    Local,
    Search,
}

impl PackageDataMode {
    pub fn label(self) -> &'static str {
        match self {
            PackageDataMode::Local => "本地覆盖/已声明",
            PackageDataMode::Search => "nixpkgs 搜索",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub enum ManagedToggle {
    #[default]
    Inherit,
    Enabled,
    Disabled,
}

impl ManagedToggle {
    pub const ALL: [ManagedToggle; 3] = [
        ManagedToggle::Inherit,
        ManagedToggle::Enabled,
        ManagedToggle::Disabled,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ManagedToggle::Inherit => "跟随现有",
            ManagedToggle::Enabled => "强制启用",
            ManagedToggle::Disabled => "强制禁用",
        }
    }

    pub fn marker(self) -> &'static str {
        match self {
            ManagedToggle::Inherit => "inherit",
            ManagedToggle::Enabled => "enabled",
            ManagedToggle::Disabled => "disabled",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub enum ManagedBarProfile {
    #[default]
    Inherit,
    Default,
    None,
}

impl ManagedBarProfile {
    pub const ALL: [ManagedBarProfile; 3] = [
        ManagedBarProfile::Inherit,
        ManagedBarProfile::Default,
        ManagedBarProfile::None,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ManagedBarProfile::Inherit => "跟随现有",
            ManagedBarProfile::Default => "default",
            ManagedBarProfile::None => "none",
        }
    }

    pub fn marker(self) -> &'static str {
        match self {
            ManagedBarProfile::Inherit => "inherit",
            ManagedBarProfile::Default => "default",
            ManagedBarProfile::None => "none",
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct HomeManagedSettings {
    pub bar_profile: ManagedBarProfile,
    pub enable_zed_entry: ManagedToggle,
    pub enable_yesplaymusic_entry: ManagedToggle,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UsersTextMode {
    ManagedUsers,
    AdminUsers,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostsTextMode {
    ProxyUrl,
    TunInterface,
    PerUserTunInterfaces,
    PerUserTunDnsPorts,
    IntelBusId,
    AmdBusId,
    NvidiaBusId,
    SpecialisationModes,
}

#[derive(Clone, Debug, Default)]
pub struct HostManagedSettings {
    pub primary_user: String,
    pub users: Vec<String>,
    pub admin_users: Vec<String>,
    pub host_role: String,
    pub user_linger: bool,
    pub cache_profile: String,
    pub proxy_mode: String,
    pub proxy_url: String,
    pub tun_interface: String,
    pub per_user_tun_enable: bool,
    pub per_user_tun_interfaces: BTreeMap<String, String>,
    pub per_user_tun_dns_ports: BTreeMap<String, u16>,
    pub gpu_mode: String,
    pub gpu_igpu_vendor: String,
    pub gpu_prime_mode: String,
    pub gpu_intel_bus: Option<String>,
    pub gpu_amd_bus: Option<String>,
    pub gpu_nvidia_bus: Option<String>,
    pub gpu_nvidia_open: bool,
    pub gpu_specialisations_enable: bool,
    pub gpu_specialisation_modes: Vec<String>,
    pub docker_enable: bool,
    pub libvirtd_enable: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CatalogEntry {
    pub id: String,
    pub name: String,
    pub category: String,
    #[serde(default)]
    pub group: Option<String>,
    pub expr: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub platforms: Vec<String>,
    #[serde(default)]
    pub desktop_entry_flag: Option<String>,
}

impl CatalogEntry {
    pub fn group_key(&self) -> &str {
        self.group.as_deref().unwrap_or(&self.category)
    }

    pub fn source_label(&self) -> &str {
        self.source.as_deref().unwrap_or("nixpkgs")
    }

    pub fn matches(&self, category: Option<&str>, query: &str) -> bool {
        if let Some(category) = category
            && self.category != category
        {
            return false;
        }

        let query = query.trim().to_lowercase();
        if query.is_empty() {
            return true;
        }

        let haystack = format!(
            "{} {} {} {} {} {} {} {} {}",
            self.id,
            self.name,
            self.category,
            self.group_key(),
            self.expr,
            self.description.as_deref().unwrap_or(""),
            self.source_label(),
            self.keywords.join(" "),
            self.platforms.join(" ")
        )
        .to_lowercase();
        haystack.contains(&query)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GroupMeta {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub order: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct HomeOptionMeta {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_home_option_area")]
    pub area: String,
    #[serde(default)]
    pub order: u32,
}

fn default_home_option_area() -> String {
    "desktop".to_string()
}
