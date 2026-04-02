use crate::domain::tui::HostManagedSettings;
use crate::write_file_atomic;
use anyhow::Result;
use std::path::{Path, PathBuf};

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

fn nix_string_literal(value: &str) -> String {
    format!("{value:?}")
}

fn nix_nullable_string(value: Option<&str>) -> String {
    match value {
        Some(value) if !value.trim().is_empty() => nix_string_literal(value),
        _ => "null".to_string(),
    }
}
