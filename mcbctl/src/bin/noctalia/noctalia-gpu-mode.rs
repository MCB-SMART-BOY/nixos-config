use anyhow::{Context, Result, anyhow};
use mcbctl::{command_exists, home_dir, prepend_paths, xdg_state_home};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

const MODE_DIR: &str = "/run/current-system/specialisation";

#[derive(Debug)]
enum MenuPickError {
    NoMenuBackend,
    Cancelled,
}

fn normalize_mode(mode: &str) -> String {
    if mode.starts_with("gpu-") {
        mode.to_string()
    } else {
        format!("gpu-{mode}")
    }
}

fn mode_file() -> Option<PathBuf> {
    let config_home = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|_| std::env::var("HOME").map(PathBuf::from))
        .ok()?;
    Some(config_home.join("noctalia/gpu-modes"))
}

fn state_file() -> Option<PathBuf> {
    let base = std::env::var("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| xdg_state_home());
    Some(base.join("noctalia/gpu-current"))
}

fn boot_epoch() -> Option<u64> {
    let content = fs::read_to_string("/proc/uptime").ok()?;
    let first = content.split_whitespace().next()?;
    let uptime: f64 = first.parse().ok()?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_secs_f64();
    let boot = if now > uptime { now - uptime } else { 0.0 };
    Some(boot.floor() as u64)
}

fn parse_mode_line(line: &str) -> Option<String> {
    let no_comment = line.split('#').next().unwrap_or_default();
    let compact: String = no_comment.chars().filter(|c| !c.is_whitespace()).collect();
    if compact.is_empty() {
        return None;
    }
    if compact == "base" {
        Some("base".to_string())
    } else {
        Some(normalize_mode(&compact))
    }
}

fn read_state_mode() -> Option<String> {
    let file = state_file()?;
    let text = fs::read_to_string(file).ok()?;
    let line = text.lines().next().unwrap_or_default();
    parse_mode_line(line)
}

fn read_state_mode_fresh() -> Option<String> {
    let file = state_file()?;
    if !file.is_file() {
        return None;
    }
    if let Some(boot) = boot_epoch()
        && let Ok(meta) = fs::metadata(&file)
        && let Ok(modified) = meta.modified()
        && let Ok(mtime) = modified.duration_since(UNIX_EPOCH)
        && mtime.as_secs() < boot
    {
        return None;
    }
    read_state_mode()
}

fn write_state_mode(mode: &str) -> Result<()> {
    let file = state_file().ok_or_else(|| anyhow!("cannot resolve state file"))?;
    if let Some(parent) = file.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(&file, format!("{mode}\n"))
        .with_context(|| format!("failed to write {}", file.display()))?;
    Ok(())
}

fn list_modes_from_file() -> Vec<String> {
    let Some(file) = mode_file() else {
        return Vec::new();
    };
    let Ok(text) = fs::read_to_string(file) else {
        return Vec::new();
    };
    text.lines().filter_map(parse_mode_line).collect()
}

fn list_modes_from_env() -> Vec<String> {
    let Ok(raw) = std::env::var("NOCTALIA_GPU_MODES") else {
        return Vec::new();
    };
    raw.split_whitespace()
        .filter(|s| !s.is_empty())
        .map(normalize_mode)
        .collect()
}

fn mode_from_path(path: &str) -> Option<String> {
    if let Some(idx) = path.find("/specialisation/") {
        let tail = &path[idx + "/specialisation/".len()..];
        return tail.split('/').next().map(ToOwned::to_owned);
    }
    if let Some(idx) = path.find("specialisation-") {
        let tail = &path[idx + "specialisation-".len()..];
        return tail.split('/').next().map(ToOwned::to_owned);
    }
    None
}

fn readlink_abs(path: &str) -> Option<String> {
    fs::canonicalize(path)
        .ok()
        .map(|p| p.to_string_lossy().to_string())
}

fn current_mode() -> String {
    for p in ["/run/current-system", "/run/booted-system"] {
        if let Some(abs) = readlink_abs(p)
            && let Some(mode) = mode_from_path(&abs)
        {
            return mode;
        }
    }

    if let Ok(cmdline) = fs::read_to_string("/proc/cmdline") {
        for token in cmdline.split_whitespace() {
            let cand = if let Some(v) = token.strip_prefix("init=") {
                Some(v)
            } else {
                token.strip_prefix("systemConfig=")
            };
            if let Some(path) = cand
                && let Some(mode) = mode_from_path(path)
            {
                return mode;
            }
        }
    }

    if let Some(mode) = read_state_mode_fresh() {
        return mode;
    }
    "base".to_string()
}

fn emit_status() {
    let mode = current_mode();
    let label = mode.strip_prefix("gpu-").unwrap_or(&mode);
    let json = serde_json::json!({
        "text": format!("GPU:{label}"),
        "tooltip": format!("GPU specialisation: {mode}")
    });
    println!("{json}");
}

fn list_modes() -> Vec<String> {
    let mut modes = Vec::new();
    if let Ok(entries) = fs::read_dir(MODE_DIR) {
        for entry in entries.flatten() {
            let Ok(ft) = entry.file_type() else {
                continue;
            };
            if !ft.is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("gpu-") {
                modes.push(name);
            }
        }
    }
    if !modes.is_empty() {
        modes.sort();
        modes.dedup();
        return modes;
    }

    let from_file = list_modes_from_file();
    if !from_file.is_empty() {
        return from_file;
    }

    list_modes_from_env()
}

fn pick_menu(lines: &[String]) -> std::result::Result<String, MenuPickError> {
    let mut candidates: Vec<(&str, Vec<&str>)> = Vec::new();
    if command_exists("fuzzel") {
        candidates.push(("fuzzel", vec!["--dmenu", "--prompt", "GPU mode: "]));
    }
    if command_exists("wofi") {
        candidates.push(("wofi", vec!["--dmenu", "-p", "GPU mode"]));
    }
    if command_exists("rofi") {
        candidates.push(("rofi", vec!["-dmenu", "-p", "GPU mode"]));
    }
    let Some((bin, args)) = candidates.into_iter().next() else {
        return Err(MenuPickError::NoMenuBackend);
    };

    let mut child = Command::new(bin)
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| MenuPickError::Cancelled)?;

    if let Some(stdin) = child.stdin.as_mut() {
        let input = format!("{}\n", lines.join("\n"));
        let _ = stdin.write_all(input.as_bytes());
    }

    let out = child
        .wait_with_output()
        .map_err(|_| MenuPickError::Cancelled)?;
    if !out.status.success() {
        return Err(MenuPickError::Cancelled);
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
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

fn run_inherit(cmd: &[String]) -> Result<ExitStatus> {
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

fn launch_in_terminal(command: &[String]) -> Result<()> {
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

fn apply_mode(target: &str) -> Result<()> {
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
        let _ = write_state_mode(&store_mode);
        Ok(())
    } else {
        Err(anyhow!(
            "apply mode failed with {}",
            status.code().unwrap_or(1)
        ))
    }
}

fn select_self_command() -> String {
    std::env::current_exe()
        .ok()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "noctalia-gpu-mode".to_string())
}

fn quote_shell(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\"'\"'"))
}

