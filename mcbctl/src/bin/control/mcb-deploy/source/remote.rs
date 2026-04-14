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
                    if let Some(commit) = run_capture_allow_fail(
                        "git",
                        &["-C", &tmp_dir.display().to_string(), "rev-parse", "HEAD"],
                    ) {
                        self.source_commit = commit.trim().to_string();
                    }
                    self.success(&format!("仓库拉取完成（{}）", self.source_commit));
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
            if let Some(commit) = run_capture_allow_fail(
                "git",
                &["-C", &tmp_dir.display().to_string(), "rev-parse", "HEAD"],
            ) {
                self.source_commit = commit.trim().to_string();
            }
            self.success(&format!("仓库拉取完成（{}）", self.source_commit));
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
