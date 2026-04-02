use super::*;
use anyhow::Context;

fn desktop_notify(summary: &str, body: &str) {
    if command_exists("notify-send") {
        let _ = Command::new("notify-send").args([summary, body]).status();
    }
}

fn topology_message(topology: state::HostGpuTopology) -> &'static str {
    match topology {
        state::HostGpuTopology::IgpuOnly => {
            "当前主机判断为单集显主机。GPU specialisation 主要用于确认默认图形拓扑，不涉及多显卡间迁移。"
        }
        state::HostGpuTopology::MultiGpu => {
            "当前主机判断为多显卡主机。切换会改变实际 GPU 拓扑，但已打开的图形应用通常不会迁移到新 GPU。"
        }
        state::HostGpuTopology::DgpuOnly => {
            "当前主机判断为独显主机。GPU specialisation 主要影响默认图形栈与桌面环境行为，不存在集显/独显之间的迁移。"
        }
    }
}

fn transition_note(from_raw: &str, to_raw: &str) -> String {
    let from_effective = state::effective_mode(from_raw);
    let to_effective = state::effective_mode(to_raw);
    let topology = state::host_topology();

    if from_effective == to_effective {
        if from_raw == to_raw {
            return format!(
                "{} 当前已经是 {to_effective} 模式，无需额外处理。",
                topology_message(topology)
            );
        }
        return format!(
            "{} 已切到 {to_raw}，但实际 GPU 模式仍是 {to_effective}。Waybar/Noctalia 会自动刷新。",
            topology_message(topology)
        );
    }

    let relog_recommended = topology == state::HostGpuTopology::MultiGpu
        && (from_effective == "hybrid"
            || to_effective == "hybrid"
            || from_effective == "dgpu"
            || to_effective == "dgpu");

    if relog_recommended {
        format!(
            "{} 已从 {from_effective} 切到 {to_effective}。Waybar/Noctalia 会自动刷新，但已打开的图形应用不会迁移到新 GPU。建议至少重启图形应用；如果出现渲染异常或性能不一致，建议注销并重新登录图形会话。",
            topology_message(topology)
        )
    } else {
        format!(
            "{} 已从 {from_effective} 切到 {to_effective}。Waybar/Noctalia 会自动刷新，但已打开的图形应用通常需要手动重启。",
            topology_message(topology)
        )
    }
}

pub(super) fn show_session_note() -> Result<()> {
    let raw_mode = state::current_mode();
    let effective = state::effective_mode(&raw_mode);
    let topology = state::host_topology();
    let message = if raw_mode == "base" {
        format!(
            "{} 当前 specialisation 是 base，实际默认 GPU 模式是 {effective}。切到其它模式后，Waybar/Noctalia 会自动刷新，但已打开的图形应用通常需要手动重启。涉及多显卡主机上的 hybrid 或 dgpu 切换，更建议注销并重新登录图形会话。",
            topology_message(topology)
        )
    } else {
        format!(
            "{} 当前 GPU 模式是 {effective}（specialisation: {raw_mode}）。切换后，Waybar/Noctalia 会自动刷新，但已打开的图形应用通常不会迁移到新 GPU。涉及多显卡主机上的 hybrid 或 dgpu 切换，更建议注销并重新登录图形会话。",
            topology_message(topology)
        )
    };
    println!("{message}");
    desktop_notify("GPU specialisation", &message);
    Ok(())
}

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
    let previous_raw = state::current_mode();
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
        let note = transition_note(&previous_raw, &store_mode);
        println!("{note}");
        desktop_notify("GPU specialisation", &note);
        Ok(())
    } else {
        Err(anyhow!(
            "apply mode failed with {}",
            status.code().unwrap_or(1)
        ))
    }
}
