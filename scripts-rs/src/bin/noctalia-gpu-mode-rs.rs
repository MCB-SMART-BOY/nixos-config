use scripts_rs::{exit_from_status, find_repo_root};
use std::path::PathBuf;
use std::process::Command;

fn resolve_script() -> Option<PathBuf> {
    if let Ok(v) = std::env::var("NOCTALIA_GPU_MODE_SH") {
        let p = PathBuf::from(v);
        if p.is_file() {
            return Some(p);
        }
    }
    if let Ok(root) = find_repo_root() {
        let p = root.join("home/users/mcbnixos/scripts/noctalia-gpu-mode");
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

fn main() {
    let Some(script) = resolve_script() else {
        eprintln!("noctalia-gpu-mode-rs: cannot locate source shell script");
        eprintln!("set NOCTALIA_GPU_MODE_SH=/path/to/noctalia-gpu-mode or run from repo subtree");
        std::process::exit(1);
    };

    let mut cmd = Command::new("bash");
    cmd.arg(script);
    cmd.args(std::env::args().skip(1));
    match cmd.status() {
        Ok(status) => exit_from_status(status),
        Err(err) => {
            eprintln!("noctalia-gpu-mode-rs: {err}");
            std::process::exit(1);
        }
    }
}
