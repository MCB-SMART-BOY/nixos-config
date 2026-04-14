use super::*;

fn directory_has_entries(path: &Path) -> Result<bool> {
    if !path.is_dir() {
        return Ok(false);
    }

    let mut entries = fs::read_dir(path)
        .with_context(|| format!("failed to read directory {}", path.display()))?;
    Ok(entries.next().is_some())
}

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
        let has_content = directory_has_entries(&self.etc_dir)?;
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
                        let ans = self.prompt_line(&format!(
                            "检测到 {} 已存在，选择 [b]备份并覆盖/[o]直接覆盖/[q]退出（默认 b）： ",
                            self.etc_dir.display()
                        ))?;
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
        fs::remove_dir_all(&preserve)
            .with_context(|| format!("failed to remove {}", preserve.display()))?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn directory_has_entries_detects_empty_and_non_empty_dirs() -> Result<()> {
        let root = create_temp_dir("mcbctl-deploy-execute")?;
        let empty = root.join("empty");
        let filled = root.join("filled");
        fs::create_dir_all(&empty)?;
        fs::create_dir_all(&filled)?;
        fs::write(filled.join("note.txt"), "payload")?;

        assert!(!directory_has_entries(&empty)?);
        assert!(directory_has_entries(&filled)?);
        assert!(!directory_has_entries(&root.join("missing"))?);

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn directory_has_entries_reports_unreadable_directory() -> Result<()> {
        use std::os::unix::fs::PermissionsExt;

        if run_capture_allow_fail("id", &["-u"]).is_some_and(|uid| uid.trim() == "0") {
            return Ok(());
        }

        let root = create_temp_dir("mcbctl-deploy-execute-unreadable")?;
        let unreadable = root.join("private");
        fs::create_dir_all(&unreadable)?;
        fs::write(unreadable.join("note.txt"), "payload")?;
        fs::set_permissions(&unreadable, fs::Permissions::from_mode(0o000))?;

        let err =
            directory_has_entries(&unreadable).expect_err("unreadable directory should error");
        assert!(err.to_string().contains("failed to read directory"));

        fs::set_permissions(&unreadable, fs::Permissions::from_mode(0o755))?;
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn prepare_etc_dir_backs_up_non_empty_directory_in_non_tty_mode() -> Result<()> {
        let root = create_temp_dir("mcbctl-deploy-execute-backup")?;
        let etc_dir = root.join("etc");
        fs::create_dir_all(&etc_dir)?;
        fs::write(etc_dir.join("flake.nix"), "payload")?;

        let mut app = test_app(root.clone());
        app.etc_dir = etc_dir.clone();
        app.sudo_cmd = None;
        app.overwrite_mode = OverwriteMode::Ask;
        app.overwrite_mode_set = true;

        app.prepare_etc_dir()?;

        assert_eq!(app.overwrite_mode, OverwriteMode::Backup);
        let backup_parent = etc_dir
            .parent()
            .context("etc test dir should have parent")?;
        let backup_name = format!(
            "{}.backup-",
            etc_dir.file_name().unwrap_or_default().to_string_lossy()
        );
        let backup_dir = fs::read_dir(backup_parent)?
            .flatten()
            .map(|entry| entry.path())
            .find(|path| {
                path.file_name()
                    .is_some_and(|name| name.to_string_lossy().starts_with(&backup_name))
            })
            .context("backup directory should be created")?;
        assert_eq!(fs::read_to_string(backup_dir.join("flake.nix"))?, "payload");

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn clean_etc_dir_keep_hardware_preserves_target_hardware_only() -> Result<()> {
        let root = create_temp_dir("mcbctl-deploy-execute-clean")?;
        let etc_dir = root.join("etc");
        let host_dir = etc_dir.join("hosts/demo");
        fs::create_dir_all(&host_dir)?;
        fs::write(
            host_dir.join("hardware-configuration.nix"),
            "{ lib, ... }: { }\n",
        )?;
        fs::write(etc_dir.join("flake.nix"), "payload")?;
        fs::write(host_dir.join("default.nix"), "stale")?;

        let mut app = test_app(root.clone());
        app.etc_dir = etc_dir.clone();
        app.sudo_cmd = None;
        app.target_name = "demo".to_string();

        app.clean_etc_dir_keep_hardware()?;

        assert!(!etc_dir.join("flake.nix").exists());
        assert!(!host_dir.join("default.nix").exists());
        assert_eq!(
            fs::read_to_string(host_dir.join("hardware-configuration.nix"))?,
            "{ lib, ... }: { }\n"
        );

        fs::remove_dir_all(root)?;
        Ok(())
    }

    fn create_temp_dir(prefix: &str) -> Result<PathBuf> {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let root = std::env::temp_dir().join(format!("{prefix}-{}-{unique}", std::process::id()));
        fs::create_dir_all(&root)?;
        Ok(root)
    }

    fn test_app(repo_dir: PathBuf) -> App {
        App {
            repo_dir,
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
