use crate::{ensure_existing_managed_file, write_managed_file};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub fn ensure_managed_host_layout(managed_dir: &Path) -> Result<()> {
    fs::create_dir_all(managed_dir)
        .with_context(|| format!("failed to create {}", managed_dir.display()))?;

    let default_path = managed_dir.join("default.nix");
    write_managed_file(
        &default_path,
        "host-managed-default",
        &render_managed_host_default_file(),
        &["# TUI / 自动化工具专用主机入口。"],
    )?;

    for (name, kind) in [
        ("users.nix", "host-users"),
        ("network.nix", "host-network"),
        ("gpu.nix", "host-gpu"),
        ("virtualization.nix", "host-virtualization"),
    ] {
        ensure_existing_managed_file(&managed_dir.join(name), kind)?;
    }

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
