use super::*;
use mcbctl::release_bundle::{default_release_repository, render_release_manifest_json};

fn summarize_cleanup_failures(context: &str, failures: &[String]) -> String {
    if failures.is_empty() {
        return context.to_string();
    }

    format!("{context}: {}", failures.join(" | "))
}

fn cleanup_temp_file(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    fs::remove_file(path)
        .with_context(|| format!("failed to remove temporary file {}", path.display()))
}

fn cleanup_temp_files(paths: &[&Path]) -> Result<()> {
    let failures = paths
        .iter()
        .filter_map(|path| cleanup_temp_file(path).err().map(|err| err.to_string()))
        .collect::<Vec<_>>();
    if failures.is_empty() {
        Ok(())
    } else {
        bail!("{}", failures.join(" | "))
    }
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

fn dirty_worktree_from_probe(status_success: bool, stdout: &str, stderr: &str) -> Result<bool> {
    if !status_success {
        let stderr = stderr.trim();
        if stderr.is_empty() {
            bail!("git status --porcelain failed");
        }
        bail!("git status --porcelain failed: {stderr}");
    }

    Ok(!stdout.trim().is_empty())
}

fn ensure_release_worktree_gate(dirty_result: Result<bool>, allow_dirty: bool) -> Result<()> {
    let dirty = dirty_result.context("探测工作区状态失败")?;
    if dirty && !allow_dirty {
        bail!("工作区存在未提交变更，发布前请先提交或设置 RELEASE_ALLOW_DIRTY=true。");
    }
    Ok(())
}

fn ensure_git_release_step(status_success: bool, args: &[&str]) -> Result<()> {
    if status_success {
        Ok(())
    } else {
        bail!("git {} 失败", args.join(" "));
    }
}

fn ensure_release_create_step(status_success: bool) -> Result<()> {
    if status_success {
        Ok(())
    } else {
        bail!("gh release create 失败");
    }
}

fn ensure_workflow_run_step(status_success: bool, workflow: &str) -> Result<()> {
    if status_success {
        Ok(())
    } else {
        bail!("gh workflow run {} 失败", workflow);
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

        let dirty = match Self::command_output("git", &["status", "--porcelain"]) {
            Ok(out) => dirty_worktree_from_probe(
                out.status.success(),
                &String::from_utf8_lossy(&out.stdout),
                &String::from_utf8_lossy(&out.stderr),
            ),
            Err(err) => Err(err),
        };
        let allow_dirty = std::env::var("RELEASE_ALLOW_DIRTY")
            .ok()
            .is_some_and(|v| v == "true");
        ensure_release_worktree_gate(dirty, allow_dirty)?;

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
            self.section(&format!("将发布版本：{version}"));
            if !last_tag.is_empty() {
                self.note(&format!("上一个版本：{last_tag}"));
            }
            self.section("Release Notes 预览：");
            for line in notes.lines() {
                self.note(line);
            }
            self.confirm_continue(&format!("确认发布 {version}？"))?;
        }

        for args in [
            vec!["tag", "-a", &version, "-m", &version],
            vec!["push", "origin", "HEAD"],
            vec!["push", "origin", &version],
        ] {
            let st = Command::new("git").args(&args).status()?;
            ensure_git_release_step(st.success(), &args)?;
        }

        let notes_file = create_temp_path("mcbctl-release-notes", "md")?;
        let manifest_file = create_temp_path("mcbctl-release-manifest", "json")?;
        fs::write(&notes_file, notes)?;
        let manifest_repo = std::env::var("RELEASE_REPOSITORY")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(default_release_repository);
        fs::write(
            &manifest_file,
            render_release_manifest_json(&manifest_repo, &version)?,
        )?;
        let st = Command::new("gh")
            .arg("release")
            .arg("create")
            .arg(&version)
            .arg("--title")
            .arg(&version)
            .arg("--notes-file")
            .arg(&notes_file)
            .arg(&manifest_file)
            .status()?;
        let release_result = ensure_release_create_step(st.success());
        let cleanup_result = cleanup_temp_files(&[notes_file.as_path(), manifest_file.as_path()]);
        finalize_with_cleanup(
            release_result,
            cleanup_result,
            "release asset cleanup failed",
        )?;

        let workflow = "release-mcbctl.yml";
        let workflow_args = release_workflow_run_args(workflow, &version);
        let st = Command::new("gh").args(&workflow_args).status()?;
        ensure_workflow_run_step(st.success(), workflow)?;

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
            source_dir_override: None,
            repo_urls: vec![],
            branch: "main".to_string(),
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
            rebuild_upgrade_set: false,
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
            "release asset cleanup failed",
            &[
                "remove temp file failed".to_string(),
                "another cleanup error".to_string(),
            ],
        );

        assert!(summary.contains("release asset cleanup failed"));
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
    fn dirty_worktree_from_probe_detects_clean_and_dirty_outputs() -> Result<()> {
        assert!(!dirty_worktree_from_probe(true, "", "")?);
        assert!(!dirty_worktree_from_probe(true, "  \n", "")?);
        assert!(dirty_worktree_from_probe(true, " M Cargo.toml\n", "")?);
        Ok(())
    }

    #[test]
    fn dirty_worktree_from_probe_surfaces_probe_failure() {
        let err = dirty_worktree_from_probe(false, "", "index locked")
            .expect_err("failed git status probe should be reported");

        assert!(err.to_string().contains("index locked"));
    }

    #[test]
    fn ensure_release_worktree_gate_allows_clean_worktree_without_override() -> Result<()> {
        ensure_release_worktree_gate(Ok(false), false)?;
        Ok(())
    }

    #[test]
    fn ensure_release_worktree_gate_allows_dirty_worktree_with_override() -> Result<()> {
        ensure_release_worktree_gate(Ok(true), true)?;
        Ok(())
    }

    #[test]
    fn ensure_release_worktree_gate_blocks_dirty_worktree_without_override() {
        let err = ensure_release_worktree_gate(Ok(true), false)
            .expect_err("dirty worktree without override should be rejected");

        assert!(err.to_string().contains("工作区存在未提交变更"));
        assert!(err.to_string().contains("RELEASE_ALLOW_DIRTY=true"));
    }

    #[test]
    fn ensure_release_worktree_gate_surfaces_probe_failure_context() {
        let err = ensure_release_worktree_gate(Err(anyhow::anyhow!("index locked")), false)
            .expect_err("probe failure should be wrapped with release gate context");

        assert!(err.to_string().contains("探测工作区状态失败"));
        assert!(
            err.chain()
                .any(|cause| cause.to_string().contains("index locked"))
        );
    }

    #[test]
    fn ensure_git_release_step_surfaces_failing_push() {
        let err = ensure_git_release_step(false, &["push", "origin", "HEAD"])
            .expect_err("failing git release step should be reported");

        assert!(err.to_string().contains("git push origin HEAD 失败"));
    }

    #[test]
    fn ensure_release_create_step_surfaces_failure() {
        let err =
            ensure_release_create_step(false).expect_err("failing release creation should surface");

        assert!(err.to_string().contains("gh release create 失败"));
    }

    #[test]
    fn finalize_with_cleanup_preserves_release_create_failure_and_cleanup_context() {
        let err = finalize_with_cleanup(
            ensure_release_create_step(false),
            Err(anyhow::anyhow!("unlink failed")),
            "release asset cleanup failed",
        )
        .expect_err("cleanup aggregation should preserve release creation failure");

        assert!(err.to_string().contains("gh release create 失败"));
        assert!(err.to_string().contains("unlink failed"));
    }

    #[test]
    fn ensure_workflow_run_step_surfaces_failure() {
        let err = ensure_workflow_run_step(false, "release-mcbctl.yml")
            .expect_err("failing workflow dispatch should be reported");

        assert!(
            err.to_string()
                .contains("gh workflow run release-mcbctl.yml 失败")
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
