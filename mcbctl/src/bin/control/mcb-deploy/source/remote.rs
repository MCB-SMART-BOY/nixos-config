use super::*;

impl App {
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

    pub(crate) fn prepare_source_repo(&mut self, tmp_dir: &Path) -> Result<()> {
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
