use anyhow::{Result, anyhow};
use std::path::PathBuf;
use std::process::Command;

fn usage() {
    eprintln!(
        "Usage:\n  update-upstream-apps          # update source pins\n  update-upstream-apps --check  # check source pins are up-to-date"
    );
}

fn resolve_peer_bin(name: &str) -> String {
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        let p = PathBuf::from(dir).join(name);
        if p.is_file() {
            return p.to_string_lossy().to_string();
        }
    }
    name.to_string()
}

fn run(name: &str, check: bool) -> Result<()> {
    let bin = resolve_peer_bin(name);
    let mut cmd = Command::new(&bin);
    if check {
        cmd.arg("--check");
    }
    let status = cmd
        .status()
        .map_err(|e| anyhow!("failed to run {bin}: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("{name} failed with {}", status.code().unwrap_or(1)))
    }
}

fn main() {
    let mut check = false;
    let mut args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.first().is_some_and(|a| a == "--check") {
        check = true;
        args.remove(0);
    }
    if !args.is_empty() {
        usage();
        std::process::exit(2);
    }

    let result =
        run("update-zed-source", check).and_then(|_| run("update-yesplaymusic-source", check));
    match result {
        Ok(_) => {
            if check {
                println!("done: upstream app source pins are up-to-date (zed, yesplaymusic)");
            } else {
                println!("done: updated upstream app source pins (zed, yesplaymusic)");
            }
        }
        Err(err) => {
            eprintln!("update-upstream-apps: {err:#}");
            std::process::exit(1);
        }
    }
}
