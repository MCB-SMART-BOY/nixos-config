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

#[derive(Clone, Debug)]
pub struct HostManagedSettings {
    pub primary_user: String,
    pub users: Vec<String>,
    pub admin_users: Vec<String>,
    pub host_role: String,
    pub user_linger: bool,
    pub cache_profile: String,
    pub custom_substituters: Vec<String>,
    pub custom_trusted_public_keys: Vec<String>,
    pub proxy_mode: String,
    pub proxy_url: String,
    pub tun_interface: String,
    pub tun_interfaces: Vec<String>,
    pub enable_proxy_dns: bool,
    pub proxy_dns_addr: String,
    pub proxy_dns_port: u16,
    pub per_user_tun_enable: bool,
    pub per_user_tun_compat_global_service_socket: bool,
    pub per_user_tun_redirect_dns: bool,
    pub per_user_tun_interfaces: BTreeMap<String, String>,
    pub per_user_tun_dns_ports: BTreeMap<String, u16>,
    pub per_user_tun_table_base: i64,
    pub per_user_tun_priority_base: i64,
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

impl Default for HostManagedSettings {
    fn default() -> Self {
        Self {
            primary_user: String::new(),
            users: Vec::new(),
            admin_users: Vec::new(),
            host_role: "desktop".to_string(),
            user_linger: false,
            cache_profile: "cn".to_string(),
            custom_substituters: Vec::new(),
            custom_trusted_public_keys: Vec::new(),
            proxy_mode: "off".to_string(),
            proxy_url: String::new(),
            tun_interface: String::new(),
            tun_interfaces: Vec::new(),
            enable_proxy_dns: true,
            proxy_dns_addr: "127.0.0.1".to_string(),
            proxy_dns_port: 53,
            per_user_tun_enable: false,
            per_user_tun_compat_global_service_socket: true,
            per_user_tun_redirect_dns: false,
            per_user_tun_interfaces: BTreeMap::new(),
            per_user_tun_dns_ports: BTreeMap::new(),
            per_user_tun_table_base: 1000,
            per_user_tun_priority_base: 10000,
            gpu_mode: "igpu".to_string(),
            gpu_igpu_vendor: "intel".to_string(),
            gpu_prime_mode: "offload".to_string(),
            gpu_intel_bus: None,
            gpu_amd_bus: None,
            gpu_nvidia_bus: None,
            gpu_nvidia_open: false,
            gpu_specialisations_enable: false,
            gpu_specialisation_modes: Vec::new(),
            docker_enable: false,
            libvirtd_enable: false,
        }
    }
}
