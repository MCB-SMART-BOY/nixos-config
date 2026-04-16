use anyhow::{Context, Result, bail};
use mcbctl::domain::deploy::DeployPlan;
use mcbctl::domain::tui::{DeployAction, DeploySource, DeployTask};
use mcbctl::repo::preferred_remote_branch;
use mcbctl::store::deploy::{
    NixosRebuildPlan, RepoSyncPlan, ensure_host_hardware_config, host_hardware_config_path,
    run_nixos_rebuild, run_repo_sync,
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
    source_dir_override: Option<PathBuf>,
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
    rebuild_upgrade_set: bool,
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
        let branch = preferred_remote_branch(&repo_dir);
        let git_clone_timeout_sec = std::env::var("GIT_CLONE_TIMEOUT_SEC")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .filter(|v| *v > 0)
            .unwrap_or(90);

        Ok(Self {
            repo_dir,
            source_dir_override: None,
            repo_urls: vec![
                "https://gitee.com/MCB-SMART-BOY/nixos-config.git".to_string(),
                "https://github.com/MCB-SMART-BOY/nixos-config.git".to_string(),
            ],
            branch,
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
            rebuild_upgrade_set: false,
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
        let mut index = 0usize;
        let mut requested_source = None::<String>;

        while index < args.len() {
            match args[index].as_str() {
                "--mode" => {
                    let mode = next_arg_value(args, &mut index, "--mode")?;
                    self.set_deploy_mode(mode)?;
                }
                "--host" => {
                    let host = next_arg_value(args, &mut index, "--host")?;
                    self.target_name = host.to_string();
                }
                "--action" => {
                    let action = next_arg_value(args, &mut index, "--action")?;
                    self.set_rebuild_mode(action)?;
                }
                "--source" => {
                    let source = next_arg_value(args, &mut index, "--source")?;
                    self.set_source_prefill(source)?;
                    requested_source = Some(source.to_string());
                }
                "--ref" => {
                    let source_ref = next_arg_value(args, &mut index, "--ref")?;
                    self.source_ref = source_ref.to_string();
                }
                "--upgrade" => {
                    self.rebuild_upgrade = true;
                    self.rebuild_upgrade_set = true;
                }
                other => {
                    self.usage();
                    bail!(
                        "此脚本已改为全交互模式；不支持参数：{other}。仅内部 handoff 可使用 --mode/--host/--action/--source/--ref/--upgrade。"
                    );
                }
            }
            index += 1;
        }

        match requested_source.as_deref() {
            Some("remote-pinned") if self.source_ref.trim().is_empty() => {
                bail!("--source remote-pinned 需要同时提供 --ref。");
            }
            Some("current-repo" | "etc-nixos" | "remote-head")
                if !self.source_ref.trim().is_empty() =>
            {
                bail!("--ref 仅能与 --source remote-pinned 搭配使用。");
            }
            None if !self.source_ref.trim().is_empty() => {
                bail!("--ref 需要与 --source remote-pinned 一起使用。");
            }
            _ => {}
        }

        Ok(())
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

    fn set_rebuild_mode(&mut self, mode: &str) -> Result<()> {
        match mode {
            "switch" | "test" | "boot" | "build" => {
                self.mode = mode.to_string();
                Ok(())
            }
            _ => bail!("不支持的部署动作：{mode}"),
        }
    }

    fn set_source_prefill(&mut self, source: &str) -> Result<()> {
        match source {
            "current-repo" => {
                self.force_remote_source = false;
                self.allow_remote_head = false;
                self.source_ref.clear();
                self.source_dir_override = None;
            }
            "etc-nixos" => {
                self.force_remote_source = false;
                self.allow_remote_head = false;
                self.source_ref.clear();
                self.source_dir_override = Some(self.etc_dir.clone());
            }
            "remote-head" => {
                self.force_remote_source = true;
                self.allow_remote_head = true;
                self.source_ref.clear();
                self.source_dir_override = None;
            }
            "remote-pinned" => {
                self.force_remote_source = true;
                self.allow_remote_head = false;
                self.source_ref.clear();
                self.source_dir_override = None;
            }
            _ => bail!("不支持的来源：{source}"),
        }
        self.source_choice_set = true;
        Ok(())
    }
}

fn next_arg_value<'a>(args: &'a [String], index: &mut usize, flag: &str) -> Result<&'a str> {
    *index += 1;
    args.get(*index)
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
        .with_context(|| format!("{flag} 需要参数"))
}

fn render_cli_error(err: &anyhow::Error) -> String {
    format!("mcbctl: {err:#}")
}

fn main() {
    let mut app = match App::new() {
        Ok(v) => v,
        Err(err) => {
            eprintln!("{}", render_cli_error(&err));
            std::process::exit(1);
        }
    };
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(err) = app.parse_args(&args).and_then(|_| app.run()) {
        eprintln!("{}", render_cli_error(&err));
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn parse_args_accepts_internal_prefill_flags() -> Result<()> {
        let mut app = test_app();
        let args = vec![
            "--mode".to_string(),
            "update-existing".to_string(),
            "--host".to_string(),
            "nixos".to_string(),
            "--action".to_string(),
            "boot".to_string(),
            "--source".to_string(),
            "etc-nixos".to_string(),
            "--upgrade".to_string(),
        ];

        app.parse_args(&args)?;

        assert_eq!(app.deploy_mode, DeployMode::UpdateExisting);
        assert!(app.deploy_mode_set);
        assert_eq!(app.target_name, "nixos");
        assert_eq!(app.mode, "boot");
        assert!(app.source_choice_set);
        assert_eq!(app.source_dir_override, Some(PathBuf::from("/etc/nixos")));
        assert!(app.rebuild_upgrade);
        assert!(app.rebuild_upgrade_set);
        Ok(())
    }

    #[test]
    fn parse_args_rejects_ref_without_remote_pinned_source() -> Result<()> {
        let mut app = test_app();
        let args = vec![
            "--source".to_string(),
            "current-repo".to_string(),
            "--ref".to_string(),
            "deadbeef".to_string(),
        ];

        let err = app
            .parse_args(&args)
            .expect_err("current-repo should reject --ref");

        assert!(
            err.to_string()
                .contains("--ref 仅能与 --source remote-pinned")
        );
        Ok(())
    }

    #[test]
    fn etc_nixos_prefill_keeps_source_detail_even_if_destination_dir_changes() -> Result<()> {
        let mut app = test_app();

        app.set_source_prefill("etc-nixos")?;
        app.etc_dir = PathBuf::from("/home/demo/.nixos");

        assert_eq!(app.deploy_plan_source(), DeploySource::EtcNixos);
        assert_eq!(
            app.deploy_plan_source_detail(),
            Some("/etc/nixos".to_string())
        );
        Ok(())
    }

    #[test]
    fn parse_args_keeps_rejecting_unknown_non_interactive_flags() -> Result<()> {
        let mut app = test_app();
        let args = vec!["--mystery".to_string()];

        let err = app
            .parse_args(&args)
            .expect_err("unknown flags should still be rejected");

        assert!(err.to_string().contains("不支持参数：--mystery"));
        Ok(())
    }

    fn test_app() -> App {
        App {
            repo_dir: PathBuf::from("/tmp/repo"),
            source_dir_override: None,
            repo_urls: Vec::new(),
            branch: "rust脚本分支".to_string(),
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
            rebuild_upgrade_set: false,
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
            git_clone_timeout_sec: 90,
        }
    }
}
