use super::*;

pub(super) fn mode_file() -> Option<PathBuf> {
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

pub(super) fn write_state_mode(mode: &str) -> Result<()> {
    let file = state_file().ok_or_else(|| anyhow!("cannot resolve state file"))?;
    if let Some(parent) = file.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&file, format!("{mode}\n"))?;
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

pub(super) fn emit_status() {
    let mode = current_mode();
    let label = mode.strip_prefix("gpu-").unwrap_or(&mode);
    let json = serde_json::json!({
        "text": format!("GPU:{label}"),
        "tooltip": format!("GPU specialisation: {mode}")
    });
    println!("{json}");
}

pub(super) fn list_modes() -> Vec<String> {
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
