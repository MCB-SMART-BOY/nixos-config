use scripts_rs::{exit_from_status, home_dir, prepend_paths, run_status_inherit};
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("niri-run-rs: missing command");
        std::process::exit(2);
    }

    let user = std::env::var("USER").unwrap_or_default();
    let home = home_dir();
    prepend_paths(&[
        PathBuf::from("/run/current-system/sw/bin"),
        PathBuf::from("/run/wrappers/bin"),
        home.join(".local/bin"),
        home.join(".nix-profile/bin"),
        PathBuf::from(format!("/etc/profiles/per-user/{user}/bin")),
    ]);

    match run_status_inherit(&args[0], &args[1..]) {
        Ok(status) => exit_from_status(status),
        Err(err) => {
            eprintln!("niri-run-rs: {err:#}");
            std::process::exit(1);
        }
    }
}
