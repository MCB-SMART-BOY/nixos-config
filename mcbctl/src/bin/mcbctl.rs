use anyhow::{Result, bail};
use mcbctl::tui::app;
use mcbctl::tui::state::{AppContext, AppState};

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

    if args.len() == 1 && (args[0] == "-h" || args[0] == "--help") {
        usage();
        return Ok(());
    }

    usage();
    bail!("mcbctl 现在是 TUI 控制台入口；直接部署请运行 mcb-deploy。")
}

fn launch_tui() -> Result<()> {
    let context = AppContext::detect()?;
    let state = AppState::new(context);
    app::run(state)
}

fn usage() {
    println!(
        "用法:\n  mcbctl\n\n说明:\n  默认进入 TUI 控制台。\n  如果你要直接进入交互式部署向导，请运行 mcb-deploy。"
    );
}
