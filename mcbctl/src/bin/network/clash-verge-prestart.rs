use anyhow::{Context, Result, anyhow, bail};
use mcbctl::run_capture;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Default)]
struct Args {
    user: String,
    group: String,
    runtime_dir: String,
    iface: String,
    config_home: String,
    data_home: String,
    cache_home: String,
    state_home: String,
}

fn parse_args() -> Result<Args> {
    let mut out = Args::default();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| anyhow!("missing value for {arg}"))?;
        match arg.as_str() {
            "--user" => out.user = value,
            "--group" => out.group = value,
            "--runtime-dir" => out.runtime_dir = value,
            "--iface" => out.iface = value,
            "--config-home" => out.config_home = value,
            "--data-home" => out.data_home = value,
            "--cache-home" => out.cache_home = value,
            "--state-home" => out.state_home = value,
            _ => bail!("unknown argument: {arg}"),
        }
    }

    if out.user.is_empty()
        || out.group.is_empty()
        || out.runtime_dir.is_empty()
        || out.config_home.is_empty()
        || out.data_home.is_empty()
        || out.cache_home.is_empty()
        || out.state_home.is_empty()
    {
        bail!("missing required arguments");
    }
    Ok(out)
}

fn chown_recursive(owner: &str, paths: &[String]) {
    let _ = Command::new("chown")
        .arg("-R")
        .arg(owner)
        .args(paths)
        .status();
}

#[cfg(unix)]
fn set_owner_only_permissions(path: &str) {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).ok();
}

#[cfg(not(unix))]
fn set_owner_only_permissions(_path: &str) {}

fn main() {
    let result = (|| -> Result<()> {
        let args = parse_args()?;
        let uid = run_capture("id", &["-u", &args.user])?;
        let uid = uid.trim().to_string();
        let owner = format!("{}:{}", args.user, args.group);

        let dirs = vec![
            format!("{}/clash-verge", args.config_home),
            format!("{}/clash-verge-rev", args.config_home),
            format!("{}/clash-verge", args.data_home),
            format!("{}/clash-verge-rev", args.data_home),
            format!("{}/clash-verge-rev", args.cache_home),
            format!("{}/clash-verge-rev", args.state_home),
        ];
        for dir in &dirs {
            fs::create_dir_all(dir).with_context(|| format!("failed to create {dir}"))?;
            set_owner_only_permissions(dir);
        }
        chown_recursive(&owner, &dirs);

        let runtime_dir = PathBuf::from(&args.runtime_dir);
        if runtime_dir.is_dir() {
            for entry in fs::read_dir(&runtime_dir).into_iter().flatten().flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("sock") {
                    fs::remove_file(path).ok();
                }
            }
        }

        if !args.iface.is_empty() {
            let exists = Command::new("ip")
                .args(["link", "show", "dev", &args.iface])
                .status()
                .ok()
                .is_some_and(|s| s.success());
            if !exists {
                let status = Command::new("ip")
                    .args([
                        "tuntap",
                        "add",
                        "dev",
                        &args.iface,
                        "mode",
                        "tun",
                        "user",
                        &uid,
                    ])
                    .status()?;
                if !status.success() {
                    bail!("failed to create tun device {}", args.iface);
                }
            }
            let _ = Command::new("ip")
                .args(["link", "set", "dev", &args.iface, "up"])
                .status();
        }

        Ok(())
    })();

    if let Err(err) = result {
        eprintln!("clash-verge-prestart: {err:#}");
        std::process::exit(1);
    }
}
