use super::*;

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

    pub(crate) fn should_require_hardware_config(&self) -> bool {
        !(self.rootless && self.mode == "build")
    }

    pub(crate) fn root_hardware_config_path(&self) -> PathBuf {
        self.etc_dir.join("hardware-configuration.nix")
    }

    pub(crate) fn ensure_root_hardware_config(&self) -> Result<()> {
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

    pub(crate) fn self_check_repo(&self, repo_dir: &Path) -> Result<()> {
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

    pub(crate) fn temp_dns_enable(&mut self) -> Result<bool> {
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

    pub(crate) fn temp_dns_disable(&mut self) {
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

    pub(crate) fn deploy_flow(&mut self) -> Result<()> {
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

    pub(crate) fn run(&mut self) -> Result<()> {
        match self.run_action {
            RunAction::Deploy => self.deploy_flow(),
            RunAction::Release => self.release_flow(),
        }
    }
}
