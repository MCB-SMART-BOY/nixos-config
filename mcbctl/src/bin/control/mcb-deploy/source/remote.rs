use super::*;

impl App {
    fn probe_remote_source_commit(&mut self, repo_dir: &Path) {
        self.source_commit.clear();
        let repo_dir_display = repo_dir.display().to_string();
        match Self::command_output("git", &["-C", &repo_dir_display, "rev-parse", "HEAD"]) {
            Ok(out) => match source_commit_from_probe(
                out.status.success(),
                &String::from_utf8_lossy(&out.stdout),
                &String::from_utf8_lossy(&out.stderr),
            ) {
                Ok(commit) => self.source_commit = commit,
                Err(err) => self.warn(&format!("远端源提交探测失败：{err}")),
            },
            Err(err) => self.warn(&format!("远端源提交探测失败：{err}")),
        }
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

    fn clone_repo(&mut self, tmp_dir: &Path, url: &str) -> Result<()> {
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
                    self.probe_remote_source_commit(tmp_dir);
                    self.success(&clone_success_message(&self.source_commit));
                    return Ok(());
                }
                bail!(
                    "已拉取仓库，但 checkout 失败：{url}（ref: {}，exit={})",
                    self.source_ref,
                    checkout.code().unwrap_or(1)
                );
            }
            if status.code() == Some(124) {
                bail!(
                    "仓库拉取超时：{url}（ref: {}，{}s）",
                    self.source_ref,
                    timeout_s
                );
            }
            bail!(
                "仓库拉取失败：{url}（ref: {}，exit={})",
                self.source_ref,
                status.code().unwrap_or(1)
            );
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
            self.probe_remote_source_commit(tmp_dir);
            self.success(&clone_success_message(&self.source_commit));
            return Ok(());
        }
        if status.code() == Some(124) {
            bail!(
                "仓库拉取超时：{url}（branch: {}，{}s）",
                self.branch,
                timeout_s
            );
        }
        bail!(
            "仓库拉取失败：{url}（branch: {}，exit={})",
            self.branch,
            status.code().unwrap_or(1)
        )
    }

    fn clone_repo_any(&mut self, tmp_dir: &Path) -> Result<()> {
        self.source_commit.clear();
        let mut failures = Vec::new();
        for (idx, url) in self.repo_urls.clone().iter().enumerate() {
            self.note(&format!(
                "尝试镜像 ({}/{})：{}",
                idx + 1,
                self.repo_urls.len(),
                url
            ));
            reset_source_workspace(tmp_dir)?;
            match self.clone_repo(tmp_dir, url) {
                Ok(()) => return Ok(()),
                Err(err) => {
                    let detail = err.to_string();
                    self.warn(&detail);
                    failures.push((url.clone(), detail));
                }
            }
        }
        bail!(
            "{}",
            summarize_source_failures("所有镜像都失败了", &failures)
        )
    }

    fn clone_repo_any_with_dns_retry(&mut self, tmp_dir: &Path) -> Result<()> {
        let first_error = match self.clone_repo_any(tmp_dir) {
            Ok(()) => return Ok(()),
            Err(err) => err.to_string(),
        };
        self.warn(&first_error);
        self.log("尝试临时切换阿里云 DNS 后重试");
        if !self.temp_dns_enable()? {
            self.warn("临时 DNS 设置失败，将继续使用当前 DNS 再重试一次。");
        }
        let retry_error = match self.clone_repo_any(tmp_dir) {
            Ok(()) => return Ok(()),
            Err(err) => err.to_string(),
        };
        bail!(
            "{}",
            summarize_source_failures(
                "仓库拉取失败",
                &[
                    ("首次尝试".to_string(), first_error),
                    ("DNS 重试".to_string(), retry_error),
                ],
            )
        )
    }

    pub(crate) fn prepare_source_repo(&mut self, tmp_dir: &Path) -> Result<()> {
        if self.force_remote_source {
            self.require_remote_source_pin()?;
            self.clone_repo_any_with_dns_retry(tmp_dir)?;
            return Ok(());
        }

        if let Some(source_dir) = self.detect_local_repo_dir() {
            self.prepare_local_source(tmp_dir, &source_dir)?;
            return Ok(());
        }

        self.require_remote_source_pin()?;
        self.clone_repo_any_with_dns_retry(tmp_dir)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn probe_remote_source_commit_clears_stale_commit_when_repo_is_not_git() -> Result<()> {
        let root = create_temp_dir("mcbctl-remote-source-probe-fail")?;
        let repo_dir = root.join("repo");
        fs::create_dir_all(&repo_dir)?;
        fs::write(repo_dir.join("flake.nix"), "{ }")?;

        let mut app = test_app(root);
        app.source_commit = "stale-commit".to_string();

        app.probe_remote_source_commit(&repo_dir);

        assert!(app.source_commit.is_empty());
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
