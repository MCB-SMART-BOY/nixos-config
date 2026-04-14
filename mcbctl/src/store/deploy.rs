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

    pub fn tar_pack_args(&self, tar_file: &Path) -> Vec<String> {
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

    pub fn tar_extract_args(&self, tar_file: &Path) -> Vec<String> {
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

pub fn host_hardware_config_path(root: &Path, host: &str) -> PathBuf {
    root.join("hosts")
        .join(host)
        .join("hardware-configuration.nix")
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
    ensure_real_hardware_config_for_rebuild(plan)?;
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

fn ensure_real_hardware_config_for_rebuild(plan: &NixosRebuildPlan) -> Result<()> {
    if matches!(plan.action, DeployAction::Build) {
        return Ok(());
    }

    let hardware_file = host_hardware_config_path(&plan.flake_root, &plan.target_host);
    if hardware_file.is_file() {
        return Ok(());
    }

    bail!(
        "{} 缺少真实 hardware-configuration.nix；`switch` / `test` / `boot` 不能使用评估 fallback。若只是做评估，请改用 `mcbctl build-host` 或 `mcbctl rebuild build`。",
        hardware_file.display()
    )
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

pub fn ensure_host_hardware_config(etc_root: &Path, host: &str, use_sudo: bool) -> Result<()> {
    if host.trim().is_empty() {
        bail!("未指定目标主机，无法生成 hosts/<host>/hardware-configuration.nix");
    }

    let target = host_hardware_config_path(etc_root, host);
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

    run_root_command_ok(
        "mkdir",
        &[
            "-p".to_string(),
            target
                .parent()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| etc_root.display().to_string()),
        ],
        use_sudo,
    )?;
    let output = Command::new("nixos-generate-config")
        .arg("--show-hardware-config")
        .output()
        .context("failed to run nixos-generate-config --show-hardware-config")?;
    if !output.status.success() {
        bail!(
            "nixos-generate-config failed with {}",
            output.status.code().unwrap_or(1)
        );
    }

    let temp_file = create_temp_path("mcbctl-hardware-config", "nix")?;
    if let Err(err) = fs::write(&temp_file, &output.stdout)
        .with_context(|| format!("failed to write {}", temp_file.display()))
    {
        let cleanup_result = cleanup_temp_file(
            &temp_file,
            "failed to remove temporary hardware configuration file",
        );
        return finalize_with_cleanup(
            Err(err),
            cleanup_result,
            "temporary hardware configuration cleanup failed",
        );
    }

    let install_result = run_root_command_ok(
        "install",
        &[
            "-Dm0644".to_string(),
            temp_file.display().to_string(),
            target.display().to_string(),
        ],
        use_sudo,
    );
    let cleanup_result = cleanup_temp_file(
        &temp_file,
        "failed to remove temporary hardware configuration file",
    );
    finalize_with_cleanup(
        install_result,
        cleanup_result,
        "temporary hardware configuration cleanup failed",
    )
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
    run_repo_sync_with_mode(
        plan,
        command_exists("rsync"),
        &mut run_local_ok,
        &mut run_root_ok,
        &mut clean_destination,
    )
}

fn run_repo_sync_with_mode<FL, FR, FC>(
    plan: &RepoSyncPlan,
    use_rsync: bool,
    run_local_ok: &mut FL,
    run_root_ok: &mut FR,
    clean_destination: &mut FC,
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

    if use_rsync {
        run_root_ok("rsync", &plan.rsync_args())?;
        return Ok(());
    }

    if plan.delete_extra {
        clean_destination()?;
    }

    let tar_file = create_temp_path("mcbctl-sync", "tar")?;
    let pack_args = plan.tar_pack_args(&tar_file);
    if let Err(err) = run_local_ok("tar", &pack_args) {
        let cleanup_result =
            cleanup_temp_file(&tar_file, "failed to remove repository sync archive");
        return finalize_with_cleanup(
            Err(err),
            cleanup_result,
            "repository sync archive cleanup failed",
        );
    }
    let unpack_args = plan.tar_extract_args(&tar_file);
    if let Err(err) = run_root_ok("tar", &unpack_args) {
        let cleanup_result =
            cleanup_temp_file(&tar_file, "failed to remove repository sync archive");
        return finalize_with_cleanup(
            Err(err),
            cleanup_result,
            "repository sync archive cleanup failed",
        );
    }
    let cleanup_result = cleanup_temp_file(&tar_file, "failed to remove repository sync archive");
    finalize_with_cleanup(
        Ok(()),
        cleanup_result,
        "repository sync archive cleanup failed",
    )
}

fn summarize_cleanup_failures(context: &str, failures: &[String]) -> String {
    if failures.is_empty() {
        return context.to_string();
    }

    format!("{context}: {}", failures.join(" | "))
}

fn cleanup_temp_file(path: &Path, label: &str) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    fs::remove_file(path).with_context(|| format!("{label}: {}", path.display()))
}

fn finalize_with_cleanup(
    primary_result: Result<()>,
    cleanup_result: Result<()>,
    cleanup_context: &str,
) -> Result<()> {
    match (primary_result, cleanup_result) {
        (Ok(()), Ok(())) => Ok(()),
        (Ok(()), Err(cleanup_err)) => {
            let failures = vec![cleanup_err.to_string()];
            bail!("{}", summarize_cleanup_failures(cleanup_context, &failures))
        }
        (Err(err), Ok(())) => Err(err),
        (Err(err), Err(cleanup_err)) => {
            let failures = vec![cleanup_err.to_string()];
            bail!(
                "{}",
                summarize_cleanup_failures(&err.to_string(), &failures)
            )
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::cell::{Cell, RefCell};
    use std::sync::{Mutex, MutexGuard, OnceLock};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn host_hardware_config_path_uses_per_host_location() {
        let _guard = test_lock();
        let path = host_hardware_config_path(Path::new("/repo"), "demo");
        assert_eq!(
            path,
            PathBuf::from("/repo/hosts/demo/hardware-configuration.nix")
        );
    }

    #[test]
    fn build_allows_eval_fallback_without_real_hardware_config() -> Result<()> {
        let _guard = test_lock();
        let root = create_temp_repo()?;
        let plan = NixosRebuildPlan {
            action: DeployAction::Build,
            upgrade: false,
            flake_root: root.clone(),
            target_host: "demo".to_string(),
        };

        ensure_real_hardware_config_for_rebuild(&plan)?;
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn switch_requires_real_hardware_config() -> Result<()> {
        let _guard = test_lock();
        let root = create_temp_repo()?;
        let plan = NixosRebuildPlan {
            action: DeployAction::Switch,
            upgrade: false,
            flake_root: root.clone(),
            target_host: "demo".to_string(),
        };

        let err = ensure_real_hardware_config_for_rebuild(&plan)
            .expect_err("switch should require a real host hardware config");
        assert!(
            err.to_string()
                .contains("缺少真实 hardware-configuration.nix")
        );

        fs::create_dir_all(root.join("hosts/demo"))?;
        fs::write(host_hardware_config_path(&root, "demo"), "{ ... }: { }\n")?;
        ensure_real_hardware_config_for_rebuild(&plan)?;

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn run_repo_sync_tar_mode_cleans_destination_and_temp_file() -> Result<()> {
        let _guard = test_lock();
        let root = create_temp_repo()?;
        let source_dir = root.join("source");
        let destination_dir = root.join("dest");
        fs::create_dir_all(&source_dir)?;
        let plan = RepoSyncPlan {
            source_dir: source_dir.clone(),
            destination_dir: destination_dir.clone(),
            delete_extra: true,
        };

        let calls = RefCell::new(Vec::<String>::new());
        let clean_called = Cell::new(false);
        let archive_path = RefCell::new(None::<PathBuf>);

        run_repo_sync_with_mode(
            &plan,
            false,
            &mut |cmd, args| {
                calls
                    .borrow_mut()
                    .push(format!("local:{cmd} {}", args.join(" ")));
                let tar_path = PathBuf::from(&args[5]);
                fs::write(&tar_path, "archive")?;
                *archive_path.borrow_mut() = Some(tar_path);
                Ok(())
            },
            &mut |cmd, args| {
                calls
                    .borrow_mut()
                    .push(format!("root:{cmd} {}", args.join(" ")));
                if cmd == "mkdir" {
                    fs::create_dir_all(&destination_dir)?;
                }
                Ok(())
            },
            &mut || {
                clean_called.set(true);
                Ok(())
            },
        )?;

        assert!(clean_called.get());
        assert_eq!(
            calls.borrow()[0],
            format!("root:mkdir -p {}", destination_dir.display())
        );
        assert!(calls.borrow()[1].starts_with("local:tar "));
        assert!(calls.borrow()[2].starts_with("root:tar "));
        assert!(!archive_path.borrow().as_ref().unwrap().exists());

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn run_repo_sync_tar_mode_preserves_primary_error_and_reports_cleanup_failure() -> Result<()> {
        let _guard = test_lock();
        let root = create_temp_repo()?;
        let source_dir = root.join("source");
        let destination_dir = root.join("dest");
        fs::create_dir_all(&source_dir)?;
        let plan = RepoSyncPlan {
            source_dir,
            destination_dir,
            delete_extra: false,
        };

        let archive_path = RefCell::new(None::<PathBuf>);
        let err = run_repo_sync_with_mode(
            &plan,
            false,
            &mut |_cmd, args| {
                let tar_path = PathBuf::from(&args[5]);
                *archive_path.borrow_mut() = Some(tar_path.clone());
                fs::write(&tar_path, "archive")?;
                fs::remove_file(&tar_path)?;
                fs::create_dir(&tar_path)?;
                bail!("pack failed")
            },
            &mut |_cmd, _args| Ok(()),
            &mut || Ok(()),
        )
        .expect_err("pack failure should be reported");

        let message = err.to_string();
        assert!(message.contains("pack failed"));
        assert!(message.contains("failed to remove repository sync archive"));

        if let Some(path) = archive_path.borrow().as_ref()
            && path.is_dir()
        {
            fs::remove_dir_all(path)?;
        }
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn run_repo_sync_tar_mode_fails_when_cleanup_after_success_fails() -> Result<()> {
        let _guard = test_lock();
        let root = create_temp_repo()?;
        let source_dir = root.join("source");
        let destination_dir = root.join("dest");
        fs::create_dir_all(&source_dir)?;
        let plan = RepoSyncPlan {
            source_dir,
            destination_dir: destination_dir.clone(),
            delete_extra: false,
        };

        let archive_path = RefCell::new(None::<PathBuf>);
        let err = run_repo_sync_with_mode(
            &plan,
            false,
            &mut |_cmd, args| {
                let tar_path = PathBuf::from(&args[5]);
                *archive_path.borrow_mut() = Some(tar_path.clone());
                fs::write(&tar_path, "archive")?;
                Ok(())
            },
            &mut |cmd, args| {
                if cmd == "mkdir" {
                    fs::create_dir_all(&destination_dir)?;
                } else {
                    let tar_path = PathBuf::from(&args[3]);
                    fs::remove_file(&tar_path)?;
                    fs::create_dir(&tar_path)?;
                }
                Ok(())
            },
            &mut || Ok(()),
        )
        .expect_err("cleanup failure after successful sync should fail");

        let message = err.to_string();
        assert!(
            message.contains("failed to remove repository sync archive"),
            "{message}"
        );

        if let Some(path) = archive_path.borrow().as_ref()
            && path.is_dir()
        {
            fs::remove_dir_all(path)?;
        }
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn run_repo_sync_rsync_mode_skips_clean_destination() -> Result<()> {
        let _guard = test_lock();
        let root = create_temp_repo()?;
        let destination_dir = root.join("dest");
        let plan = RepoSyncPlan {
            source_dir: root.join("source"),
            destination_dir: destination_dir.clone(),
            delete_extra: true,
        };

        let clean_called = Cell::new(false);
        let root_calls = RefCell::new(Vec::<String>::new());

        run_repo_sync_with_mode(
            &plan,
            true,
            &mut |_cmd, _args| Ok(()),
            &mut |cmd, args| {
                root_calls
                    .borrow_mut()
                    .push(format!("{cmd} {}", args.join(" ")));
                if cmd == "mkdir" {
                    fs::create_dir_all(&destination_dir)?;
                }
                Ok(())
            },
            &mut || {
                clean_called.set(true);
                Ok(())
            },
        )?;

        assert!(!clean_called.get());
        assert_eq!(root_calls.borrow().len(), 2);
        assert!(root_calls.borrow()[1].starts_with("rsync -a --delete "));

        fs::remove_dir_all(root)?;
        Ok(())
    }

    fn create_temp_repo() -> Result<PathBuf> {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let root = std::env::temp_dir().join(format!(
            "mcbctl-deploy-store-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&root)?;
        Ok(root)
    }

    fn test_lock() -> MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
