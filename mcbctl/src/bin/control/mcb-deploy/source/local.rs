use super::*;

fn summarize_cleanup_failures(context: &str, failures: &[String]) -> String {
    if failures.is_empty() {
        return context.to_string();
    }

    format!("{context}: {}", failures.join(" | "))
}

fn cleanup_local_source_archive(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    fs::remove_file(path)
        .with_context(|| format!("failed to remove local source archive {}", path.display()))
}

fn finalize_with_cleanup(
    primary_result: Result<()>,
    cleanup_result: Result<()>,
    cleanup_context: &str,
) -> Result<()> {
    match (primary_result, cleanup_result) {
        (Ok(()), Ok(())) => Ok(()),
        (Ok(()), Err(cleanup_err)) => {
            let failures = vec![cleanup_err.to_string()];
            bail!("{}", summarize_cleanup_failures(cleanup_context, &failures))
        }
        (Err(err), Ok(())) => Err(err),
        (Err(err), Err(cleanup_err)) => {
            let failures = vec![cleanup_err.to_string()];
            bail!(
                "{}",
                summarize_cleanup_failures(&err.to_string(), &failures)
            )
        }
    }
}

impl App {
    pub(crate) fn detect_local_repo_dir(&self) -> Option<PathBuf> {
        let cwd = std::env::current_dir().ok();
        let mut candidates = Vec::new();
        if let Some(c) = cwd {
            candidates.push(c);
        }
        candidates.push(self.repo_dir.clone());
        candidates.into_iter().find(|d| path_looks_repo(d))
    }

    pub(crate) fn prepare_local_source(&mut self, tmp_dir: &Path, source_dir: &Path) -> Result<()> {
        self.log(&format!("使用本地仓库：{}", source_dir.display()));
        reset_source_workspace(tmp_dir)?;
        self.source_commit.clear();

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
            let tar_file = create_temp_path("mcbctl-local-src", "tar")?;
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
            let extract_status = Self::run_status_inherit("tar", &args)?;
            let extract_result = if extract_status.success() {
                Ok(())
            } else {
                Err(anyhow::anyhow!("解包本地仓库失败"))
            };
            let cleanup_result = cleanup_local_source_archive(&tar_file);
            finalize_with_cleanup(
                extract_result,
                cleanup_result,
                "local source archive cleanup failed",
            )?;
        }

        if command_exists("git") {
            let source_dir_display = source_dir.display().to_string();
            match Self::command_output("git", &["-C", &source_dir_display, "rev-parse", "HEAD"]) {
                Ok(out) => match source_commit_from_probe(
                    out.status.success(),
                    &String::from_utf8_lossy(&out.stdout),
                    &String::from_utf8_lossy(&out.stderr),
                ) {
                    Ok(commit) => self.source_commit = commit,
                    Err(err) => self.warn(&format!("本地源提交探测失败：{err}")),
                },
                Err(err) => self.warn(&format!("本地源提交探测失败：{err}")),
            }
        }
        if !self.source_commit.is_empty() {
            self.note(&format!("本地源提交：{}", self.source_commit));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn prepare_local_source_clears_stale_source_commit_when_probe_fails() -> Result<()> {
        let root = create_temp_dir("mcbctl-local-source-probe-fail")?;
        let source_dir = root.join("source");
        let tmp_dir = root.join("workspace");
        fs::create_dir_all(&source_dir)?;
        fs::write(source_dir.join("flake.nix"), "{ }")?;
        fs::write(source_dir.join("README.md"), "demo")?;

        let mut app = test_app(root);
        app.source_commit = "stale-commit".to_string();

        app.prepare_local_source(&tmp_dir, &source_dir)?;

        assert!(app.source_commit.is_empty());
        assert_eq!(fs::read_to_string(tmp_dir.join("flake.nix"))?, "{ }");
        assert_eq!(fs::read_to_string(tmp_dir.join("README.md"))?, "demo");
        Ok(())
    }

    fn create_temp_dir(prefix: &str) -> Result<PathBuf> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
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
            etc_dir: PathBuf::from("/tmp/etc-nixos"),
            dns_enabled: false,
            temp_dns_backend: String::new(),
            temp_dns_backup: None,
            temp_dns_iface: String::new(),
            tmp_dir: None,
            sudo_cmd: None,
            rootless: false,
            run_action: RunAction::Deploy,
            progress_total: 7,
            progress_current: 0,
            git_clone_timeout_sec: 90,
        }
    }
}
