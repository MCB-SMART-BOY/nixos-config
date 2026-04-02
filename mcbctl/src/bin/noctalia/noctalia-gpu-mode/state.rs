use super::*;
use mcbctl::run_capture_allow_fail;
use serde::Deserialize;

pub(super) fn mode_file() -> Option<PathBuf> {
    let config_home = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|_| std::env::var("HOME").map(PathBuf::from))
        .ok()?;
    Some(config_home.join("noctalia/gpu-modes"))
}

fn default_mode_file() -> Option<PathBuf> {
    let config_home = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|_| std::env::var("HOME").map(PathBuf::from))
        .ok()?;
    Some(config_home.join("noctalia/gpu-default-mode"))
}

fn topology_file() -> Option<PathBuf> {
    let config_home = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|_| std::env::var("HOME").map(PathBuf::from))
        .ok()?;
    Some(config_home.join("noctalia/gpu-topology.json"))
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TopologyMetadata {
    #[serde(default)]
    host_topology: String,
    #[serde(default)]
    default_mode: String,
    #[serde(default)]
    intel_bus_id: Option<String>,
    #[serde(default)]
    amdgpu_bus_id: Option<String>,
    #[serde(default)]
    nvidia_bus_id: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum HostGpuTopology {
    IgpuOnly,
    MultiGpu,
    DgpuOnly,
}

impl HostGpuTopology {
    pub(super) fn id(self) -> &'static str {
        match self {
            Self::IgpuOnly => "igpu-only",
            Self::MultiGpu => "multi-gpu",
            Self::DgpuOnly => "dgpu-only",
        }
    }

    pub(super) fn summary(self) -> &'static str {
        match self {
            Self::IgpuOnly => "单集显主机",
            Self::MultiGpu => "多显卡主机",
            Self::DgpuOnly => "独显主机",
        }
    }
}

impl TopologyMetadata {
    fn default_mode(&self) -> String {
        parse_effective_mode_line(&self.default_mode).unwrap_or_else(|| "igpu".to_string())
    }

    fn topology(&self) -> HostGpuTopology {
        match self.host_topology.as_str() {
            "multi-gpu" => HostGpuTopology::MultiGpu,
            "dgpu-only" => HostGpuTopology::DgpuOnly,
            "igpu-only" => HostGpuTopology::IgpuOnly,
            _ => {
                if self.nvidia_bus_id.is_some()
                    && (self.intel_bus_id.is_some() || self.amdgpu_bus_id.is_some())
                {
                    HostGpuTopology::MultiGpu
                } else if self.nvidia_bus_id.is_some() || self.default_mode() == "dgpu" {
                    HostGpuTopology::DgpuOnly
                } else {
                    HostGpuTopology::IgpuOnly
                }
            }
        }
    }
}

fn topology_metadata() -> Option<TopologyMetadata> {
    let file = topology_file()?;
    let text = fs::read_to_string(file).ok()?;
    serde_json::from_str(&text).ok()
}

fn parse_effective_mode_line(line: &str) -> Option<String> {
    let no_comment = line.split('#').next().unwrap_or_default().trim();
    if no_comment.is_empty() {
        return None;
    }
    let normalized = normalize_mode(no_comment);
    Some(
        normalized
            .strip_prefix("gpu-")
            .unwrap_or(&normalized)
            .to_string(),
    )
}

pub(super) fn default_effective_mode() -> String {
    if let Some(meta) = topology_metadata() {
        return meta.default_mode();
    }

    if let Ok(raw) = std::env::var("NOCTALIA_GPU_DEFAULT_MODE")
        && let Some(mode) = parse_effective_mode_line(&raw)
    {
        return mode;
    }

    if let Some(file) = default_mode_file()
        && let Ok(text) = fs::read_to_string(file)
        && let Some(mode) = text.lines().find_map(parse_effective_mode_line)
    {
        return mode;
    }

    "igpu".to_string()
}

pub(super) fn host_topology() -> HostGpuTopology {
    if let Some(meta) = topology_metadata() {
        return meta.topology();
    }

    let lspci = run_capture_allow_fail("lspci", &["-nn"]).unwrap_or_default();
    let lower = lspci.to_lowercase();
    let has_nvidia = lower.contains(" nvidia ");
    let has_other_gpu = lower.contains(" intel corporation ")
        || lower.contains(" advanced micro devices")
        || lower.contains(" amd/ati ");

    if has_nvidia && has_other_gpu {
        HostGpuTopology::MultiGpu
    } else if has_nvidia {
        HostGpuTopology::DgpuOnly
    } else {
        HostGpuTopology::IgpuOnly
    }
}

pub(super) fn effective_mode(raw: &str) -> String {
    if raw.is_empty() || raw == "base" {
        return default_effective_mode();
    }

    let normalized = normalize_mode(raw);
    normalized
        .strip_prefix("gpu-")
        .unwrap_or(&normalized)
        .to_string()
}

pub(super) fn current_mode() -> String {
    current_mode_inner()
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

fn current_mode_inner() -> String {
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
    let raw_mode = current_mode_inner();
    let effective = effective_mode(&raw_mode);
    let topology = host_topology();
    if topology != HostGpuTopology::MultiGpu {
        let json = serde_json::json!({
            "text": "",
            "alt": effective,
            "class": ["gpu-mode", "hidden", topology.id()],
            "tooltip": format!(
                "Host topology: {}\n当前主机不是多显卡机器，GPU 模式切换入口已隐藏。",
                topology.summary()
            )
        });
        println!("{json}");
        return;
    }

    let specialisation = if raw_mode == "base" {
        format!("base (default: {effective})")
    } else {
        raw_mode.clone()
    };
    let class = if raw_mode == "base" {
        vec![
            "gpu-mode".to_string(),
            "gpu-base".to_string(),
            format!("gpu-{effective}"),
        ]
    } else {
        vec!["gpu-mode".to_string(), raw_mode.clone()]
    };
    let json = serde_json::json!({
        "text": format!("GPU:{effective}"),
        "alt": effective,
        "class": class,
        "tooltip": format!(
            "Host topology: {}\nGPU specialisation: {specialisation}\nEffective mode: {effective}\n切换后 Waybar/Noctalia 会自动刷新，但已打开的图形应用通常需要手动重启。",
            topology.summary()
        )
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
