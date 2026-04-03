use std::collections::BTreeMap;

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
