use anyhow::{Context, Result, anyhow};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};

pub mod domain;
pub mod health;
pub mod release_bundle;
pub mod repo;
pub mod store;
pub mod tui;

const MANAGED_KIND_PREFIX: &str = "# mcbctl-managed: ";
const MANAGED_CHECKSUM_PREFIX: &str = "# mcbctl-checksum: ";

pub fn emit_json(text: &str, tooltip: &str, class: &str) {
    println!(
        "{}",
        json!({
            "text": text,
            "tooltip": tooltip,
            "class": class,
        })
    );
}

pub fn command_exists(name: &str) -> bool {
    env::var_os("PATH")
        .map(|paths| {
            env::split_paths(&paths).any(|dir| {
                command_candidates(&dir, name)
                    .into_iter()
                    .any(|full| full.is_file() && is_executable(&full))
            })
        })
        .unwrap_or(false)
}

#[cfg(windows)]
fn command_candidates(dir: &Path, name: &str) -> Vec<PathBuf> {
    let candidate = dir.join(name);
    if candidate.extension().is_some() {
        return vec![candidate];
    }

    let mut candidates = vec![candidate];
    for ext in [".exe", ".cmd", ".bat", ".com"] {
        candidates.push(dir.join(format!("{name}{ext}")));
    }
    candidates
}

#[cfg(not(windows))]
fn command_candidates(dir: &Path, name: &str) -> Vec<PathBuf> {
    vec![dir.join(name)]
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(path)
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}

pub fn run_capture(cmd: &str, args: &[&str]) -> Result<String> {
    let out = Command::new(cmd)
        .args(args)
        .output()
        .with_context(|| format!("failed to run {cmd}"))?;
    if !out.status.success() {
        return Err(anyhow!(
            "{cmd} exited with {}",
            out.status.code().unwrap_or_default()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

pub fn run_capture_allow_fail(cmd: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(cmd).args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).to_string())
}

pub fn run_status(cmd: &str, args: &[&str]) -> Result<ExitStatus> {
    Command::new(cmd)
        .args(args)
        .status()
        .with_context(|| format!("failed to run {cmd}"))
}

pub fn run_status_inherit(cmd: &str, args: &[String]) -> Result<ExitStatus> {
    Command::new(cmd)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("failed to run {cmd}"))
}

pub fn resolve_sibling_binary(name: &str) -> Result<PathBuf> {
    let current = env::current_exe().context("failed to locate current executable")?;
    let Some(dir) = current.parent() else {
        return Err(anyhow!("failed to resolve executable directory"));
    };
    let candidate = dir.join(name);
    if candidate.is_file() {
        Ok(candidate)
    } else {
        Err(anyhow!(
            "failed to locate sibling binary: {}",
            candidate.display()
        ))
    }
}

pub fn run_sibling_status(name: &str, args: &[String]) -> Result<ExitStatus> {
    let binary = resolve_sibling_binary(name)?;
    Command::new(&binary)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("failed to run {}", binary.display()))
}

pub fn xdg_cache_home() -> PathBuf {
    if let Ok(v) = env::var("XDG_CACHE_HOME") {
        return PathBuf::from(v);
    }
    home_dir().join(".cache")
}

pub fn xdg_config_home() -> PathBuf {
    if let Ok(v) = env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(v);
    }
    home_dir().join(".config")
}

pub fn xdg_state_home() -> PathBuf {
    if let Ok(v) = env::var("XDG_STATE_HOME") {
        return PathBuf::from(v);
    }
    home_dir().join(".local/state")
}

pub fn home_dir() -> PathBuf {
    env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/"))
}

pub fn prepend_paths(paths: &[PathBuf]) {
    let current = env::var("PATH").unwrap_or_default();
    let mut parts: Vec<String> = current
        .split(':')
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
        .collect();
    for p in paths.iter().rev() {
        let p = p.to_string_lossy().to_string();
        if !parts.iter().any(|x| x == &p) {
            parts.insert(0, p);
        }
    }
    // SAFETY: this CLI runs single-threaded and updates PATH before spawning child processes.
    unsafe {
        env::set_var("PATH", parts.join(":"));
    }
}