fn menu_flow() -> Result<()> {
    let modes = list_modes();
    if modes.is_empty() {
        if command_exists("notify-send") {
            let mut hint = "No GPU specialisations found.".to_string();
            if let Some(path) = mode_file() {
                hint.push_str(&format!(
                    " Provide modes via {} or NOCTALIA_GPU_MODES.",
                    path.display()
                ));
            }
            let _ = Command::new("notify-send")
                .args(["GPU specialisation", &hint])
                .status();
        }
        return Ok(());
    }

    let mut labels = vec!["base".to_string()];
    let mut label_to_mode = HashMap::new();
    for m in &modes {
        let label = m.strip_prefix("gpu-").unwrap_or(m).to_string();
        labels.push(label.clone());
        label_to_mode.insert(label, m.clone());
    }

    let selection = match pick_menu(&labels) {
        Ok(v) => v,
        Err(MenuPickError::NoMenuBackend) => {
            let cmd_path = select_self_command();
            let shell_cmd = format!("{} --menu-cli", quote_shell(&cmd_path));
            let cmd = vec!["bash".to_string(), "-lc".to_string(), shell_cmd];
            let _ = launch_in_terminal(&cmd);
            return Ok(());
        }
        Err(MenuPickError::Cancelled) => return Ok(()),
    };

    if selection.is_empty() || selection == "cancel" {
        return Ok(());
    }

    let target = if selection == "base" {
        "base".to_string()
    } else {
        match label_to_mode.get(&selection) {
            Some(v) => v.clone(),
            None => return Ok(()),
        }
    };

    let cmd_path = select_self_command();
    let cmd = vec![
        cmd_path,
        "--apply".to_string(),
        if target.is_empty() {
            "base".to_string()
        } else {
            target
        },
    ];
    let _ = launch_in_terminal(&cmd);
    Ok(())
}

