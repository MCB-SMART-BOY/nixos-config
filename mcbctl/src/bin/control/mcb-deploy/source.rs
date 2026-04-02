use super::*;

impl App {
    pub(super) fn prompt_source_strategy(&mut self) -> Result<()> {
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

    pub(super) fn validate_mode_conflicts(&self) -> Result<()> {
        if self.deploy_mode == DeployMode::UpdateExisting && !self.target_users.is_empty() {
            bail!("仅更新模式不允许修改用户列表；该模式会保留现有用户与权限。");
        }
        Ok(())
    }

    pub(super) fn require_remote_source_pin(&self) -> Result<()> {
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

    pub(super) fn detect_local_repo_dir(&self) -> Option<PathBuf> {
        let cwd = std::env::current_dir().ok();
        let mut candidates = Vec::new();
        if let Some(c) = cwd {
            candidates.push(c);
        }
        candidates.push(self.repo_dir.clone());
        candidates.into_iter().find(|d| path_looks_repo(d))
    }

    pub(super) fn prepare_local_source(&mut self, tmp_dir: &Path, source_dir: &Path) -> Result<()> {
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

    pub(super) fn run_git_with_timeout(&self, args: &[String]) -> Result<ExitStatus> {
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

    pub(super) fn clone_repo(&mut self, tmp_dir: &Path, url: &str) -> Result<bool> {
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

    pub(super) fn clone_repo_any(&mut self, tmp_dir: &Path) -> Result<bool> {
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

    pub(super) fn clone_repo_any_with_dns_retry(&mut self, tmp_dir: &Path) -> Result<bool> {
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

    pub(super) fn prepare_source_repo(&mut self, tmp_dir: &Path) -> Result<()> {
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
}
