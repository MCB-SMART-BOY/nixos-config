use crate::run_capture_allow_fail;
use anyhow::{Result, bail};
use std::fs;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegrityFinding {
    pub path: String,
    pub rule: &'static str,
    pub detail: String,
}

#[derive(Clone, Debug, Default)]
pub struct IntegrityReport {
    pub findings: Vec<IntegrityFinding>,
}

impl IntegrityReport {
    pub fn is_clean(&self) -> bool {
        self.findings.is_empty()
    }

    fn push(&mut self, path: String, rule: &'static str, detail: impl Into<String>) {
        self.findings.push(IntegrityFinding {
            path,
            rule,
            detail: detail.into(),
        });
    }

    pub fn render_lines(&self) -> Vec<String> {
        if self.findings.is_empty() {
            return vec!["repository integrity check passed".to_string()];
        }

        let mut lines = vec![format!(
            "repository integrity check failed with {} finding(s):",
            self.findings.len()
        )];
        for finding in &self.findings {
            lines.push(format!(
                "- [{}] {}: {}",
                finding.rule, finding.path, finding.detail
            ));
        }
        lines
    }
}

pub fn ensure_repository_integrity(root: &Path) -> Result<()> {
    let report = audit_repository(root)?;
    if report.is_clean() {
        return Ok(());
    }
    bail!("{}", report.render_lines().join("\n"));
}

pub fn detect_current_branch(repo_root: &Path) -> Option<String> {
    let repo = repo_root.display().to_string();
    let branch = run_capture_allow_fail("git", &["-C", &repo, "branch", "--show-current"])?;
    let branch = branch.trim();
    if branch.is_empty() || branch == "HEAD" {
        None
    } else {
        Some(branch.to_string())
    }
}

pub fn preferred_remote_branch(repo_root: &Path) -> String {
    detect_current_branch(repo_root).unwrap_or_else(|| "rust脚本分支".to_string())
}

pub fn audit_repository(root: &Path) -> Result<IntegrityReport> {
    let mut report = IntegrityReport::default();

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|entry| !should_skip_entry(entry))
        .flatten()
    {
        if entry.path() == root {
            continue;
        }

        let rel = relative_path(root, entry.path());
        check_forbidden_path(&mut report, &rel, &entry);
        if !entry.file_type().is_file() {
            continue;
        }
        check_forbidden_extension(&mut report, &rel);
        check_forbidden_content(&mut report, entry.path(), &rel);
    }

    report.findings.sort_by(|left, right| {
        left.rule
            .cmp(right.rule)
            .then_with(|| left.path.cmp(&right.path))
            .then_with(|| left.detail.cmp(&right.detail))
    });
    report.findings.dedup();

    Ok(report)
}

fn should_skip_entry(entry: &DirEntry) -> bool {
    if entry.depth() == 0 {
        return false;
    }
    let Some(name) = entry.file_name().to_str() else {
        return false;
    };
    entry.file_type().is_dir()
        && (name == ".git" || name == "target" || name == "result" || name.starts_with("result-"))
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn check_forbidden_path(report: &mut IntegrityReport, rel: &str, entry: &DirEntry) {
    let legacy_runner = legacy_runner_file();
    let legacy_root = legacy_root_dir();
    let legacy_root_prefix = format!("{legacy_root}/");
    let legacy_rs = legacy_rs_dir();
    let legacy_rs_prefix = format!("{legacy_rs}/");

    if rel == legacy_runner {
        report.push(
            rel.to_string(),
            "legacy-path",
            "legacy root launcher file must not exist in the Rust-only branch",
        );
    }

    if rel == legacy_root || rel.starts_with(&legacy_root_prefix) {
        report.push(
            rel.to_string(),
            "legacy-path",
            "legacy root script directory must be removed; project logic belongs in mcbctl/",
        );
    }

    if rel == legacy_rs || rel.starts_with(&legacy_rs_prefix) {
        report.push(
            rel.to_string(),
            "legacy-path",
            "legacy Rust script directory must be removed; use mcbctl/src/bin/* instead",
        );
    }

    if is_home_user_scripts_path(rel) {
        report.push(
            rel.to_string(),
            "legacy-path",
            "user-side script subtrees are forbidden; user commands must come from Rust binaries",
        );
    }

    if is_pkg_scripts_path(rel) {
        report.push(
            rel.to_string(),
            "legacy-path",
            "package-side script subtrees are forbidden; package logic must stay in Rust/Nix",
        );
    }

    if entry.file_type().is_dir() && (rel.ends_with("/scripts") || rel == legacy_root) {
        report.push(
            rel.to_string(),
            "legacy-path",
            "script directories are not allowed in the converged branch",
        );
    }
}

fn is_home_user_scripts_path(rel: &str) -> bool {
    let parts = rel.split('/').collect::<Vec<_>>();
    parts.len() >= 4 && parts[0] == "home" && parts[1] == "users" && parts[3] == "scripts"
}

fn is_pkg_scripts_path(rel: &str) -> bool {
    let parts = rel.split('/').collect::<Vec<_>>();
    parts.len() >= 3 && parts[0] == "pkgs" && parts[2] == "scripts"
}

fn legacy_runner_file() -> String {
    ["run", ".sh"].concat()
}

fn legacy_root_dir() -> &'static str {
    "scripts"
}

