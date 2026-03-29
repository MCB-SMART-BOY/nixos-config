use crate::run_capture_allow_fail;
use anyhow::{Result, bail};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub fn detect_repo_root() -> Result<PathBuf> {
    if let Ok(root) = env::var("MCB_FLAKE_ROOT") {
        let path = PathBuf::from(root);
        if path.join("flake.nix").is_file() {
            return Ok(path);
        }
    }

    let mut current = env::current_dir()?;
    loop {
        if current.join("flake.nix").is_file() && current.join("hosts").is_dir() {
            return Ok(current);
        }
        if !current.pop() {
            break;
        }
    }

    let etc_root = PathBuf::from("/etc/nixos");
    if etc_root.join("flake.nix").is_file() {
        return Ok(etc_root);
    }

    bail!("unable to find flake root")
}

pub fn detect_hostname() -> String {
    if let Ok(host) = fs::read_to_string("/etc/hostname") {
        let trimmed = host.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    run_capture_allow_fail("hostname", &[])
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

pub fn detect_privilege_mode() -> String {
    let uid = run_capture_allow_fail("id", &["-u"]).unwrap_or_default();
    if uid.trim() == "0" {
        return "root".to_string();
    }
    if env::var_os("SUDO_USER").is_some() {
        return "sudo-session".to_string();
    }
    if crate::command_exists("sudo") {
        return "sudo-available".to_string();
    }
    "rootless".to_string()
}

pub fn detect_nix_system() -> String {
    if let Ok(system) = env::var("NIX_SYSTEM")
        && !system.trim().is_empty()
    {
        return system;
    }

    run_capture_allow_fail(
        "nix",
        &[
            "--extra-experimental-features",
            "nix-command flakes",
            "eval",
            "--impure",
            "--raw",
            "--expr",
            "builtins.currentSystem",
        ],
    )
    .map(|value| value.trim().to_string())
    .filter(|value| !value.is_empty())
    .unwrap_or_else(|| "x86_64-linux".to_string())
}

pub fn list_hosts(repo_root: &Path) -> Vec<String> {
    list_child_dirs(repo_root.join("hosts"), |name| {
        name != "profiles" && name != "templates"
    })
}

pub fn list_users(repo_root: &Path) -> Vec<String> {
    list_child_dirs(repo_root.join("home/users"), |name| !name.starts_with('_'))
}

fn list_child_dirs<F>(root: PathBuf, include: F) -> Vec<String>
where
    F: Fn(&str) -> bool,
{
    let mut items = Vec::new();
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if include(&name) {
                items.push(name);
            }
        }
    }
    items.sort();
    items
}
