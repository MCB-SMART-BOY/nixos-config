use crate::domain::tui::HostManagedSettings;
use crate::{run_capture, write_file_atomic};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

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

pub fn ensure_managed_host_layout(managed_dir: &Path) -> Result<()> {
    fs::create_dir_all(managed_dir)
        .with_context(|| format!("failed to create {}", managed_dir.display()))?;

    let default_path = managed_dir.join("default.nix");
    write_file_atomic(&default_path, &render_managed_host_default_file())?;

    Ok(())
}

pub fn managed_host_users_path(repo_root: &Path, host: &str) -> PathBuf {
    repo_root.join("hosts").join(host).join("managed/users.nix")
}

pub fn managed_host_network_path(repo_root: &Path, host: &str) -> PathBuf {
    repo_root
        .join("hosts")
        .join(host)
        .join("managed/network.nix")
}

pub fn managed_host_gpu_path(repo_root: &Path, host: &str) -> PathBuf {
    repo_root.join("hosts").join(host).join("managed/gpu.nix")
}

pub fn managed_host_virtualization_path(repo_root: &Path, host: &str) -> PathBuf {
    repo_root
        .join("hosts")
        .join(host)
        .join("managed/virtualization.nix")
}

pub fn write_host_users_fragment(
    managed_dir: &Path,
    settings: &HostManagedSettings,
) -> Result<PathBuf> {
    let path = managed_dir.join("users.nix");
    write_file_atomic(&path, &render_host_users_file(settings))?;
    Ok(path)
}

pub fn write_host_runtime_fragments(
    managed_dir: &Path,
    settings: &HostManagedSettings,
) -> Result<Vec<PathBuf>> {
    let network_path = managed_dir.join("network.nix");
    write_file_atomic(&network_path, &render_host_network_file(settings))?;

    let gpu_path = managed_dir.join("gpu.nix");
    write_file_atomic(&gpu_path, &render_host_gpu_file(settings))?;

    let virtualization_path = managed_dir.join("virtualization.nix");
    write_file_atomic(
        &virtualization_path,
        &render_host_virtualization_file(settings),
    )?;

    Ok(vec![network_path, gpu_path, virtualization_path])
}

fn render_host_users_file(settings: &HostManagedSettings) -> String {
    let users_list = settings
        .users
        .iter()
        .map(|user| format!(" \"{user}\""))
        .collect::<String>();
    let admin_list = settings
        .admin_users
        .iter()
        .map(|user| format!(" \"{user}\""))
        .collect::<String>();
    let lines = vec![
        "# 机器管理的用户结构分片（由 mcbctl 维护）。".to_string(),
        "# 适合放主用户、用户列表、管理员和主机角色。".to_string(),
        "".to_string(),
        "{ lib, ... }:".to_string(),
        "".to_string(),
        "{".to_string(),
        format!(
            "  mcb.user = lib.mkForce {};",
            nix_string_literal(&settings.primary_user)
        ),
        format!("  mcb.users = lib.mkForce [{users_list} ];"),
        format!("  mcb.adminUsers = lib.mkForce [{admin_list} ];"),
        format!(
            "  mcb.hostRole = lib.mkForce {};",
            nix_string_literal(&settings.host_role)
        ),
        format!("  mcb.userLinger = lib.mkForce {};", settings.user_linger),
        "}".to_string(),
        "".to_string(),
    ];

    lines.join("\n")
}

fn render_host_network_file(settings: &HostManagedSettings) -> String {
    let mut lines = vec![
        "# 机器管理的网络/TUN 分片（由 mcbctl 维护）。".to_string(),
        "# 适合放缓存源策略、代理模式和 per-user TUN 结构化设置。".to_string(),
        "".to_string(),
        "{ lib, ... }:".to_string(),
        "".to_string(),
        "{".to_string(),
        format!(
            "  mcb.nix.cacheProfile = lib.mkForce {};",
            nix_string_literal(&settings.cache_profile)
        ),
        format!(
            "  mcb.proxyMode = lib.mkForce {};",
            nix_string_literal(&settings.proxy_mode)
        ),
        format!(
            "  mcb.proxyUrl = lib.mkForce {};",
            nix_string_literal(&settings.proxy_url)
        ),
        format!(
            "  mcb.tunInterface = lib.mkForce {};",
            nix_string_literal(&settings.tun_interface)
        ),
        format!(
            "  mcb.perUserTun.enable = lib.mkForce {};",
            settings.per_user_tun_enable
        ),
    ];

    lines.push("  mcb.perUserTun.interfaces = lib.mkForce {".to_string());
    for (user, iface) in &settings.per_user_tun_interfaces {
        lines.push(format!("    {user} = {};", nix_string_literal(iface)));
    }
    lines.push("  };".to_string());

    lines.push("  mcb.perUserTun.dnsPorts = lib.mkForce {".to_string());
    for (user, port) in &settings.per_user_tun_dns_ports {
        lines.push(format!("    {user} = {port};"));
    }
    lines.push("  };".to_string());
    lines.push("}".to_string());
    lines.push("".to_string());

    lines.join("\n")
}

