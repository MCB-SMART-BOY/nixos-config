use crate::command_exists;
use crate::domain::tui::DeployAction;
use anyhow::{Context, Result, bail};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};

#[derive(Clone, Debug)]
pub struct NixosRebuildPlan {
    pub action: DeployAction,
    pub upgrade: bool,
    pub flake_root: PathBuf,
    pub target_host: String,
}

impl NixosRebuildPlan {
    pub fn args(&self) -> Vec<String> {
        let mut args = vec![
            self.action.rebuild_mode().to_string(),
            "--show-trace".to_string(),
        ];
        if self.upgrade {
            args.push("--upgrade".to_string());
        }
        args.push("--flake".to_string());
        args.push(format!(
            "{}#{}",
            self.flake_root.display(),
            self.target_host
        ));
        args
    }

    pub fn command_preview(&self, use_sudo: bool) -> String {
        let args = self.args().join(" ");
        if use_sudo {
            format!("sudo -E env NIX_CONFIG='<merged>' nixos-rebuild {args}")
        } else {
            format!("env NIX_CONFIG='<merged>' nixos-rebuild {args}")
        }
    }
}

#[derive(Clone, Debug)]
pub struct RepoSyncPlan {
    pub source_dir: PathBuf,
    pub destination_dir: PathBuf,
    pub delete_extra: bool,
}

impl RepoSyncPlan {
    pub fn rsync_args(&self) -> Vec<String> {
        let mut args = vec!["-a".to_string()];
        if self.delete_extra {
            args.push("--delete".to_string());
        }
        args.extend([
            "--exclude".to_string(),
            ".git/".to_string(),
            "--exclude".to_string(),
            "hardware-configuration.nix".to_string(),
            format!("{}/", self.source_dir.display()),
            format!("{}/", self.destination_dir.display()),
        ]);
        args
    }

    pub fn tar_pack_args(&self, tar_file: &PathBuf) -> Vec<String> {
        vec![
            "-C".to_string(),
            self.source_dir.display().to_string(),
            "--exclude=.git".to_string(),
            "--exclude=hardware-configuration.nix".to_string(),
            "-cf".to_string(),
            tar_file.display().to_string(),
            ".".to_string(),
        ]
    }

    pub fn tar_extract_args(&self, tar_file: &PathBuf) -> Vec<String> {
        vec![
            "-C".to_string(),
            self.destination_dir.display().to_string(),
            "-xf".to_string(),
            tar_file.display().to_string(),
        ]
    }

    pub fn command_preview(&self) -> String {
        if command_exists("rsync") {
            format!("rsync {}", self.rsync_args().join(" "))
        } else {
            format!(
                "tar {} && tar {}",
                self.tar_pack_args(&PathBuf::from("<tmp>.tar")).join(" "),
                self.tar_extract_args(&PathBuf::from("<tmp>.tar")).join(" ")
            )
        }
    }
}

pub fn merged_nix_config() -> String {
    let mut nix_config = "experimental-features = nix-command flakes".to_string();
    if let Ok(extra) = std::env::var("NIX_CONFIG")
        && !extra.trim().is_empty()
    {
        nix_config = format!("{extra}\n{nix_config}");
    }
    nix_config
}

pub fn run_nixos_rebuild(plan: &NixosRebuildPlan, use_sudo: bool) -> Result<ExitStatus> {
    let nix_config = merged_nix_config();
    let args = plan.args();

    let status = if use_sudo {
        let mut cmd = Command::new("sudo");
        cmd.arg("-E")
            .arg("env")
            .arg(format!("NIX_CONFIG={nix_config}"))
            .arg("nixos-rebuild")
            .args(&args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
        cmd.status().context("failed to run sudo nixos-rebuild")?
    } else {
        let mut cmd = Command::new("env");
        cmd.arg(format!("NIX_CONFIG={nix_config}"))
            .arg("nixos-rebuild")
            .args(&args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
        cmd.status().context("failed to run nixos-rebuild")?
    };

    Ok(status)
}

pub fn run_root_command(cmd: &str, args: &[String], use_sudo: bool) -> Result<ExitStatus> {
    let status = if use_sudo {
        let mut command = Command::new("sudo");
        command
            .arg("-E")
            .arg(cmd)
            .args(args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
        command
            .status()
            .with_context(|| format!("failed to run sudo {cmd}"))?
    } else {
        let mut command = Command::new(cmd);
        command
            .args(args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
        command
            .status()
            .with_context(|| format!("failed to run {cmd}"))?
    };

    Ok(status)
}

pub fn run_root_command_ok(cmd: &str, args: &[String], use_sudo: bool) -> Result<()> {
    let status = run_root_command(cmd, args, use_sudo)?;
    if status.success() {
        Ok(())
    } else {
        bail!("{cmd} failed with {}", status.code().unwrap_or(1))
    }
}

pub fn ensure_root_hardware_config(etc_root: &Path, use_sudo: bool) -> Result<()> {
    let target = etc_root.join("hardware-configuration.nix");
    if target.is_file() {
        return Ok(());
    }
    if !command_exists("nixos-generate-config") {
        bail!(
            "缺少 {}，无法自动生成 {}。",
            "nixos-generate-config",
            target.display()
        );
    }

    run_root_command_ok("mkdir", &["-p".to_string(), etc_root.display().to_string()], use_sudo)?;
    run_root_command_ok(
        "env",
        &[
            format!("HW_FILE={}", target.display()),
            "sh".to_string(),
            "-c".to_string(),
            "umask 022 && nixos-generate-config --show-hardware-config > \"$HW_FILE\""
                .to_string(),
        ],
        use_sudo,
    )?;
    Ok(())
}

pub fn run_repo_sync<FL, FR, FC>(
    plan: &RepoSyncPlan,
    mut run_local_ok: FL,
    mut run_root_ok: FR,
    mut clean_destination: FC,
) -> Result<()>
where
    FL: FnMut(&str, &[String]) -> Result<()>,
    FR: FnMut(&str, &[String]) -> Result<()>,
    FC: FnMut() -> Result<()>,
{
    run_root_ok(
        "mkdir",
        &["-p".to_string(), plan.destination_dir.display().to_string()],
    )?;

    if command_exists("rsync") {
        run_root_ok("rsync", &plan.rsync_args())?;
        return Ok(());
    }

    if plan.delete_extra {
        clean_destination()?;
    }

    let tar_file = create_temp_path("mcbctl-sync", "tar")?;
    let pack_args = plan.tar_pack_args(&tar_file);
    if let Err(err) = run_local_ok("tar", &pack_args) {
        fs::remove_file(&tar_file).ok();
        return Err(err);
    }
    let unpack_args = plan.tar_extract_args(&tar_file);
    if let Err(err) = run_root_ok("tar", &unpack_args) {
        fs::remove_file(&tar_file).ok();
        return Err(err);
    }
    fs::remove_file(&tar_file).ok();
    Ok(())
}

fn create_temp_path(prefix: &str, ext: &str) -> Result<PathBuf> {
    let base = std::env::temp_dir();
    for n in 0..2048u32 {
        let p = base.join(format!(
            "{prefix}-{}-{}-{n}.{ext}",
            std::process::id(),
            chrono_like_millis()
        ));
        if !p.exists() {
            return Ok(p);
        }
    }
    bail!("failed to allocate temporary path");
}

fn chrono_like_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}
