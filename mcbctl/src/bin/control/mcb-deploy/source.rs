use super::*;

#[path = "source/local.rs"]
mod local;
#[path = "source/prompt.rs"]
mod prompt;
#[path = "source/remote.rs"]
mod remote;

fn reset_source_workspace(tmp_dir: &Path) -> Result<()> {
    if tmp_dir.exists() {
        fs::remove_dir_all(tmp_dir)
            .with_context(|| format!("failed to clear source workspace {}", tmp_dir.display()))?;
    }
    fs::create_dir_all(tmp_dir)
        .with_context(|| format!("failed to create source workspace {}", tmp_dir.display()))?;
    Ok(())
}

fn summarize_source_failures(context: &str, failures: &[(String, String)]) -> String {
    if failures.is_empty() {
        return context.to_string();
    }

    let joined = failures
        .iter()
        .map(|(label, detail)| format!("{label}: {detail}"))
        .collect::<Vec<_>>()
        .join(" | ");
    format!("{context}: {joined}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn reset_source_workspace_recreates_directory() -> Result<()> {
        let root = create_temp_dir("mcbctl-source-workspace-test")?;
        let tmp_dir = root.join("source");
        fs::create_dir_all(tmp_dir.join("nested"))?;
        fs::write(tmp_dir.join("nested/file.txt"), "stale")?;

        reset_source_workspace(&tmp_dir)?;

        assert!(tmp_dir.is_dir());
        assert!(!tmp_dir.join("nested/file.txt").exists());

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn summarize_source_failures_joins_attempts() {
        let message = summarize_source_failures(
            "准备源代码失败",
            &[
                ("mirror-a".to_string(), "clone exited with 128".to_string()),
                ("mirror-b".to_string(), "checkout failed".to_string()),
            ],
        );

        assert!(message.contains("准备源代码失败"));
        assert!(message.contains("mirror-a: clone exited with 128"));
        assert!(message.contains("mirror-b: checkout failed"));
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
}