fn render_host_gpu_file(settings: &HostManagedSettings) -> String {
    let gpu_modes = settings
        .gpu_specialisation_modes
        .iter()
        .map(|mode| format!(" \"{mode}\""))
        .collect::<String>();

    let mut lines = vec![
        "# 机器管理的 GPU 分片（由 mcbctl 维护）。".to_string(),
        "# 适合放 GPU 模式、PRIME busId 和 specialisations。".to_string(),
        "".to_string(),
        "{ lib, ... }:".to_string(),
        "".to_string(),
        "{".to_string(),
    ];

    lines.push(format!(
        "  mcb.hardware.gpu.mode = lib.mkForce {};",
        nix_string_literal(&settings.gpu_mode)
    ));
    lines.push(format!(
        "  mcb.hardware.gpu.igpuVendor = lib.mkForce {};",
        nix_string_literal(&settings.gpu_igpu_vendor)
    ));
    lines.push("  mcb.hardware.gpu.prime = lib.mkForce {".to_string());
    lines.push(format!(
        "    mode = {};",
        nix_string_literal(&settings.gpu_prime_mode)
    ));
    lines.push(format!(
        "    intelBusId = {};",
        nix_nullable_string(settings.gpu_intel_bus.as_deref())
    ));
    lines.push(format!(
        "    amdgpuBusId = {};",
        nix_nullable_string(settings.gpu_amd_bus.as_deref())
    ));
    lines.push(format!(
        "    nvidiaBusId = {};",
        nix_nullable_string(settings.gpu_nvidia_bus.as_deref())
    ));
    lines.push("  };".to_string());
    lines.push(format!(
        "  mcb.hardware.gpu.nvidia.open = lib.mkForce {};",
        settings.gpu_nvidia_open
    ));
    lines.push(format!(
        "  mcb.hardware.gpu.specialisations.enable = lib.mkForce {};",
        settings.gpu_specialisations_enable
    ));
    lines.push(format!(
        "  mcb.hardware.gpu.specialisations.modes = lib.mkForce [{gpu_modes} ];"
    ));
    lines.push("}".to_string());
    lines.push("".to_string());

    lines.join("\n")
}

fn render_host_virtualization_file(settings: &HostManagedSettings) -> String {
    let mut lines = vec![
        "# 机器管理的虚拟化分片（由 mcbctl 维护）。".to_string(),
        "# 适合放 Docker / Libvirt 这类主机级运行时能力。".to_string(),
        "".to_string(),
        "{ lib, ... }:".to_string(),
        "".to_string(),
        "{".to_string(),
    ];

    lines.push(format!(
        "  mcb.virtualisation.docker.enable = lib.mkForce {};",
        settings.docker_enable
    ));
    lines.push(format!(
        "  mcb.virtualisation.libvirtd.enable = lib.mkForce {};",
        settings.libvirtd_enable
    ));
    lines.push("}".to_string());
    lines.push("".to_string());

    lines.join("\n")
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

fn render_managed_host_default_file() -> String {
    [
        "# TUI / 自动化工具专用主机入口。",
        "",
        "{ lib, ... }:",
        "",
        "let",
        "  splitImports = lib.concatLists [",
        "    (lib.optional (builtins.pathExists ./users.nix) ./users.nix)",
        "    (lib.optional (builtins.pathExists ./network.nix) ./network.nix)",
        "    (lib.optional (builtins.pathExists ./gpu.nix) ./gpu.nix)",
        "    (lib.optional (builtins.pathExists ./virtualization.nix) ./virtualization.nix)",
        "  ];",
        "in",
        "{",
        "  imports = splitImports ++ lib.optional (builtins.pathExists ./local.nix) ./local.nix;",
        "}",
        "",
    ]
    .join("\n")
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

fn nix_string_literal(value: &str) -> String {
    format!("{value:?}")
}

fn nix_nullable_string(value: Option<&str>) -> String {
    match value {
        Some(value) if !value.trim().is_empty() => nix_string_literal(value),
        _ => "null".to_string(),
    }
}
