use anyhow::{Context, Result, bail};
use mcbctl::domain::deploy::DeployPlan;
use mcbctl::domain::tui::{DeployAction, DeploySource, DeployTask};
use mcbctl::store::deploy::{
    NixosRebuildPlan, RepoSyncPlan, ensure_root_hardware_config, run_nixos_rebuild, run_repo_sync,
};
use mcbctl::{command_exists, find_repo_root, run_capture_allow_fail};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Output, Stdio};
use walkdir::WalkDir;

#[path = "mcb-deploy/execute.rs"]
mod execute;
#[path = "mcb-deploy/orchestrate.rs"]
mod orchestrate;
#[path = "mcb-deploy/plan.rs"]
mod plan;
#[path = "mcb-deploy/release.rs"]
mod release;
#[path = "mcb-deploy/runtime.rs"]
mod runtime;
#[path = "mcb-deploy/scaffold.rs"]
mod scaffold;
#[path = "mcb-deploy/selection.rs"]
mod selection;
#[path = "mcb-deploy/source.rs"]
mod source;
#[path = "mcb-deploy/ui.rs"]
mod ui;
#[path = "mcb-deploy/utils.rs"]
mod utils;
#[path = "mcb-deploy/wizard.rs"]
mod wizard;

pub(crate) use utils::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DeployMode {
    ManageUsers,
    UpdateExisting,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OverwriteMode {
    Ask,
    Backup,
    Overwrite,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HostProfileKind {
    Unknown,
    Desktop,
    Server,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DetectedGpuTopology {
    Unknown,
    IgpuOnly,
    MultiGpu,
    DgpuOnly,
}

impl DetectedGpuTopology {
    fn summary(self) -> &'static str {
        match self {
            Self::Unknown => "未识别",
            Self::IgpuOnly => "单集显主机",
            Self::MultiGpu => "多显卡主机",
            Self::DgpuOnly => "独显主机",
        }
    }

    fn recommended_mode(self) -> &'static str {
        match self {
            Self::Unknown => "igpu",
            Self::IgpuOnly => "igpu",
            Self::MultiGpu => "hybrid",
            Self::DgpuOnly => "dgpu",
        }
    }
}

#[derive(Clone, Debug, Default)]
struct DetectedGpuProfile {
    topology: Option<DetectedGpuTopology>,
    igpu_vendor: String,
    intel_bus: String,
    amd_bus: String,
    nvidia_bus: String,
}

impl DetectedGpuProfile {
    fn topology(&self) -> DetectedGpuTopology {
        self.topology.unwrap_or(DetectedGpuTopology::Unknown)
    }

    fn summary_line(&self) -> String {
        let mut parts = vec![self.topology().summary().to_string()];
        if !self.igpu_vendor.is_empty() {
            parts.push(format!("iGPU={}", self.igpu_vendor));
        }
        if !self.intel_bus.is_empty() {
            parts.push(format!("Intel {}", self.intel_bus));
        }
        if !self.amd_bus.is_empty() {
            parts.push(format!("AMD {}", self.amd_bus));
        }
        if !self.nvidia_bus.is_empty() {
            parts.push(format!("NVIDIA {}", self.nvidia_bus));
        }
        parts.join("，")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RunAction {
    Deploy,
    Release,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WizardAction {
    Continue,
    Back,
}

struct App {
    repo_dir: PathBuf,
    repo_urls: Vec<String>,
    branch: String,
    source_ref: String,
    allow_remote_head: bool,
    source_commit: String,
    source_choice_set: bool,
    target_name: String,
    target_users: Vec<String>,
    target_admin_users: Vec<String>,
    deploy_mode: DeployMode,
    deploy_mode_set: bool,
    force_remote_source: bool,
    overwrite_mode: OverwriteMode,
    overwrite_mode_set: bool,
    per_user_tun_enabled: bool,
    host_profile_kind: HostProfileKind,
    user_tun: BTreeMap<String, String>,
    user_dns: BTreeMap<String, u16>,
    server_overrides_enabled: bool,
    server_enable_network_cli: String,
    server_enable_network_gui: String,
    server_enable_shell_tools: String,
    server_enable_wayland_tools: String,
    server_enable_system_tools: String,
    server_enable_geek_tools: String,
    server_enable_gaming: String,
    server_enable_insecure_tools: String,
    server_enable_docker: String,
    server_enable_libvirtd: String,
    created_home_users: Vec<String>,
    gpu_override: bool,
    gpu_override_from_detection: bool,
    gpu_mode: String,
    gpu_igpu_vendor: String,
    gpu_prime_mode: String,
    gpu_intel_bus: String,
    gpu_amd_bus: String,
    gpu_nvidia_bus: String,
    gpu_nvidia_open: String,
    gpu_specialisations_enabled: bool,
    gpu_specialisations_set: bool,
    gpu_specialisation_modes: Vec<String>,
    detected_gpu: DetectedGpuProfile,
    mode: String,
    rebuild_upgrade: bool,
    etc_dir: PathBuf,
    dns_enabled: bool,
    temp_dns_backend: String,
    temp_dns_backup: Option<PathBuf>,
    temp_dns_iface: String,
    tmp_dir: Option<PathBuf>,
    sudo_cmd: Option<String>,
    rootless: bool,
    run_action: RunAction,
    progress_total: u32,
    progress_current: u32,
    git_clone_timeout_sec: u64,
}

impl App {
    fn new() -> Result<Self> {
        let repo_dir = detect_repo_dir()?;
        let git_clone_timeout_sec = std::env::var("GIT_CLONE_TIMEOUT_SEC")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .filter(|v| *v > 0)
            .unwrap_or(90);

        Ok(Self {
            repo_dir,
            repo_urls: vec![
                "https://gitee.com/MCB-SMART-BOY/nixos-config.git".to_string(),
                "https://github.com/MCB-SMART-BOY/nixos-config.git".to_string(),
            ],
            branch: "master".to_string(),
            source_ref: String::new(),
            allow_remote_head: false,
            source_commit: String::new(),
            source_choice_set: false,
            target_name: String::new(),
            target_users: Vec::new(),
            target_admin_users: Vec::new(),
            deploy_mode: DeployMode::ManageUsers,
            deploy_mode_set: false,
            force_remote_source: false,
            overwrite_mode: OverwriteMode::Ask,
            overwrite_mode_set: false,
            per_user_tun_enabled: false,
            host_profile_kind: HostProfileKind::Unknown,
            user_tun: BTreeMap::new(),
            user_dns: BTreeMap::new(),
            server_overrides_enabled: false,
            server_enable_network_cli: String::new(),
            server_enable_network_gui: String::new(),
            server_enable_shell_tools: String::new(),
            server_enable_wayland_tools: String::new(),
            server_enable_system_tools: String::new(),
            server_enable_geek_tools: String::new(),
            server_enable_gaming: String::new(),
            server_enable_insecure_tools: String::new(),
            server_enable_docker: String::new(),
            server_enable_libvirtd: String::new(),
            created_home_users: Vec::new(),
            gpu_override: false,
            gpu_override_from_detection: false,
            gpu_mode: String::new(),
            gpu_igpu_vendor: String::new(),
            gpu_prime_mode: String::new(),
            gpu_intel_bus: String::new(),
            gpu_amd_bus: String::new(),
            gpu_nvidia_bus: String::new(),
            gpu_nvidia_open: String::new(),
            gpu_specialisations_enabled: false,
            gpu_specialisations_set: false,
            gpu_specialisation_modes: Vec::new(),
            detected_gpu: DetectedGpuProfile::default(),
            mode: "switch".to_string(),
            rebuild_upgrade: false,
            etc_dir: PathBuf::from("/etc/nixos"),
            dns_enabled: false,
            temp_dns_backend: String::new(),
            temp_dns_backup: None,
            temp_dns_iface: String::new(),
            tmp_dir: None,
            sudo_cmd: Some("sudo".to_string()),
            rootless: false,
            run_action: RunAction::Deploy,
            progress_total: 7,
            progress_current: 0,
            git_clone_timeout_sec,
        })
    }

    fn parse_args(&mut self, args: &[String]) -> Result<()> {
        if args.is_empty() {
            return Ok(());
        }
        if args.len() == 1 && (args[0] == "-h" || args[0] == "--help") {
            self.usage();
            std::process::exit(0);
        }
        if args.len() == 1 && (args[0] == "release" || args[0] == "--release") {
            self.run_action = RunAction::Release;
            return Ok(());
        }
        self.usage();
        bail!("此脚本已改为全交互模式，请直接运行 mcb-deploy（不需要参数）。");
    }

    fn set_deploy_mode(&mut self, mode: &str) -> Result<()> {
        match mode {
            "manage-users" | "users" => {
                self.deploy_mode = DeployMode::ManageUsers;
                self.force_remote_source = false;
            }
            "update-existing" | "update" => {
                self.deploy_mode = DeployMode::UpdateExisting;
                self.force_remote_source = true;
            }
            _ => bail!("不支持的部署模式：{mode}"),
        }
        self.deploy_mode_set = true;
        Ok(())
    }
}

fn main() {
    let mut app = match App::new() {
        Ok(v) => v,
        Err(err) => {
            eprintln!("mcbctl: {err:#}");
            std::process::exit(1);
        }
    };
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(err) = app.parse_args(&args).and_then(|_| app.run()) {
        eprintln!("mcbctl: {err:#}");
        std::process::exit(1);
    }
}
