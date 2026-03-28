use scripts_rs::{exit_from_status, find_repo_root};
use std::path::PathBuf;
use std::process::Command;

fn resolve_run_sh() -> Option<PathBuf> {
    if let Ok(v) = std::env::var("RUN_SH_PATH") {
        let p = PathBuf::from(v);
        if p.is_file() {
            return Some(p);
        }
    }
    if let Ok(root) = find_repo_root() {
        let p = root.join("run.sh");
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

fn main() {
    let Some(run_sh) = resolve_run_sh() else {
        eprintln!("run-rs: cannot locate run.sh");
        eprintln!("set RUN_SH_PATH=/path/to/run.sh or run from repo subtree");
        std::process::exit(1);
    };

    let mut cmd = Command::new("bash");
    cmd.arg(run_sh);
    cmd.args(std::env::args().skip(1));

    match cmd.status() {
        Ok(status) => exit_from_status(status),
        Err(err) => {
            eprintln!("run-rs: {err}");
            std::process::exit(1);
        }
    }
}
