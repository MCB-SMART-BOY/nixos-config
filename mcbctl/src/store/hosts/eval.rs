use crate::domain::tui::HostManagedSettings;
use crate::run_capture;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Clone, Debug, Default, Deserialize)]
struct EvaluatedNixSettings {
    #[serde(default, rename = "cacheProfile")]
    cache_profile: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct EvaluatedPerUserTun {
    #[serde(default)]
    enable: bool,
    #[serde(default)]
    interfaces: BTreeMap<String, String>,
    #[serde(default, rename = "dnsPorts")]
    dns_ports: BTreeMap<String, u16>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct EvaluatedGpuPrime {
    #[serde(default)]
    mode: String,
    #[serde(default, rename = "intelBusId")]
    intel_bus_id: Option<String>,
    #[serde(default, rename = "amdgpuBusId")]
    amdgpu_bus_id: Option<String>,
    #[serde(default, rename = "nvidiaBusId")]
    nvidia_bus_id: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct EvaluatedGpuNvidia {
    #[serde(default)]
    open: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct EvaluatedGpuSpecialisations {
    #[serde(default)]
    enable: bool,
    #[serde(default)]
    modes: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct EvaluatedGpu {
    #[serde(default)]
    mode: String,
    #[serde(default, rename = "igpuVendor")]
    igpu_vendor: String,
    #[serde(default)]
    prime: EvaluatedGpuPrime,
    #[serde(default)]
    nvidia: EvaluatedGpuNvidia,
    #[serde(default)]
    specialisations: EvaluatedGpuSpecialisations,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct EvaluatedHardware {
    #[serde(default)]
    gpu: EvaluatedGpu,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct EvaluatedVirtSwitch {
    #[serde(default)]
    enable: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct EvaluatedVirtualisation {
    #[serde(default)]
    docker: EvaluatedVirtSwitch,
    #[serde(default)]
    libvirtd: EvaluatedVirtSwitch,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct EvaluatedMcbConfig {
    #[serde(default)]
    user: String,
    #[serde(default)]
    users: Vec<String>,
    #[serde(default, rename = "adminUsers")]
    admin_users: Vec<String>,
    #[serde(default, rename = "hostRole")]
    host_role: String,
    #[serde(default, rename = "userLinger")]
    user_linger: bool,
    #[serde(default)]
    nix: EvaluatedNixSettings,
    #[serde(default, rename = "proxyMode")]
    proxy_mode: String,
    #[serde(default, rename = "proxyUrl")]
    proxy_url: String,
    #[serde(default, rename = "tunInterface")]
    tun_interface: String,
    #[serde(default, rename = "perUserTun")]
    per_user_tun: EvaluatedPerUserTun,
    #[serde(default)]
    hardware: EvaluatedHardware,
    #[serde(default)]
    virtualisation: EvaluatedVirtualisation,
}

pub fn load_host_settings(
    repo_root: &Path,
    hosts: &[String],
) -> BTreeMap<String, HostManagedSettings> {
    let mut settings = BTreeMap::new();

    for host in hosts {
        settings.insert(host.clone(), load_single_host_settings(repo_root, host));
    }

    settings
}

fn load_single_host_settings(repo_root: &Path, host: &str) -> HostManagedSettings {
    let flake_ref = format!(
        "path:{}#nixosConfigurations.{host}.config.mcb",
        repo_root.display()
    );
    let args = [
        "--extra-experimental-features",
        "nix-command flakes",
        "eval",
        "--json",
        flake_ref.as_str(),
    ];

    let Ok(output) = run_capture("nix", &args) else {
        return HostManagedSettings::default();
    };
    let Ok(parsed) = serde_json::from_str::<EvaluatedMcbConfig>(&output) else {
        return HostManagedSettings::default();
    };

    host_settings_from_eval(parsed)
}

fn host_settings_from_eval(value: EvaluatedMcbConfig) -> HostManagedSettings {
    let mut users = dedup_string_list(if value.users.is_empty() {
        if value.user.trim().is_empty() {
            Vec::new()
        } else {
            vec![value.user.clone()]
        }
    } else {
        value.users
    });

    let mut primary_user = value.user;
    if primary_user.trim().is_empty() {
        primary_user = users.first().cloned().unwrap_or_default();
    } else if !users.contains(&primary_user) {
        users.insert(0, primary_user.clone());
    }

    let admin_users = dedup_string_list(if value.admin_users.is_empty() {
        if primary_user.trim().is_empty() {
            Vec::new()
        } else {
            vec![primary_user.clone()]
        }
    } else {
        value.admin_users
    });

    HostManagedSettings {
        primary_user,
        users,
        admin_users,
        host_role: default_if_empty(value.host_role, "desktop"),
        user_linger: value.user_linger,
        cache_profile: default_if_empty(value.nix.cache_profile, "cn"),
        proxy_mode: default_if_empty(value.proxy_mode, "off"),
        proxy_url: value.proxy_url,
        tun_interface: value.tun_interface,
        per_user_tun_enable: value.per_user_tun.enable,
        per_user_tun_interfaces: value.per_user_tun.interfaces,
        per_user_tun_dns_ports: value.per_user_tun.dns_ports,
        gpu_mode: default_if_empty(value.hardware.gpu.mode, "igpu"),
        gpu_igpu_vendor: default_if_empty(value.hardware.gpu.igpu_vendor, "intel"),
        gpu_prime_mode: default_if_empty(value.hardware.gpu.prime.mode, "offload"),
        gpu_intel_bus: value.hardware.gpu.prime.intel_bus_id,
        gpu_amd_bus: value.hardware.gpu.prime.amdgpu_bus_id,
        gpu_nvidia_bus: value.hardware.gpu.prime.nvidia_bus_id,
        gpu_nvidia_open: value.hardware.gpu.nvidia.open,
        gpu_specialisations_enable: value.hardware.gpu.specialisations.enable,
        gpu_specialisation_modes: dedup_string_list(value.hardware.gpu.specialisations.modes),
        docker_enable: value.virtualisation.docker.enable,
        libvirtd_enable: value.virtualisation.libvirtd.enable,
    }
}

fn dedup_string_list(items: Vec<String>) -> Vec<String> {
    let mut output = Vec::new();
    for item in items {
        if !output.contains(&item) {
            output.push(item);
        }
    }
    output
}

fn default_if_empty(value: String, fallback: &str) -> String {
    if value.trim().is_empty() {
        fallback.to_string()
    } else {
        value
    }
}
