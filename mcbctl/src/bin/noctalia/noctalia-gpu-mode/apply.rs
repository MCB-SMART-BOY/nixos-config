use super::*;
use anyhow::Context;

fn host_name() -> String {
    if let Ok(h) = fs::read_to_string("/etc/hostname") {
        let trimmed = h.trim().to_string();
        if !trimmed.is_empty() {
            return trimmed;
        }
    }
    let out = Command::new("hostname").output();
    if let Ok(out) = out
        && out.status.success()
    {
        let v = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !v.is_empty() {
            return v;
        }
    }
    String::new()
}

fn build_rebuild_cmd(specialisation: Option<&str>) -> Vec<String> {
    let switch_path = if let Some(spec) = specialisation {
        format!("/nix/var/nix/profiles/system/specialisation/{spec}/bin/switch-to-configuration")
    } else {
        "/nix/var/nix/profiles/system/bin/switch-to-configuration".to_string()
    };
    if Path::new(&switch_path).is_file() {
        return vec!["sudo".to_string(), switch_path, "switch".to_string()];
    }

    let switch_path = if let Some(spec) = specialisation {
        format!("/run/current-system/specialisation/{spec}/bin/switch-to-configuration")
    } else {
        "/run/current-system/bin/switch-to-configuration".to_string()
    };
    if Path::new(&switch_path).is_file() {
        return vec!["sudo".to_string(), switch_path, "switch".to_string()];
    }

    let mut cmd = vec![
        "sudo".to_string(),
        "nixos-rebuild".to_string(),
        "switch".to_string(),
    ];
    if let Some(spec) = specialisation {
        cmd.push("--specialisation".to_string());
        cmd.push(spec.to_string());
    }

    let mut flake = std::env::var("NOCTALIA_FLAKE").unwrap_or_default();
    let home = std::env::var("HOME").unwrap_or_else(|_| home_dir().to_string_lossy().to_string());
    if flake.is_empty() {
        let h = host_name();
        if Path::new("/etc/nixos/flake.nix").is_file() {
            flake = if h.is_empty() {
                "/etc/nixos".to_string()
            } else {
                format!("/etc/nixos#{h}")
            };
        } else {
            let p = PathBuf::from(&home).join("nixos-config/flake.nix");
            if p.is_file() {
                flake = if h.is_empty() {
                    format!("{home}/nixos-config")
                } else {
                    format!("{home}/nixos-config#{h}")
                };
            }
        }
    }
    if !flake.is_empty() {
        cmd.push("--flake".to_string());
        cmd.push(flake);
    }
    cmd
}

pub(super) fn run_inherit(cmd: &[String]) -> Result<ExitStatus> {
    if cmd.is_empty() {
        return Err(anyhow!("empty command"));
    }
    let status = Command::new(&cmd[0])
        .args(&cmd[1..])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("failed to run {}", cmd[0]))?;
    Ok(status)
}

pub(super) fn launch_in_terminal(command: &[String]) -> Result<()> {
    let mut term = std::env::var("TERMINAL").unwrap_or_default();
    if term.is_empty() {
        for cand in ["alacritty", "foot", "kitty", "wezterm"] {
            if command_exists(cand) {
                term = cand.to_string();
                break;
            }
        }
    }
    if term.is_empty() {
        if command_exists("notify-send") {
            let _ = Command::new("notify-send")
                .args([
                    "GPU specialisation",
                    "No terminal found to run nixos-rebuild",
                ])
                .status();
        }
        return Err(anyhow!("no terminal found"));
    }

    let args: Vec<String> = if command_exists("niri-run") {
        if term == "wezterm" {
            let mut v = vec![
                "niri-run".to_string(),
                "wezterm".to_string(),
                "start".to_string(),
                "--".to_string(),
            ];
            v.extend(command.iter().cloned());
            v
        } else {
            let mut v = vec!["niri-run".to_string(), term.clone(), "-e".to_string()];
            v.extend(command.iter().cloned());
            v
        }
    } else if term == "wezterm" {
        let mut v = vec!["wezterm".to_string(), "start".to_string(), "--".to_string()];
        v.extend(command.iter().cloned());
        v
    } else {
        let mut v = vec![term.clone(), "-e".to_string()];
        v.extend(command.iter().cloned());
        v
    };

    let status = run_inherit(&args)?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow!(
            "terminal command failed with {}",
            status.code().unwrap_or(1)
        ))
    }
}

pub(super) fn apply_mode(target: &str) -> Result<()> {
    let (specialisation, store_mode) = if target.is_empty() || target == "base" {
        (None, "base".to_string())
    } else {
        let m = normalize_mode(target);
        (Some(m.clone()), m)
    };
    let cmd = build_rebuild_cmd(specialisation.as_deref());
    println!("Running: {}", cmd.join(" "));
    let status = run_inherit(&cmd)?;
    if status.success() {
        let _ = state::write_state_mode(&store_mode);
        Ok(())
    } else {
        Err(anyhow!(
            "apply mode failed with {}",
            status.code().unwrap_or(1)
        ))
    }
}
