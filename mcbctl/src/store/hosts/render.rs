use crate::domain::tui::HostManagedSettings;
use crate::write_managed_file;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub fn write_host_users_fragment(
    managed_dir: &Path,
    settings: &HostManagedSettings,
) -> Result<PathBuf> {
    let path = managed_dir.join("users.nix");
    write_managed_file(
        &path,
        "host-users",
        &render_host_users_file(settings),
        &["# 机器管理的用户结构分片"],
    )?;
    Ok(path)
}

pub fn write_host_runtime_fragments(
    managed_dir: &Path,
    settings: &HostManagedSettings,
) -> Result<Vec<PathBuf>> {
    let network_path = managed_dir.join("network.nix");
    write_managed_file(
        &network_path,
        "host-network",
        &render_host_network_file(settings),
        &["# 机器管理的网络/TUN 分片"],
    )?;

    let gpu_path = managed_dir.join("gpu.nix");
    write_managed_file(
        &gpu_path,
        "host-gpu",
        &render_host_gpu_file(settings),
        &["# 机器管理的 GPU 分片"],
    )?;

    let virtualization_path = managed_dir.join("virtualization.nix");
    write_managed_file(
        &virtualization_path,
        "host-virtualization",
        &render_host_virtualization_file(settings),
        &["# 机器管理的虚拟化分片"],
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
            "  mcb.nix.customSubstituters = lib.mkForce {};",
            nix_string_list(&settings.custom_substituters)
        ),
        format!(
            "  mcb.nix.customTrustedPublicKeys = lib.mkForce {};",
            nix_string_list(&settings.custom_trusted_public_keys)
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
            "  mcb.tunInterfaces = lib.mkForce {};",
            nix_string_list(&settings.tun_interfaces)
        ),
        format!(
            "  mcb.enableProxyDns = lib.mkForce {};",
            settings.enable_proxy_dns
        ),
        format!(
            "  mcb.proxyDnsAddr = lib.mkForce {};",
            nix_string_literal(&settings.proxy_dns_addr)
        ),
        format!(
            "  mcb.proxyDnsPort = lib.mkForce {};",
            settings.proxy_dns_port
        ),
        format!(
            "  mcb.perUserTun.enable = lib.mkForce {};",
            settings.per_user_tun_enable
        ),
        format!(
            "  mcb.perUserTun.compatGlobalServiceSocket = lib.mkForce {};",
            settings.per_user_tun_compat_global_service_socket
        ),
        format!(
            "  mcb.perUserTun.redirectDns = lib.mkForce {};",
            settings.per_user_tun_redirect_dns
        ),
        format!(
            "  mcb.perUserTun.tableBase = lib.mkForce {};",
            settings.per_user_tun_table_base
        ),
        format!(
            "  mcb.perUserTun.priorityBase = lib.mkForce {};",
            settings.per_user_tun_priority_base
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

fn nix_string_list(values: &[String]) -> String {
    let items = values
        .iter()
        .map(|value| format!(" {}", nix_string_literal(value)))
        .collect::<String>();
    format!("[{items} ]")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_host_network_file_includes_extended_managed_fields() {
        let mut settings = HostManagedSettings {
            cache_profile: "custom".to_string(),
            custom_substituters: vec!["https://cache.example.org".to_string()],
            custom_trusted_public_keys: vec!["cache.example.org-1:abc".to_string()],
            proxy_mode: "tun".to_string(),
            proxy_url: "http://127.0.0.1:7890".to_string(),
            tun_interface: "tun0".to_string(),
            tun_interfaces: vec!["mihomo".to_string()],
            enable_proxy_dns: false,
            proxy_dns_addr: "127.0.0.1".to_string(),
            proxy_dns_port: 5353,
            per_user_tun_enable: true,
            per_user_tun_compat_global_service_socket: false,
            per_user_tun_redirect_dns: true,
            per_user_tun_table_base: 2000,
            per_user_tun_priority_base: 30000,
            ..HostManagedSettings::default()
        };
        settings
            .per_user_tun_interfaces
            .insert("alice".to_string(), "tun-alice".to_string());
        settings
            .per_user_tun_dns_ports
            .insert("alice".to_string(), 1053);

        let rendered = render_host_network_file(&settings);

        for needle in [
            "mcb.nix.cacheProfile = lib.mkForce \"custom\";",
            "mcb.nix.customSubstituters = lib.mkForce [ \"https://cache.example.org\" ];",
            "mcb.nix.customTrustedPublicKeys = lib.mkForce [ \"cache.example.org-1:abc\" ];",
            "mcb.tunInterfaces = lib.mkForce [ \"mihomo\" ];",
            "mcb.perUserTun.compatGlobalServiceSocket = lib.mkForce false;",
            "mcb.perUserTun.tableBase = lib.mkForce 2000;",
            "mcb.perUserTun.priorityBase = lib.mkForce 30000;",
            "alice = \"tun-alice\";",
            "alice = 1053;",
        ] {
            assert!(
                rendered.contains(needle),
                "rendered network fragment should contain: {needle}\n{rendered}"
            );
        }
    }

    #[test]
    fn render_host_gpu_file_includes_managed_fields() {
        let settings = HostManagedSettings {
            gpu_mode: "hybrid".to_string(),
            gpu_igpu_vendor: "amd".to_string(),
            gpu_prime_mode: "sync".to_string(),
            gpu_intel_bus: None,
            gpu_amd_bus: Some("PCI:4:0:0".to_string()),
            gpu_nvidia_bus: Some("PCI:1:0:0".to_string()),
            gpu_nvidia_open: true,
            gpu_specialisations_enable: true,
            gpu_specialisation_modes: vec!["igpu".to_string(), "hybrid".to_string()],
            ..HostManagedSettings::default()
        };

        let rendered = render_host_gpu_file(&settings);

        for needle in [
            "mcb.hardware.gpu.mode = lib.mkForce \"hybrid\";",
            "mcb.hardware.gpu.igpuVendor = lib.mkForce \"amd\";",
            "mode = \"sync\";",
            "intelBusId = null;",
            "amdgpuBusId = \"PCI:4:0:0\";",
            "nvidiaBusId = \"PCI:1:0:0\";",
            "mcb.hardware.gpu.nvidia.open = lib.mkForce true;",
            "mcb.hardware.gpu.specialisations.enable = lib.mkForce true;",
            "mcb.hardware.gpu.specialisations.modes = lib.mkForce [ \"igpu\" \"hybrid\" ];",
        ] {
            assert!(
                rendered.contains(needle),
                "rendered gpu fragment should contain: {needle}\n{rendered}"
            );
        }
    }

    #[test]
    fn render_host_virtualization_file_includes_managed_fields() {
        let settings = HostManagedSettings {
            docker_enable: true,
            libvirtd_enable: true,
            ..HostManagedSettings::default()
        };

        let rendered = render_host_virtualization_file(&settings);

        for needle in [
            "mcb.virtualisation.docker.enable = lib.mkForce true;",
            "mcb.virtualisation.libvirtd.enable = lib.mkForce true;",
        ] {
            assert!(
                rendered.contains(needle),
                "rendered virtualization fragment should contain: {needle}\n{rendered}"
            );
        }
    }
}
