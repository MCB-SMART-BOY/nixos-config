use anyhow::{Context, Result, anyhow, bail};
use reqwest::blocking::Client;
use scripts_rs::{
    command_exists, find_repo_root, run_capture, run_capture_allow_fail, write_file_atomic,
};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

fn usage() {
    eprintln!(
        "Usage:\n  update-zed-source-rs          # update source.nix to latest upstream stable release\n  update-zed-source-rs --check  # check whether source.nix is up-to-date"
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
    let source_file = root.join("pkgs/zed/source.nix");
    let client = Client::builder().build()?;
    let release_json = client
        .get("https://api.github.com/repos/zed-industries/zed/releases/latest")
        .header("User-Agent", "nixos-config-scripts-rs")
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
    let find_asset = |name: &str| -> Option<&Value> {
        assets.iter().find(|a| {
            a.get("name")
                .and_then(|v| v.as_str())
                .is_some_and(|n| n == name)
        })
    };

    let x86 = find_asset("zed-linux-x86_64.tar.gz")
        .ok_or_else(|| anyhow!("missing zed-linux-x86_64.tar.gz in release {tag}"))?;
    let arm = find_asset("zed-linux-aarch64.tar.gz")
        .ok_or_else(|| anyhow!("missing zed-linux-aarch64.tar.gz in release {tag}"))?;

    let x86_url = x86
        .get("browser_download_url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("missing x86 browser_download_url"))?;
    let arm_url = arm
        .get("browser_download_url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("missing arm browser_download_url"))?;

    let x86_digest = x86.get("digest").and_then(|v| v.as_str());
    let arm_digest = arm.get("digest").and_then(|v| v.as_str());

    let x86_hash = to_sri_hash(x86_url, x86_digest)?;
    let arm_hash = to_sri_hash(arm_url, arm_digest)?;

    let content = format!(
        "{{\n  x86_64-linux = {{\n    version = \"{version}\";\n    url = \"{x86_url}\";\n    hash = \"{x86_hash}\";\n  }};\n\n  aarch64-linux = {{\n    version = \"{version}\";\n    url = \"{arm_url}\";\n    hash = \"{arm_hash}\";\n  }};\n}}\n"
    );

    if check_only {
        let old = fs::read_to_string(&source_file).unwrap_or_default();
        if old == content {
            println!("up-to-date: {} ({tag})", source_file.display());
            return Ok(0);
        }
        eprintln!("outdated: {} (latest {tag})", source_file.display());
        if command_exists("diff") && source_file.is_file() {
            let tmp = PathBuf::from("/tmp/zed-source-rs.nix");
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
    println!("zed official stable -> {tag}");
    Ok(0)
}

fn main() {
    match main_impl() {
        Ok(code) => std::process::exit(code),
        Err(err) => {
            eprintln!("update-zed-source-rs: {err:#}");
            std::process::exit(1);
        }
    }
}
