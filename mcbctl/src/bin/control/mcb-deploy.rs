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
#[path = "mcb-deploy/wizard.rs"]
mod wizard;

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

    fn msg(&self, level: &str, text: &str) {
        println!("[{}] {}", level, text);
    }

    fn log(&self, text: &str) {
        self.msg("信息", text);
    }

    fn success(&self, text: &str) {
        self.msg("完成", text);
    }

    fn warn(&self, text: &str) {
        self.msg("警告", text);
    }

    fn section(&self, text: &str) {
        println!("\n{text}");
    }

    fn note(&self, text: &str) {
        println!("{text}");
    }

    fn banner(&self) {
        println!("==========================================");
        println!("  NixOS 一键部署（mcbctl）");
        println!("==========================================");
    }

    fn is_tty(&self) -> bool {
        io::stdin().is_terminal() && io::stdout().is_terminal()
    }

    fn progress_step(&mut self, label: &str) {
        self.progress_current = self.progress_current.saturating_add(1);
        let width = 24u32;
        let filled = (self.progress_current * width) / self.progress_total.max(1);
        let empty = width.saturating_sub(filled);
        let bar = format!(
            "{}{}",
            "#".repeat(filled as usize),
            "-".repeat(empty as usize)
        );
        println!(
            "进度: [{}] {}/{} {}",
            bar, self.progress_current, self.progress_total, label
        );
    }

    fn usage(&self) {
        println!(
            "用法:
  mcb-deploy
  mcb-deploy release

说明:
  默认模式为全交互部署向导，不需要任何命令行参数。
  所有配置项（部署模式、来源、覆盖策略、用户、权限、GPU、TUN 等）
  均在执行过程中通过菜单选择。

  release 模式用于发布新版本：创建 tag，并发布 GitHub Release。"
        );
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

    fn menu_prompt(&self, title: &str, default_index: usize, options: &[String]) -> Result<usize> {
        if options.is_empty() {
            bail!("菜单选项不能为空");
        }
        loop {
            println!("\n{title}");
            for (idx, opt) in options.iter().enumerate() {
                println!("  {}) {}", idx + 1, opt);
            }
            print!(
                "请选择 [1-{}]（默认 {}，输入 q 退出）： ",
                options.len(),
                default_index
            );
            io::stdout().flush().ok();
            let mut input = String::new();
            io::stdin().read_line(&mut input).context("读取输入失败")?;
            let input = input.trim();
            if input.eq_ignore_ascii_case("q") {
                bail!("已退出");
            }
            if input.is_empty() {
                return Ok(default_index);
            }
            if let Ok(v) = input.parse::<usize>()
                && v >= 1
                && v <= options.len()
            {
                return Ok(v);
            }
            println!("无效选择，请重试。");
        }
    }

    fn ask_bool(&self, prompt: &str, default: bool) -> Result<bool> {
        if !self.is_tty() {
            return Ok(default);
        }
        let default_idx = if default { 1 } else { 2 };
        let pick = self.menu_prompt(
            prompt,
            default_idx,
            &["是 (true)".to_string(), "否 (false)".to_string()],
        )?;
        Ok(pick == 1)
    }

    fn wizard_back_or_quit(&self, prompt: &str) -> Result<WizardAction> {
        print!("{prompt} [c继续/b返回/q退出]（默认 c）： ");
        io::stdout().flush().ok();
        let mut answer = String::new();
        io::stdin().read_line(&mut answer).ok();
        let a = answer.trim();
        if a.eq_ignore_ascii_case("b") {
            Ok(WizardAction::Back)
        } else if a.eq_ignore_ascii_case("q") {
            bail!("已退出")
        } else {
            Ok(WizardAction::Continue)
        }
    }

    fn confirm_continue(&self, prompt: &str) -> Result<()> {
        if !self.is_tty() {
            return Ok(());
        }
        print!("{prompt} [Y/n] ");
        io::stdout().flush().ok();
        let mut answer = String::new();
        io::stdin().read_line(&mut answer).ok();
        if answer.trim().eq_ignore_ascii_case("n") {
            bail!("已退出");
        }
        Ok(())
    }

    fn command_output(cmd: &str, args: &[&str]) -> Option<Output> {
        Command::new(cmd).args(args).output().ok()
    }

    fn run_status_inherit(cmd: &str, args: &[String]) -> Result<ExitStatus> {
        Command::new(cmd)
            .args(args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .with_context(|| format!("failed to run {cmd}"))
    }

    fn run_as_root_inherit(&self, cmd: &str, args: &[String]) -> Result<ExitStatus> {
        if let Some(sudo) = &self.sudo_cmd {
            Command::new(sudo)
                .arg(cmd)
                .args(args)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()
                .with_context(|| format!("failed to run {sudo} {cmd}"))
        } else {
            Command::new(cmd)
                .args(args)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()
                .with_context(|| format!("failed to run {cmd}"))
        }
    }

    fn run_as_root_ok(&self, cmd: &str, args: &[String]) -> Result<()> {
        let status = self.run_as_root_inherit(cmd, args)?;
        if status.success() {
            Ok(())
        } else {
            bail!("{cmd} failed with {}", status.code().unwrap_or(1));
        }
    }

    fn check_env(&mut self) -> Result<()> {
        self.log("检查环境...");

        let is_root = run_capture_allow_fail("id", &["-u"])
            .map(|s| s.trim() == "0")
            .unwrap_or(false);
        if is_root {
            self.warn("检测到 root，将跳过 sudo。");
            self.sudo_cmd = None;
        } else if !command_exists("sudo") {
            self.warn("未找到 sudo，进入 rootless 模式。");
            self.sudo_cmd = None;
            self.rootless = true;
        }

        if !command_exists("git") {
            bail!("未找到 git。");
        }
        if !command_exists("nixos-rebuild") {
            bail!("未找到 nixos-rebuild。");
        }

        if self.sudo_cmd.is_some() {
            if let Some(out) = Self::command_output("sudo", &["-n", "true"]) {
                if !out.status.success() {
                    let stderr = String::from_utf8_lossy(&out.stderr).to_lowercase();
                    if stderr.contains("no new privileges") {
                        self.warn("sudo 无法提权（no new privileges），进入 rootless 模式。");
                        self.sudo_cmd = None;
                        self.rootless = true;
                    } else {
                        self.warn("sudo 需要交互输入密码，将在需要时提示。");
                    }
                }
            }
        }

        if self.rootless {
            if !can_write_dir(&self.etc_dir) {
                let alt_dir = home_dir().join(".nixos");
                if self.is_tty() {
                    print!(
                        "无权限写入 {}，改用 {}？ [Y/n] ",
                        self.etc_dir.display(),
                        alt_dir.display()
                    );
                    io::stdout().flush().ok();
                    let mut ans = String::new();
                    io::stdin().read_line(&mut ans).ok();
                    if ans.trim().eq_ignore_ascii_case("n") {
                        bail!(
                            "无法写入 {}，请使用 root 运行或修改权限。",
                            self.etc_dir.display()
                        );
                    }
                }
                self.etc_dir = alt_dir;
                self.log(&format!(
                    "rootless 模式使用目录：{}",
                    self.etc_dir.display()
                ));
            }
            if self.mode == "switch" || self.mode == "test" {
                self.warn("rootless 模式无法切换系统，将自动改为 build。");
                self.mode = "build".to_string();
            }
        }

        if self.should_require_hardware_config() {
            self.ensure_root_hardware_config()?;
        } else {
            self.note("rootless + build 模式：跳过硬件配置强制检查（仅构建/评估）。");
        }
        Ok(())
    }

    fn should_require_hardware_config(&self) -> bool {
        !(self.rootless && self.mode == "build")
    }

    fn root_hardware_config_path(&self) -> PathBuf {
        self.etc_dir.join("hardware-configuration.nix")
    }

    fn ensure_root_hardware_config(&self) -> Result<()> {
        if !self.should_require_hardware_config() {
            return Ok(());
        }
        let target = self.root_hardware_config_path();
        if target.is_file() {
            return Ok(());
        }
        self.warn(&format!("未发现 {}，将自动生成。", target.display()));
        ensure_root_hardware_config(&self.etc_dir, self.sudo_cmd.is_some())?;
        self.log(&format!("已生成 {}", target.display()));
        Ok(())
    }

    fn is_legacy_shell_path(rel: &str) -> bool {
        rel == "run.sh"
            || rel.starts_with("scripts/run/")
            || (rel.starts_with("pkgs/") && rel.contains("/scripts/") && rel.ends_with(".sh"))
            || (rel.starts_with("home/users/") && rel.contains("/scripts/"))
    }

    fn self_check_repo(&self, repo_dir: &Path) -> Result<()> {
        self.log("仓库自检...");

        let mut legacy_shell_files = Vec::<String>::new();
        for entry in WalkDir::new(repo_dir).into_iter().flatten() {
            if !entry.file_type().is_file() {
                continue;
            }
            let rel = entry
                .path()
                .strip_prefix(repo_dir)
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_else(|_| entry.path().to_string_lossy().replace('\\', "/"));
            if Self::is_legacy_shell_path(&rel) {
                legacy_shell_files.push(rel);
            }
        }

        legacy_shell_files.sort();
        legacy_shell_files.dedup();
        if !legacy_shell_files.is_empty() {
            let sample = legacy_shell_files
                .iter()
                .take(12)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");
            bail!(
                "检测到遗留 Shell 脚本入口（需要完全迁移到 Rust）：{}{}",
                sample,
                if legacy_shell_files.len() > 12 {
                    " ..."
                } else {
                    ""
                }
            );
        }

        let cargo_toml = repo_dir.join("mcbctl/Cargo.toml");
        if cargo_toml.is_file() {
            if command_exists("cargo") {
                let status = Command::new("cargo")
                    .args(["check", "--quiet"])
                    .current_dir(repo_dir.join("mcbctl"))
                    .stdin(Stdio::null())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .status()
                    .context("failed to run cargo check for mcbctl")?;
                if !status.success() {
                    bail!("mcbctl cargo check 失败");
                }
            } else {
                self.warn("未检测到 cargo，跳过 mcbctl 编译自检。");
            }
        } else {
            self.warn("未找到 mcbctl/Cargo.toml，跳过 Rust 脚本编译自检。");
        }

        self.success("仓库自检完成");
        Ok(())
    }

    fn set_deploy_mode_prompt(&mut self) -> Result<()> {
        if self.deploy_mode_set || !self.is_tty() {
            return Ok(());
        }
        let pick = self.menu_prompt(
            "选择部署模式",
            1,
            &[
                "新增/调整用户并部署（可修改用户/权限）".to_string(),
                "仅更新当前配置（网络仓库最新，不改用户/权限）".to_string(),
            ],
        )?;
        if pick == 1 {
            self.set_deploy_mode("manage-users")
        } else {
            self.set_deploy_mode("update-existing")
        }
    }

    fn prompt_overwrite_mode(&mut self) -> Result<()> {
        if self.overwrite_mode_set {
            return Ok(());
        }
        if !self.is_tty() {
            self.overwrite_mode = OverwriteMode::Backup;
            self.overwrite_mode_set = true;
            return Ok(());
        }
        let pick = self.menu_prompt(
            "选择覆盖策略（/etc/nixos 已存在时）",
            1,
            &[
                "先备份再覆盖（推荐）".to_string(),
                "直接覆盖（不备份）".to_string(),
                "执行时再询问".to_string(),
            ],
        )?;
        self.overwrite_mode = match pick {
            1 => OverwriteMode::Backup,
            2 => OverwriteMode::Overwrite,
            _ => OverwriteMode::Ask,
        };
        self.overwrite_mode_set = true;
        Ok(())
    }

    fn prompt_rebuild_upgrade(&mut self) -> Result<()> {
        if !self.is_tty() {
            self.rebuild_upgrade = false;
            return Ok(());
        }
        self.rebuild_upgrade = self.ask_bool("重建时升级上游依赖？", false)?;
        Ok(())
    }

    fn detect_default_iface(&self) -> Option<String> {
        let out = run_capture_allow_fail("ip", &["route", "show", "default"])?;
        let line = out.lines().next()?;
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() >= 5 {
            Some(cols[4].to_string())
        } else {
            None
        }
    }

    fn temp_dns_enable(&mut self) -> Result<bool> {
        let servers = vec!["223.5.5.5".to_string(), "223.6.6.6".to_string()];
        if self.rootless {
            self.warn("rootless 模式无法临时设置 DNS，跳过。");
            return Ok(false);
        }

        if command_exists("resolvectl") && command_exists("systemctl") {
            let active = Command::new("systemctl")
                .args(["is-active", "--quiet", "systemd-resolved"])
                .status()
                .ok()
                .is_some_and(|s| s.success());
            if active && let Some(iface) = self.detect_default_iface() {
                self.log(&format!(
                    "临时 DNS（resolvectl {}）：{}",
                    iface,
                    servers.join(" ")
                ));
                let mut dns_args = vec!["dns".to_string(), iface.clone()];
                dns_args.extend(servers.clone());
                self.run_as_root_ok("resolvectl", &dns_args)?;
                self.run_as_root_ok(
                    "resolvectl",
                    &["domain".to_string(), iface.clone(), "~.".to_string()],
                )?;
                self.temp_dns_backend = "resolvectl".to_string();
                self.temp_dns_iface = iface;
                self.dns_enabled = true;
                return Ok(true);
            }
        }

        let resolv = PathBuf::from("/etc/resolv.conf");
        if resolv.exists() {
            let backup = create_temp_path("mcbctl-resolv", "conf")?;
            self.run_as_root_ok(
                "cp",
                &[
                    "-a".to_string(),
                    "/etc/resolv.conf".to_string(),
                    backup.display().to_string(),
                ],
            )?;
            self.run_as_root_ok("rm", &["-f".to_string(), "/etc/resolv.conf".to_string()])?;

            let content_file = create_temp_path("mcbctl-resolv-new", "conf")?;
            let content = servers
                .iter()
                .map(|s| format!("nameserver {s}"))
                .collect::<Vec<_>>()
                .join("\n")
                + "\n";
            fs::write(&content_file, content)?;
            self.run_as_root_ok(
                "cp",
                &[
                    "-a".to_string(),
                    content_file.display().to_string(),
                    "/etc/resolv.conf".to_string(),
                ],
            )?;
            fs::remove_file(content_file).ok();

            self.log(&format!(
                "临时 DNS（/etc/resolv.conf）：{}",
                servers.join(" ")
            ));
            self.temp_dns_backend = "resolv.conf".to_string();
            self.temp_dns_backup = Some(backup);
            self.dns_enabled = true;
            return Ok(true);
        }

        bail!("无法设置临时 DNS（无 resolvectl 且缺少 /etc/resolv.conf）。")
    }

    fn temp_dns_disable(&mut self) {
        if self.temp_dns_backend == "resolvectl" {
            if !self.temp_dns_iface.is_empty() {
                self.log(&format!("恢复 DNS（resolvectl {}）", self.temp_dns_iface));
                let _ = self.run_as_root_inherit(
                    "resolvectl",
                    &["revert".to_string(), self.temp_dns_iface.clone()],
                );
                let _ = self.run_as_root_inherit("resolvectl", &["flush-caches".to_string()]);
            }
        } else if self.temp_dns_backend == "resolv.conf"
            && let Some(backup) = &self.temp_dns_backup
            && backup.is_file()
        {
            self.log("恢复 /etc/resolv.conf");
            let _ = self.run_as_root_inherit(
                "cp",
                &[
                    "-a".to_string(),
                    backup.display().to_string(),
                    "/etc/resolv.conf".to_string(),
                ],
            );
            fs::remove_file(backup).ok();
        }
        self.temp_dns_backend.clear();
        self.temp_dns_iface.clear();
        self.temp_dns_backup = None;
    }

    fn deploy_flow(&mut self) -> Result<()> {
        self.banner();
        self.set_deploy_mode_prompt()?;
        self.validate_mode_conflicts()?;
        self.prompt_overwrite_mode()?;
        self.prompt_rebuild_upgrade()?;
        self.prompt_source_strategy()?;

        if !self.source_ref.is_empty() && self.allow_remote_head {
            self.warn("检测到来源策略冲突，将优先使用固定版本。");
            self.allow_remote_head = false;
        }

        self.section("环境检查");
        self.check_env()?;
        self.progress_step("环境检查");

        let tmp_dir = create_temp_dir("mcbctl-source")?;
        self.tmp_dir = Some(tmp_dir.clone());

        let result = (|| -> Result<()> {
            self.section("准备源代码");
            loop {
                if self.prepare_source_repo(&tmp_dir).is_ok() {
                    break;
                }
                if !self.is_tty() {
                    bail!("仓库拉取失败，请检查网络或更换来源策略");
                }
                let pick = self.menu_prompt(
                    "准备源代码失败，下一步",
                    1,
                    &[
                        "重试当前来源".to_string(),
                        "重新选择来源策略".to_string(),
                        "退出".to_string(),
                    ],
                )?;
                match pick {
                    1 => continue,
                    2 => {
                        self.source_choice_set = false;
                        self.prompt_source_strategy()?;
                    }
                    3 => bail!("已退出"),
                    _ => {}
                }
            }
            self.progress_step("准备源代码");

            self.section("仓库自检");
            self.self_check_repo(&tmp_dir)?;
            self.progress_step("仓库自检");

            self.wizard_flow(&tmp_dir)?;
            if self.deploy_mode == DeployMode::UpdateExisting {
                self.preserve_existing_local_override(&tmp_dir)?;
            } else {
                self.ensure_host_entry(&tmp_dir)?;
                self.ensure_user_home_entries(&tmp_dir)?;
                if !self.created_home_users.is_empty() {
                    self.warn(&format!(
                        "已自动创建用户 Home Manager 模板：{}",
                        self.created_home_users.join(" ")
                    ));
                }
                self.write_local_override(&tmp_dir)?;
            }
            self.ensure_root_hardware_config()?;
            self.progress_step("收集配置");
            self.confirm_continue("确认以上配置并继续同步？")?;

            self.section("同步与构建");
            self.prepare_etc_dir()?;
            self.progress_step("准备覆盖策略");

            self.sync_repo_to_etc(&tmp_dir)?;
            self.progress_step("同步配置");
            self.confirm_continue("配置已同步，继续重建系统？")?;
            if !self.rebuild_system()? {
                if !self.dns_enabled {
                    self.log("尝试临时切换阿里云 DNS 后重试重建");
                    if !self.temp_dns_enable()? {
                        self.warn("临时 DNS 设置失败，将继续使用当前 DNS 重试重建。");
                    }
                    if !self.rebuild_system()? {
                        bail!("系统重建失败，请检查日志");
                    }
                } else {
                    bail!("系统重建失败，请检查日志");
                }
            }
            self.progress_step("系统重建");
            Ok(())
        })();

        self.temp_dns_disable();
        if let Some(tmp) = self.tmp_dir.take() {
            fs::remove_dir_all(tmp).ok();
        }
        result
    }

    fn run(&mut self) -> Result<()> {
        match self.run_action {
            RunAction::Deploy => self.deploy_flow(),
            RunAction::Release => self.release_flow(),
        }
    }
}

fn strip_comment(line: &str) -> &str {
    line.split('#').next().unwrap_or("")
}

fn first_quoted(line: &str) -> Option<String> {
    let start = line.find('"')?;
    let rest = &line[start + 1..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn is_valid_username(v: &str) -> bool {
    let mut chars = v.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first.is_ascii_lowercase()) {
        return false;
    }
    chars.all(|c| c == '_' || c == '-' || c.is_ascii_lowercase() || c.is_ascii_digit())
}

fn can_write_dir(path: &Path) -> bool {
    if fs::create_dir_all(path).is_err() {
        return false;
    }
    let probe = path.join(format!(".mcbctl-write-{}", std::process::id()));
    match fs::write(&probe, b"ok") {
        Ok(_) => {
            fs::remove_file(probe).ok();
            true
        }
        Err(_) => false,
    }
}

fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/"))
}

fn path_looks_repo(dir: &Path) -> bool {
    dir.join("flake.nix").is_file()
        && dir.join("hosts").is_dir()
        && dir.join("modules").is_dir()
        && dir.join("home").is_dir()
}

fn detect_repo_dir() -> Result<PathBuf> {
    if let Ok(root) = find_repo_root() {
        return Ok(root);
    }
    if let Ok(exe) = std::env::current_exe() {
        let mut cur = exe.parent().map(|p| p.to_path_buf());
        while let Some(dir) = cur {
            if path_looks_repo(&dir) {
                return Ok(dir);
            }
            cur = dir.parent().map(|p| p.to_path_buf());
        }
    }
    let cwd = std::env::current_dir()?;
    if path_looks_repo(&cwd) {
        return Ok(cwd);
    }
    bail!("mcbctl: cannot locate repository root");
}

fn create_temp_dir(prefix: &str) -> Result<PathBuf> {
    let base = std::env::temp_dir();
    for n in 0..2048u32 {
        let p = base.join(format!(
            "{prefix}-{}-{}-{n}",
            std::process::id(),
            chrono_like_millis()
        ));
        if fs::create_dir(&p).is_ok() {
            return Ok(p);
        }
    }
    bail!("failed to create temporary directory");
}

fn create_temp_path(prefix: &str, ext: &str) -> Result<PathBuf> {
    let base = std::env::temp_dir();
    for n in 0..2048u32 {
        let p = base.join(format!(
            "{prefix}-{}-{}-{n}.{ext}",
            std::process::id(),
            chrono_like_millis()
        ));
        if !p.exists() {
            return Ok(p);
        }
    }
    bail!("failed to allocate temporary path");
}

fn chrono_like_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

fn copy_recursively(src: &Path, dst: &Path) -> Result<()> {
    if src.is_file() {
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(src, dst)?;
        return Ok(());
    }
    fs::create_dir_all(dst)?;
    for entry in WalkDir::new(src).into_iter().flatten() {
        let path = entry.path();
        let rel = path.strip_prefix(src).unwrap_or(path);
        let target = dst.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(path, target)?;
        }
    }
    Ok(())
}

fn copy_recursively_if_missing(src: &Path, dst: &Path) -> Result<()> {
    if src.is_file() {
        if dst.exists() {
            return Ok(());
        }
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(src, dst)?;
        return Ok(());
    }

    fs::create_dir_all(dst)?;
    for entry in WalkDir::new(src).into_iter().flatten() {
        let path = entry.path();
        let rel = path.strip_prefix(src).unwrap_or(path);
        let target = dst.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)?;
        } else if entry.file_type().is_file() {
            if target.exists() {
                continue;
            }
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(path, target)?;
        }
    }
    Ok(())
}

fn is_valid_host_name(v: &str) -> bool {
    if v.is_empty() || v.len() > 63 {
        return false;
    }
    if v.starts_with('-') || v.ends_with('-') {
        return false;
    }
    v.bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
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
