use anyhow::{Result, bail};
use mcbctl::tui::state::{AppContext, AppState};
use mcbctl::{exit_from_status, tui::app};
use std::path::PathBuf;
use std::process::Command;

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
        _ => {}
    }

    usage();
    bail!("不支持的子命令；部署请使用 `mcbctl deploy` 或 `mcb-deploy`。")
}

fn launch_tui() -> Result<()> {
    let context = AppContext::detect()?;
    let state = AppState::new(context);
    app::run(state)
}

fn usage() {
    println!(
        "用法:\n  mcbctl\n  mcbctl tui\n  mcbctl deploy\n  mcbctl release\n\n说明:\n  默认进入 TUI 控制台。\n  `mcbctl deploy` 会启动交互式部署向导。\n  `mcbctl release` 会转发到发布流程。"
    );
}

fn run_sibling(name: &str, args: &[String]) -> Result<()> {
    let binary = resolve_sibling_binary(name)?;
    let status = Command::new(&binary).args(args).status()?;
    if status.success() {
        Ok(())
    } else {
        exit_from_status(status)
    }
}

fn resolve_sibling_binary(name: &str) -> Result<PathBuf> {
    let current = std::env::current_exe()?;
    let Some(dir) = current.parent() else {
        bail!("无法定位当前可执行文件目录。");
    };
    let candidate = dir.join(name);
    if candidate.is_file() {
        Ok(candidate)
    } else {
        bail!("未找到同级命令：{}", candidate.display());
    }
}
