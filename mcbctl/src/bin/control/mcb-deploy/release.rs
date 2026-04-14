use super::*;

fn summarize_cleanup_failures(context: &str, failures: &[String]) -> String {
    if failures.is_empty() {
        return context.to_string();
    }

    format!("{context}: {}", failures.join(" | "))
}

fn cleanup_release_notes_file(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    fs::remove_file(path)
        .with_context(|| format!("failed to remove release notes file {}", path.display()))
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

fn next_release_version_for_base(base: &str, tag_output: &str) -> String {
    let mut max = -1i64;
    for tag in tag_output.lines() {
        if tag == base {
            max = 0;
        } else if let Some(sfx) = tag.strip_prefix(&(base.to_string() + "."))
            && let Ok(num) = sfx.parse::<i64>()
            && num > max
        {
            max = num;
        }
    }
    if max >= 0 {
        format!("{base}.{}", max + 1)
    } else {
        base.to_string()
    }
}

fn render_release_notes_from_log(last_tag: &str, log_output: Option<&str>) -> String {
    let Some(out) = log_output else {
        return if last_tag.is_empty() {
            "Git log unavailable; no release notes generated.".to_string()
        } else {
            format!("Git log unavailable since {last_tag}; no release notes generated.")
        };
    };
    let lines: Vec<&str> = out.lines().collect();
    if lines.is_empty() {
        if last_tag.is_empty() {
            "No code changes found.".to_string()
        } else {
            format!("No code changes since {last_tag}.")
        }
    } else {
        let header = if last_tag.is_empty() {
            "Changes".to_string()
        } else {
            format!("Changes since {last_tag}")
        };
        let mut notes = format!("## {header}\n");
        for line in lines {
            notes.push_str(&format!("- {line}\n"));
        }
        notes
    }
}

impl App {
    pub(super) fn default_release_version(&self) -> String {
        let today = run_capture_allow_fail("date", &["+%Y.%m.%d"])
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| {
                self.warn("无法读取当前日期，发布版本前缀将回退到 1970.01.01。");
                "1970.01.01".to_string()
            });
        let base = format!("v{today}");
        let out = run_capture_allow_fail("git", &["tag", "--list", &base, &format!("{base}.*")]);
        if out.is_none() {
            self.warn("无法读取现有 release tag，发布版本将按当天基础版本回退。");
        }
        next_release_version_for_base(&base, out.as_deref().unwrap_or(""))
    }

    pub(super) fn resolve_release_version(&self) -> Result<String> {
        let mut version = std::env::var("RELEASE_VERSION").unwrap_or_default();
        let default_version = self.default_release_version();
        if version.is_empty() && self.is_tty() {
            let input =
                self.prompt_line(&format!("请输入发布版本（默认 {default_version}）： "))?;
            version = input.trim().to_string();
        }
        if version.is_empty() {
            version = default_version;
        }
        if !version.starts_with('v') {
            version = format!("v{version}");
        }
        Ok(version)
    }

    pub(super) fn find_last_release_tag(&self) -> String {
        run_capture_allow_fail("git", &["describe", "--tags", "--abbrev=0"])
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| {
                self.warn("无法探测上一个 release tag，将按首次发布生成 release notes。");
                String::new()
            })
    }

    pub(super) fn generate_release_notes(&self, last_tag: &str) -> String {
        let range = if last_tag.is_empty() {
            "HEAD".to_string()
        } else {
            format!("{last_tag}..HEAD")
        };
        let out = run_capture_allow_fail("git", &["log", "--oneline", "--no-merges", &range]);
        if out.is_none() {
            self.warn("无法读取 git log，将生成回退版 release notes。");
        }
        render_release_notes_from_log(last_tag, out.as_deref())
    }

    pub(super) fn release_flow(&mut self) -> Result<()> {
        self.banner();
        if !command_exists("git") {
            bail!("未找到 git。");
        }
        if !command_exists("gh") {
            bail!("未找到 GitHub CLI (gh)。");
        }
        let auth = Command::new("gh").args(["auth", "status"]).status()?;
        if !auth.success() {
            bail!("gh 未登录，请先执行 gh auth login。");
        }

        std::env::set_current_dir(&self.repo_dir)
            .with_context(|| format!("无法进入仓库目录：{}", self.repo_dir.display()))?;
        if !self.repo_dir.join(".git").is_dir() {
            bail!("当前目录不是 git 仓库：{}", self.repo_dir.display());
        }

        let dirty = run_capture_allow_fail("git", &["status", "--porcelain"]).unwrap_or_default();
        let allow_dirty = std::env::var("RELEASE_ALLOW_DIRTY")
            .ok()
            .is_some_and(|v| v == "true");
        if !dirty.trim().is_empty() && !allow_dirty {
            bail!("工作区存在未提交变更，发布前请先提交或设置 RELEASE_ALLOW_DIRTY=true。");
        }

        let version = self.resolve_release_version()?;
        let exists = Command::new("git").args(["rev-parse", &version]).status()?;
        if exists.success() {
            bail!("标签已存在：{version}");
        }

        let last_tag = self.find_last_release_tag();
        let mut notes = std::env::var("RELEASE_NOTES").unwrap_or_default();
        if notes.is_empty() {
            notes = self.generate_release_notes(&last_tag);
        }

        if self.is_tty() {
            println!("\n将发布版本：{version}");
            if !last_tag.is_empty() {
                println!("上一个版本：{last_tag}");
            }
            println!("\nRelease Notes 预览：\n{notes}\n");
            self.confirm_continue(&format!("确认发布 {version}？"))?;
        }

        for args in [
            vec!["tag", "-a", &version, "-m", &version],
            vec!["push", "origin", "HEAD"],
            vec!["push", "origin", &version],
        ] {
            let st = Command::new("git").args(&args).status()?;
            if !st.success() {
                bail!("git {} 失败", args.join(" "));
            }
        }

        let notes_file = create_temp_path("mcbctl-release-notes", "md")?;
        fs::write(&notes_file, notes)?;
        let st = Command::new("gh")
            .args([
                "release",
                "create",
                &version,
                "--title",
                &version,
                "--notes-file",
                &notes_file.display().to_string(),
            ])
            .status()?;
        let release_result = if st.success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("gh release create 失败"))
        };
        let cleanup_result = cleanup_release_notes_file(&notes_file);
        finalize_with_cleanup(
            release_result,
            cleanup_result,
            "release notes cleanup failed",
        )?;

        let workflow = "release-mcbctl.yml";
        let st = Command::new("gh")
            .args([
                "workflow",
                "run",
                workflow,
                "--ref",
                &self.branch,
                "-f",
                &format!("tag={version}"),
            ])
            .status()?;
        if !st.success() {
            bail!("gh workflow run {} 失败", workflow);
        }

        self.success(&format!(
            "Release 已发布：{version}；已触发跨平台预编译产物构建。"
        ));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarize_cleanup_failures_joins_messages() {
        let summary = summarize_cleanup_failures(
            "release notes cleanup failed",
            &[
                "remove temp file failed".to_string(),
                "another cleanup error".to_string(),
            ],
        );

        assert!(summary.contains("release notes cleanup failed"));
        assert!(summary.contains("remove temp file failed"));
        assert!(summary.contains("another cleanup error"));
    }

    #[test]
    fn next_release_version_for_base_picks_next_suffix() {
        let tags = "v2026.04.14\nv2026.04.14.1\nv2026.04.14.3\nv2026.04.13.9\n";

        assert_eq!(
            next_release_version_for_base("v2026.04.14", tags),
            "v2026.04.14.4"
        );
    }

    #[test]
    fn next_release_version_for_base_returns_base_when_no_matching_tag_exists() {
        assert_eq!(
            next_release_version_for_base("v2026.04.14", "v2026.04.13\n"),
            "v2026.04.14"
        );
    }

    #[test]
    fn render_release_notes_from_log_renders_changes_when_log_is_available() {
        let notes = render_release_notes_from_log("v1.0.0", Some("abc fix bug\ndef add test"));

        assert!(notes.contains("## Changes since v1.0.0"));
        assert!(notes.contains("- abc fix bug"));
        assert!(notes.contains("- def add test"));
    }

    #[test]
    fn render_release_notes_from_log_falls_back_when_log_is_unavailable() {
        let notes = render_release_notes_from_log("v1.0.0", None);

        assert_eq!(
            notes,
            "Git log unavailable since v1.0.0; no release notes generated."
        );
    }
}
