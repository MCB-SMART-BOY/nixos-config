use anyhow::{Context, Result, bail};
use mcbctl::domain::tui::DeployAction;
#[cfg(test)]
use mcbctl::health::{DoctorToolStatus, assess_doctor_environment};
use mcbctl::health::{collect_doctor_report, ensure_required_layout, tool_status_label};
use mcbctl::release_bundle::{ReleaseBundleOptions, build_release_bundle};
use mcbctl::repo::{
    ensure_repository_integrity, extract_manual_managed_files, migrate_managed_files,
    migrate_root_hardware_config,
};
use mcbctl::store::deploy::{NixosRebuildPlan, merged_nix_config, run_nixos_rebuild};
use mcbctl::tui::state::{AppContext, AppState};
use mcbctl::{
    command_exists, exit_from_status, find_repo_root, home_dir, run_capture_allow_fail,
    run_sibling_status, tui::app, xdg_cache_home,
};
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Clone, Copy)]
enum TerminalAction {
    FlakeStatus,
    FlakeHint,
    Sensors,
    Memory,
    Disk,
}

#[derive(Clone, Copy)]
enum ScreenshotMode {
    Full,
    Region,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SudoMode {
    Auto,
    Always,
    Never,
}

#[derive(Clone, Debug)]
struct RebuildRequest {
    plan: NixosRebuildPlan,
    sudo_mode: SudoMode,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BuildHostRequest {
    build_target: String,
    dry_run: bool,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("mcbctl: {err:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        return launch_tui();
    }

    match args[0].as_str() {
        "-h" | "--help" => {
            usage();
            return Ok(());
        }
        "tui" => {
            return launch_tui();
        }
        "deploy" => {
            return run_sibling("mcb-deploy", &args[1..]);
        }
        "release" => {
            return run_sibling("mcb-deploy", &["release".to_string()]);
        }
        "rebuild" => {
            return run_rebuild(&args[1..]);
        }
        "build-host" => {
            return run_build_host(&args[1..]);
        }
        "repo-integrity" => {
            return run_repo_integrity(&args[1..]);
        }
        "migrate-managed" => {
            return run_migrate_managed(&args[1..]);
        }
        "extract-managed" => {
            return run_extract_managed(&args[1..]);
        }
        "migrate-hardware-config" => {
            return run_migrate_hardware_config(&args[1..]);
        }
        "release-bundle" => {
            return run_release_bundle(&args[1..]);
        }
        "lint-repo" => {
            return run_lint_repo(&args[1..]);
        }
        "doctor" => {
            return run_doctor(&args[1..]);
        }
        "terminal-action" => {
            return run_terminal_action(&args[1..]);
        }
        "screenshot-edit" => {
            return run_screenshot_edit(&args[1..]);
        }
        _ => {}
    }

    usage();
    bail!("不支持的子命令。")
}

fn launch_tui() -> Result<()> {
    let context = AppContext::detect()?;
    let state = AppState::new(context);
    app::run(state)
}

fn usage() {
    println!(
        "用法:\n  mcbctl\n  mcbctl tui\n  mcbctl deploy [--help]\n  mcbctl release\n  mcbctl rebuild <switch|test|boot|build> [host] [--flake <path>] [--upgrade] [--sudo|--no-sudo]\n  mcbctl build-host [host] [--flake <path>] [--dry-run]\n  mcbctl repo-integrity [--root <path>]\n  mcbctl migrate-managed [--root <path>]\n  mcbctl extract-managed [--root <path>]\n  mcbctl migrate-hardware-config [--root <path>] [--host <name>]\n  mcbctl release-bundle --target <triple> --bin-dir <path> --out-dir <path> [--version <tag>]\n  mcbctl lint-repo [--root <path>]\n  mcbctl doctor [--root <path>]\n  mcbctl terminal-action <flake-status|flake-hint|sensors|memory|disk>\n  mcbctl screenshot-edit <full|region>\n\n说明:\n  默认进入 TUI 控制台。\n  `mcbctl deploy` 会转发到交互式部署向导。\n  `mcbctl release` 会转发到发布流程。\n  `rebuild` / `build-host` 是 fish 快捷入口背后的 Rust 主线命令。\n  `repo-integrity` / `migrate-managed` / `extract-managed` / `migrate-hardware-config` / `release-bundle` / `lint-repo` / `doctor` 用于 Rust 主线下的仓库校验、迁移与发布产物打包。"
    );
}

fn run_sibling(name: &str, args: &[String]) -> Result<()> {
    let status = run_sibling_status(name, args)?;
    if status.success() {
        Ok(())
    } else {
        exit_from_status(status)
    }
}

fn run_repo_integrity(args: &[String]) -> Result<()> {
    if has_help_flag(args) {
        println!("用法:\n  mcbctl repo-integrity [--root <path>]");
        return Ok(());
    }
    let root = parse_root_arg(args)?;
    ensure_repository_integrity(&root)?;
    println!("repo-integrity: ok ({})", root.display());
    Ok(())
}

fn run_migrate_managed(args: &[String]) -> Result<()> {
    if has_help_flag(args) {
        println!("用法:\n  mcbctl migrate-managed [--root <path>]");
        return Ok(());
    }

    let root = parse_root_arg(args)?;
    let report = migrate_managed_files(&root)?;
    println!("migrate-managed");
    println!("repo root: {}", root.display());
    println!("migrated: {}", report.migrated.len());
    for path in &report.migrated {
        println!("  migrated {path}");
    }
    println!("skipped: {}", report.skipped.len());
    for path in &report.skipped {
        println!("  ok {path}");
    }
    Ok(())
}

fn run_extract_managed(args: &[String]) -> Result<()> {
    if has_help_flag(args) {
        println!("用法:\n  mcbctl extract-managed [--root <path>]");
        return Ok(());
    }

    let root = parse_root_arg(args)?;
    let report = extract_manual_managed_files(&root)?;
    println!("extract-managed");
    println!("repo root: {}", root.display());
    println!("extracted: {}", report.extracted.len());
    for path in &report.extracted {
        println!("  extracted {path}");
    }
    println!("already-managed: {}", report.skipped_valid.len());
    println!("legacy-needs-migrate: {}", report.skipped_legacy.len());
    for path in &report.skipped_legacy {
        println!("  migrate first {path}");
    }
    Ok(())
}

fn run_migrate_hardware_config(args: &[String]) -> Result<()> {
    if has_help_flag(args) {
        println!("用法:\n  mcbctl migrate-hardware-config [--root <path>] [--host <name>]");
        return Ok(());
    }

    let (root, host) = parse_root_host_args(args)?;
    let report = migrate_root_hardware_config(&root, &host)?;
    println!("migrate-hardware-config");
    println!("repo root: {}", root.display());
    println!("target host: {host}");
    println!(
        "{}: {}",
        if report.moved {
            "migrated"
        } else {
            "already-present"
        },
        report.destination
    );
    Ok(())
}

fn run_release_bundle(args: &[String]) -> Result<()> {
    if has_help_flag(args) {
        println!(
            "用法:\n  mcbctl release-bundle --target <triple> --bin-dir <path> --out-dir <path> [--version <tag>]"
        );
        return Ok(());
    }

    let mut target = None;
    let mut bin_dir = None;
    let mut out_dir = None;
    let mut version = None;
    let mut idx = 0usize;
    while idx < args.len() {
        match args[idx].as_str() {
            "--target" => {
                let Some(value) = args.get(idx + 1) else {
                    bail!("--target 缺少三元组");
                };
                target = Some(value.to_string());
                idx += 2;
            }
            "--bin-dir" => {
                let Some(value) = args.get(idx + 1) else {
                    bail!("--bin-dir 缺少路径");
                };
                bin_dir = Some(PathBuf::from(value));
                idx += 2;
            }
            "--out-dir" => {
                let Some(value) = args.get(idx + 1) else {
                    bail!("--out-dir 缺少路径");
                };
                out_dir = Some(PathBuf::from(value));
                idx += 2;
            }
            "--version" => {
                let Some(value) = args.get(idx + 1) else {
                    bail!("--version 缺少版本");
                };
                version = Some(value.to_string());
                idx += 2;
            }
            other => bail!("不支持的参数：{other}"),
        }
    }

    let options = ReleaseBundleOptions {
        target: target.ok_or_else(|| anyhow::anyhow!("缺少 --target"))?,
        version: version.unwrap_or_else(default_release_bundle_version),
        bin_dir: bin_dir.ok_or_else(|| anyhow::anyhow!("缺少 --bin-dir"))?,
        out_dir: out_dir.ok_or_else(|| anyhow::anyhow!("缺少 --out-dir"))?,
    };
    let report = build_release_bundle(&options)?;
    println!("release-bundle");
    println!("target: {}", options.target);
    println!("version: {}", options.version);
    println!("archive: {}", report.archive.display());
    println!("checksum: {}", report.checksum_file.display());
    Ok(())
}

fn run_rebuild(args: &[String]) -> Result<()> {
    if has_help_flag(args) || args.is_empty() {
        println!(
            "用法:\n  mcbctl rebuild <switch|test|boot|build> [host] [--flake <path>] [--upgrade] [--sudo|--no-sudo]"
        );
        return Ok(());
    }

    let request = parse_rebuild_request(args, current_host_name(), default_flake_root())?;
    let status = run_nixos_rebuild(&request.plan, resolve_sudo_mode(request.sudo_mode))?;
    if status.success() {
        Ok(())
    } else {
        exit_from_status(status)
    }
}

fn run_build_host(args: &[String]) -> Result<()> {
    if has_help_flag(args) {
        println!(
            "用法:\n  mcbctl build-host [host] [--flake <path>] [--dry-run]\n  mcbctl build-host --target <flake-ref> [--dry-run]"
        );
        return Ok(());
    }

    let request = parse_build_host_request(args, current_host_name(), default_flake_root())?;
    run_nix_build_target(&request.build_target, request.dry_run)
}

fn run_lint_repo(args: &[String]) -> Result<()> {
    if has_help_flag(args) {
        println!("用法:\n  mcbctl lint-repo [--root <path>]");
        return Ok(());
    }
    let root = parse_root_arg(args)?;
    ensure_repository_integrity(&root)?;
    ensure_required_layout(&root)?;
    println!("lint-repo: ok ({})", root.display());
    Ok(())
}

fn run_doctor(args: &[String]) -> Result<()> {
    if has_help_flag(args) {
        println!("用法:\n  mcbctl doctor [--root <path>]");
        return Ok(());
    }
    let root = parse_root_arg(args)?;
    let report = collect_doctor_report(&root)?;

    println!("doctor");
    println!("repo root: {}", report.repo_root.display());
    println!("remote branch: {}", report.remote_branch);
    println!("git: {}", tool_status_label(report.tools.git));
    println!("nix: {}", tool_status_label(report.tools.nix));
    println!(
        "nixos-rebuild: {}",
        tool_status_label(report.tools.nixos_rebuild)
    );
    println!("cargo: {}", tool_status_label(report.tools.cargo));
    println!("repo hardware config: {}", report.repo_hardware);
    println!(
        "legacy root hardware config: {}",
        if report.legacy_root_hardware {
            "present (run `mcbctl migrate-hardware-config`)"
        } else {
            "absent"
        }
    );
    println!("user: {}", report.current_user);
    println!("uid: {}", report.current_uid);
    println!(
        "repo layout: {}",
        report
            .layout_error
            .as_ref()
            .map(|err| format!("failed ({err})"))
            .unwrap_or_else(|| "ok".to_string())
    );
    println!(
        "deployment environment: {}",
        if report.assessment.blocking_issues.is_empty() {
            "ok"
        } else {
            "failed"
        }
    );
    for issue in &report.assessment.blocking_issues {
        println!("- {issue}");
    }
    if !report.assessment.warnings.is_empty() {
        println!("environment warnings:");
        for warning in &report.assessment.warnings {
            println!("- {warning}");
        }
    }

    if report.integrity_clean {
        println!("repo integrity: ok");
    } else {
        println!("repo integrity: failed");
        for line in report.integrity_lines.iter().skip(1) {
            println!("{line}");
        }
    }

    let failures = report.failure_lines();
    if failures.is_empty() {
        return Ok(());
    }
    bail!("doctor failed: {}", failures.join(" | "))
}

fn resolve_sudo_mode(mode: SudoMode) -> bool {
    match mode {
        SudoMode::Always => true,
        SudoMode::Never => false,
        SudoMode::Auto => run_capture_allow_fail("id", &["-u"])
            .map(|uid| uid.trim() != "0")
            .unwrap_or(true),
    }
}

fn default_flake_root() -> PathBuf {
    let etc_nixos = PathBuf::from("/etc/nixos");
    if looks_like_repo(&etc_nixos) {
        return etc_nixos;
    }
    find_repo_root().unwrap_or_else(|_| preferred_terminal_repo_dir())
}

fn parse_rebuild_request(
    args: &[String],
    current_host: Option<String>,
    fallback_flake_root: PathBuf,
) -> Result<RebuildRequest> {
    let Some(mode) = args.first() else {
        bail!("缺少 rebuild 模式");
    };
    let action = parse_rebuild_action(mode)?;

    let mut host = None;
    let mut flake_root = None;
    let mut upgrade = false;
    let mut sudo_mode = SudoMode::Auto;
    let mut idx = 1usize;
    while idx < args.len() {
        match args[idx].as_str() {
            "--flake" => {
                let Some(value) = args.get(idx + 1) else {
                    bail!("--flake 缺少路径");
                };
                flake_root = Some(PathBuf::from(value));
                idx += 2;
            }
            "--upgrade" => {
                upgrade = true;
                idx += 1;
            }
            "--sudo" => {
                sudo_mode = SudoMode::Always;
                idx += 1;
            }
            "--no-sudo" => {
                sudo_mode = SudoMode::Never;
                idx += 1;
            }
            other if other.starts_with('-') => bail!("不支持的参数：{other}"),
            other => {
                if host.replace(other.to_string()).is_some() {
                    bail!("只能指定一个目标主机");
                }
                idx += 1;
            }
        }
    }

    let target_host = resolve_required_host(host, current_host, "无法推断主机名，请显式传入 host")?;
    Ok(RebuildRequest {
        plan: NixosRebuildPlan {
            action,
            upgrade,
            flake_root: flake_root.unwrap_or(fallback_flake_root),
            target_host,
        },
        sudo_mode,
    })
}

fn parse_build_host_request(
    args: &[String],
    current_host: Option<String>,
    fallback_flake_root: PathBuf,
) -> Result<BuildHostRequest> {
    let mut host = None;
    let mut flake_root = None;
    let mut dry_run = false;
    let mut target = None;
    let mut idx = 0usize;
    while idx < args.len() {
        match args[idx].as_str() {
            "--flake" => {
                let Some(value) = args.get(idx + 1) else {
                    bail!("--flake 缺少路径");
                };
                flake_root = Some(PathBuf::from(value));
                idx += 2;
            }
            "--dry-run" => {
                dry_run = true;
                idx += 1;
            }
            "--target" => {
                let Some(value) = args.get(idx + 1) else {
                    bail!("--target 缺少 flake 引用");
                };
                target = Some(value.to_string());
                idx += 2;
            }
            other if other.starts_with('-') => bail!("不支持的参数：{other}"),
            other => {
                if host.replace(other.to_string()).is_some() {
                    bail!("只能指定一个目标主机");
                }
                idx += 1;
            }
        }
    }

    if target.is_some() && (host.is_some() || flake_root.is_some()) {
        bail!("--target 不能和 host/--flake 同时使用");
    }

    let build_target = if let Some(target) = target {
        target
    } else {
        let target_host = resolve_required_host(
            host,
            current_host,
            "无法推断主机名，请显式传入 host 或 --target",
        )?;
        format!(
            "{}#nixosConfigurations.{}.config.system.build.toplevel",
            flake_root.unwrap_or(fallback_flake_root).display(),
            target_host
        )
    };

    Ok(BuildHostRequest {
        build_target,
        dry_run,
    })
}

fn parse_rebuild_action(mode: &str) -> Result<DeployAction> {
    match mode {
        "switch" => Ok(DeployAction::Switch),
        "test" => Ok(DeployAction::Test),
        "boot" => Ok(DeployAction::Boot),
        "build" => Ok(DeployAction::Build),
        other => bail!("不支持的 rebuild 模式：{other}"),
    }
}

fn resolve_required_host(
    explicit_host: Option<String>,
    current_host: Option<String>,
    missing_message: &str,
) -> Result<String> {
    explicit_host
        .or(current_host)
        .filter(|host| !host.trim().is_empty() && host != "<hostname>")
        .ok_or_else(|| anyhow::anyhow!(missing_message.to_string()))
}

fn run_nix_build_target(target: &str, dry_run: bool) -> Result<()> {
    if !command_exists("nix") {
        bail!("未找到 nix");
    }

    let mut cmd = Command::new("env");
    cmd.arg(format!("NIX_CONFIG={}", merged_nix_config()))
        .arg("nix")
        .arg("build")
        .arg(target)
        .arg("--accept-flake-config")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    if dry_run {
        cmd.arg("--dry-run");
    }

    let status = cmd.status().context("failed to run nix build")?;
    if status.success() {
        Ok(())
    } else {
        exit_from_status(status)
    }
}

fn run_terminal_action(args: &[String]) -> Result<()> {
    if has_help_flag(args) || args.len() != 1 {
        println!("用法:\n  mcbctl terminal-action <flake-status|flake-hint|sensors|memory|disk>");
        return Ok(());
    }

    let action = match args[0].as_str() {
        "flake-status" => TerminalAction::FlakeStatus,
        "flake-hint" => TerminalAction::FlakeHint,
        "sensors" => TerminalAction::Sensors,
        "memory" => TerminalAction::Memory,
        "disk" => TerminalAction::Disk,
        other => bail!("不支持的 terminal-action：{other}"),
    };

    match action {
        TerminalAction::FlakeStatus => terminal_flake_status(),
        TerminalAction::FlakeHint => terminal_flake_hint(),
        TerminalAction::Sensors => terminal_single_command("sensors", &[]),
        TerminalAction::Memory => terminal_memory(),
        TerminalAction::Disk => terminal_disk(),
    }
}

fn run_screenshot_edit(args: &[String]) -> Result<()> {
    if has_help_flag(args) || args.len() != 1 {
        println!("用法:\n  mcbctl screenshot-edit <full|region>");
        return Ok(());
    }

    let mode = match args[0].as_str() {
        "full" => ScreenshotMode::Full,
        "region" => ScreenshotMode::Region,
        other => bail!("不支持的截图模式：{other}"),
    };

    if !command_exists("grim") {
        bail!("缺少 grim");
    }
    if !command_exists("swappy") {
        bail!("缺少 swappy");
    }
    if matches!(mode, ScreenshotMode::Region) && !command_exists("slurp") {
        bail!("区域截图缺少 slurp");
    }

    let cache_dir = xdg_cache_home().join("mcbctl/screenshots");
    fs::create_dir_all(&cache_dir)
        .with_context(|| format!("failed to create {}", cache_dir.display()))?;
    let target = cache_dir.join(format!(
        "capture-{}-{}.png",
        std::process::id(),
        chrono_like_millis()
    ));

    let grim_status = match mode {
        ScreenshotMode::Full => Command::new("grim")
            .arg(&target)
            .status()
            .context("failed to run grim")?,
        ScreenshotMode::Region => {
            let output = Command::new("slurp")
                .output()
                .context("failed to run slurp")?;
            if !output.status.success() {
                bail!("slurp cancelled");
            }
            let geometry = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if geometry.is_empty() {
                bail!("slurp returned empty selection");
            }
            Command::new("grim")
                .args(["-g", geometry.as_str(), target.to_string_lossy().as_ref()])
                .status()
                .context("failed to run grim -g")?
        }
    };
    if !grim_status.success() {
        bail!("grim failed with {}", grim_status.code().unwrap_or(1));
    }

    let swappy_status = Command::new("swappy")
        .args(["-f", target.to_string_lossy().as_ref()])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("failed to run swappy")?;
    fs::remove_file(&target).ok();
    if swappy_status.success() {
        Ok(())
    } else {
        bail!("swappy failed with {}", swappy_status.code().unwrap_or(1))
    }
}

fn parse_root_arg(args: &[String]) -> Result<PathBuf> {
    let mut root = None;
    let mut idx = 0usize;
    while idx < args.len() {
        match args[idx].as_str() {
            "--root" => {
                let Some(value) = args.get(idx + 1) else {
                    bail!("--root 缺少路径");
                };
                root = Some(PathBuf::from(value));
                idx += 2;
            }
            other => bail!("不支持的参数：{other}"),
        }
    }

    if let Some(root) = root {
        Ok(root)
    } else {
        find_repo_root()
    }
}

fn parse_root_host_args(args: &[String]) -> Result<(PathBuf, String)> {
    let mut root = None;
    let mut host = None;
    let mut idx = 0usize;
    while idx < args.len() {
        match args[idx].as_str() {
            "--root" => {
                let Some(value) = args.get(idx + 1) else {
                    bail!("--root 缺少路径");
                };
                root = Some(PathBuf::from(value));
                idx += 2;
            }
            "--host" => {
                let Some(value) = args.get(idx + 1) else {
                    bail!("--host 缺少主机名");
                };
                host = Some(value.to_string());
                idx += 2;
            }
            other => bail!("不支持的参数：{other}"),
        }
    }

    let root = if let Some(root) = root {
        root
    } else {
        find_repo_root()?
    };

    let host = host
        .or_else(current_host_name)
        .or_else(|| infer_only_repo_host(&root))
        .ok_or_else(|| anyhow::anyhow!("无法推断主机名；请显式传入 --host"))?;
    Ok((root, host))
}

fn has_help_flag(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "-h" || arg == "--help")
}

fn terminal_flake_status() -> Result<()> {
    let repo = preferred_terminal_repo_dir();
    println!("repo: {}", repo.display());
    println!();

    if repo.join(".git").exists() {
        let _ = run_command_inherit("git", &["status"], Some(&repo));
        println!();
    } else {
        println!("git status: skipped (.git not present)");
        println!();
    }

    let nix_args = [
        "--extra-experimental-features",
        "nix-command flakes",
        "flake",
        "check",
        "--no-build",
    ];
    let _ = run_command_inherit("nix", &nix_args, Some(&repo));
    println!();
    prompt_close()
}

fn terminal_flake_hint() -> Result<()> {
    let repo = preferred_terminal_repo_dir();
    let repo_label = if repo.join("flake.nix").is_file() {
        repo.display().to_string()
    } else {
        ".".to_string()
    };
    let host = current_host_name().unwrap_or_else(|| "<hostname>".to_string());

    println!("repo: {}", repo.display());
    println!();
    println!("推荐动作：");
    println!("  cd {repo_label}");
    println!("  nix flake update");
    println!("  nix run .#update-upstream-apps -- --check");
    println!("  sudo nixos-rebuild switch --flake .#{host}");
    println!();
    prompt_close()
}

fn terminal_memory() -> Result<()> {
    let _ = run_command_inherit("free", &["-h"], None);
    println!();

    if command_exists("vmstat") {
        let output = Command::new("vmstat")
            .arg("-s")
            .output()
            .context("failed to run vmstat -s")?;
        if output.status.success() {
            for line in String::from_utf8_lossy(&output.stdout).lines().take(20) {
                println!("{line}");
            }
        } else {
            println!(
                "vmstat -s failed with {}",
                output.status.code().unwrap_or(1)
            );
        }
    } else {
        println!("vmstat: missing");
    }

    println!();
    prompt_close()
}

fn terminal_disk() -> Result<()> {
    let _ = run_command_inherit("df", &["-h"], None);
    println!();
    let _ = run_command_inherit("lsblk", &[], None);
    println!();
    prompt_close()
}

fn terminal_single_command(cmd: &str, args: &[&str]) -> Result<()> {
    let _ = run_command_inherit(cmd, args, None);
    println!();
    prompt_close()
}

fn run_command_inherit(cmd: &str, args: &[&str], cwd: Option<&Path>) -> Result<()> {
    if !command_exists(cmd) {
        println!("{cmd}: missing");
        return Ok(());
    }

    let mut command = Command::new(cmd);
    command
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    let status = command
        .status()
        .with_context(|| format!("failed to run {cmd}"))?;
    if !status.success() {
        println!("{cmd} exited with {}", status.code().unwrap_or(1));
    }
    Ok(())
}

fn prompt_close() -> Result<()> {
    if !io::stdin().is_terminal() {
        return Ok(());
    }
    print!("Press Enter to close...");
    io::stdout().flush().ok();
    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .context("failed to read terminal confirmation")?;
    Ok(())
}

fn preferred_terminal_repo_dir() -> PathBuf {
    let mut candidates = Vec::<PathBuf>::new();
    candidates.push(PathBuf::from("/etc/nixos"));
    candidates.push(home_dir().join("nixos-config"));
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd);
    }
    candidates.push(home_dir());

    for candidate in candidates {
        if looks_like_repo(&candidate) {
            return candidate;
        }
    }
    home_dir()
}