pub fn parse_df_root() -> Option<(String, String, String)> {
    let out = run_capture_allow_fail("df", &["-P", "/"])?;
    let line = out.lines().nth(1)?;
    let cols: Vec<&str> = line.split_whitespace().collect();
    if cols.len() < 5 {
        return None;
    }
    Some((
        cols[1].to_string(),
        cols[2].to_string(),
        cols[4].to_string(),
    ))
}

pub fn format_rate(mut rate: u64) -> String {
    let units = ["B/s", "KB/s", "MB/s", "GB/s", "TB/s"];
    let mut idx = 0usize;
    let mut rem = 0u64;
    while rate >= 1024 && idx < units.len() - 1 {
        rem = rate % 1024;
        rate /= 1024;
        idx += 1;
    }
    if idx == 0 {
        format!("{rate}{}", units[idx])
    } else {
        let frac = (rem * 10) / 1024;
        format!("{rate}.{frac}{}", units[idx])
    }
}

pub fn find_repo_root() -> Result<PathBuf> {
    let mut dir = env::current_dir().context("failed to get current dir")?;
    loop {
        if dir.join("flake.nix").is_file() && dir.join("pkgs").is_dir() {
            return Ok(dir);
        }
        if !dir.pop() {
            break;
        }
    }
    Err(anyhow!("could not locate repo root (flake.nix + pkgs)"))
}

pub fn write_file_atomic(path: &Path, content: &str) -> Result<()> {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, content).with_context(|| format!("failed to write {}", tmp.display()))?;
    fs::rename(&tmp, path)
        .with_context(|| format!("failed to rename {} -> {}", tmp.display(), path.display()))?;
    Ok(())
}

pub fn render_managed_file(kind: &str, body: &str) -> String {
    format!(
        "{MANAGED_KIND_PREFIX}{kind}\n{MANAGED_CHECKSUM_PREFIX}{}\n{body}",
        managed_checksum(body)
    )
}

pub fn managed_file_kind(content: &str) -> Option<&str> {
    let (kind, _, _) = parse_managed_file(content)?;
    Some(kind)
}

pub fn managed_file_is_valid(content: &str) -> bool {
    parse_managed_file(content)
        .is_some_and(|(_, checksum, body)| checksum == managed_checksum(body))
}

pub fn write_managed_file(
    path: &Path,
    kind: &str,
    body: &str,
    _legacy_prefixes: &[&str],
) -> Result<()> {
    let rendered = render_managed_file(kind, body);
    if let Ok(existing) = fs::read_to_string(path) {
        if existing == rendered {
            return Ok(());
        }
        if !managed_content_is_safe_to_replace(&existing, kind) {
            return Err(anyhow!(
                "refusing to overwrite {}: existing content is not a recognized {kind} managed file",
                path.display()
            ));
        }
    }

    write_file_atomic(path, &rendered)
}

fn managed_content_is_safe_to_replace(existing: &str, kind: &str) -> bool {
    parse_managed_file(existing).is_some_and(|(existing_kind, checksum, existing_body)| {
        existing_kind == kind && checksum == managed_checksum(existing_body)
    })
}

fn parse_managed_file(content: &str) -> Option<(&str, &str, &str)> {
    let first_newline = content.find('\n')?;
    let first_line = &content[..first_newline];
    let rest = &content[first_newline + 1..];
    let second_newline = rest.find('\n')?;
    let second_line = &rest[..second_newline];
    let body = &rest[second_newline + 1..];
    let kind = first_line.strip_prefix(MANAGED_KIND_PREFIX)?.trim();
    let checksum = second_line.strip_prefix(MANAGED_CHECKSUM_PREFIX)?.trim();
    Some((kind, checksum, body))
}

