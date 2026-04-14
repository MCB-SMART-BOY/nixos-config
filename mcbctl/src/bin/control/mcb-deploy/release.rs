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

fn default_release_version_for_package_version(package_version: &str) -> String {
    if package_version.starts_with('v') {
        package_version.to_string()
    } else {
        format!("v{package_version}")
    }
}

fn release_workflow_run_args(workflow: &str, version: &str) -> Vec<String> {
    vec![
        "workflow".to_string(),
        "run".to_string(),
        workflow.to_string(),
        "--ref".to_string(),
        version.to_string(),
        "-f".to_string(),
        format!("tag={version}"),
    ]
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
        default_release_version_for_package_version(env!("CARGO_PKG_VERSION"))
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
        let workflow_args = release_workflow_run_args(workflow, &version);
        let st = Command::new("gh").args(&workflow_args).status()?;
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
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    fn test_app() -> App {
        App {
            repo_dir: PathBuf::from("/tmp/repo"),
            repo_urls: vec![],
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
            run_action: RunAction::Release,
            progress_total: 7,
            progress_current: 0,
            git_clone_timeout_sec: 90,
        }
    }

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

    #[test]
    fn default_release_version_for_package_version_adds_v_prefix() {
        assert_eq!(
            default_release_version_for_package_version("3.0.0"),
            "v3.0.0"
        );
        assert_eq!(
            default_release_version_for_package_version("v3.0.0"),
            "v3.0.0"
        );
    }

    #[test]
    fn app_default_release_version_uses_package_version() {
        let app = test_app();

        assert_eq!(
            app.default_release_version(),
            format!("v{}", env!("CARGO_PKG_VERSION"))
        );
    }

    #[test]
    fn release_workflow_run_args_use_release_tag_as_ref() {
        let args = release_workflow_run_args("release-mcbctl.yml", "v3.0.0");

        assert_eq!(
            args,
            vec![
                "workflow".to_string(),
                "run".to_string(),
                "release-mcbctl.yml".to_string(),
                "--ref".to_string(),
                "v3.0.0".to_string(),
                "-f".to_string(),
                "tag=v3.0.0".to_string(),
            ]
        );
    }
}
