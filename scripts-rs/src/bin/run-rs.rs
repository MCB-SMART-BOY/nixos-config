use anyhow::{Context, Result, bail};
use scripts_rs::{command_exists, find_repo_root, run_capture_allow_fail};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Output, Stdio};
use walkdir::WalkDir;

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
    script_dir: PathBuf,
    run_sh_version: String,
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
        let script_dir = detect_script_dir()?;
        let version_file = script_dir.join("VERSION");
        let mut run_sh_version = "v2026.03.20".to_string();
        if let Ok(v) = fs::read_to_string(&version_file) {
            let trimmed = v.trim();
            if !trimmed.is_empty() {
                run_sh_version = trimmed.to_string();
            }
        }
        let git_clone_timeout_sec = std::env::var("GIT_CLONE_TIMEOUT_SEC")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .filter(|v| *v > 0)
            .unwrap_or(90);

        Ok(Self {
            script_dir,
            run_sh_version,
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
        println!("  NixOS 一键部署（run-rs {}）", self.run_sh_version);
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
  run-rs
  run-rs release

说明:
  默认模式为全交互部署向导，不需要任何命令行参数。
  所有配置项（部署模式、来源、覆盖策略、用户、权限、GPU、TUN 等）
  均在执行过程中通过菜单选择。

  release 模式用于发布新版本：更新 VERSION、创建 tag，并发布 GitHub Release。"
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
        bail!("此脚本已改为全交互模式，请直接运行 run-rs（不需要参数）。");
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
            if !self.has_any_hardware_config() {
                bail!(
                    "缺少硬件配置：{}/hardware-configuration.nix 或 {}/hosts/<hostname>/hardware-configuration.nix；请先运行 nixos-generate-config。",
                    self.etc_dir.display(),
                    self.etc_dir.display()
                );
            }
        } else {
            self.note("rootless + build 模式：跳过硬件配置强制检查（仅构建/评估）。");
        }
        Ok(())
    }

    fn should_require_hardware_config(&self) -> bool {
        !(self.rootless && self.mode == "build")
    }

    fn has_any_hardware_config(&self) -> bool {
        if self.etc_dir.join("hardware-configuration.nix").is_file() {
            return true;
        }
        if !self.target_name.is_empty()
            && self
                .etc_dir
                .join("hosts")
                .join(&self.target_name)
                .join("hardware-configuration.nix")
                .is_file()
        {
            return true;
        }
        let hosts = self.etc_dir.join("hosts");
        if hosts.is_dir() {
            for entry in WalkDir::new(&hosts)
                .max_depth(3)
                .into_iter()
                .flatten()
                .filter(|e| e.file_type().is_file())
            {
                if entry.file_name() == "hardware-configuration.nix" {
                    return true;
                }
            }
        }
        false
    }

    fn ensure_host_hardware_config(&self) -> Result<()> {
        if !self.should_require_hardware_config() {
            return Ok(());
        }
        if self.etc_dir.join("hardware-configuration.nix").is_file() {
            return Ok(());
        }
        if !self.target_name.is_empty()
            && self
                .etc_dir
                .join("hosts")
                .join(&self.target_name)
                .join("hardware-configuration.nix")
                .is_file()
        {
            return Ok(());
        }
        bail!(
            "缺少硬件配置：{}/hardware-configuration.nix 或 {}/hosts/{}/hardware-configuration.nix；请先运行 nixos-generate-config。",
            self.etc_dir.display(),
            self.etc_dir.display(),
            self.target_name
        );
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

        let cargo_toml = repo_dir.join("scripts-rs/Cargo.toml");
        if cargo_toml.is_file() {
            if command_exists("cargo") {
                let status = Command::new("cargo")
                    .args(["check", "--quiet"])
                    .current_dir(repo_dir.join("scripts-rs"))
                    .stdin(Stdio::null())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .status()
                    .context("failed to run cargo check for scripts-rs")?;
                if !status.success() {
                    bail!("scripts-rs cargo check 失败");
                }
            } else {
                self.warn("未检测到 cargo，跳过 scripts-rs 编译自检。");
            }
        } else {
            self.warn("未找到 scripts-rs/Cargo.toml，跳过 Rust 脚本编译自检。");
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

    fn prompt_source_strategy(&mut self) -> Result<()> {
        if self.source_choice_set {
            return Ok(());
        }
        let local_repo = self.detect_local_repo_dir();
        if !self.is_tty() {
            if self.deploy_mode == DeployMode::UpdateExisting {
                self.force_remote_source = true;
                self.allow_remote_head = true;
                self.source_ref.clear();
            } else if local_repo.is_some() {
                self.force_remote_source = false;
                self.allow_remote_head = false;
                self.source_ref.clear();
            } else {
                self.force_remote_source = true;
                self.allow_remote_head = false;
            }
            self.source_choice_set = true;
            return Ok(());
        }

        let mut options = Vec::<String>::new();
        let mut default_index = 1usize;
        if let Some(local) = &local_repo {
            options.push(format!("使用本地仓库（推荐）: {}", local.display()));
        }
        options.push("使用网络仓库固定版本（输入 commit/tag）".to_string());
        options.push("使用网络仓库最新版本（HEAD）".to_string());
        if self.deploy_mode == DeployMode::UpdateExisting {
            default_index = options.len();
        }
        let pick = self.menu_prompt("选择配置来源", default_index, &options)?;

        if local_repo.is_some() && pick == 1 {
            self.force_remote_source = false;
            self.allow_remote_head = false;
            self.source_ref.clear();
        } else {
            let mut remote_pick = pick;
            if local_repo.is_some() {
                remote_pick = pick.saturating_sub(1);
            }
            match remote_pick {
                1 => {
                    self.force_remote_source = true;
                    self.allow_remote_head = false;
                    loop {
                        print!("请输入远端固定版本（commit/tag）： ");
                        io::stdout().flush().ok();
                        let mut line = String::new();
                        io::stdin().read_line(&mut line).ok();
                        let v = line.trim();
                        if !v.is_empty() {
                            self.source_ref = v.to_string();
                            break;
                        }
                        println!("版本不能为空，请重试。");
                    }
                }
                2 => {
                    self.force_remote_source = true;
                    self.allow_remote_head = true;
                    self.source_ref.clear();
                }
                _ => {}
            }
        }
        self.source_choice_set = true;
        Ok(())
    }

    fn validate_mode_conflicts(&self) -> Result<()> {
        if self.deploy_mode == DeployMode::UpdateExisting && !self.target_users.is_empty() {
            bail!("仅更新模式不允许修改用户列表；该模式会保留现有用户与权限。");
        }
        Ok(())
    }

    fn require_remote_source_pin(&self) -> Result<()> {
        if self.allow_remote_head {
            self.warn("当前将跟随远端分支最新提交（存在供应链风险）。");
            return Ok(());
        }
        if self.source_ref.is_empty() {
            bail!(
                "未检测到本地仓库，且未选择远端固定版本；请在向导中选择固定版本或明确选择远端最新版本。"
            );
        }
        Ok(())
    }

    fn detect_local_repo_dir(&self) -> Option<PathBuf> {
        let cwd = std::env::current_dir().ok();
        let mut candidates = Vec::new();
        if let Some(c) = cwd {
            candidates.push(c);
        }
        candidates.push(self.script_dir.clone());
        candidates.into_iter().find(|d| path_looks_repo(d))
    }

    fn prepare_local_source(&mut self, tmp_dir: &Path, source_dir: &Path) -> Result<()> {
        self.log(&format!("使用本地仓库：{}", source_dir.display()));
        if tmp_dir.exists() {
            fs::remove_dir_all(tmp_dir).ok();
        }
        fs::create_dir_all(tmp_dir)?;

        if command_exists("rsync") {
            let args = vec![
                "-a".to_string(),
                "--exclude".to_string(),
                ".git/".to_string(),
                format!("{}/", source_dir.display()),
                format!("{}/", tmp_dir.display()),
            ];
            let status = Self::run_status_inherit("rsync", &args)?;
            if !status.success() {
                bail!("rsync 复制本地仓库失败");
            }
        } else {
            let tar_file = create_temp_path("run-rs-local-src", "tar")?;
            let args = vec![
                "-C".to_string(),
                source_dir.display().to_string(),
                "--exclude=.git".to_string(),
                "-cf".to_string(),
                tar_file.display().to_string(),
                ".".to_string(),
            ];
            let st = Self::run_status_inherit("tar", &args)?;
            if !st.success() {
                bail!("打包本地仓库失败");
            }
            let args = vec![
                "-C".to_string(),
                tmp_dir.display().to_string(),
                "-xf".to_string(),
                tar_file.display().to_string(),
            ];
            let st = Self::run_status_inherit("tar", &args)?;
            fs::remove_file(&tar_file).ok();
            if !st.success() {
                bail!("解包本地仓库失败");
            }
        }

        if command_exists("git") {
            let out = Command::new("git")
                .args(["-C", &source_dir.display().to_string(), "rev-parse", "HEAD"])
                .output();
            if let Ok(out) = out
                && out.status.success()
            {
                self.source_commit = String::from_utf8_lossy(&out.stdout).trim().to_string();
            }
        }
        if !self.source_commit.is_empty() {
            self.note(&format!("本地源提交：{}", self.source_commit));
        }
        Ok(())
    }

    fn run_git_with_timeout(&self, args: &[String]) -> Result<ExitStatus> {
        if command_exists("timeout") {
            let mut cmd = Command::new("timeout");
            cmd.arg("--foreground")
                .arg(self.git_clone_timeout_sec.to_string())
                .arg("git")
                .args(args)
                .env("GIT_TERMINAL_PROMPT", "0")
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit());
            Ok(cmd.status()?)
        } else {
            let mut cmd = Command::new("git");
            cmd.args(args)
                .env("GIT_TERMINAL_PROMPT", "0")
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit());
            Ok(cmd.status()?)
        }
    }

    fn clone_repo(&mut self, tmp_dir: &Path, url: &str) -> Result<bool> {
        let timeout_s = self.git_clone_timeout_sec;
        if !self.source_ref.is_empty() {
            self.log(&format!(
                "拉取仓库：{url}（固定 ref: {}，超时 {}s）",
                self.source_ref, timeout_s
            ));
            let args = vec![
                "-c".to_string(),
                "http.lowSpeedLimit=1024".to_string(),
                "-c".to_string(),
                "http.lowSpeedTime=20".to_string(),
                "clone".to_string(),
                url.to_string(),
                tmp_dir.display().to_string(),
            ];
            let status = self.run_git_with_timeout(&args)?;
            if status.success() {
                let checkout = Command::new("git")
                    .args([
                        "-C",
                        &tmp_dir.display().to_string(),
                        "checkout",
                        "--detach",
                        &self.source_ref,
                    ])
                    .status()?;
                if checkout.success() {
                    if let Some(commit) = run_capture_allow_fail(
                        "git",
                        &["-C", &tmp_dir.display().to_string(), "rev-parse", "HEAD"],
                    ) {
                        self.source_commit = commit.trim().to_string();
                    }
                    self.success(&format!("仓库拉取完成（{}）", self.source_commit));
                    return Ok(true);
                }
                self.warn(&format!(
                    "已拉取仓库，但 checkout 失败：{url}（ref: {}）",
                    self.source_ref
                ));
                return Ok(false);
            }
            if status.code() == Some(124) {
                self.warn(&format!("仓库拉取超时：{url}（{}s）", timeout_s));
            }
            self.warn(&format!(
                "仓库拉取或 checkout 失败：{url}（ref: {}）",
                self.source_ref
            ));
            return Ok(false);
        }

        self.log(&format!(
            "拉取仓库：{url}（{}，超时 {}s）",
            self.branch, timeout_s
        ));
        let args = vec![
            "-c".to_string(),
            "http.lowSpeedLimit=1024".to_string(),
            "-c".to_string(),
            "http.lowSpeedTime=20".to_string(),
            "clone".to_string(),
            "--depth".to_string(),
            "1".to_string(),
            "--branch".to_string(),
            self.branch.clone(),
            url.to_string(),
            tmp_dir.display().to_string(),
        ];
        let status = self.run_git_with_timeout(&args)?;
        if status.success() {
            if let Some(commit) = run_capture_allow_fail(
                "git",
                &["-C", &tmp_dir.display().to_string(), "rev-parse", "HEAD"],
            ) {
                self.source_commit = commit.trim().to_string();
            }
            self.success(&format!("仓库拉取完成（{}）", self.source_commit));
            return Ok(true);
        }
        if status.code() == Some(124) {
            self.warn(&format!("仓库拉取超时：{url}（{}s）", timeout_s));
        }
        self.warn(&format!("仓库拉取失败：{url}"));
        Ok(false)
    }

    fn clone_repo_any(&mut self, tmp_dir: &Path) -> Result<bool> {
        self.source_commit.clear();
        for (idx, url) in self.repo_urls.clone().iter().enumerate() {
            self.note(&format!(
                "尝试镜像 ({}/{})：{}",
                idx + 1,
                self.repo_urls.len(),
                url
            ));
            if tmp_dir.exists() {
                fs::remove_dir_all(tmp_dir).ok();
            }
            fs::create_dir_all(tmp_dir).ok();
            if self.clone_repo(tmp_dir, url)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn clone_repo_any_with_dns_retry(&mut self, tmp_dir: &Path) -> Result<bool> {
        if self.clone_repo_any(tmp_dir)? {
            return Ok(true);
        }
        self.log("尝试临时切换阿里云 DNS 后重试");
        if !self.temp_dns_enable()? {
            self.warn("临时 DNS 设置失败，将继续使用当前 DNS 再重试一次。");
        }
        if tmp_dir.exists() {
            fs::remove_dir_all(tmp_dir).ok();
        }
        fs::create_dir_all(tmp_dir).ok();
        self.clone_repo_any(tmp_dir)
    }

    fn prepare_source_repo(&mut self, tmp_dir: &Path) -> Result<()> {
        if self.force_remote_source {
            self.require_remote_source_pin()?;
            if !self.clone_repo_any_with_dns_retry(tmp_dir)? {
                bail!("仓库拉取失败");
            }
            return Ok(());
        }

        if let Some(source_dir) = self.detect_local_repo_dir() {
            self.prepare_local_source(tmp_dir, &source_dir)?;
            return Ok(());
        }

        self.require_remote_source_pin()?;
        if !self.clone_repo_any_with_dns_retry(tmp_dir)? {
            bail!("仓库拉取失败");
        }
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
            let backup = create_temp_path("run-rs-resolv", "conf")?;
            self.run_as_root_ok(
                "cp",
                &[
                    "-a".to_string(),
                    "/etc/resolv.conf".to_string(),
                    backup.display().to_string(),
                ],
            )?;
            self.run_as_root_ok("rm", &["-f".to_string(), "/etc/resolv.conf".to_string()])?;

            let content_file = create_temp_path("run-rs-resolv-new", "conf")?;
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

    fn list_hosts(&self, repo_dir: &Path) -> Vec<String> {
        let mut hosts = Vec::new();
        let host_dir = repo_dir.join("hosts");
        if host_dir.is_dir()
            && let Ok(entries) = fs::read_dir(host_dir)
        {
            for entry in entries.flatten() {
                if !entry.path().is_dir() {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().to_string();
                if name != "profiles" {
                    hosts.push(name);
                }
            }
        }
        hosts.sort();
        hosts
    }

    fn select_host(&mut self, repo_dir: &Path) -> Result<()> {
        if !self.target_name.is_empty() {
            return Ok(());
        }
        if self.is_tty() {
            let hosts = self.list_hosts(repo_dir);
            if hosts.is_empty() {
                bail!("未找到可用的 hosts 目录。");
            }
            let mut default_index = 1usize;
            for (i, h) in hosts.iter().enumerate() {
                if h == "nixos" {
                    default_index = i + 1;
                    break;
                }
            }
            let pick = self.menu_prompt("选择主机", default_index, &hosts)?;
            self.target_name = hosts[pick - 1].clone();
        } else {
            self.target_name = "nixos".to_string();
        }
        Ok(())
    }

    fn validate_host(&self, repo_dir: &Path) -> Result<()> {
        if self.target_name.is_empty() {
            bail!("未指定主机名称。");
        }
        if !repo_dir.join("hosts").join(&self.target_name).is_dir() {
            bail!("主机不存在：hosts/{}", self.target_name);
        }
        Ok(())
    }

    fn detect_host_profile_kind(&mut self, repo_dir: &Path) {
        self.host_profile_kind = HostProfileKind::Unknown;
        let host_file = repo_dir
            .join("hosts")
            .join(&self.target_name)
            .join("default.nix");
        if let Ok(text) = fs::read_to_string(host_file) {
            if text.contains("../profiles/server.nix") {
                self.host_profile_kind = HostProfileKind::Server;
            } else if text.contains("../profiles/desktop.nix") {
                self.host_profile_kind = HostProfileKind::Desktop;
            }
        }
    }

    fn detect_per_user_tun(&self, repo_dir: &Path) -> bool {
        if command_exists("nix") {
            let mut nix_config = "experimental-features = nix-command flakes".to_string();
            if let Ok(extra) = std::env::var("NIX_CONFIG")
                && !extra.trim().is_empty()
            {
                nix_config = format!("{extra}\n{nix_config}");
            }
            let target = format!(
                "{}#nixosConfigurations.{}.config.mcb.perUserTun.enable",
                repo_dir.display(),
                self.target_name
            );
            let out = Command::new("nix")
                .env("NIX_CONFIG", nix_config)
                .args(["eval", "--raw", &target])
                .output();
            if let Ok(out) = out
                && out.status.success()
            {
                let v = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if v == "true" {
                    return true;
                }
                if v == "false" {
                    return false;
                }
            }
        }

        let files = vec![
            repo_dir
                .join("hosts")
                .join(&self.target_name)
                .join("local.nix"),
            repo_dir
                .join("hosts")
                .join(&self.target_name)
                .join("default.nix"),
        ];
        for file in files {
            let Ok(text) = fs::read_to_string(file) else {
                continue;
            };
            if text
                .lines()
                .map(strip_comment)
                .any(|l| l.contains("mcb.perUserTun.enable") && l.contains("true"))
            {
                return true;
            }
            let mut in_block = false;
            for line in text.lines().map(strip_comment) {
                if line.contains("perUserTun") && line.contains('{') {
                    in_block = true;
                }
                if in_block && line.contains("enable") && line.contains("true") {
                    return true;
                }
                if in_block && line.contains('}') {
                    in_block = false;
                }
            }
        }
        false
    }

    fn extract_user_from_file(file: &Path) -> Option<String> {
        let text = fs::read_to_string(file).ok()?;
        for line in text.lines() {
            let l = strip_comment(line);
            if l.contains("mcb.user") && l.contains('=') && l.contains('"') {
                if let Some(v) = first_quoted(l) {
                    return Some(v);
                }
            }
            if l.trim_start().starts_with("user")
                && l.contains('=')
                && l.contains('"')
                && let Some(v) = first_quoted(l)
            {
                return Some(v);
            }
        }
        None
    }

    fn resolve_default_user(&self) -> String {
        let mut files = Vec::new();
        if let Some(tmp_dir) = &self.tmp_dir {
            files.push(
                tmp_dir
                    .join("hosts")
                    .join(&self.target_name)
                    .join("local.nix"),
            );
            files.push(
                tmp_dir
                    .join("hosts")
                    .join(&self.target_name)
                    .join("default.nix"),
            );
        }
        files.push(
            self.etc_dir
                .join("hosts")
                .join(&self.target_name)
                .join("local.nix"),
        );
        files.push(
            self.etc_dir
                .join("hosts")
                .join(&self.target_name)
                .join("default.nix"),
        );
        for file in files {
            if let Some(v) = Self::extract_user_from_file(&file) {
                return v;
            }
        }
        "mcbnixos".to_string()
    }

    fn list_existing_home_users(&self, repo_dir: &Path) -> Vec<String> {
        let mut users = Vec::new();
        let users_dir = repo_dir.join("home/users");
        if users_dir.is_dir()
            && let Ok(entries) = fs::read_dir(users_dir)
        {
            for entry in entries.flatten() {
                if !entry.path().is_dir() {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().to_string();
                if is_valid_username(&name) {
                    users.push(name);
                }
            }
        }
        users.sort();
        users
    }

    fn add_target_user(&mut self, user: &str) {
        if !self.target_users.iter().any(|u| u == user) {
            self.target_users.push(user.to_string());
        }
    }

    fn remove_target_user(&mut self, user: &str) {
        self.target_users.retain(|u| u != user);
    }

    fn toggle_target_user(&mut self, user: &str) {
        if self.target_users.iter().any(|u| u == user) {
            self.remove_target_user(user);
        } else {
            self.add_target_user(user);
        }
    }

    fn add_admin_user(&mut self, user: &str) {
        if !self.target_admin_users.iter().any(|u| u == user) {
            self.target_admin_users.push(user.to_string());
        }
    }

    fn remove_admin_user(&mut self, user: &str) {
        self.target_admin_users.retain(|u| u != user);
    }

    fn toggle_admin_user(&mut self, user: &str) {
        if self.target_admin_users.iter().any(|u| u == user) {
            self.remove_admin_user(user);
        } else {
            self.add_admin_user(user);
        }
    }

    fn select_existing_users_menu(&mut self, users: &[String]) -> Result<bool> {
        loop {
            let mut options = Vec::new();
            for user in users {
                if self.target_users.iter().any(|u| u == user) {
                    options.push(format!("[x] {user}"));
                } else {
                    options.push(format!("[ ] {user}"));
                }
            }
            options.push("完成".to_string());
            options.push("返回".to_string());
            let pick = self.menu_prompt("勾选已有用户（可重复切换）", 1, &options)?;
            if pick >= 1 && pick <= users.len() {
                self.toggle_target_user(&users[pick - 1]);
                continue;
            }
            if pick == users.len() + 1 {
                return Ok(true);
            }
            return Ok(false);
        }
    }

    fn select_admin_users_menu(&mut self) -> Result<bool> {
        loop {
            let mut options = Vec::new();
            for user in &self.target_users {
                if self.target_admin_users.iter().any(|u| u == user) {
                    options.push(format!("[x] {user}"));
                } else {
                    options.push(format!("[ ] {user}"));
                }
            }
            options.push("完成".to_string());
            options.push("返回".to_string());
            let pick = self.menu_prompt("勾选管理员用户（可重复切换）", 1, &options)?;
            if pick >= 1 && pick <= self.target_users.len() {
                let user = self.target_users[pick - 1].clone();
                self.toggle_admin_user(&user);
                continue;
            }
            if pick == self.target_users.len() + 1 {
                return Ok(true);
            }
            return Ok(false);
        }
    }

    fn prompt_users(&mut self, repo_dir: &Path) -> Result<WizardAction> {
        let default_user = self.resolve_default_user();
        if !self.is_tty() {
            if self.target_users.is_empty() {
                self.target_users = vec![default_user];
            }
            return Ok(WizardAction::Continue);
        }
        if self.target_users.is_empty() {
            self.target_users = vec![default_user.clone()];
        }

        loop {
            let current = if self.target_users.is_empty() {
                "未选择".to_string()
            } else {
                self.target_users.join(" ")
            };
            let pick = self.menu_prompt(
                &format!("选择用户（当前：{current}）"),
                1,
                &[
                    format!("仅使用默认用户 ({default_user})"),
                    "从已有 Home 用户中选择".to_string(),
                    "新增用户（手写用户名）".to_string(),
                    "清空已选用户".to_string(),
                    "完成".to_string(),
                    "返回".to_string(),
                    "退出".to_string(),
                ],
            )?;
            match pick {
                1 => {
                    self.target_users = vec![default_user.clone()];
                }
                2 => {
                    let mut existing = self.list_existing_home_users(repo_dir);
                    existing.sort();
                    existing.dedup();
                    if existing.is_empty() {
                        self.warn("未发现可选的已有 Home 用户目录。");
                        continue;
                    }
                    let _ = self.select_existing_users_menu(&existing)?;
                }
                3 => {
                    print!("输入新增用户名（留空取消）： ");
                    io::stdout().flush().ok();
                    let mut input = String::new();
                    io::stdin().read_line(&mut input).ok();
                    let input = input.trim();
                    if input.is_empty() {
                        continue;
                    }
                    if !is_valid_username(input) {
                        self.warn(&format!("用户名不合法：{input}"));
                        continue;
                    }
                    self.add_target_user(input);
                }
                4 => {
                    self.target_users.clear();
                }
                5 => {
                    if self.target_users.is_empty() {
                        self.warn("请至少选择一个用户。");
                        continue;
                    }
                    return Ok(WizardAction::Continue);
                }
                6 => return Ok(WizardAction::Back),
                7 => bail!("已退出"),
                _ => {}
            }
        }
    }

    fn prompt_admin_users(&mut self) -> Result<WizardAction> {
        if self.target_users.is_empty() {
            bail!("用户列表为空，无法选择管理员。");
        }
        let default_admin = self.target_users[0].clone();
        if !self.is_tty() {
            if self.target_admin_users.is_empty() {
                self.target_admin_users = vec![default_admin];
            }
            return Ok(WizardAction::Continue);
        }
        if self.target_admin_users.is_empty() {
            self.target_admin_users = vec![default_admin.clone()];
        }

        loop {
            let current = if self.target_admin_users.is_empty() {
                "未选择".to_string()
            } else {
                self.target_admin_users.join(" ")
            };
            let pick = self.menu_prompt(
                &format!("管理员权限（wheel，当前：{current}）"),
                1,
                &[
                    format!("仅主用户 ({default_admin})"),
                    "所有用户".to_string(),
                    "自定义勾选管理员".to_string(),
                    "清空管理员".to_string(),
                    "完成".to_string(),
                    "返回".to_string(),
                    "退出".to_string(),
                ],
            )?;
            match pick {
                1 => self.target_admin_users = vec![default_admin.clone()],
                2 => self.target_admin_users = self.target_users.clone(),
                3 => {
                    let _ = self.select_admin_users_menu()?;
                }
                4 => self.target_admin_users.clear(),
                5 => {
                    if self.target_admin_users.is_empty() {
                        self.warn("至少需要一个管理员用户。");
                        continue;
                    }
                    return Ok(WizardAction::Continue);
                }
                6 => return Ok(WizardAction::Back),
                7 => bail!("已退出"),
                _ => {}
            }
        }
    }

    fn dedupe_users(&mut self) {
        let mut set = BTreeSet::new();
        let mut out = Vec::new();
        for u in &self.target_users {
            if set.insert(u.clone()) {
                out.push(u.clone());
            }
        }
        self.target_users = out;
    }

    fn dedupe_admin_users(&mut self) {
        let mut set = BTreeSet::new();
        let mut out = Vec::new();
        for u in &self.target_admin_users {
            if set.insert(u.clone()) {
                out.push(u.clone());
            }
        }
        self.target_admin_users = out;
    }

    fn validate_users(&self) -> Result<()> {
        for user in &self.target_users {
            if !is_valid_username(user) {
                bail!("用户名不合法：{user}");
            }
        }
        Ok(())
    }

    fn validate_admin_users(&mut self) -> Result<()> {
        if self.target_admin_users.is_empty() && !self.target_users.is_empty() {
            self.target_admin_users = vec![self.target_users[0].clone()];
        }
        for user in &self.target_admin_users {
            if !is_valid_username(user) {
                bail!("管理员用户名不合法：{user}");
            }
            if !self.target_users.iter().any(|u| u == user) {
                bail!("管理员用户必须包含在用户列表中：{user}");
            }
        }
        Ok(())
    }

    fn reset_tun_maps(&mut self) {
        self.user_tun.clear();
        self.user_dns.clear();
    }

    fn reset_admin_users(&mut self) {
        self.target_admin_users.clear();
    }

    fn reset_server_overrides(&mut self) {
        self.server_overrides_enabled = false;
        self.server_enable_network_cli.clear();
        self.server_enable_network_gui.clear();
        self.server_enable_shell_tools.clear();
        self.server_enable_wayland_tools.clear();
        self.server_enable_system_tools.clear();
        self.server_enable_geek_tools.clear();
        self.server_enable_gaming.clear();
        self.server_enable_insecure_tools.clear();
        self.server_enable_docker.clear();
        self.server_enable_libvirtd.clear();
    }

    fn reset_gpu_override(&mut self) {
        self.gpu_override = false;
        self.gpu_mode.clear();
        self.gpu_igpu_vendor.clear();
        self.gpu_prime_mode.clear();
        self.gpu_intel_bus.clear();
        self.gpu_amd_bus.clear();
        self.gpu_nvidia_bus.clear();
        self.gpu_nvidia_open.clear();
        self.gpu_specialisations_enabled = false;
        self.gpu_specialisations_set = false;
        self.gpu_specialisation_modes.clear();
    }

    fn configure_per_user_tun(&mut self) -> Result<WizardAction> {
        if !self.per_user_tun_enabled {
            return Ok(WizardAction::Continue);
        }
        if !self.is_tty() {
            self.reset_tun_maps();
            return Ok(WizardAction::Continue);
        }
        let pick = self.menu_prompt(
            "TUN 配置方式",
            1,
            &[
                "沿用主机配置".to_string(),
                "使用默认接口/端口 (tun0/tun1 + 1053..)".to_string(),
                "使用常见接口名 (Meta/Mihomo/clash0)".to_string(),
                "返回".to_string(),
            ],
        )?;
        match pick {
            4 => return Ok(WizardAction::Back),
            1 => {
                self.reset_tun_maps();
                return Ok(WizardAction::Continue);
            }
            2 => {
                self.reset_tun_maps();
                for (idx, user) in self.target_users.iter().enumerate() {
                    self.user_tun.insert(user.clone(), format!("tun{idx}"));
                    self.user_dns.insert(user.clone(), 1053 + idx as u16);
                }
            }
            3 => {
                self.reset_tun_maps();
                let common = ["Meta", "Mihomo", "clash0", "tun0", "tun1", "tun2"];
                for (idx, user) in self.target_users.iter().enumerate() {
                    let iface = common
                        .get(idx)
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| format!("tun{idx}"));
                    self.user_tun.insert(user.clone(), iface);
                    self.user_dns.insert(user.clone(), 1053 + idx as u16);
                }
            }
            _ => {}
        }
        Ok(WizardAction::Continue)
    }

    fn strip_leading_zeros(v: &str) -> String {
        let trimmed = v.trim_start_matches('0');
        if trimmed.is_empty() {
            "0".to_string()
        } else {
            trimmed.to_string()
        }
    }

    fn normalize_pci_bus_id(addr: &str) -> Option<String> {
        let raw = addr.strip_prefix("0000:").unwrap_or(addr);
        let mut split = raw.split(':');
        let bus = split.next()?;
        let rest = split.next()?;
        let mut rest_split = rest.split('.');
        let dev = rest_split.next()?;
        let func = rest_split.next()?;
        Some(format!(
            "PCI:{}:{}:{}",
            Self::strip_leading_zeros(bus),
            Self::strip_leading_zeros(dev),
            Self::strip_leading_zeros(func)
        ))
    }

    fn detect_bus_ids_from_lspci(vendor: &str) -> Vec<String> {
        let mut out = Vec::new();
        if !command_exists("lspci") {
            return out;
        }
        let Some(text) = run_capture_allow_fail("lspci", &["-D", "-d", "::03xx"]) else {
            return out;
        };
        for line in text.lines() {
            let ok = match vendor {
                "intel" => line.contains("Intel"),
                "amd" => line.contains("AMD") || line.contains("Advanced Micro Devices"),
                "nvidia" => line.contains("NVIDIA"),
                _ => false,
            };
            if !ok {
                continue;
            }
            if let Some(addr) = line.split_whitespace().next()
                && let Some(norm) = Self::normalize_pci_bus_id(addr)
            {
                out.push(norm);
            }
        }
        out
    }

    fn extract_bus_id_from_file(file: &Path, key: &str) -> Option<String> {
        let text = fs::read_to_string(file).ok()?;
        for line in text.lines() {
            let l = strip_comment(line);
            if l.contains(key) && l.contains('=') && l.contains('"') {
                return first_quoted(l);
            }
        }
        None
    }

    fn resolve_bus_id_default(&self, vendor: &str) -> Option<String> {
        if let Some(v) = Self::detect_bus_ids_from_lspci(vendor).first() {
            return Some(v.clone());
        }
        let key = match vendor {
            "intel" => "intelBusId",
            "amd" => "amdgpuBusId",
            "nvidia" => "nvidiaBusId",
            _ => return None,
        };
        let mut files = Vec::new();
        files.push(
            self.etc_dir
                .join("hosts")
                .join(&self.target_name)
                .join("local.nix"),
        );
        files.push(
            self.etc_dir
                .join("hosts")
                .join(&self.target_name)
                .join("default.nix"),
        );
        if let Some(tmp) = &self.tmp_dir {
            files.push(tmp.join("hosts").join(&self.target_name).join("local.nix"));
            files.push(
                tmp.join("hosts")
                    .join(&self.target_name)
                    .join("default.nix"),
            );
        }
        for f in files {
            if let Some(v) = Self::extract_bus_id_from_file(&f, key) {
                return Some(v);
            }
        }
        None
    }

    fn bus_candidates_for_vendor(&self, vendor: &str) -> Vec<String> {
        let mut seen = BTreeSet::new();
        let mut out = Vec::new();
        if let Some(fallback) = self.resolve_bus_id_default(vendor)
            && seen.insert(fallback.clone())
        {
            out.push(fallback);
        }
        for v in Self::detect_bus_ids_from_lspci(vendor) {
            if seen.insert(v.clone()) {
                out.push(v);
            }
        }
        out
    }

    fn configure_gpu(&mut self) -> Result<WizardAction> {
        if !self.is_tty() {
            self.reset_gpu_override();
            return Ok(WizardAction::Continue);
        }

        let pick = self.menu_prompt(
            "GPU 配置方式",
            1,
            &[
                "沿用主机配置".to_string(),
                "选择 GPU 模式".to_string(),
                "返回".to_string(),
            ],
        )?;
        match pick {
            1 => {
                self.reset_gpu_override();
                return Ok(WizardAction::Continue);
            }
            2 => self.gpu_override = true,
            3 => return Ok(WizardAction::Back),
            _ => {}
        }

        let pick = self.menu_prompt(
            "选择 GPU 模式",
            2,
            &[
                "核显 (igpu)".to_string(),
                "混合 (hybrid)".to_string(),
                "独显 (dgpu)".to_string(),
                "返回".to_string(),
            ],
        )?;
        match pick {
            1 => self.gpu_mode = "igpu".to_string(),
            2 => self.gpu_mode = "hybrid".to_string(),
            3 => self.gpu_mode = "dgpu".to_string(),
            4 => return Ok(WizardAction::Back),
            _ => {}
        }

        if self.gpu_mode == "igpu" || self.gpu_mode == "hybrid" {
            let pick = self.menu_prompt(
                "核显厂商",
                1,
                &["Intel".to_string(), "AMD".to_string(), "返回".to_string()],
            )?;
            match pick {
                1 => self.gpu_igpu_vendor = "intel".to_string(),
                2 => self.gpu_igpu_vendor = "amd".to_string(),
                3 => return Ok(WizardAction::Back),
                _ => {}
            }
        }

        if self.gpu_mode == "hybrid" {
            let pick = self.menu_prompt(
                "PRIME 模式",
                1,
                &[
                    "offload（推荐，Wayland）".to_string(),
                    "sync（偏向 X11）".to_string(),
                    "reverseSync（偏向 X11）".to_string(),
                    "返回".to_string(),
                ],
            )?;
            match pick {
                1 => self.gpu_prime_mode = "offload".to_string(),
                2 => self.gpu_prime_mode = "sync".to_string(),
                3 => self.gpu_prime_mode = "reverseSync".to_string(),
                4 => return Ok(WizardAction::Back),
                _ => {}
            }

            if self.gpu_igpu_vendor == "intel" {
                let candidates = self.bus_candidates_for_vendor("intel");
                if candidates.is_empty() {
                    let pick = self.menu_prompt(
                        "未检测到 Intel iGPU busId",
                        1,
                        &["沿用主机配置".to_string(), "返回".to_string()],
                    )?;
                    if pick == 1 {
                        self.reset_gpu_override();
                        return Ok(WizardAction::Continue);
                    }
                    return Ok(WizardAction::Back);
                }
                let mut options = candidates.clone();
                options.push("返回".to_string());
                let pick = self.menu_prompt("选择 Intel iGPU busId", 1, &options)?;
                if pick == options.len() {
                    return Ok(WizardAction::Back);
                }
                self.gpu_intel_bus = options[pick - 1].clone();
            } else {
                let candidates = self.bus_candidates_for_vendor("amd");
                if candidates.is_empty() {
                    let pick = self.menu_prompt(
                        "未检测到 AMD iGPU busId",
                        1,
                        &["沿用主机配置".to_string(), "返回".to_string()],
                    )?;
                    if pick == 1 {
                        self.reset_gpu_override();
                        return Ok(WizardAction::Continue);
                    }
                    return Ok(WizardAction::Back);
                }
                let mut options = candidates.clone();
                options.push("返回".to_string());
                let pick = self.menu_prompt("选择 AMD iGPU busId", 1, &options)?;
                if pick == options.len() {
                    return Ok(WizardAction::Back);
                }
                self.gpu_amd_bus = options[pick - 1].clone();
            }

            let nvidia = self.bus_candidates_for_vendor("nvidia");
            if nvidia.is_empty() {
                let pick = self.menu_prompt(
                    "未检测到 NVIDIA dGPU busId",
                    1,
                    &["沿用主机配置".to_string(), "返回".to_string()],
                )?;
                if pick == 1 {
                    self.reset_gpu_override();
                    return Ok(WizardAction::Continue);
                }
                return Ok(WizardAction::Back);
            }
            let mut options = nvidia.clone();
            options.push("返回".to_string());
            let pick = self.menu_prompt("选择 NVIDIA dGPU busId", 1, &options)?;
            if pick == options.len() {
                return Ok(WizardAction::Back);
            }
            self.gpu_nvidia_bus = options[pick - 1].clone();
        }

        if self.gpu_mode == "hybrid" || self.gpu_mode == "dgpu" {
            let pick = self.menu_prompt(
                "NVIDIA 使用开源内核模块？",
                1,
                &[
                    "是（open=true）".to_string(),
                    "否（open=false）".to_string(),
                    "返回".to_string(),
                ],
            )?;
            match pick {
                1 => self.gpu_nvidia_open = "true".to_string(),
                2 => self.gpu_nvidia_open = "false".to_string(),
                3 => return Ok(WizardAction::Back),
                _ => {}
            }
        }

        if self.gpu_mode == "hybrid" {
            let pick = self.menu_prompt(
                "生成 GPU specialisation（igpu/hybrid/dgpu）以便切换？",
                1,
                &["是".to_string(), "否".to_string(), "返回".to_string()],
            )?;
            match pick {
                1 => {
                    self.gpu_specialisations_enabled = true;
                    self.gpu_specialisations_set = true;
                }
                2 => {
                    self.gpu_specialisations_enabled = false;
                    self.gpu_specialisations_set = true;
                }
                3 => return Ok(WizardAction::Back),
                _ => {}
            }
            if self.gpu_specialisations_enabled {
                self.gpu_specialisation_modes =
                    vec!["igpu".to_string(), "hybrid".to_string(), "dgpu".to_string()];
            }
        }
        Ok(WizardAction::Continue)
    }

    fn configure_server_overrides(&mut self) -> Result<WizardAction> {
        if self.host_profile_kind != HostProfileKind::Server {
            self.reset_server_overrides();
            return Ok(WizardAction::Continue);
        }
        if !self.is_tty() {
            self.reset_server_overrides();
            return Ok(WizardAction::Continue);
        }
        let pick = self.menu_prompt(
            "服务器软件配置",
            1,
            &[
                "沿用主机配置".to_string(),
                "运维服务器预设（CLI + Geek + Docker）".to_string(),
                "自定义开关".to_string(),
                "返回".to_string(),
            ],
        )?;
        match pick {
            1 => {
                self.reset_server_overrides();
            }
            2 => {
                self.server_overrides_enabled = true;
                self.server_enable_network_cli = "true".to_string();
                self.server_enable_network_gui = "false".to_string();
                self.server_enable_shell_tools = "true".to_string();
                self.server_enable_wayland_tools = "false".to_string();
                self.server_enable_system_tools = "true".to_string();
                self.server_enable_geek_tools = "true".to_string();
                self.server_enable_gaming = "false".to_string();
                self.server_enable_insecure_tools = "false".to_string();
                self.server_enable_docker = "true".to_string();
                self.server_enable_libvirtd = "false".to_string();
            }
            3 => {
                self.server_overrides_enabled = true;
                self.server_enable_network_cli = self
                    .ask_bool("启用网络/代理 CLI（mcb.packages.enableNetworkCli）？", true)?
                    .to_string();
                self.server_enable_network_gui = self
                    .ask_bool("启用网络图形工具（mcb.packages.enableNetworkGui）？", false)?
                    .to_string();
                self.server_enable_shell_tools = self
                    .ask_bool("启用命令行工具组（mcb.packages.enableShellTools）？", true)?
                    .to_string();
                self.server_enable_wayland_tools = self
                    .ask_bool(
                        "启用 Wayland 工具组（mcb.packages.enableWaylandTools）？",
                        false,
                    )?
                    .to_string();
                self.server_enable_system_tools = self
                    .ask_bool("启用系统工具组（mcb.packages.enableSystemTools）？", true)?
                    .to_string();
                self.server_enable_geek_tools = self
                    .ask_bool("启用调试/诊断工具（mcb.packages.enableGeekTools）？", false)?
                    .to_string();
                self.server_enable_gaming = self
                    .ask_bool("启用游戏工具组（mcb.packages.enableGaming）？", false)?
                    .to_string();
                self.server_enable_insecure_tools = self
                    .ask_bool(
                        "启用不安全软件组（mcb.packages.enableInsecureTools）？",
                        false,
                    )?
                    .to_string();
                self.server_enable_docker = self
                    .ask_bool("启用 Docker（mcb.virtualisation.docker.enable）？", false)?
                    .to_string();
                self.server_enable_libvirtd = self
                    .ask_bool(
                        "启用 Libvirt/KVM（mcb.virtualisation.libvirtd.enable）？",
                        false,
                    )?
                    .to_string();
            }
            4 => return Ok(WizardAction::Back),
            _ => {}
        }
        Ok(WizardAction::Continue)
    }

    fn print_summary(&mut self) {
        self.section("部署概要");
        match self.deploy_mode {
            DeployMode::UpdateExisting => {
                println!("部署模式：仅更新当前配置（保留用户/权限）");
            }
            DeployMode::ManageUsers => {
                println!("部署模式：新增/调整用户并部署");
            }
        }
        println!("主机：{}", self.target_name);
        if self.deploy_mode == DeployMode::UpdateExisting {
            if !self.source_ref.is_empty() {
                println!("源策略：网络仓库固定版本 ({})", self.source_ref);
            } else {
                println!("源策略：网络仓库最新 HEAD");
            }
            println!("用户/权限：保持当前主机 local.nix");
        } else {
            println!("用户：{}", self.target_users.join(" "));
            println!("管理员：{}", self.target_admin_users.join(" "));
        }
        if !self.source_commit.is_empty() {
            println!("源提交：{}", self.source_commit);
        }
        println!(
            "覆盖策略：{}",
            match self.overwrite_mode {
                OverwriteMode::Ask => "ask",
                OverwriteMode::Backup => "backup",
                OverwriteMode::Overwrite => "overwrite",
            }
        );
        println!(
            "依赖升级：{}",
            if self.rebuild_upgrade {
                "启用"
            } else {
                "关闭"
            }
        );

        if self.deploy_mode == DeployMode::UpdateExisting {
            return;
        }

        if self.per_user_tun_enabled {
            if self.user_tun.is_empty() {
                println!("Per-user TUN：已启用（沿用主机配置）");
            } else {
                println!("Per-user TUN：已启用");
                for user in &self.target_users {
                    let iface = self.user_tun.get(user).cloned().unwrap_or_default();
                    let dns = self.user_dns.get(user).copied().unwrap_or_default();
                    println!("  - {user} -> {iface} (DNS {dns})");
                }
            }
        } else {
            println!("Per-user TUN：未启用");
        }

        if self.gpu_override {
            println!("GPU：{}", self.gpu_mode);
            if !self.gpu_igpu_vendor.is_empty() {
                println!("  - iGPU 厂商：{}", self.gpu_igpu_vendor);
            }
            if !self.gpu_prime_mode.is_empty() {
                println!("  - PRIME：{}", self.gpu_prime_mode);
            }
            if !self.gpu_intel_bus.is_empty() {
                println!("  - Intel busId：{}", self.gpu_intel_bus);
            }
            if !self.gpu_amd_bus.is_empty() {
                println!("  - AMD busId：{}", self.gpu_amd_bus);
            }
            if !self.gpu_nvidia_bus.is_empty() {
                println!("  - NVIDIA busId：{}", self.gpu_nvidia_bus);
            }
            if !self.gpu_nvidia_open.is_empty() {
                println!("  - NVIDIA open：{}", self.gpu_nvidia_open);
            }
            if self.gpu_specialisations_enabled {
                println!(
                    "  - specialisation：启用 ({})",
                    self.gpu_specialisation_modes.join(" ")
                );
            }
        } else {
            println!("GPU：沿用主机配置");
        }

        if self.server_overrides_enabled {
            println!("服务器软件覆盖：已启用");
            println!("  - enableNetworkCli={}", self.server_enable_network_cli);
            println!("  - enableNetworkGui={}", self.server_enable_network_gui);
            println!("  - enableShellTools={}", self.server_enable_shell_tools);
            println!(
                "  - enableWaylandTools={}",
                self.server_enable_wayland_tools
            );
            println!("  - enableSystemTools={}", self.server_enable_system_tools);
            println!("  - enableGeekTools={}", self.server_enable_geek_tools);
            println!("  - enableGaming={}", self.server_enable_gaming);
            println!(
                "  - enableInsecureTools={}",
                self.server_enable_insecure_tools
            );
            println!("  - docker.enable={}", self.server_enable_docker);
            println!("  - libvirtd.enable={}", self.server_enable_libvirtd);
        }
    }

    fn wizard_flow(&mut self, repo_dir: &Path) -> Result<()> {
        let mut step = 1u8;

        if self.deploy_mode == DeployMode::UpdateExisting {
            loop {
                match step {
                    1 => {
                        self.select_host(repo_dir)?;
                        self.validate_host(repo_dir)?;
                        self.detect_host_profile_kind(repo_dir);
                        step = 2;
                    }
                    2 => {
                        self.print_summary();
                        if self.is_tty() {
                            match self.wizard_back_or_quit("确认仅更新当前配置并继续？")?
                            {
                                WizardAction::Back => {
                                    self.target_name.clear();
                                    step = 1;
                                }
                                WizardAction::Continue => return Ok(()),
                            }
                        } else {
                            return Ok(());
                        }
                    }
                    _ => return Ok(()),
                }
            }
        }

        loop {
            match step {
                1 => {
                    self.select_host(repo_dir)?;
                    self.validate_host(repo_dir)?;
                    self.detect_host_profile_kind(repo_dir);
                    step = 2;
                }
                2 => {
                    match self.prompt_users(repo_dir)? {
                        WizardAction::Back => {
                            self.target_users.clear();
                            self.reset_admin_users();
                            self.reset_tun_maps();
                            self.reset_gpu_override();
                            self.reset_server_overrides();
                            self.target_name.clear();
                            step = 1;
                            continue;
                        }
                        WizardAction::Continue => {}
                    }
                    self.dedupe_users();
                    self.validate_users()?;
                    self.reset_admin_users();
                    self.reset_tun_maps();
                    self.reset_gpu_override();
                    self.reset_server_overrides();
                    step = 3;
                }
                3 => {
                    match self.prompt_admin_users()? {
                        WizardAction::Back => {
                            self.reset_admin_users();
                            step = 2;
                            continue;
                        }
                        WizardAction::Continue => {}
                    }
                    self.dedupe_admin_users();
                    self.validate_admin_users()?;
                    step = 4;
                }
                4 => {
                    self.per_user_tun_enabled = self.detect_per_user_tun(repo_dir);
                    if self.per_user_tun_enabled {
                        match self.configure_per_user_tun()? {
                            WizardAction::Back => {
                                self.reset_tun_maps();
                                step = 3;
                                continue;
                            }
                            WizardAction::Continue => {}
                        }
                    } else {
                        self.reset_tun_maps();
                    }
                    step = 5;
                }
                5 => {
                    if self.host_profile_kind == HostProfileKind::Server {
                        self.reset_gpu_override();
                        step = 6;
                        continue;
                    }
                    match self.configure_gpu()? {
                        WizardAction::Back => {
                            self.reset_gpu_override();
                            step = 4;
                            continue;
                        }
                        WizardAction::Continue => {}
                    }
                    step = 6;
                }
                6 => {
                    if self.host_profile_kind != HostProfileKind::Server {
                        self.reset_server_overrides();
                        step = 7;
                        continue;
                    }
                    match self.configure_server_overrides()? {
                        WizardAction::Back => {
                            self.reset_server_overrides();
                            step = 5;
                            continue;
                        }
                        WizardAction::Continue => {}
                    }
                    step = 7;
                }
                7 => {
                    self.print_summary();
                    if self.is_tty() {
                        match self.wizard_back_or_quit("确认以上配置")? {
                            WizardAction::Back => {
                                step = if self.host_profile_kind == HostProfileKind::Server {
                                    6
                                } else {
                                    5
                                };
                            }
                            WizardAction::Continue => return Ok(()),
                        }
                    } else {
                        return Ok(());
                    }
                }
                _ => return Ok(()),
            }
        }
    }

    fn ensure_user_home_entries(&mut self, repo_dir: &Path) -> Result<()> {
        let mut profile_import = "../../profiles/full.nix";
        let extra_imports = vec!["./git.nix", "./packages.nix"];
        let mut include_user_files = true;
        if self.host_profile_kind == HostProfileKind::Server {
            profile_import = "../../profiles/minimal.nix";
            include_user_files = false;
        }

        let default_user = self.resolve_default_user();
        let mut template_user = String::new();
        let mut template_dir = None::<PathBuf>;
        let default_dir = repo_dir.join("home/users").join(&default_user);
        if default_dir.is_dir() {
            template_user = default_user.clone();
            template_dir = Some(default_dir);
        } else {
            let mcb = repo_dir.join("home/users/mcbnixos");
            if mcb.is_dir() {
                template_user = "mcbnixos".to_string();
                template_dir = Some(mcb);
            }
        }
        if !template_user.is_empty() {
            self.note(&format!("新用户模板来源：home/users/{template_user}"));
        }

        let copy_template_content = std::env::var("RUN_SH_COPY_USER_TEMPLATE")
            .ok()
            .is_some_and(|v| v == "true");
        if copy_template_content {
            self.note("将复制模板用户目录内容（RUN_SH_COPY_USER_TEMPLATE=true）");
        } else {
            self.note("默认仅生成最小用户模板（不复制 config/assets/scripts）；如需复制可设置 RUN_SH_COPY_USER_TEMPLATE=true");
        }

        for user in self.target_users.clone() {
            let user_dir = repo_dir.join("home/users").join(&user);
            let user_file = user_dir.join("default.nix");
            let create_default = !user_file.is_file();

            fs::create_dir_all(&user_dir)?;
            if create_default
                && let Some(template_dir) = &template_dir
                && user_dir != *template_dir
                && include_user_files
                && copy_template_content
            {
                for item in ["config", "assets", "scripts"] {
                    let src = template_dir.join(item);
                    let dst = user_dir.join(item);
                    if src.exists() && !dst.exists() {
                        copy_recursively(&src, &dst)?;
                    }
                }
                for template_file in ["files.nix", "scripts.nix"] {
                    let src = template_dir.join(template_file);
                    let dst = user_dir.join(template_file);
                    if src.is_file() && !dst.exists() {
                        fs::copy(src, dst).ok();
                    }
                }
            }

            let git_file = user_dir.join("git.nix");
            if !git_file.is_file() {
                fs::write(
                    &git_file,
                    r#"# 默认 Git 身份（请按需修改）
{ config, ... }:

{
  programs.git.settings.user = {
    name = config.home.username;
    # email = "you@example.com";
  };
}
"#,
                )?;
            }

            let packages_file = user_dir.join("packages.nix");
            if !packages_file.is_file() {
                if let Some(template_dir) = &template_dir {
                    let src = template_dir.join("packages.nix");
                    if src.is_file() && user_dir != *template_dir {
                        fs::copy(src, &packages_file).ok();
                    }
                }
                if !packages_file.is_file() {
                    if self.host_profile_kind == HostProfileKind::Server {
                        fs::write(
                            &packages_file,
                            r#"# 用户个人软件入口（服务器最小模板）
{ pkgs, ... }:

{
  home.packages = with pkgs; [
    # tmux
    # htop
    # rsync
  ];
}
"#,
                        )?;
                    } else {
                        fs::write(
                            &packages_file,
                            r#"# 用户个人软件入口（按需启用，不影响其他用户可见性）
{ pkgs, ... }:

{
  mcb.desktopEntries = {
    enableZed = false;
    enableYesPlayMusic = false;
  };

  # 逐个声明该用户的软件（仅此用户可见）
  home.packages = with pkgs; [
    # firefox
    # helix
    # (callPackage ../../../pkgs/zed { })            # 同时把 enableZed 改为 true
    # (callPackage ../../../pkgs/yesplaymusic { })   # 同时把 enableYesPlayMusic 改为 true
  ];
}
"#,
                        )?;
                    }
                }
            }

            let local_example = user_dir.join("local.nix.example");
            if !local_example.is_file() {
                fs::write(
                    &local_example,
                    r#"# 用户私有覆盖示例（按需复制为 local.nix）
{ ... }:

{
  # 仅当前用户生效的个性化开关示例：
  # home.packages = with pkgs; [ localsend ];
}
"#,
                )?;
            }

            if !create_default {
                continue;
            }

            let mut import_lines = vec![format!("    {profile_import}")];
            for extra in &extra_imports {
                if user_dir.join(extra).is_file() {
                    import_lines.push(format!("    {extra}"));
                }
            }
            if include_user_files {
                if user_dir.join("files.nix").is_file() {
                    import_lines.push("    ./files.nix".to_string());
                }
                if user_dir.join("scripts.nix").is_file() {
                    import_lines.push("    ./scripts.nix".to_string());
                }
            }
            let content = format!(
                r#"{{
  lib, ...
}}:

let
  user = "{user}";
in
{{
  imports = [
{}
  ] ++ lib.optional (builtins.pathExists ./local.nix) ./local.nix;

  home.username = user;
  home.homeDirectory = "/home/${{user}}";
  home.stateVersion = "25.11";

  programs.home-manager.enable = true;
  xdg.enable = true;
}}
"#,
                import_lines.join("\n")
            );
            fs::write(&user_file, content)?;
            self.created_home_users.push(user.clone());
            self.warn(&format!(
                "已为新用户自动生成 Home Manager 入口：home/users/{user}/default.nix"
            ));
        }
        Ok(())
    }

    fn preserve_existing_local_override(&self, repo_dir: &Path) -> Result<()> {
        if self.deploy_mode != DeployMode::UpdateExisting {
            return Ok(());
        }
        if self.target_name.is_empty() {
            return Ok(());
        }
        let src = self
            .etc_dir
            .join("hosts")
            .join(&self.target_name)
            .join("local.nix");
        let dst = repo_dir
            .join("hosts")
            .join(&self.target_name)
            .join("local.nix");
        if src.is_file() {
            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&src, &dst).with_context(|| {
                format!(
                    "仅更新模式：复制现有 local.nix 失败：{} -> {}",
                    src.display(),
                    dst.display()
                )
            })?;
            self.note(&format!(
                "仅更新模式：已保留现有 hosts/{}/local.nix",
                self.target_name
            ));
        } else {
            self.note("仅更新模式：未发现现有 hosts/<host>/local.nix，将按仓库默认配置更新。");
        }
        Ok(())
    }

    fn write_local_override(&mut self, repo_dir: &Path) -> Result<()> {
        if self.target_users.is_empty() {
            return Ok(());
        }
        let host_dir = repo_dir.join("hosts").join(&self.target_name);
        if !host_dir.is_dir() {
            bail!("主机目录不存在：{}", host_dir.display());
        }
        let file = host_dir.join("local.nix");

        if self.target_admin_users.is_empty() {
            self.target_admin_users = vec![self.target_users[0].clone()];
        }
        let primary = &self.target_users[0];
        let users_list = self
            .target_users
            .iter()
            .map(|u| format!(" \"{u}\""))
            .collect::<String>();
        let admins_list = self
            .target_admin_users
            .iter()
            .map(|u| format!(" \"{u}\""))
            .collect::<String>();

        let mut out = String::new();
        out.push_str("{ lib, ... }:\n\n{\n");
        out.push_str(&format!("  mcb.user = lib.mkForce \"{primary}\";\n"));
        out.push_str(&format!("  mcb.users = lib.mkForce [{users_list} ];\n"));
        out.push_str(&format!(
            "  mcb.adminUsers = lib.mkForce [{admins_list} ];\n"
        ));

        if self.per_user_tun_enabled && !self.user_tun.is_empty() {
            out.push_str("  mcb.perUserTun.interfaces = lib.mkForce {\n");
            for user in &self.target_users {
                if let Some(v) = self.user_tun.get(user) {
                    out.push_str(&format!("    {user} = \"{v}\";\n"));
                }
            }
            out.push_str("  };\n");
            out.push_str("  mcb.perUserTun.dnsPorts = lib.mkForce {\n");
            for user in &self.target_users {
                if let Some(v) = self.user_dns.get(user) {
                    out.push_str(&format!("    {user} = {v};\n"));
                }
            }
            out.push_str("  };\n");
        }

        if self.gpu_override {
            out.push_str(&format!(
                "  mcb.hardware.gpu.mode = lib.mkForce \"{}\";\n",
                self.gpu_mode
            ));
            if !self.gpu_igpu_vendor.is_empty() {
                out.push_str(&format!(
                    "  mcb.hardware.gpu.igpuVendor = lib.mkForce \"{}\";\n",
                    self.gpu_igpu_vendor
                ));
            }
            if !self.gpu_nvidia_open.is_empty() {
                out.push_str(&format!(
                    "  mcb.hardware.gpu.nvidia.open = lib.mkForce {};\n",
                    self.gpu_nvidia_open
                ));
            }
            if !self.gpu_prime_mode.is_empty()
                || !self.gpu_intel_bus.is_empty()
                || !self.gpu_amd_bus.is_empty()
                || !self.gpu_nvidia_bus.is_empty()
            {
                out.push_str("  mcb.hardware.gpu.prime = lib.mkForce {\n");
                if !self.gpu_prime_mode.is_empty() {
                    out.push_str(&format!("    mode = \"{}\";\n", self.gpu_prime_mode));
                }
                if !self.gpu_intel_bus.is_empty() {
                    out.push_str(&format!("    intelBusId = \"{}\";\n", self.gpu_intel_bus));
                }
                if !self.gpu_amd_bus.is_empty() {
                    out.push_str(&format!("    amdgpuBusId = \"{}\";\n", self.gpu_amd_bus));
                }
                if !self.gpu_nvidia_bus.is_empty() {
                    out.push_str(&format!("    nvidiaBusId = \"{}\";\n", self.gpu_nvidia_bus));
                }
                out.push_str("  };\n");
            }
            if self.gpu_specialisations_set {
                out.push_str(&format!(
                    "  mcb.hardware.gpu.specialisations.enable = lib.mkForce {};\n",
                    self.gpu_specialisations_enabled
                ));
                if self.gpu_specialisations_enabled && !self.gpu_specialisation_modes.is_empty() {
                    let mode_list = self
                        .gpu_specialisation_modes
                        .iter()
                        .map(|m| format!(" \"{m}\""))
                        .collect::<String>();
                    out.push_str(&format!(
                        "  mcb.hardware.gpu.specialisations.modes = lib.mkForce [{mode_list} ];\n"
                    ));
                }
            }
        }

        if self.server_overrides_enabled {
            out.push_str(&format!(
                "  mcb.packages.enableNetworkCli = lib.mkForce {};\n",
                self.server_enable_network_cli
            ));
            out.push_str(&format!(
                "  mcb.packages.enableNetworkGui = lib.mkForce {};\n",
                self.server_enable_network_gui
            ));
            out.push_str(&format!(
                "  mcb.packages.enableShellTools = lib.mkForce {};\n",
                self.server_enable_shell_tools
            ));
            out.push_str(&format!(
                "  mcb.packages.enableWaylandTools = lib.mkForce {};\n",
                self.server_enable_wayland_tools
            ));
            out.push_str(&format!(
                "  mcb.packages.enableSystemTools = lib.mkForce {};\n",
                self.server_enable_system_tools
            ));
            out.push_str(&format!(
                "  mcb.packages.enableGeekTools = lib.mkForce {};\n",
                self.server_enable_geek_tools
            ));
            out.push_str(&format!(
                "  mcb.packages.enableGaming = lib.mkForce {};\n",
                self.server_enable_gaming
            ));
            out.push_str(&format!(
                "  mcb.packages.enableInsecureTools = lib.mkForce {};\n",
                self.server_enable_insecure_tools
            ));
            out.push_str(&format!(
                "  mcb.virtualisation.docker.enable = lib.mkForce {};\n",
                self.server_enable_docker
            ));
            out.push_str(&format!(
                "  mcb.virtualisation.libvirtd.enable = lib.mkForce {};\n",
                self.server_enable_libvirtd
            ));
        }
        out.push_str("}\n");
        fs::write(file, out)?;
        Ok(())
    }

    fn backup_etc(&self) -> Result<()> {
        let ts = run_capture_allow_fail("date", &["+%Y%m%d-%H%M%S"])
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let backup_dir = PathBuf::from(format!("{}.backup-{ts}", self.etc_dir.display()));
        self.log(&format!(
            "备份 {} -> {}",
            self.etc_dir.display(),
            backup_dir.display()
        ));
        self.run_as_root_ok(
            "mkdir",
            &["-p".to_string(), backup_dir.display().to_string()],
        )?;
        if command_exists("rsync") {
            self.run_as_root_ok(
                "rsync",
                &[
                    "-a".to_string(),
                    format!("{}/", self.etc_dir.display()),
                    format!("{}/", backup_dir.display()),
                ],
            )?;
        } else {
            self.run_as_root_ok(
                "cp",
                &[
                    "-a".to_string(),
                    format!("{}/.", self.etc_dir.display()),
                    backup_dir.display().to_string(),
                ],
            )?;
        }
        self.success("备份完成");
        Ok(())
    }

    fn prepare_etc_dir(&mut self) -> Result<()> {
        let has_content = self.etc_dir.is_dir()
            && fs::read_dir(&self.etc_dir)
                .ok()
                .and_then(|mut it| it.next())
                .is_some();
        if !has_content {
            return Ok(());
        }
        match self.overwrite_mode {
            OverwriteMode::Backup => self.backup_etc()?,
            OverwriteMode::Overwrite => {
                self.note(&format!("将覆盖 {}（未启用备份）", self.etc_dir.display()));
            }
            OverwriteMode::Ask => {
                if self.is_tty() {
                    loop {
                        print!(
                            "检测到 {} 已存在，选择 [b]备份并覆盖/[o]直接覆盖/[q]退出（默认 b）： ",
                            self.etc_dir.display()
                        );
                        io::stdout().flush().ok();
                        let mut ans = String::new();
                        io::stdin().read_line(&mut ans).ok();
                        let ans = ans.trim();
                        if ans.eq_ignore_ascii_case("q") {
                            bail!("已退出");
                        } else if ans.eq_ignore_ascii_case("o") {
                            self.overwrite_mode = OverwriteMode::Overwrite;
                            break;
                        } else if ans.is_empty() || ans.eq_ignore_ascii_case("b") {
                            self.backup_etc()?;
                            self.overwrite_mode = OverwriteMode::Backup;
                            break;
                        } else {
                            println!("无效选择，请重试。");
                        }
                    }
                } else {
                    self.backup_etc()?;
                    self.overwrite_mode = OverwriteMode::Backup;
                }
            }
        }
        Ok(())
    }

    fn clean_etc_dir_keep_hardware(&self) -> Result<()> {
        if self.etc_dir.as_os_str().is_empty() || self.etc_dir == Path::new("/") {
            bail!("ETC_DIR 无效，拒绝清理：{}", self.etc_dir.display());
        }
        if !self.etc_dir.is_dir() {
            return Ok(());
        }

        let preserve = create_temp_dir("run-rs-preserve")?;
        let etc_hw = self.etc_dir.join("hardware-configuration.nix");
        if etc_hw.is_file() {
            fs::create_dir_all(&preserve)?;
            self.run_as_root_ok(
                "cp",
                &[
                    "-a".to_string(),
                    etc_hw.display().to_string(),
                    preserve.display().to_string(),
                ],
            )?;
        }
        let etc_hosts = self.etc_dir.join("hosts");
        if etc_hosts.is_dir() {
            for entry in WalkDir::new(&etc_hosts)
                .max_depth(3)
                .into_iter()
                .flatten()
                .filter(|e| {
                    e.file_type().is_file() && e.file_name() == "hardware-configuration.nix"
                })
            {
                let file = entry.path();
                let rel = file.strip_prefix(&self.etc_dir).unwrap_or(file);
                let dst = preserve.join(rel);
                if let Some(parent) = dst.parent() {
                    fs::create_dir_all(parent).ok();
                }
                self.run_as_root_ok(
                    "cp",
                    &[
                        "-a".to_string(),
                        file.display().to_string(),
                        dst.display().to_string(),
                    ],
                )?;
            }
        }

        self.run_as_root_ok(
            "find",
            &[
                self.etc_dir.display().to_string(),
                "-mindepth".to_string(),
                "1".to_string(),
                "-maxdepth".to_string(),
                "1".to_string(),
                "-exec".to_string(),
                "rm".to_string(),
                "-rf".to_string(),
                "{}".to_string(),
                "+".to_string(),
            ],
        )?;

        let preserved_root = preserve.join("hardware-configuration.nix");
        if preserved_root.is_file() {
            self.run_as_root_ok(
                "cp",
                &[
                    "-a".to_string(),
                    preserved_root.display().to_string(),
                    self.etc_dir.display().to_string(),
                ],
            )?;
        }

        let preserved_hosts = preserve.join("hosts");
        if preserved_hosts.is_dir() {
            self.run_as_root_ok(
                "mkdir",
                &[
                    "-p".to_string(),
                    self.etc_dir.join("hosts").display().to_string(),
                ],
            )?;
            self.run_as_root_ok(
                "cp",
                &[
                    "-a".to_string(),
                    format!("{}/.", preserved_hosts.display()),
                    self.etc_dir.join("hosts").display().to_string(),
                ],
            )?;
        }
        fs::remove_dir_all(preserve).ok();
        Ok(())
    }

    fn sync_repo_to_etc(&self, repo_dir: &Path) -> Result<()> {
        self.log(&format!("同步到 {}", self.etc_dir.display()));
        self.run_as_root_ok(
            "mkdir",
            &["-p".to_string(), self.etc_dir.display().to_string()],
        )?;

        let delete = matches!(
            self.overwrite_mode,
            OverwriteMode::Overwrite | OverwriteMode::Backup
        );
        if command_exists("rsync") {
            let mut args = vec!["-a".to_string()];
            if delete {
                args.push("--delete".to_string());
            }
            args.extend([
                "--exclude".to_string(),
                ".git/".to_string(),
                "--exclude".to_string(),
                "hardware-configuration.nix".to_string(),
                "--exclude".to_string(),
                "hosts/*/hardware-configuration.nix".to_string(),
                format!("{}/", repo_dir.display()),
                format!("{}/", self.etc_dir.display()),
            ]);
            self.run_as_root_ok("rsync", &args)?;
        } else {
            if delete {
                self.clean_etc_dir_keep_hardware()?;
            }
            let tar_file = create_temp_path("run-rs-sync", "tar")?;
            let args = vec![
                "-C".to_string(),
                repo_dir.display().to_string(),
                "--exclude=.git".to_string(),
                "--exclude=hardware-configuration.nix".to_string(),
                "--exclude=hosts/*/hardware-configuration.nix".to_string(),
                "-cf".to_string(),
                tar_file.display().to_string(),
                ".".to_string(),
            ];
            let st = Self::run_status_inherit("tar", &args)?;
            if !st.success() {
                bail!("打包同步内容失败");
            }
            self.run_as_root_ok(
                "tar",
                &[
                    "-C".to_string(),
                    self.etc_dir.display().to_string(),
                    "-xf".to_string(),
                    tar_file.display().to_string(),
                ],
            )?;
            fs::remove_file(tar_file).ok();
        }
        self.success("配置同步完成");
        Ok(())
    }

    fn rebuild_system(&self) -> Result<bool> {
        self.log(&format!(
            "重建系统（{}），目标：{}",
            self.mode, self.target_name
        ));
        let mut nix_config = "experimental-features = nix-command flakes".to_string();
        if let Ok(extra) = std::env::var("NIX_CONFIG")
            && !extra.trim().is_empty()
        {
            nix_config = format!("{extra}\n{nix_config}");
        }

        let mut rebuild_args = vec![self.mode.clone(), "--show-trace".to_string()];
        if self.rebuild_upgrade {
            rebuild_args.push("--upgrade".to_string());
        }
        rebuild_args.push("--flake".to_string());
        rebuild_args.push(format!("{}#{}", self.etc_dir.display(), self.target_name));

        let status = if self.sudo_cmd.is_some() {
            let mut cmd = Command::new("sudo");
            cmd.arg("-E")
                .arg("env")
                .arg(format!("NIX_CONFIG={nix_config}"))
                .arg("nixos-rebuild")
                .args(&rebuild_args)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit());
            cmd.status()?
        } else {
            let mut cmd = Command::new("env");
            cmd.arg(format!("NIX_CONFIG={nix_config}"))
                .arg("nixos-rebuild")
                .args(&rebuild_args)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit());
            cmd.status()?
        };

        if status.success() {
            self.success("系统重建完成");
            Ok(true)
        } else {
            self.warn("系统重建失败");
            Ok(false)
        }
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

        let tmp_dir = create_temp_dir("run-rs-source")?;
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
                self.ensure_user_home_entries(&tmp_dir)?;
                if !self.created_home_users.is_empty() {
                    self.warn(&format!(
                        "已自动创建用户 Home Manager 模板：{}",
                        self.created_home_users.join(" ")
                    ));
                }
                self.write_local_override(&tmp_dir)?;
            }
            self.ensure_host_hardware_config()?;
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

    fn default_release_version(&self) -> String {
        let today = run_capture_allow_fail("date", &["+%Y.%m.%d"])
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "1970.01.01".to_string());
        let base = format!("v{today}");
        let mut max = -1i64;
        let out = run_capture_allow_fail("git", &["tag", "--list", &base, &format!("{base}.*")])
            .unwrap_or_default();
        for tag in out.lines() {
            if tag == base {
                max = 0;
            } else if let Some(sfx) = tag.strip_prefix(&(base.clone() + "."))
                && let Ok(num) = sfx.parse::<i64>()
                && num > max
            {
                max = num;
            }
        }
        if max >= 0 {
            format!("{base}.{}", max + 1)
        } else {
            base
        }
    }

    fn resolve_release_version(&self) -> Result<String> {
        let mut version = std::env::var("RELEASE_VERSION").unwrap_or_default();
        let default_version = self.default_release_version();
        if version.is_empty() && self.is_tty() {
            print!("请输入发布版本（默认 {default_version}）： ");
            io::stdout().flush().ok();
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            version = input.trim().to_string();
        }
        if version.is_empty() {
            version = default_version;
        }
        if !version.starts_with('v') {
            version = format!("v{version}");
        }
        Ok(version)
    }

    fn find_last_release_tag(&self) -> String {
        run_capture_allow_fail("git", &["describe", "--tags", "--abbrev=0"])
            .map(|s| s.trim().to_string())
            .unwrap_or_default()
    }

    fn generate_release_notes(&self, last_tag: &str) -> String {
        let range = if last_tag.is_empty() {
            "HEAD".to_string()
        } else {
            format!("{last_tag}..HEAD")
        };
        let out = run_capture_allow_fail("git", &["log", "--oneline", "--no-merges", &range])
            .unwrap_or_default();
        let lines: Vec<&str> = out.lines().collect();
        if lines.is_empty() {
            if last_tag.is_empty() {
                "No code changes found.".to_string()
            } else {
                format!("No code changes since {last_tag}.")
            }
        } else {
            let header = if last_tag.is_empty() {
                "Changes".to_string()
            } else {
                format!("Changes since {last_tag}")
            };
            let mut notes = format!("## {header}\n");
            for line in lines {
                notes.push_str(&format!("- {line}\n"));
            }
            notes
        }
    }

    fn update_version_files(&self, version: &str) -> Result<()> {
        fs::write(self.script_dir.join("VERSION"), format!("{version}\n"))?;
        Ok(())
    }

    fn release_flow(&mut self) -> Result<()> {
        self.banner();
        if !command_exists("git") {
            bail!("未找到 git。");
        }
        if !command_exists("gh") {
            bail!("未找到 GitHub CLI (gh)。");
        }
        let auth = Command::new("gh").args(["auth", "status"]).status()?;
        if !auth.success() {
            bail!("gh 未登录，请先执行 gh auth login。");
        }

        std::env::set_current_dir(&self.script_dir)
            .with_context(|| format!("无法进入仓库目录：{}", self.script_dir.display()))?;
        if !self.script_dir.join(".git").is_dir() {
            bail!("当前目录不是 git 仓库：{}", self.script_dir.display());
        }

        let dirty = run_capture_allow_fail("git", &["status", "--porcelain"]).unwrap_or_default();
        let allow_dirty = std::env::var("RELEASE_ALLOW_DIRTY")
            .ok()
            .is_some_and(|v| v == "true");
        if !dirty.trim().is_empty() && !allow_dirty {
            bail!("工作区存在未提交变更，发布前请先提交或设置 RELEASE_ALLOW_DIRTY=true。");
        }

        let version = self.resolve_release_version()?;
        let exists = Command::new("git").args(["rev-parse", &version]).status()?;
        if exists.success() {
            bail!("标签已存在：{version}");
        }

        let last_tag = self.find_last_release_tag();
        let mut notes = std::env::var("RELEASE_NOTES").unwrap_or_default();
        if notes.is_empty() {
            notes = self.generate_release_notes(&last_tag);
        }

        if self.is_tty() {
            println!("\n将发布版本：{version}");
            if !last_tag.is_empty() {
                println!("上一个版本：{last_tag}");
            }
            println!("\nRelease Notes 预览：\n{notes}\n");
            self.confirm_continue(&format!("确认发布 {version}？"))?;
        }

        self.update_version_files(&version)?;
        let add = Command::new("git").args(["add", "VERSION"]).status()?;
        if !add.success() {
            bail!("git add 失败");
        }
        let cached = Command::new("git")
            .args(["diff", "--cached", "--quiet"])
            .status()?;
        if !cached.success() {
            let commit = Command::new("git")
                .args(["commit", "-m", &format!("release: {version}")])
                .status()?;
            if !commit.success() {
                bail!("版本提交失败");
            }
        } else {
            self.warn("VERSION 未变化，跳过版本提交。");
        }

        for args in [
            vec!["tag", "-a", &version, "-m", &version],
            vec!["push", "origin", "HEAD"],
            vec!["push", "origin", &version],
        ] {
            let st = Command::new("git").args(&args).status()?;
            if !st.success() {
                bail!("git {} 失败", args.join(" "));
            }
        }

        let notes_file = create_temp_path("run-rs-release-notes", "md")?;
        fs::write(&notes_file, notes)?;
        let st = Command::new("gh")
            .args([
                "release",
                "create",
                &version,
                "--title",
                &version,
                "--notes-file",
                &notes_file.display().to_string(),
            ])
            .status()?;
        fs::remove_file(notes_file).ok();
        if !st.success() {
            bail!("gh release create 失败");
        }
        self.success(&format!("Release 已发布：{version}"));
        Ok(())
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
    let probe = path.join(format!(".run-rs-write-{}", std::process::id()));
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

fn detect_script_dir() -> Result<PathBuf> {
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
    bail!("run-rs: cannot locate repository root");
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

fn main() {
    let mut app = match App::new() {
        Ok(v) => v,
        Err(err) => {
            eprintln!("run-rs: {err:#}");
            std::process::exit(1);
        }
    };
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(err) = app.parse_args(&args).and_then(|_| app.run()) {
        eprintln!("run-rs: {err:#}");
        std::process::exit(1);
    }
}
