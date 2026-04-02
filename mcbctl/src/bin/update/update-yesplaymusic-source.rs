use anyhow::{Context, Result, anyhow, bail};
use mcbctl::{
    command_exists, find_repo_root, run_capture, run_capture_allow_fail, write_file_atomic,
};
use reqwest::blocking::Client;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

fn usage() {
    eprintln!(
        "Usage:\n  update-yesplaymusic-source          # update source.nix to latest upstream stable release\n  update-yesplaymusic-source --check  # check whether source.nix is up-to-date"
    );
}

fn to_sri_hash(url: &str, digest: Option<&str>) -> Result<String> {
    if let Some(d) = digest
        && d.starts_with("sha256:")
    {
        let hex = d.trim_start_matches("sha256:");
        let out = run_capture(
            "nix",
            &[
                "--extra-experimental-features",
                "nix-command",
                "hash",
                "convert",
                "--hash-algo",
                "sha256",
                "--to",
                "sri",
                hex,
            ],
        )?;
        return Ok(out.trim().to_string());
    }

    let out = run_capture(
        "nix",
        &[
            "--extra-experimental-features",
            "nix-command",
            "store",
            "prefetch-file",
            "--json",
            url,
        ],
    )?;
    let v: Value = serde_json::from_str(&out).context("failed to parse nix prefetch json")?;
    v.get("hash")
        .and_then(|h| h.as_str())
        .map(ToOwned::to_owned)
        .ok_or_else(|| anyhow!("missing hash in prefetch output"))
}

fn main_impl() -> Result<i32> {
    let mut check_only = false;
    let mut args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.first().is_some_and(|a| a == "--check") {
        check_only = true;
        args.remove(0);
    }
    if !args.is_empty() {
        usage();
        return Ok(2);
    }

    for cmd in ["nix"] {
        if !command_exists(cmd) {
            bail!("missing command: {cmd}");
        }
    }

    let root = find_repo_root()?;
    let source_file = root.join("pkgs/yesplaymusic/source.nix");
    let client = Client::builder().build()?;
    let release_json = client
        .get("https://api.github.com/repos/qier222/YesPlayMusic/releases/latest")
        .header("User-Agent", "nixos-config-mcbctl")
        .send()?
        .error_for_status()?
        .text()?;
    let parsed: Value = serde_json::from_str(&release_json)?;
    let tag = parsed
        .get("tag_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("missing tag_name"))?;
    let version = tag.trim_start_matches('v');

    let assets = parsed
        .get("assets")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("missing assets"))?;

    let asset = assets
        .iter()
        .find(|a| {
            a.get("name").and_then(|v| v.as_str()).is_some_and(|name| {
                name.starts_with("YesPlayMusic-") && name.ends_with(".AppImage")
            })
        })
        .ok_or_else(|| anyhow!("failed to locate AppImage asset in release {tag}"))?;

    let url = asset
        .get("browser_download_url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("missing browser_download_url"))?;
    let digest = asset.get("digest").and_then(|v| v.as_str());
    let hash = to_sri_hash(url, digest)?;

    let content =
        format!("{{\n  version = \"{version}\";\n  url = \"{url}\";\n  hash = \"{hash}\";\n}}\n");

    if check_only {
        let old = fs::read_to_string(&source_file).unwrap_or_default();
        if old == content {
            println!("up-to-date: {} ({tag})", source_file.display());
            return Ok(0);
        }
        eprintln!("outdated: {} (latest {tag})", source_file.display());
        if command_exists("diff") && source_file.is_file() {
            let tmp = PathBuf::from("/tmp/yesplaymusic-source.nix");
            let _ = fs::write(&tmp, &content);
            let _ = run_capture_allow_fail(
                "diff",
                &["-u", &source_file.to_string_lossy(), &tmp.to_string_lossy()],
            )
            .map(|d| eprintln!("{d}"));
            let _ = fs::remove_file(tmp);
        }
        return Ok(1);
    }

    write_file_atomic(&source_file, &content)?;
    println!("updated {}", source_file.display());
    println!("yesplaymusic official stable -> {tag}");
    Ok(0)
}

fn main() {
    match main_impl() {
        Ok(code) => std::process::exit(code),
        Err(err) => {
            eprintln!("update-yesplaymusic-source: {err:#}");
            std::process::exit(1);
        }
    }
}
