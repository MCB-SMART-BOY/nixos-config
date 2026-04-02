use anyhow::{Result, anyhow, bail};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Default, Deserialize)]
struct ConfigFile {
    stamp: String,
    managed_apps_file: String,
    enable_flathub: bool,
    apps: Vec<String>,
    filesystem: Vec<String>,
    envs: Vec<String>,
    extra_args: Vec<String>,
}

#[derive(Default)]
struct Args {
    config: String,
    stamp: String,
    managed_apps_file: String,
    enable_flathub: bool,
    apps: Vec<String>,
    filesystem: Vec<String>,
    envs: Vec<String>,
    extra_args: Vec<String>,
}

fn parse_args() -> Result<Args> {
    let mut out = Args::default();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--config" => out.config = args.next().ok_or_else(|| anyhow!("missing config"))?,
            "--stamp" => out.stamp = args.next().ok_or_else(|| anyhow!("missing stamp"))?,
            "--managed-apps-file" => {
                out.managed_apps_file = args
                    .next()
                    .ok_or_else(|| anyhow!("missing managed apps file"))?
            }
            "--enable-flathub" => out.enable_flathub = true,
            "--app" => out
                .apps
                .push(args.next().ok_or_else(|| anyhow!("missing app"))?),
            "--filesystem" => out
                .filesystem
                .push(args.next().ok_or_else(|| anyhow!("missing filesystem"))?),
            "--env" => out
                .envs
                .push(args.next().ok_or_else(|| anyhow!("missing env"))?),
            "--extra-arg" => out
                .extra_args
                .push(args.next().ok_or_else(|| anyhow!("missing extra arg"))?),
            _ => bail!("unknown argument: {arg}"),
        }
    }
    if !out.config.is_empty() {
        let config: ConfigFile = serde_json::from_str(&fs::read_to_string(&out.config)?)?;
        out.stamp = config.stamp;
        out.managed_apps_file = config.managed_apps_file;
        out.enable_flathub = config.enable_flathub;
        out.apps = config.apps;
        out.filesystem = config.filesystem;
        out.envs = config.envs;
        out.extra_args = config.extra_args;
    }
    if out.stamp.is_empty() || out.managed_apps_file.is_empty() {
        bail!("missing required arguments");
    }
    Ok(out)
}

fn run_ok(cmd: &str, args: &[String]) -> Result<()> {
    let status = Command::new(cmd).args(args).status()?;
    if status.success() {
        Ok(())
    } else {
        bail!("{cmd} failed with {}", status.code().unwrap_or(1));
    }
}

fn main() {
    let result = (|| -> Result<()> {
        let args = parse_args()?;
        let stamp = PathBuf::from(&args.stamp);
        if let Some(parent) = stamp.parent() {
            fs::create_dir_all(parent)?;
        }

        if args.enable_flathub {
            run_ok(
                "flatpak",
                &[
                    "remote-add".to_string(),
                    "--system".to_string(),
                    "--if-not-exists".to_string(),
                    "flathub".to_string(),
                    "https://flathub.org/repo/flathub.flatpakrepo".to_string(),
                ],
            )?;
        }

        let desired: BTreeSet<String> = args.apps.iter().cloned().collect();
        let managed_file = PathBuf::from(&args.managed_apps_file);
        if managed_file.is_file() {
            let old = fs::read_to_string(&managed_file).unwrap_or_default();
            for app in old.lines().map(str::trim).filter(|s| !s.is_empty()) {
                if !desired.contains(app) {
                    let _ = Command::new("flatpak")
                        .args(["uninstall", "--system", "-y", "--noninteractive", app])
                        .status();
                }
            }
        }

        if let Some(parent) = managed_file.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&managed_file, "")?;
        for app in &args.apps {
            run_ok(
                "flatpak",
                &[
                    "install".to_string(),
                    "--system".to_string(),
                    "-y".to_string(),
                    "--noninteractive".to_string(),
                    "flathub".to_string(),
                    app.clone(),
                ],
            )?;
            let mut old = fs::read_to_string(&managed_file).unwrap_or_default();
            old.push_str(app);
            old.push('\n');
            fs::write(&managed_file, old)?;
        }

        let mut override_args = vec!["override".to_string(), "--system".to_string()];
        for fs_path in &args.filesystem {
            override_args.push(format!("--filesystem={fs_path}"));
        }
        for envv in &args.envs {
            override_args.push(format!("--env={envv}"));
        }
        override_args.extend(args.extra_args.clone());
        if override_args.len() > 2 {
            run_ok("flatpak", &override_args)?;
        }

        fs::write(stamp, "")?;
        Ok(())
    })();

    if let Err(err) = result {
        eprintln!("flatpak-setup: {err:#}");
        std::process::exit(1);
    }
}
