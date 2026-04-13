use mcbctl::current_gpu_mode_label;
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::process::Command;

fn exec_or_exit(mut cmd: Command, label: &str, code: i32) -> ! {
    #[cfg(unix)]
    {
        let err = cmd.exec();
        eprintln!("{label}: {err}");
        std::process::exit(code);
    }

    #[cfg(not(unix))]
    {
        match cmd.status() {
            Ok(status) => std::process::exit(status.code().unwrap_or(code)),
            Err(err) => {
                eprintln!("{label}: {err}");
                std::process::exit(code);
            }
        }
    }
}

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

    exec_or_exit(
        cmd,
        &format!("electron-auto-gpu: failed to exec {app}"),
        127,
    );
}
