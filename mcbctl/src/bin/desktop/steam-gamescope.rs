use mcbctl::{command_exists, home_dir, prepend_paths};
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

fn exec_or_exit(mut cmd: Command, label: &str) -> ! {
    #[cfg(unix)]
    {
        let err = cmd.exec();
        eprintln!("{label}: {err}");
        std::process::exit(1);
    }

    #[cfg(not(unix))]
    {
        match cmd.status() {
            Ok(status) => std::process::exit(status.code().unwrap_or(1)),
            Err(err) => {
                eprintln!("{label}: {err}");
                std::process::exit(1);
            }
        }
    }
}

fn main() {
    let user = std::env::var("USER").unwrap_or_default();
    let home = home_dir();
    prepend_paths(&[
        PathBuf::from("/run/current-system/sw/bin"),
        PathBuf::from("/run/wrappers/bin"),
        home.join(".nix-profile/bin"),
        home.join(".local/bin"),
        PathBuf::from(format!("/etc/profiles/per-user/{user}/bin")),
    ]);

    if !command_exists("gamescope") {
        eprintln!("steam-gamescope: gamescope not found in PATH");
        std::process::exit(1);
    }
    if !command_exists("steam") {
        eprintln!("steam-gamescope: steam not found in PATH");
        std::process::exit(1);
    }

    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut cmd = Command::new("gamescope");
    cmd.arg("-f").arg("--").arg("steam");
    if args.is_empty() {
        cmd.arg("-vgui");
    } else {
        cmd.args(args);
    }
    cmd.env_remove("VK_DRIVER_FILES");
    cmd.env_remove("VK_ICD_FILENAMES");

    exec_or_exit(cmd, "steam-gamescope");
}
