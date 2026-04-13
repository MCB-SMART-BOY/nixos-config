use super::*;

impl App {
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

    pub(super) fn prepare_etc_dir(&mut self) -> Result<()> {
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

        let preserve = create_temp_dir("mcbctl-preserve")?;
        let etc_hw = host_hardware_config_path(&self.etc_dir, &self.target_name);
        if etc_hw.is_file() {
            let Some(parent) = preserved_hardware_parent(&preserve, &self.target_name) else {
                bail!("目标主机尚未确定，无法保留硬件配置");
            };
            fs::create_dir_all(&parent)?;
            self.run_as_root_ok(
                "cp",
                &[
                    "-a".to_string(),
                    etc_hw.display().to_string(),
                    parent.display().to_string(),
                ],
            )?;
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

        let preserved_root = preserve
            .join("hosts")
            .join(&self.target_name)
            .join("hardware-configuration.nix");
        if preserved_root.is_file() {
            if let Some(parent) = etc_hw.parent() {
                self.run_as_root_ok("mkdir", &["-p".to_string(), parent.display().to_string()])?;
            }
            self.run_as_root_ok(
                "cp",
                &[
                    "-a".to_string(),
                    preserved_root.display().to_string(),
                    etc_hw.display().to_string(),
                ],
            )?;
        }
        fs::remove_dir_all(preserve).ok();
        Ok(())
    }

    pub(super) fn sync_repo_to_etc(&self, repo_dir: &Path) -> Result<()> {
        self.log(&format!("同步到 {}", self.etc_dir.display()));
        let plan = self.repo_sync_plan(repo_dir);
        self.note(&format!("同步预览：{}", plan.command_preview()));
        run_repo_sync(
            &plan,
            |cmd, args| {
                let status = Self::run_status_inherit(cmd, args)?;
                if status.success() {
                    Ok(())
                } else {
                    bail!("{cmd} failed with {}", status.code().unwrap_or(1));
                }
            },
            |cmd, args| self.run_as_root_ok(cmd, args),
            || self.clean_etc_dir_keep_hardware(),
        )?;
        self.success("配置同步完成");
        Ok(())
    }

    pub(super) fn rebuild_system(&self) -> Result<bool> {
        let plan = self.rebuild_plan();
        self.log(&format!(
            "重建系统（{}），目标：{}",
            plan.action.label(),
            plan.target_host
        ));
        self.note(&format!(
            "命令预览：{}",
            plan.command_preview(self.sudo_cmd.is_some())
        ));

        let status = run_nixos_rebuild(&plan, self.sudo_cmd.is_some())?;

        if status.success() {
            self.success("系统重建完成");
            Ok(true)
        } else {
            self.warn("系统重建失败");
            Ok(false)
        }
    }
}

fn preserved_hardware_parent(root: &Path, host: &str) -> Option<PathBuf> {
    if host.trim().is_empty() {
        None
    } else {
        Some(root.join("hosts").join(host))
    }
}
