use super::super::*;
use mcbctl::repo::ensure_repository_integrity;

impl App {
    pub(crate) fn command_output(cmd: &str, args: &[&str]) -> Option<Output> {
        Command::new(cmd).args(args).output().ok()
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

        if self.sudo_cmd.is_some()
            && let Some(out) = Self::command_output("sudo", &["-n", "true"])
            && !out.status.success()
        {
            let stderr = String::from_utf8_lossy(&out.stderr).to_lowercase();
            if stderr.contains("no new privileges") {
                self.warn("sudo 无法提权（no new privileges），进入 rootless 模式。");
                self.sudo_cmd = None;
                self.rootless = true;
            } else {
                self.warn("sudo 需要交互输入密码，将在需要时提示。");
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
        if !self.is_tty() {
            self.rebuild_upgrade = false;
            return Ok(());
        }
        self.rebuild_upgrade = self.ask_bool("重建时升级上游依赖？", false)?;
        Ok(())
    }
}
