use super::super::*;
use mcbctl::run_capture_allow_fail;
use serde::Deserialize;

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
pub(crate) enum HostGpuTopology {
    IgpuOnly,
    MultiGpu,
    DgpuOnly,
}

impl HostGpuTopology {
    pub(crate) fn id(self) -> &'static str {
        match self {
            Self::IgpuOnly => "igpu-only",
            Self::MultiGpu => "multi-gpu",
            Self::DgpuOnly => "dgpu-only",
        }
    }

    pub(crate) fn summary(self) -> &'static str {
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

pub(crate) fn default_effective_mode() -> String {
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

pub(crate) fn host_topology() -> HostGpuTopology {
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

pub(crate) fn effective_mode(raw: &str) -> String {
    if raw.is_empty() || raw == "base" {
        return default_effective_mode();
    }

    let normalized = normalize_mode(raw);
    normalized
        .strip_prefix("gpu-")
        .unwrap_or(&normalized)
        .to_string()
}
