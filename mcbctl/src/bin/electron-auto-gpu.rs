use mcbctl::current_gpu_mode_label;
use std::os::unix::process::CommandExt;
use std::process::Command;

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("Usage: electron-auto-gpu <command> [args...]");
        std::process::exit(2);
    }

    let app = args.remove(0);
    let mode = current_gpu_mode_label();
    let mut cmd = Command::new(&app);
    cmd.args(&args);

    if mode == "dgpu" {
        cmd.env("NIXOS_OZONE_WL", "0");
        cmd.env("ELECTRON_OZONE_PLATFORM_HINT", "x11");
        cmd.env("OZONE_PLATFORM", "x11");
    }

    let err = cmd.exec();
    eprintln!("electron-auto-gpu: failed to exec {app}: {err}");
    std::process::exit(127);
}
