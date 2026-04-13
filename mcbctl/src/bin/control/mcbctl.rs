use anyhow::{Context, Result, bail};
use mcbctl::domain::tui::DeployAction;
use mcbctl::repo::{audit_repository, ensure_repository_integrity, preferred_remote_branch};
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

#[derive(Clone, Copy)]
enum SudoMode {
    Auto,
    Always,
    Never,
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
        "用法:\n  mcbctl\n  mcbctl tui\n  mcbctl deploy [--help]\n  mcbctl release\n  mcbctl rebuild <switch|test|boot|build> [host] [--flake <path>] [--upgrade] [--sudo|--no-sudo]\n  mcbctl build-host [host] [--flake <path>] [--dry-run]\n  mcbctl repo-integrity [--root <path>]\n  mcbctl lint-repo [--root <path>]\n  mcbctl doctor [--root <path>]\n  mcbctl terminal-action <flake-status|flake-hint|sensors|memory|disk>\n  mcbctl screenshot-edit <full|region>\n\n说明:\n  默认进入 TUI 控制台。\n  `mcbctl deploy` 会转发到交互式部署向导。\n  `mcbctl release` 会转发到发布流程。\n  `rebuild` / `build-host` 是 fish 快捷入口背后的 Rust 主线命令。\n  `repo-integrity` / `lint-repo` / `doctor` 用于 Rust 主线下的仓库校验。"
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

fn run_rebuild(args: &[String]) -> Result<()> {
    if has_help_flag(args) || args.is_empty() {
        println!(
            "用法:\n  mcbctl rebuild <switch|test|boot|build> [host] [--flake <path>] [--upgrade] [--sudo|--no-sudo]"
        );
        return Ok(());
    }

    let action = match args[0].as_str() {
        "switch" => DeployAction::Switch,
        "test" => DeployAction::Test,
        "boot" => DeployAction::Boot,
        "build" => DeployAction::Build,
        other => bail!("不支持的 rebuild 模式：{other}"),
    };

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

    let target_host =
        host.unwrap_or_else(|| current_host_name().unwrap_or_else(|| "<hostname>".to_string()));
    if target_host == "<hostname>" {
        bail!("无法推断主机名，请显式传入 host");
    }

    let plan = NixosRebuildPlan {
        action,
        upgrade,
        flake_root: flake_root.unwrap_or_else(default_flake_root),
        target_host,
    };

    let status = run_nixos_rebuild(&plan, resolve_sudo_mode(sudo_mode))?;
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
        let target_host =
            host.unwrap_or_else(|| current_host_name().unwrap_or_else(|| "<hostname>".to_string()));
        if target_host == "<hostname>" {
            bail!("无法推断主机名，请显式传入 host 或 --target");
        }
        format!(
            "{}#nixosConfigurations.{}.config.system.build.toplevel",
            flake_root.unwrap_or_else(default_flake_root).display(),
            target_host
        )
    };

    run_nix_build_target(&build_target, dry_run)
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
    let report = audit_repository(&root)?;
    ensure_required_layout(&root)?;

    println!("doctor");
    println!("repo root: {}", root.display());
    println!("remote branch: {}", preferred_remote_branch(&root));
    println!(
        "git: {}",
        if command_exists("git") {
            "ok"
        } else {
            "missing"
        }
    );
    println!(
        "nix: {}",
        if command_exists("nix") {
            "ok"
        } else {
            "missing"
        }
    );
    println!(
        "nixos-rebuild: {}",
        if command_exists("nixos-rebuild") {
            "ok"
        } else {
            "missing"
        }
    );
    println!(
        "cargo: {}",
        if command_exists("cargo") {
            "ok"
        } else {
            "missing"
        }
    );
    println!(
        "user: {}",
        run_capture_allow_fail("id", &["-un"])
            .map(|user| user.trim().to_string())
            .filter(|user| !user.is_empty())
            .unwrap_or_else(|| "unknown".to_string())
    );
    println!(
        "uid: {}",
        run_capture_allow_fail("id", &["-u"])
            .map(|uid| uid.trim().to_string())
            .filter(|uid| !uid.is_empty())
            .unwrap_or_else(|| "unknown".to_string())
    );

    if report.is_clean() {
        println!("repo integrity: ok");
        return Ok(());
    }

    println!("repo integrity: failed");
    for line in report.render_lines().into_iter().skip(1) {
        println!("{line}");
    }
    bail!("doctor failed")
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

fn ensure_required_layout(root: &Path) -> Result<()> {
    for rel in [
        "flake.nix",
        "mcbctl/Cargo.toml",
        "hosts",
        "hosts/templates",
        "modules",
        "home",
        "home/templates/users",
        "catalog",
        "pkgs",
        "pkgs/mcbctl/default.nix",
    ] {
        let path = root.join(rel);
        if !path.exists() {
            bail!("缺少必须保留的仓库边界：{}", path.display());
        }
    }
    Ok(())
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

fn chrono_like_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}