fn looks_like_repo(path: &Path) -> bool {
    path.join("flake.nix").is_file()
        && path.join("hosts").is_dir()
        && path.join("modules").is_dir()
        && path.join("home").is_dir()
}

fn current_host_name() -> Option<String> {
    fs::read_to_string("/etc/hostname")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| {
            run_capture_allow_fail("hostname", &[])
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        })
}

fn infer_only_repo_host(root: &Path) -> Option<String> {
    let mut hosts = fs::read_dir(root.join("hosts"))
        .ok()?
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            (!matches!(name.as_str(), "_support" | "profiles" | "templates")).then_some(name)
        })
        .collect::<Vec<_>>();
    hosts.sort();
    (hosts.len() == 1).then(|| hosts.remove(0))
}

fn chrono_like_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

fn default_release_bundle_version() -> String {
    std::env::var("GITHUB_REF_NAME")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            run_capture_allow_fail("git", &["describe", "--tags", "--abbrev=0"])
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        })
        .unwrap_or_else(|| "dev".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_rebuild_request_parses_flags_and_host() -> Result<()> {
        let request = parse_rebuild_request(
            &[
                "switch".to_string(),
                "demo".to_string(),
                "--upgrade".to_string(),
                "--sudo".to_string(),
                "--flake".to_string(),
                "/tmp/flake".to_string(),
            ],
            Some("ignored".to_string()),
            PathBuf::from("/fallback"),
        )?;

        assert_eq!(request.plan.action, DeployAction::Switch);
        assert!(request.plan.upgrade);
        assert_eq!(request.plan.target_host, "demo");
        assert_eq!(request.plan.flake_root, PathBuf::from("/tmp/flake"));
        assert_eq!(request.sudo_mode, SudoMode::Always);
        Ok(())
    }

    #[test]
    fn parse_rebuild_request_uses_detected_host_when_missing() -> Result<()> {
        let request = parse_rebuild_request(
            &["build".to_string()],
            Some("nixos".to_string()),
            PathBuf::from("/repo"),
        )?;

        assert_eq!(request.plan.action, DeployAction::Build);
        assert_eq!(request.plan.target_host, "nixos");
        assert_eq!(request.plan.flake_root, PathBuf::from("/repo"));
        assert_eq!(request.sudo_mode, SudoMode::Auto);
        Ok(())
    }

    #[test]
    fn parse_rebuild_request_rejects_multiple_hosts() {
        let err = parse_rebuild_request(
            &[
                "switch".to_string(),
                "host-a".to_string(),
                "host-b".to_string(),
            ],
            None,
            PathBuf::from("/repo"),
        )
        .expect_err("multiple hosts should be rejected");
        assert!(err.to_string().contains("只能指定一个目标主机"));
    }

    #[test]
    fn parse_rebuild_request_requires_host_when_undetectable() {
        let err = parse_rebuild_request(&["switch".to_string()], None, PathBuf::from("/repo"))
            .expect_err("missing host should be rejected");
        assert!(err.to_string().contains("无法推断主机名"));
    }

    #[test]
    fn parse_build_host_request_builds_default_target() -> Result<()> {
        let request = parse_build_host_request(
            &["demo".to_string(), "--dry-run".to_string()],
            Some("ignored".to_string()),
            PathBuf::from("/repo"),
        )?;

        assert_eq!(
            request.build_target,
            "/repo#nixosConfigurations.demo.config.system.build.toplevel"
        );
        assert!(request.dry_run);
        Ok(())
    }

    #[test]
    fn parse_build_host_request_accepts_explicit_target() -> Result<()> {
        let request = parse_build_host_request(
            &[
                "--target".to_string(),
                ".#nixosConfigurations.demo.config.system.build.toplevel".to_string(),
            ],
            None,
            PathBuf::from("/repo"),
        )?;

        assert_eq!(
            request.build_target,
            ".#nixosConfigurations.demo.config.system.build.toplevel"
        );
        assert!(!request.dry_run);
        Ok(())
    }

    #[test]
    fn parse_build_host_request_rejects_target_with_host_or_flake() {
        let err = parse_build_host_request(
            &[
                "--target".to_string(),
                ".#foo".to_string(),
                "--flake".to_string(),
                "/repo".to_string(),
            ],
            None,
            PathBuf::from("/fallback"),
        )
        .expect_err("explicit target should not allow host or flake");
        assert!(
            err.to_string()
                .contains("--target 不能和 host/--flake 同时使用")
        );
    }

    #[test]
    fn parse_build_host_request_requires_target_or_host_when_undetectable() {
        let err = parse_build_host_request(&[], None, PathBuf::from("/repo"))
            .expect_err("missing target should be rejected");
        assert!(err.to_string().contains("无法推断主机名"));
    }

    #[test]
    fn resolve_required_host_prefers_explicit_value() -> Result<()> {
        let host = resolve_required_host(
            Some("explicit".to_string()),
            Some("detected".to_string()),
            "missing",
        )?;
        assert_eq!(host, "explicit");
        Ok(())
    }

    #[test]
    fn infer_only_repo_host_ignores_support_directories() -> Result<()> {
        let root = create_temp_repo()?;
        for name in ["_support", "profiles", "templates", "demo"] {
            std::fs::create_dir_all(root.join("hosts").join(name))?;
        }

        assert_eq!(infer_only_repo_host(&root), Some("demo".to_string()));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn infer_only_repo_host_returns_none_for_multiple_hosts() -> Result<()> {
        let root = create_temp_repo()?;
        for name in ["demo-a", "demo-b"] {
            std::fs::create_dir_all(root.join("hosts").join(name))?;
        }

        assert_eq!(infer_only_repo_host(&root), None);

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn assess_doctor_environment_requires_nix_and_warns_for_nixos_rebuild() {
        let assessment = assess_doctor_environment(DoctorToolStatus {
            git: true,
            nix: false,
            nixos_rebuild: false,
            cargo: true,
        });

        assert_eq!(assessment.blocking_issues.len(), 1);
        assert!(
            assessment
                .blocking_issues
                .iter()
                .any(|issue| issue.contains("缺少 nix"))
        );
        assert!(
            assessment
                .warnings
                .iter()
                .any(|warning| warning.contains("缺少 nixos-rebuild"))
        );
        assert_eq!(assessment.warnings.len(), 1);
    }

    #[test]
    fn assess_doctor_environment_only_warns_for_git_and_cargo() {
        let assessment = assess_doctor_environment(DoctorToolStatus {
            git: false,
            nix: true,
            nixos_rebuild: true,
            cargo: false,
        });

        assert!(assessment.blocking_issues.is_empty());
        assert_eq!(assessment.warnings.len(), 2);
        assert!(
            assessment
                .warnings
                .iter()
                .any(|warning| warning.contains("缺少 git"))
        );
        assert!(
            assessment
                .warnings
                .iter()
                .any(|warning| warning.contains("缺少 cargo"))
        );
    }

    fn create_temp_repo() -> Result<PathBuf> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let root =
            std::env::temp_dir().join(format!("mcbctl-cli-parse-{}-{unique}", std::process::id()));
        std::fs::create_dir_all(root.join("hosts"))?;
        Ok(root)
    }
}