fn legacy_rs_dir() -> String {
    [legacy_root_dir(), "rs"].join("-")
}

fn check_forbidden_extension(report: &mut IntegrityReport, rel: &str) {
    let ext = Path::new(rel)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default();
    if matches!(ext, "sh" | "bash" | "py") {
        report.push(
            rel.to_string(),
            "forbidden-extension",
            format!("*.{ext} files are not allowed in this branch"),
        );
    }
}

fn check_forbidden_content(report: &mut IntegrityReport, path: &Path, rel: &str) {
    let ext = Path::new(rel).extension().and_then(|ext| ext.to_str());
    if ext.is_some_and(|ext| ext == "md") {
        return;
    }

    let Ok(bytes) = fs::read(path) else {
        return;
    };
    let Ok(content) = String::from_utf8(bytes) else {
        return;
    };

    if ext.is_some_and(|ext| ext == "rs") {
        for (rule, pattern, detail) in [
            (
                "rust-shell-bridge",
                "Command::new(\"sh\")",
                "Rust must not spawn sh directly",
            ),
            (
                "rust-shell-bridge",
                "Command::new(\"bash\")",
                "Rust must not spawn bash directly",
            ),
            (
                "rust-shell-bridge",
                "Command::new(\"python\")",
                "Rust must not spawn python directly",
            ),
        ] {
            if content.contains(pattern) {
                report.push(rel.to_string(), rule, detail);
            }
        }
        return;
    }

    for (rule, pattern, detail) in [
        (
            "shebang",
            format!("#!/usr/bin/env {}", "bash"),
            "bash shebang is forbidden; move logic to Rust or Nix",
        ),
        (
            "shebang",
            format!("#!/usr/bin/env {}", "sh"),
            "sh shebang is forbidden; move logic to Rust or Nix",
        ),
        (
            "shebang",
            format!("#!/usr/bin/env {}", "python"),
            "python shebang is forbidden; move logic to Rust or Nix",
        ),
        (
            "shell-bridge",
            nix_shell_helper(&["write", "Shell", "Application"]),
            "Nix shell helper is forbidden; package logic must stay out of shell wrappers",
        ),
        (
            "shell-bridge",
            nix_shell_helper(&["write", "Shell", "Script", "Bin"]),
            "Nix shell helper is forbidden; package logic must stay out of shell wrappers",
        ),
        (
            "shell-bridge",
            nix_shell_helper(&["write", "Shell", "Script"]),
            "Nix shell helper is forbidden; package logic must stay out of shell wrappers",
        ),
        (
            "shell-bridge",
            shell_flag_pattern("sh", "-c"),
            "shell command strings are forbidden; invoke Rust/Nix logic directly",
        ),
        (
            "shell-bridge",
            shell_flag_pattern("bash", "-c"),
            "shell command strings are forbidden; invoke Rust/Nix logic directly",
        ),
        (
            "shell-bridge",
            shell_flag_pattern("bash", "-lc"),
            "shell command strings are forbidden; invoke Rust/Nix logic directly",
        ),
        (
            "shell-bridge",
            shell_flag_pattern("python", "-c"),
            "python command strings are forbidden; move logic to Rust",
        ),
        (
            "shell-bridge",
            shell_flag_pattern("fish", "-ic"),
            "interactive fish bridges are forbidden; use Rust helpers instead",
        ),
        (
            "shell-bridge",
            shell_flag_pattern("fish", "-c"),
            "fish bridges are forbidden; use Rust helpers instead",
        ),
    ] {
        if content.contains(&pattern) {
            report.push(rel.to_string(), rule, detail);
        }
    }
}

fn nix_shell_helper(parts: &[&str]) -> String {
    parts.join("")
}

fn shell_flag_pattern(command: &str, flag: &str) -> String {
    [command, flag].join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn flags_shell_wrappers_and_legacy_paths() -> Result<()> {
        let root = create_temp_repo()?;
        fs::create_dir_all(root.join(legacy_root_dir()))?;
        fs::write(root.join(legacy_root_dir()).join("run"), "echo hi")?;
        fs::write(
            root.join("flake.nix"),
            format!(
                r#"
              {{
                outputs = {{ ... }}: {{
                  packages.x86_64-linux.default = {} "demo" "";
                }};
              }}
            "#,
                nix_shell_helper(&["write", "Shell", "Script", "Bin"])
            ),
        )?;

        let report = audit_repository(&root)?;
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.rule == "legacy-path" && finding.path == legacy_root_dir())
        );
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.rule == "shell-bridge")
        );
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn ignores_markdown_delete_lists() -> Result<()> {
        let root = create_temp_repo()?;
        fs::write(root.join("README.md"), "删除旧壳层桥接即可。")?;

        let report = audit_repository(&root)?;
        assert!(report.is_clean());
        fs::remove_dir_all(root)?;
        Ok(())
    }

    fn create_temp_repo() -> Result<std::path::PathBuf> {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let root =
            std::env::temp_dir().join(format!("mcbctl-repo-audit-{}-{unique}", std::process::id()));
        fs::create_dir_all(&root)?;
        Ok(root)
    }
}
