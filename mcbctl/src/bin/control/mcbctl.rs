use anyhow::{Result, bail};
use mcbctl::tui::state::{AppContext, AppState};
use mcbctl::{exit_from_status, run_sibling_status, tui::app};

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
    let status = run_sibling_status(name, args)?;
    if status.success() {
        Ok(())
    } else {
        exit_from_status(status)
    }
}