fn managed_checksum(body: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(body.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn exit_from_status(status: ExitStatus) -> ! {
    std::process::exit(status.code().unwrap_or(1))
}

pub fn normalize_gpu_mode_label(mode: &str) -> String {
    match mode {
        "gpu-dgpu" | "dgpu" => "dgpu".to_string(),
        "gpu-hybrid" | "hybrid" => "hybrid".to_string(),
        "gpu-igpu" | "igpu" => "igpu".to_string(),
        _ => "base".to_string(),
    }
}

fn gpu_mode_from_path(path: &str) -> Option<String> {
    if let Some(idx) = path.find("/specialisation/gpu-") {
        let tail = &path[idx + "/specialisation/".len()..];
        return tail.split('/').next().map(ToOwned::to_owned);
    }
    if let Some(idx) = path.find("specialisation-gpu-") {
        let tail = &path[idx + "specialisation-".len()..];
        return tail.split('/').next().map(ToOwned::to_owned);
    }
    None
}

pub fn current_gpu_specialisation() -> String {
    if let Ok(mode) = env::var("MCB_GPU_MODE")
        && !mode.trim().is_empty()
    {
        return match mode.as_str() {
            "igpu" | "hybrid" | "dgpu" => format!("gpu-{mode}"),
            _ => mode,
        };
    }

    for path in ["/run/current-system", "/run/booted-system"] {
        if let Ok(canonical) = fs::canonicalize(path) {
            let p = canonical.to_string_lossy();
            if let Some(mode) = gpu_mode_from_path(&p) {
                return mode;
            }
        }
    }

    if let Ok(cmdline) = fs::read_to_string("/proc/cmdline") {
        for token in cmdline.split_whitespace() {
            let maybe_path = if let Some(v) = token.strip_prefix("init=") {
                Some(v)
            } else {
                token.strip_prefix("systemConfig=")
            };
            if let Some(path) = maybe_path
                && let Some(mode) = gpu_mode_from_path(path)
            {
                return mode;
            }
        }
    }

    "base".to_string()
}

pub fn current_gpu_mode_label() -> String {
    normalize_gpu_mode_label(&current_gpu_specialisation())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn managed_file_roundtrip_is_valid() {
        let rendered = render_managed_file("host-network", "# body\n{ }\n");
        assert_eq!(managed_file_kind(&rendered), Some("host-network"));
        assert!(managed_file_is_valid(&rendered));
    }

    #[test]
    fn write_managed_file_rejects_tampered_content() -> Result<()> {
        let unique = format!("{}-{}", std::process::id(), rand::random::<u64>());
        let dir = std::env::temp_dir().join(format!("mcbctl-managed-write-{unique}"));
        fs::create_dir_all(&dir)?;
        let path = dir.join("network.nix");

        fs::write(&path, render_managed_file("host-network", "# next\n{ }\n"))?;

        let tampered = fs::read_to_string(&path)?.replace("{ }\n", "{ a = 1; }\n");
        fs::write(&path, tampered)?;
        let err = write_managed_file(&path, "host-network", "# final\n{ }\n", &[])
            .expect_err("tampered managed file should be rejected");
        assert!(err.to_string().contains("refusing to overwrite"));

        fs::remove_dir_all(&dir)?;
        Ok(())
    }

    #[test]
    fn write_managed_file_rejects_legacy_unmarked_content() -> Result<()> {
        let unique = format!("{}-{}", std::process::id(), rand::random::<u64>());
        let dir = std::env::temp_dir().join(format!("mcbctl-managed-legacy-{unique}"));
        fs::create_dir_all(&dir)?;
        let path = dir.join("network.nix");

        fs::write(&path, "# 机器管理的网络/TUN 分片。\n\n{ ... }:\n\n{ }\n")?;
        let err = write_managed_file(
            &path,
            "host-network",
            "# final\n{ }\n",
            &["# 机器管理的网络/TUN 分片。"],
        )
        .expect_err("legacy unmarked file should require explicit migration");
        assert!(err.to_string().contains("refusing to overwrite"));

        fs::remove_dir_all(&dir)?;
        Ok(())
    }
}