fn read_choice(max: usize) -> Option<usize> {
    let mut line = String::new();
    loop {
        line.clear();
        print!("GPU mode: ");
        let _ = io::stdout().flush();
        if io::stdin().read_line(&mut line).is_err() {
            return None;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(idx) = trimmed.parse::<usize>()
            && idx >= 1
            && idx <= max
        {
            return Some(idx);
        }
    }
}

fn menu_flow_cli() -> Result<()> {
    let modes = list_modes();
    if modes.is_empty() {
        let mut hint = "No GPU specialisations found.".to_string();
        if let Some(path) = mode_file() {
            hint.push_str(&format!(
                " Provide modes via {} or NOCTALIA_GPU_MODES.",
                path.display()
            ));
        }
        println!("{hint}");
        return Ok(());
    }

    let mut labels = vec!["base".to_string()];
    let mut label_to_mode = HashMap::new();
    for m in &modes {
        let label = m.strip_prefix("gpu-").unwrap_or(m).to_string();
        labels.push(label.clone());
        label_to_mode.insert(label, m.clone());
    }

    println!("Select GPU mode:");
    for (idx, label) in labels.iter().enumerate() {
        println!("  {}) {}", idx + 1, label);
    }
    let Some(choice) = read_choice(labels.len()) else {
        return Ok(());
    };
    let selection = &labels[choice - 1];

    if selection == "base" {
        apply_mode("base")
    } else if let Some(mode) = label_to_mode.get(selection) {
        apply_mode(mode)
    } else {
        Ok(())
    }
}

fn init_path() {
    let mut extra = Vec::new();
    extra.push(PathBuf::from("/run/wrappers/bin"));
    extra.push(PathBuf::from("/run/current-system/sw/bin"));
    if let Ok(user) = std::env::var("USER") {
        extra.push(PathBuf::from(format!("/etc/profiles/per-user/{user}/bin")));
    }
    extra.push(home_dir().join(".nix-profile/bin"));
    extra.push(home_dir().join(".local/bin"));
    prepend_paths(&extra);
}

fn real_main() -> Result<()> {
    init_path();

    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("--menu") => menu_flow(),
        Some("--menu-cli") => menu_flow_cli(),
        Some("--apply") => {
            let target = args.next().unwrap_or_default();
            apply_mode(&target)
        }
        Some("--set-state") => {
            if let Some(mode) = args.next() {
                if mode == "base" {
                    let _ = write_state_mode("base");
                } else {
                    let _ = write_state_mode(&normalize_mode(&mode));
                }
            }
            Ok(())
        }
        _ => {
            emit_status();
            Ok(())
        }
    }
}

fn main() {
    if let Err(err) = real_main() {
        eprintln!("noctalia-gpu-mode: {err:#}");
        std::process::exit(1);
    }
}
