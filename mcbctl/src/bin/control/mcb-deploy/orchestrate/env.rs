use super::super::*;
use mcbctl::repo::ensure_repository_integrity;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SudoProbeStatus {
    Available,
    NeedsInteractiveAuth,
    RootlessNoNewPrivileges,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RepoCargoSelfCheckPlan {
    Run,
    SkipMissingCargo,
    SkipMissingManifest,
}

fn classify_sudo_probe(status_success: bool, stderr: &str) -> SudoProbeStatus {
    if status_success {
        return SudoProbeStatus::Available;
    }

    if stderr.to_lowercase().contains("no new privileges") {
        SudoProbeStatus::RootlessNoNewPrivileges
    } else {
        SudoProbeStatus::NeedsInteractiveAuth
    }
}

fn current_uid_from_probe(status_success: bool, stdout: &str, stderr: &str) -> Result<u32> {
    if !status_success {
        let stderr = stderr.trim();
        if stderr.is_empty() {
            bail!("id -u failed");
        }
        bail!("id -u failed: {stderr}");
    }

    stdout
        .trim()
        .parse::<u32>()
        .with_context(|| format!("id -u 输出无效：{:?}", stdout.trim()))
}

fn plan_repo_cargo_self_check(
    cargo_toml_exists: bool,
    cargo_available: bool,
) -> RepoCargoSelfCheckPlan {
    if !cargo_toml_exists {
        RepoCargoSelfCheckPlan::SkipMissingManifest
    } else if !cargo_available {
        RepoCargoSelfCheckPlan::SkipMissingCargo
    } else {
        RepoCargoSelfCheckPlan::Run
    }
}

fn ensure_repo_cargo_self_check_status(status_success: bool) -> Result<()> {
    if status_success {
        Ok(())
    } else {
        bail!("mcbctl cargo check 失败");
    }
}

impl App {
    fn probe_current_uid(&self) -> Result<Option<u32>> {
        if !command_exists("id") {
            return Ok(None);
        }

        let output = Self::command_output("id", &["-u"]).context("探测当前 uid 失败")?;
        current_uid_from_probe(
            output.status.success(),
            &String::from_utf8_lossy(&output.stdout),
            &String::from_utf8_lossy(&output.stderr),
        )
        .map(Some)
    }

    pub(crate) fn command_output(cmd: &str, args: &[&str]) -> Result<Output> {
        Command::new(cmd)
            .args(args)
            .output()
            .with_context(|| format!("failed to run {cmd}"))
    }

    pub(crate) fn run_status_inherit(cmd: &str, args: &[String]) -> Result<ExitStatus> {
        Command::new(cmd)
            .args(args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .with_context(|| format!("failed to run {cmd}"))
    }

    pub(crate) fn run_as_root_inherit(&self, cmd: &str, args: &[String]) -> Result<ExitStatus> {
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

    pub(crate) fn run_as_root_ok(&self, cmd: &str, args: &[String]) -> Result<()> {
        let status = self.run_as_root_inherit(cmd, args)?;
        if status.success() {
            Ok(())
        } else {
            bail!("{cmd} failed with {}", status.code().unwrap_or(1));
        }
    }

    pub(crate) fn check_env(&mut self) -> Result<()> {
        self.log("检查环境...");

        let is_root = match self.probe_current_uid() {
            Ok(Some(uid)) => uid == 0,
            Ok(None) => {
                self.warn("未找到 id，按非 root 环境处理。");
                false
            }
            Err(err) => {
                self.warn(&format!("当前 uid 探测失败：{err}；按非 root 环境处理。"));
                false
            }
        };
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
            let out =
                Self::command_output("sudo", &["-n", "true"]).context("检查 sudo 提权能力失败")?;
            match classify_sudo_probe(out.status.success(), &String::from_utf8_lossy(&out.stderr)) {
                SudoProbeStatus::Available => {}
                SudoProbeStatus::NeedsInteractiveAuth => {
                    self.warn("sudo 需要交互输入密码，将在需要时提示。");
                }
                SudoProbeStatus::RootlessNoNewPrivileges => {
                    self.warn("sudo 无法提权（no new privileges），进入 rootless 模式。");
                    self.sudo_cmd = None;
                    self.rootless = true;
                }
            }
        }

        if self.rootless {
            if !can_write_dir(&self.etc_dir) {
                let alt_dir = home_dir().join(".nixos");
                if self.is_tty() {
                    let ans = self.prompt_line(&format!(
                        "无权限写入 {}，改用 {}？ [Y/n] ",
                        self.etc_dir.display(),
                        alt_dir.display()
                    ))?;
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

        if !self.should_require_hardware_config() {
            self.note("rootless + build 模式：跳过硬件配置强制检查（仅构建/评估）。");
        }
        Ok(())
    }

    pub(crate) fn should_require_hardware_config(&self) -> bool {
        !(self.rootless && self.mode == "build")
    }

    pub(crate) fn target_hardware_config_path(&self) -> Result<PathBuf> {
        if self.target_name.trim().is_empty() {
            bail!("目标主机尚未确定，无法定位 hosts/<host>/hardware-configuration.nix");
        }
        Ok(host_hardware_config_path(&self.etc_dir, &self.target_name))
    }

    pub(crate) fn ensure_target_hardware_config(&self) -> Result<()> {
        if !self.should_require_hardware_config() {
            return Ok(());
        }
        let target = self.target_hardware_config_path()?;
        if target.is_file() {
            return Ok(());
        }
        self.warn(&format!("未发现 {}，将自动生成。", target.display()));
        ensure_host_hardware_config(&self.etc_dir, &self.target_name, self.sudo_cmd.is_some())?;
        self.log(&format!("已生成 {}", target.display()));
        Ok(())
    }

    pub(crate) fn self_check_repo(&self, repo_dir: &Path) -> Result<()> {
        self.log("仓库自检...");
        ensure_repository_integrity(repo_dir)?;

        let cargo_toml = repo_dir.join("mcbctl/Cargo.toml");
        match plan_repo_cargo_self_check(cargo_toml.is_file(), command_exists("cargo")) {
            RepoCargoSelfCheckPlan::Run => {
                let status = Command::new("cargo")
                    .args(["check", "--quiet"])
                    .current_dir(repo_dir.join("mcbctl"))
                    .stdin(Stdio::null())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .status()
                    .context("failed to run cargo check for mcbctl")?;
                ensure_repo_cargo_self_check_status(status.success())?;
            }
            RepoCargoSelfCheckPlan::SkipMissingCargo => {
                self.warn("未检测到 cargo，跳过 mcbctl 编译自检。");
            }
            RepoCargoSelfCheckPlan::SkipMissingManifest => {
                self.warn("未找到 mcbctl/Cargo.toml，跳过 Rust 脚本编译自检。");
            }
        }

        self.success("仓库自检完成");
        Ok(())
    }

    pub(crate) fn set_deploy_mode_prompt(&mut self) -> Result<()> {
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

    pub(crate) fn prompt_overwrite_mode(&mut self) -> Result<()> {
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

    pub(crate) fn prompt_rebuild_upgrade(&mut self) -> Result<()> {
        if self.rebuild_upgrade_set {
            return Ok(());
        }
        if !self.is_tty() {
            self.rebuild_upgrade = false;
            return Ok(());
        }
        self.rebuild_upgrade = self.ask_bool("重建时升级上游依赖？", false)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_sudo_probe_accepts_success() {
        assert_eq!(classify_sudo_probe(true, ""), SudoProbeStatus::Available);
    }

    #[test]
    fn classify_sudo_probe_detects_no_new_privileges() {
        assert_eq!(
            classify_sudo_probe(false, "sudo: The \"no new privileges\" flag is set"),
            SudoProbeStatus::RootlessNoNewPrivileges
        );
    }

    #[test]
    fn classify_sudo_probe_treats_other_failures_as_interactive_auth() {
        assert_eq!(
            classify_sudo_probe(false, "sudo: a password is required"),
            SudoProbeStatus::NeedsInteractiveAuth
        );
        assert_eq!(
            classify_sudo_probe(false, ""),
            SudoProbeStatus::NeedsInteractiveAuth
        );
    }

    #[test]
    fn current_uid_from_probe_accepts_valid_uid_output() -> Result<()> {
        assert_eq!(current_uid_from_probe(true, "0\n", "")?, 0);
        assert_eq!(current_uid_from_probe(true, "1000\n", "")?, 1000);
        Ok(())
    }

    #[test]
    fn current_uid_from_probe_rejects_invalid_or_failed_output() {
        let invalid = current_uid_from_probe(true, "root", "")
            .expect_err("non-numeric uid output should fail");
        assert!(invalid.to_string().contains("输出无效"));

        let failed = current_uid_from_probe(false, "", "permission denied")
            .expect_err("non-zero exit should fail");
        assert!(failed.to_string().contains("permission denied"));
    }

    #[test]
    fn plan_repo_cargo_self_check_prefers_missing_manifest_over_missing_cargo() {
        assert_eq!(
            plan_repo_cargo_self_check(false, true),
            RepoCargoSelfCheckPlan::SkipMissingManifest
        );
        assert_eq!(
            plan_repo_cargo_self_check(false, false),
            RepoCargoSelfCheckPlan::SkipMissingManifest
        );
    }

    #[test]
    fn plan_repo_cargo_self_check_skips_only_when_cargo_is_missing() {
        assert_eq!(
            plan_repo_cargo_self_check(true, false),
            RepoCargoSelfCheckPlan::SkipMissingCargo
        );
        assert_eq!(
            plan_repo_cargo_self_check(true, true),
            RepoCargoSelfCheckPlan::Run
        );
    }

    #[test]
    fn ensure_repo_cargo_self_check_status_rejects_non_zero_exit() {
        ensure_repo_cargo_self_check_status(true).expect("successful cargo check should pass");

        let err = ensure_repo_cargo_self_check_status(false)
            .expect_err("non-zero cargo check should fail");
        assert!(err.to_string().contains("cargo check"));
    }
}
