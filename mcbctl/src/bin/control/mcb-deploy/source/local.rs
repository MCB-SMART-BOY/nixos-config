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
}
