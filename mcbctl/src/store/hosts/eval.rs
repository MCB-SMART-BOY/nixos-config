use crate::domain::tui::HostManagedSettings;
use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

#[derive(Clone, Debug, Default)]
pub struct LoadedHostSettings {
    pub settings_by_name: BTreeMap<String, HostManagedSettings>,
    pub errors_by_name: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct EvaluatedNixSettings {
    #[serde(default, rename = "cacheProfile")]
    cache_profile: String,
    #[serde(default, rename = "customSubstituters")]
    custom_substituters: Vec<String>,
    #[serde(default, rename = "customTrustedPublicKeys")]
    custom_trusted_public_keys: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct EvaluatedPerUserTun {
    #[serde(default)]
    enable: bool,
    #[serde(default, rename = "redirectDns")]
    redirect_dns: bool,
    #[serde(default, rename = "compatGlobalServiceSocket")]
    compat_global_service_socket: bool,
    #[serde(default)]
    interfaces: BTreeMap<String, String>,
    #[serde(default, rename = "dnsPorts")]
    dns_ports: BTreeMap<String, u16>,
    #[serde(default, rename = "tableBase")]
    table_base: i64,
    #[serde(default, rename = "priorityBase")]
    priority_base: i64,
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
    #[serde(default, rename = "tunInterfaces")]
    tun_interfaces: Vec<String>,
    #[serde(default, rename = "enableProxyDns")]
    enable_proxy_dns: bool,
    #[serde(default, rename = "proxyDnsAddr")]
    proxy_dns_addr: String,
    #[serde(default, rename = "proxyDnsPort")]
    proxy_dns_port: u16,
    #[serde(default, rename = "perUserTun")]
    per_user_tun: EvaluatedPerUserTun,
    #[serde(default)]
    hardware: EvaluatedHardware,
    #[serde(default)]
    virtualisation: EvaluatedVirtualisation,
}

pub fn load_host_settings(repo_root: &Path, hosts: &[String]) -> LoadedHostSettings {
    let mut loaded = LoadedHostSettings::default();

    for host in hosts {
        match load_single_host_settings(repo_root, host) {
            Ok(settings) => {
                loaded.settings_by_name.insert(host.clone(), settings);
            }
            Err(err) => {
                loaded.errors_by_name.insert(host.clone(), err.to_string());
            }
        }
    }

    loaded
}

fn load_single_host_settings(repo_root: &Path, host: &str) -> Result<HostManagedSettings> {
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

    let output = Command::new("nix")
        .args(args)
        .output()
        .with_context(|| format!("failed to run nix eval for host {host}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if stderr.is_empty() {
            bail!(
                "nix eval for host {host} exited with {}",
                output.status.code().unwrap_or(1)
            );
        }
        bail!("nix eval for host {host} failed: {stderr}");
    }
    let parsed =
        serde_json::from_str::<EvaluatedMcbConfig>(&String::from_utf8_lossy(&output.stdout))
            .with_context(|| format!("failed to parse evaluated host config for {host}"))?;

    Ok(host_settings_from_eval(parsed))
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
        custom_substituters: dedup_string_list(value.nix.custom_substituters),
        custom_trusted_public_keys: dedup_string_list(value.nix.custom_trusted_public_keys),
        proxy_mode: default_if_empty(value.proxy_mode, "off"),
        proxy_url: value.proxy_url,
        tun_interface: value.tun_interface,
        tun_interfaces: dedup_string_list(value.tun_interfaces),
        enable_proxy_dns: value.enable_proxy_dns,
        proxy_dns_addr: default_if_empty(value.proxy_dns_addr, "127.0.0.1"),
        proxy_dns_port: default_port(value.proxy_dns_port, 53),
        per_user_tun_enable: value.per_user_tun.enable,
        per_user_tun_compat_global_service_socket: value.per_user_tun.compat_global_service_socket,
        per_user_tun_redirect_dns: value.per_user_tun.redirect_dns,
        per_user_tun_interfaces: value.per_user_tun.interfaces,
        per_user_tun_dns_ports: value.per_user_tun.dns_ports,
        per_user_tun_table_base: default_i64(value.per_user_tun.table_base, 1000),
        per_user_tun_priority_base: default_i64(value.per_user_tun.priority_base, 10000),
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

fn default_port(value: u16, fallback: u16) -> u16 {
    if value == 0 { fallback } else { value }
}

fn default_i64(value: i64, fallback: i64) -> i64 {
    if value == 0 { fallback } else { value }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_settings_from_eval_preserves_extended_network_fields() {
        let mut interfaces = BTreeMap::new();
        interfaces.insert("alice".to_string(), "tun-alice".to_string());
        let mut dns_ports = BTreeMap::new();
        dns_ports.insert("alice".to_string(), 1053);

        let settings = host_settings_from_eval(EvaluatedMcbConfig {
            user: "alice".to_string(),
            users: vec!["alice".to_string()],
            admin_users: vec!["alice".to_string()],
            host_role: "desktop".to_string(),
            user_linger: true,
            nix: EvaluatedNixSettings {
                cache_profile: "custom".to_string(),
                custom_substituters: vec!["https://cache.example.org".to_string()],
                custom_trusted_public_keys: vec!["cache.example.org-1:abc".to_string()],
            },
            proxy_mode: "tun".to_string(),
            proxy_url: String::new(),
            tun_interface: "tun0".to_string(),
            tun_interfaces: vec!["mihomo".to_string(), "mihomo".to_string()],
            enable_proxy_dns: false,
            proxy_dns_addr: "127.0.0.1".to_string(),
            proxy_dns_port: 5353,
            per_user_tun: EvaluatedPerUserTun {
                enable: true,
                redirect_dns: true,
                compat_global_service_socket: false,
                interfaces,
                dns_ports,
                table_base: 2000,
                priority_base: 30000,
            },
            hardware: EvaluatedHardware {
                gpu: EvaluatedGpu {
                    mode: "hybrid".to_string(),
                    igpu_vendor: "amd".to_string(),
                    prime: EvaluatedGpuPrime {
                        mode: "sync".to_string(),
                        intel_bus_id: None,
                        amdgpu_bus_id: Some("PCI:4:0:0".to_string()),
                        nvidia_bus_id: Some("PCI:1:0:0".to_string()),
                    },
                    nvidia: EvaluatedGpuNvidia { open: true },
                    specialisations: EvaluatedGpuSpecialisations {
                        enable: true,
                        modes: vec!["igpu".to_string(), "hybrid".to_string(), "igpu".to_string()],
                    },
                },
            },
            virtualisation: EvaluatedVirtualisation {
                docker: EvaluatedVirtSwitch { enable: true },
                libvirtd: EvaluatedVirtSwitch { enable: true },
            },
        });

        assert_eq!(settings.cache_profile, "custom");
        assert_eq!(
            settings.custom_substituters,
            vec!["https://cache.example.org".to_string()]
        );
        assert_eq!(
            settings.custom_trusted_public_keys,
            vec!["cache.example.org-1:abc".to_string()]
        );
        assert_eq!(settings.tun_interfaces, vec!["mihomo".to_string()]);
        assert!(!settings.enable_proxy_dns);
        assert!(settings.per_user_tun_enable);
        assert!(!settings.per_user_tun_compat_global_service_socket);
        assert!(settings.per_user_tun_redirect_dns);
        assert_eq!(settings.per_user_tun_table_base, 2000);
        assert_eq!(settings.per_user_tun_priority_base, 30000);
        assert_eq!(
            settings
                .per_user_tun_interfaces
                .get("alice")
                .map(String::as_str),
            Some("tun-alice")
        );
        assert_eq!(settings.per_user_tun_dns_ports.get("alice"), Some(&1053));
        assert_eq!(settings.gpu_mode, "hybrid");
        assert_eq!(settings.gpu_igpu_vendor, "amd");
        assert_eq!(settings.gpu_prime_mode, "sync");
        assert_eq!(settings.gpu_amd_bus.as_deref(), Some("PCI:4:0:0"));
        assert_eq!(settings.gpu_nvidia_bus.as_deref(), Some("PCI:1:0:0"));
        assert!(settings.gpu_nvidia_open);
        assert!(settings.gpu_specialisations_enable);
        assert_eq!(
            settings.gpu_specialisation_modes,
            vec!["igpu".to_string(), "hybrid".to_string()]
        );
        assert!(settings.docker_enable);
        assert!(settings.libvirtd_enable);
    }
}
