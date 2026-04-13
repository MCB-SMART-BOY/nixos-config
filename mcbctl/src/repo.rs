use crate::{
    managed_file_is_valid, managed_file_kind, render_managed_file, run_capture_allow_fail,
    write_file_atomic,
};
use anyhow::{Context, Result, bail};
use std::fs;
use std::path::{Path, PathBuf};
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

#[derive(Clone, Debug, Default)]
pub struct ManagedMigrationReport {
    pub migrated: Vec<String>,
    pub skipped: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct ManagedExtractionReport {
    pub extracted: Vec<String>,
    pub skipped_valid: Vec<String>,
    pub skipped_legacy: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct HardwareConfigMigrationReport {
    pub destination: String,
    pub moved: bool,
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

pub fn migrate_managed_files(root: &Path) -> Result<ManagedMigrationReport> {
    let mut report = ManagedMigrationReport::default();

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|entry| !should_skip_entry(entry))
        .flatten()
    {
        if !entry.file_type().is_file() {
            continue;
        }

        let rel = relative_path(root, entry.path());
        let Some(kind) = expected_managed_kind(&rel) else {
            continue;
        };

        let content = fs::read_to_string(entry.path())?;
        if managed_file_kind(&content) == Some(kind.as_str()) && managed_file_is_valid(&content) {
            report.skipped.push(rel);
            continue;
        }

        if managed_file_kind(&content).is_some() {
            bail!(
                "refusing to migrate {}: existing managed marker is invalid or kind mismatches {}",
                rel,
                kind
            );
        }

        if !is_recognized_legacy_managed_content(&rel, &content) {
            bail!(
                "refusing to migrate {}: content does not match a recognized legacy managed file for {}",
                rel,
                kind
            );
        }

        write_file_atomic(entry.path(), &render_managed_file(&kind, &content))?;
        report.migrated.push(rel);
    }

    report.migrated.sort();
    report.skipped.sort();
    Ok(report)
}

pub fn extract_manual_managed_files(root: &Path) -> Result<ManagedExtractionReport> {
    let mut report = ManagedExtractionReport::default();

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|entry| !should_skip_entry(entry))
        .flatten()
    {
        if !entry.file_type().is_file() {
            continue;
        }

        let rel = relative_path(root, entry.path());
        let Some(kind) = expected_managed_kind(&rel) else {
            continue;
        };

        let content = fs::read_to_string(entry.path())?;
        if managed_file_kind(&content) == Some(kind.as_str()) && managed_file_is_valid(&content) {
            report.skipped_valid.push(rel);
            continue;
        }

        if is_recognized_legacy_managed_content(&rel, &content) {
            report.skipped_legacy.push(rel);
            continue;
        }

        let plan = extraction_plan(&rel)?;
        let extracted_path = root.join(&plan.extracted_rel);
        if let Some(parent) = extracted_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        let extracted_body = render_extracted_manual_module(&plan.source_tail, &content);
        if extracted_path.is_file() {
            let existing = fs::read_to_string(&extracted_path)
                .with_context(|| format!("failed to read {}", extracted_path.display()))?;
            if existing != extracted_body {
                bail!(
                    "refusing to overwrite extracted manual file {}; review and merge it manually first",
                    extracted_path.display()
                );
            }
        } else {
            write_file_atomic(&extracted_path, &extracted_body)?;
        }

        let local_auto_path = root.join(&plan.local_auto_rel);
        let local_auto_body = render_local_auto_file();
        if let Ok(existing) = fs::read_to_string(&local_auto_path) {
            if existing != local_auto_body {
                bail!(
                    "refusing to overwrite {}: local.auto.nix is not in mcbctl-generated form",
                    local_auto_path.display()
                );
            }
        } else {
            if let Some(parent) = local_auto_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create {}", parent.display()))?;
            }
            write_file_atomic(&local_auto_path, &local_auto_body)?;
        }

        let replacement = replacement_managed_body(&rel, &kind)
            .with_context(|| format!("failed to resolve replacement body for {}", rel))?;
        write_file_atomic(entry.path(), &render_managed_file(&kind, &replacement))?;
        report.extracted.push(rel);
    }

    report.extracted.sort();
    report.skipped_valid.sort();
    report.skipped_legacy.sort();
    Ok(report)
}

pub fn migrate_root_hardware_config(
    root: &Path,
    host: &str,
) -> Result<HardwareConfigMigrationReport> {
    if host.trim().is_empty() {
        bail!("未指定目标主机，无法迁移 hardware-configuration.nix");
    }

    let source = root.join("hardware-configuration.nix");
    let destination = root
        .join("hosts")
        .join(host)
        .join("hardware-configuration.nix");

    if !source.exists() {
        if destination.is_file() {
            return Ok(HardwareConfigMigrationReport {
                destination: relative_path(root, &destination),
                moved: false,
            });
        }
        bail!(
            "未发现 {}；也未发现 {}",
            source.display(),
            destination.display()
        );
    }

    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    if destination.is_file() {
        let source_content = fs::read_to_string(&source)
            .with_context(|| format!("failed to read {}", source.display()))?;
        let destination_content = fs::read_to_string(&destination)
            .with_context(|| format!("failed to read {}", destination.display()))?;
        if source_content != destination_content {
            bail!(
                "目标 {} 已存在且内容不同；请先手工确认再迁移",
                destination.display()
            );
        }
        fs::remove_file(&source)
            .with_context(|| format!("failed to remove {}", source.display()))?;
        return Ok(HardwareConfigMigrationReport {
            destination: relative_path(root, &destination),
            moved: true,
        });
    }

    fs::rename(&source, &destination).or_else(|_| {
        fs::copy(&source, &destination)
            .with_context(|| {
                format!(
                    "failed to copy {} -> {}",
                    source.display(),
                    destination.display()
                )
            })
            .and_then(|_| {
                fs::remove_file(&source)
                    .with_context(|| format!("failed to remove {}", source.display()))
            })
    })?;

    Ok(HardwareConfigMigrationReport {
        destination: relative_path(root, &destination),
        moved: true,
    })
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

    if rel == "hardware-configuration.nix" {
        report.push(
            rel.to_string(),
            "legacy-hardware-path",
            "repository root hardware-configuration.nix is obsolete; move it to hosts/<host>/hardware-configuration.nix via `mcbctl migrate-hardware-config`",
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

    check_managed_file_protocol(report, rel, &content);
}

fn check_managed_file_protocol(report: &mut IntegrityReport, rel: &str, content: &str) {
    let Some(expected_kind) = expected_managed_kind(rel) else {
        return;
    };

    let valid_kind = managed_file_kind(content)
        .filter(|kind| *kind == expected_kind.as_str() && managed_file_is_valid(content));
    if valid_kind.is_some() {
        return;
    }

    let detail = if let Some(existing_kind) = managed_file_kind(content) {
        format!(
            "managed file must use valid mcbctl marker kind {}; found {}",
            expected_kind, existing_kind
        )
    } else {
        format!(
            "managed file must use valid mcbctl marker kind {}; run `mcbctl migrate-managed`",
            expected_kind
        )
    };
    report.push(rel.to_string(), "managed-protocol", detail);
}

fn nix_shell_helper(parts: &[&str]) -> String {
    parts.join("")
}

fn shell_flag_pattern(command: &str, flag: &str) -> String {
    [command, flag].join(" ")
}

fn expected_managed_kind(rel: &str) -> Option<String> {
    let parts = rel.split('/').collect::<Vec<_>>();

    if parts.len() == 4 && parts[0] == "hosts" && parts[2] == "managed" {
        return match parts[3] {
            "default.nix" => Some("host-managed-default".to_string()),
            "users.nix" => Some("host-users".to_string()),
            "network.nix" => Some("host-network".to_string()),
            "gpu.nix" => Some("host-gpu".to_string()),
            "virtualization.nix" => Some("host-virtualization".to_string()),
            _ => None,
        };
    }

    if parts.len() == 5 && parts[0] == "hosts" && parts[1] == "templates" && parts[3] == "managed" {
        return match parts[4] {
            "default.nix" => Some("host-managed-default".to_string()),
            "users.nix" => Some("host-users".to_string()),
            "network.nix" => Some("host-network".to_string()),
            "gpu.nix" => Some("host-gpu".to_string()),
            "virtualization.nix" => Some("host-virtualization".to_string()),
            _ => None,
        };
    }

    if parts.len() >= 5 && parts[0] == "home" {
        let managed_idx = match parts[1] {
            "users" if parts.len() >= 5 => Some(3usize),
            "templates" if parts.len() >= 6 && parts[2] == "users" => Some(4usize),
            _ => None,
        }?;

        if parts.get(managed_idx) != Some(&"managed") {
            return None;
        }

        let tail = &parts[managed_idx + 1..];
        return match tail {
            ["default.nix"] => Some("home-managed-default".to_string()),
            ["packages.nix"] => Some("home-packages-aggregator".to_string()),
            ["settings", "default.nix"] => Some("home-settings-default".to_string()),
            ["settings", "desktop.nix"] => Some("home-settings-desktop".to_string()),
            ["settings", "session.nix"] => Some("home-settings-session".to_string()),
            ["settings", "mime.nix"] => Some("home-settings-mime".to_string()),
            ["packages", file] if file.ends_with(".nix") => {
                Some(format!("package-group:{}", file.trim_end_matches(".nix")))
            }
            _ => None,
        };
    }

    None
}

struct ExtractionPlan {
    source_tail: String,
    local_auto_rel: PathBuf,
    extracted_rel: PathBuf,
}

fn extraction_plan(rel: &str) -> Result<ExtractionPlan> {
    let parts = rel.split('/').collect::<Vec<_>>();
    let (owner_parts, managed_idx) = match parts.as_slice() {
        ["hosts", host, "managed", ..] => (vec!["hosts", host], 2usize),
        ["hosts", "templates", template, "managed", ..] => {
            (vec!["hosts", "templates", template], 3usize)
        }
        ["home", "users", user, "managed", ..] => (vec!["home", "users", user], 3usize),
        ["home", "templates", "users", template, "managed", ..] => {
            (vec!["home", "templates", "users", template], 4usize)
        }
        _ => bail!("{} is not a supported managed path", rel),
    };

    let tail = &parts[managed_idx + 1..];
    if tail.is_empty() {
        bail!("{} is missing the managed file tail", rel);
    }

    let owner_rel = owner_parts.iter().collect::<PathBuf>();
    let source_tail = tail.join("/");
    let extracted_name = format!(
        "managed-{}.nix",
        tail.iter()
            .map(|part| part.trim_end_matches(".nix"))
            .collect::<Vec<_>>()
            .join("-")
    );

    Ok(ExtractionPlan {
        source_tail,
        local_auto_rel: owner_rel.join("local.auto.nix"),
        extracted_rel: owner_rel.join("local-extracted").join(extracted_name),
    })
}

fn render_local_auto_file() -> String {
    [
        "# mcbctl-local-auto",
        "# 这个文件由 mcbctl 维护，用于导入从 managed/ 中抽离出的手写模块。",
        "# 不要在这里放长期手写逻辑；请在确认后手动折叠到 local.nix。",
        "",
        "{ lib, ... }:",
        "",
        "let",
        "  extractedDir = ./local-extracted;",
        "  extractedImports =",
        "    if builtins.pathExists extractedDir then",
        "      builtins.map (name: extractedDir + \"/${name}\") (",
        "        lib.sort lib.lessThan (",
        "          lib.filter (name: lib.hasSuffix \".nix\" name) (builtins.attrNames (builtins.readDir extractedDir))",
        "        )",
        "      )",
        "    else",
        "      [ ];",
        "in",
        "{",
        "  imports = extractedImports;",
        "}",
        "",
    ]
    .join("\n")
}

fn render_extracted_manual_module(source_tail: &str, content: &str) -> String {
    format!(
        "# mcbctl-extracted-from: managed/{source_tail}\n# review and fold this module into local.nix when convenient\n\n{}",
        content.trim_end()
    ) + "\n"
}

fn replacement_managed_body(rel: &str, kind: &str) -> Option<String> {
    match kind {
        "host-managed-default" => {
            if rel.starts_with("hosts/templates/") {
                Some(legacy_host_default_body_template())
            } else {
                Some(legacy_host_default_body())
            }
        }
        "host-users" => Some(legacy_host_placeholder_body(
            "用户结构",
            "mcbctl 的 Users 页保存时",
        )),
        "host-network" => Some(legacy_host_placeholder_body(
            "网络/TUN",
            "mcbctl 的 Hosts 页保存网络相关设置时",
        )),
        "host-gpu" => Some(legacy_host_placeholder_body(
            "GPU",
            "mcbctl 的 Hosts 页保存 GPU 相关设置时",
        )),
        "host-virtualization" => Some(legacy_host_placeholder_body(
            "虚拟化",
            "mcbctl 的 Hosts 页保存 Docker / Libvirt 设置时",
        )),
        "home-managed-default" => {
            if rel.starts_with("home/templates/users/") {
                Some(legacy_home_default_body_template())
            } else {
                Some(legacy_home_default_body_user())
            }
        }
        "home-packages-aggregator" => Some(legacy_home_packages_body()),
        "home-settings-default" => Some(legacy_home_settings_default_body()),
        "home-settings-desktop" => Some(legacy_home_settings_placeholder_body(
            "桌面",
            Some("mcbctl 的 Home 页保存桌面结构化设置时"),
        )),
        "home-settings-session" => Some(legacy_home_settings_placeholder_body(
            "session",
            Some("mcbctl 后续接入 session 相关结构化设置时"),
        )),
        "home-settings-mime" => Some(legacy_home_settings_placeholder_body(
            "MIME",
            Some("mcbctl 后续接入 MIME 相关结构化设置时"),
        )),
        _ if kind.starts_with("package-group:") => Some(render_empty_package_group_file(
            kind.trim_start_matches("package-group:"),
        )),
        _ => None,
    }
}

fn render_empty_package_group_file(group: &str) -> String {
    [
        format!("# 机器管理的软件组：{group}（由 mcbctl 维护）。"),
        "# 当前为空；若曾有手写逻辑，已抽离到 ../local-extracted/。".to_string(),
        "".to_string(),
        "{ ... }:".to_string(),
        "".to_string(),
        "{ }".to_string(),
        "".to_string(),
    ]
    .join("\n")
}

fn is_recognized_legacy_managed_content(rel: &str, content: &str) -> bool {
    if content.trim().is_empty() {
        return false;
    }

    let Some(kind) = expected_managed_kind(rel) else {
        return false;
    };

    let recognized_placeholder = match kind.as_str() {
        "host-users" => matches_legacy_placeholder_module(
            content,
            "# 机器管理的用户结构分片。",
            &["# 当前为空；当 mcbctl 的 Users 页保存时，会写入这里。"],
            false,
        ),
        "host-network" => matches_legacy_placeholder_module(
            content,
            "# 机器管理的网络/TUN 分片。",
            &["# 当前为空；当 mcbctl 的 Hosts 页保存网络相关设置时，会写入这里。"],
            false,
        ),
        "host-gpu" => matches_legacy_placeholder_module(
            content,
            "# 机器管理的 GPU 分片。",
            &["# 当前为空；当 mcbctl 的 Hosts 页保存 GPU 相关设置时，会写入这里。"],
            false,
        ),
        "host-virtualization" => matches_legacy_placeholder_module(
            content,
            "# 机器管理的虚拟化分片。",
            &["# 当前为空；当 mcbctl 的 Hosts 页保存 Docker / Libvirt 设置时，会写入这里。"],
            false,
        ),
        "home-settings-desktop" => matches_legacy_placeholder_module(
            content,
            "# 机器管理的桌面设置分片。",
            &[
                "# 当前为空；当 mcbctl 的 Home 页保存桌面结构化设置时，会写入这里。",
                "# 当前为空；当 mcbctl 后续接入对应页面时，会写入这里。",
            ],
            true,
        ),
        "home-settings-session" => matches_legacy_placeholder_module(
            content,
            "# 机器管理的 session 设置分片。",
            &[
                "# 当前为空；当 mcbctl 后续接入 session 相关结构化设置时，会写入这里。",
                "# 当前为空；当 mcbctl 后续接入对应页面时，会写入这里。",
            ],
            true,
        ),
        "home-settings-mime" => matches_legacy_placeholder_module(
            content,
            "# 机器管理的 MIME 设置分片。",
            &[
                "# 当前为空；当 mcbctl 后续接入 MIME 相关结构化设置时，会写入这里。",
                "# 当前为空；当 mcbctl 后续接入对应页面时，会写入这里。",
            ],
            true,
        ),
        _ => false,
    };
    if recognized_placeholder {
        return true;
    }

    let static_matches = legacy_static_variants(&kind)
        .iter()
        .any(|expected| trimmed_content_eq(content, expected));
    if static_matches {
        return true;
    }

    kind.starts_with("package-group:")
        && content.trim_start().starts_with("# 机器管理的软件组：")
        && content.contains("home.packages = [")
        && content.contains("# managed-id:")
}

fn trimmed_content_eq(content: &str, expected: &str) -> bool {
    content.trim_end() == expected.trim_end()
}

fn matches_legacy_placeholder_module(
    content: &str,
    title: &str,
    details: &[&str],
    allow_without_detail: bool,
) -> bool {
    if allow_without_detail
        && trimmed_content_eq(content, &format!("{title}\n\n{{ ... }}:\n\n{{ }}\n"))
    {
        return true;
    }

    details.iter().any(|detail| {
        trimmed_content_eq(
            content,
            &format!("{title}\n{detail}\n\n{{ ... }}:\n\n{{ }}\n"),
        )
    })
}

fn legacy_static_variants(kind: &str) -> Vec<String> {
    match kind {
        "host-managed-default" => vec![
            legacy_host_default_body(),
            legacy_host_default_body_template(),
        ],
        "host-users" => vec![legacy_host_placeholder_body(
            "用户结构",
            "mcbctl 的 Users 页保存时",
        )],
        "host-network" => vec![legacy_host_placeholder_body(
            "网络/TUN",
            "mcbctl 的 Hosts 页保存网络相关设置时",
        )],
        "host-gpu" => vec![legacy_host_placeholder_body(
            "GPU",
            "mcbctl 的 Hosts 页保存 GPU 相关设置时",
        )],
        "host-virtualization" => vec![legacy_host_placeholder_body(
            "虚拟化",
            "mcbctl 的 Hosts 页保存 Docker / Libvirt 设置时",
        )],
        "home-managed-default" => vec![
            legacy_home_default_body_template(),
            legacy_home_default_body_user(),
        ],
        "home-packages-aggregator" => vec![legacy_home_packages_body()],
        "home-settings-default" => vec![legacy_home_settings_default_body()],
        "home-settings-desktop" => vec![
            legacy_home_settings_placeholder_body("桌面", None),
            legacy_home_settings_placeholder_body(
                "桌面",
                Some("mcbctl 的 Home 页保存桌面结构化设置时"),
            ),
            legacy_home_settings_placeholder_body("桌面", Some("mcbctl 后续接入对应页面时")),
        ],
        "home-settings-session" => vec![
            legacy_home_settings_placeholder_body("session", None),
            legacy_home_settings_placeholder_body(
                "session",
                Some("mcbctl 后续接入 session 相关结构化设置时"),
            ),
            legacy_home_settings_placeholder_body("session", Some("mcbctl 后续接入对应页面时")),
        ],
        "home-settings-mime" => vec![
            legacy_home_settings_placeholder_body("MIME", None),
            legacy_home_settings_placeholder_body(
                "MIME",
                Some("mcbctl 后续接入 MIME 相关结构化设置时"),
            ),
            legacy_home_settings_placeholder_body("mime", Some("mcbctl 后续接入对应页面时")),
        ],
        _ => Vec::new(),
    }
}

fn legacy_host_default_body() -> String {
    [
        "# TUI / 自动化工具专用主机入口。",
        "",
        "{ lib, ... }:",
        "",
        "let",
        "  splitImports = lib.concatLists [",
        "    (lib.optional (builtins.pathExists ./users.nix) ./users.nix)",
        "    (lib.optional (builtins.pathExists ./network.nix) ./network.nix)",
        "    (lib.optional (builtins.pathExists ./gpu.nix) ./gpu.nix)",
        "    (lib.optional (builtins.pathExists ./virtualization.nix) ./virtualization.nix)",
        "  ];",
        "in",
        "{",
        "  imports = splitImports ++ lib.optional (builtins.pathExists ./local.nix) ./local.nix;",
        "}",
        "",
    ]
    .join("\n")
}

fn legacy_host_default_body_template() -> String {
    [
        "# TUI / 自动化工具专用主机入口。",
        "",
        "{ lib, ... }:",
        "",
        "{",
        "  imports = [ ] ++ lib.optional (builtins.pathExists ./local.nix) ./local.nix;",
        "}",
        "",
    ]
    .join("\n")
}

fn legacy_host_placeholder_body(title: &str, detail: &str) -> String {
    [
        format!("# 机器管理的 {title} 分片。"),
        format!("# 当前为空；当 {detail}，会写入这里。"),
        "".to_string(),
        "{ ... }:".to_string(),
        "".to_string(),
        "{ }".to_string(),
        "".to_string(),
    ]
    .join("\n")
}

fn legacy_home_default_body_template() -> String {
    [
        "# TUI / 自动化工具专用入口。",
        "",
        "{ lib, ... }:",
        "",
        "{",
        "  imports = [",
        "    ./packages.nix",
        "  ]",
        "  ++ lib.optional (builtins.pathExists ./settings/default.nix) ./settings/default.nix",
        "  ++ lib.optional (builtins.pathExists ./local.nix) ./local.nix;",
        "}",
        "",
    ]
    .join("\n")
}

fn legacy_home_default_body_user() -> String {
    [
        "# TUI / 自动化工具专用入口。",
        "# 约定：机器写入的用户级改动只落在 managed/，不要直接改手写 packages.nix / config/。",
        "",
        "{ lib, ... }:",
        "",
        "{",
        "  imports = [",
        "    ./packages.nix",
        "  ]",
        "  ++ lib.optional (builtins.pathExists ./settings/default.nix) ./settings/default.nix",
        "  ++ lib.optional (builtins.pathExists ./local.nix) ./local.nix;",
        "}",
        "",
    ]
    .join("\n")
}

fn legacy_home_packages_body() -> String {
    [
        "# 机器管理的用户软件入口（由 mcbctl 维护）。",
        "# 说明：真正的软件组会按文件写入 ./packages/*.nix，这里只负责聚合导入。",
        "",
        "{ lib, ... }:",
        "",
        "let",
        "  packageDir = ./packages;",
        "  packageImports =",
        "    if builtins.pathExists packageDir then",
        "      builtins.map (name: packageDir + \"/${name}\") (",
        "        lib.sort lib.lessThan (",
        "          lib.filter (name: lib.hasSuffix \".nix\" name) (builtins.attrNames (builtins.readDir packageDir))",
        "        )",
        "      )",
        "    else",
        "      [ ];",
        "in",
        "{",
        "  imports = packageImports;",
        "}",
        "",
    ]
    .join("\n")
}

fn legacy_home_settings_default_body() -> String {
    [
        "# 机器管理的用户设置聚合入口。",
        "",
        "{ lib, ... }:",
        "",
        "let",
        "  splitImports = lib.concatLists [",
        "    (lib.optional (builtins.pathExists ./desktop.nix) ./desktop.nix)",
        "    (lib.optional (builtins.pathExists ./session.nix) ./session.nix)",
        "    (lib.optional (builtins.pathExists ./mime.nix) ./mime.nix)",
        "  ];",
        "in",
        "{",
        "  imports = splitImports;",
        "}",
        "",
    ]
    .join("\n")
}

fn legacy_home_settings_placeholder_body(title: &str, detail: Option<&str>) -> String {
    let mut lines = vec![format!("# 机器管理的 {title} 设置分片。")];
    if let Some(detail) = detail {
        lines.push(format!("# 当前为空；当 {detail}，会写入这里。"));
    }
    lines.extend([
        "".to_string(),
        "{ ... }:".to_string(),
        "".to_string(),
        "{ }".to_string(),
        "".to_string(),
    ]);
    lines.join("\n")
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

    #[test]
    fn flags_legacy_managed_files() -> Result<()> {
        let root = create_temp_repo()?;
        let path = root.join("hosts/demo/managed/network.nix");
        fs::create_dir_all(path.parent().expect("managed dir"))?;
        fs::write(
            &path,
            "# 机器管理的网络/TUN 分片。\n# 当前为空；当 mcbctl 的 Hosts 页保存网络相关设置时，会写入这里。\n\n{ ... }:\n\n{ }\n",
        )?;

        let report = audit_repository(&root)?;
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.rule == "managed-protocol"
                    && finding.path == "hosts/demo/managed/network.nix")
        );
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn migrate_managed_files_wraps_recognized_legacy_content() -> Result<()> {
        let root = create_temp_repo()?;
        let path = root.join("hosts/demo/managed/network.nix");
        fs::create_dir_all(path.parent().expect("managed dir"))?;
        let legacy = "# 机器管理的网络/TUN 分片。\n# 当前为空；当 mcbctl 的 Hosts 页保存网络相关设置时，会写入这里。\n\n{ ... }:\n\n{ }\n";
        fs::write(&path, legacy)?;

        let report = migrate_managed_files(&root)?;
        assert_eq!(report.migrated, vec!["hosts/demo/managed/network.nix"]);
        let content = fs::read_to_string(&path)?;
        assert_eq!(managed_file_kind(&content), Some("host-network"));
        assert!(managed_file_is_valid(&content));

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn migrate_managed_files_includes_host_templates() -> Result<()> {
        let root = create_temp_repo()?;
        let path = root.join("hosts/templates/laptop/managed/default.nix");
        fs::create_dir_all(path.parent().expect("managed dir"))?;
        fs::write(&path, legacy_host_default_body())?;

        let report = migrate_managed_files(&root)?;
        assert_eq!(
            report.migrated,
            vec!["hosts/templates/laptop/managed/default.nix"]
        );
        let content = fs::read_to_string(&path)?;
        assert_eq!(managed_file_kind(&content), Some("host-managed-default"));
        assert!(managed_file_is_valid(&content));

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn migrate_managed_files_rejects_unknown_content() -> Result<()> {
        let root = create_temp_repo()?;
        let path = root.join("home/users/demo/managed/settings/desktop.nix");
        fs::create_dir_all(path.parent().expect("settings dir"))?;
        fs::write(
            &path,
            "{ lib, ... }: { mcb.noctalia.barProfile = lib.mkForce \"default\"; }\n",
        )?;

        let err =
            migrate_managed_files(&root).expect_err("manual content should not be auto-migrated");
        assert!(
            err.to_string()
                .contains("does not match a recognized legacy managed file")
        );

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn extract_manual_managed_files_moves_content_out_of_managed() -> Result<()> {
        let root = create_temp_repo()?;
        let source = root.join("hosts/demo/managed/network.nix");
        fs::create_dir_all(source.parent().expect("managed dir"))?;
        fs::write(
            &source,
            "{ lib, ... }: { networking.useDHCP = lib.mkForce false; }\n",
        )?;

        let report = extract_manual_managed_files(&root)?;
        assert_eq!(report.extracted, vec!["hosts/demo/managed/network.nix"]);

        let extracted = root.join("hosts/demo/local-extracted/managed-network.nix");
        assert!(extracted.is_file());
        let local_auto = root.join("hosts/demo/local.auto.nix");
        assert_eq!(fs::read_to_string(local_auto)?, render_local_auto_file());

        let replacement = fs::read_to_string(&source)?;
        assert_eq!(managed_file_kind(&replacement), Some("host-network"));
        assert!(managed_file_is_valid(&replacement));

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn migrate_root_hardware_config_moves_into_host_directory() -> Result<()> {
        let root = create_temp_repo()?;
        let source = root.join("hardware-configuration.nix");
        fs::create_dir_all(root.join("hosts/demo"))?;
        fs::write(&source, "{ ... }: { }\n")?;

        let report = migrate_root_hardware_config(&root, "demo")?;
        assert!(report.moved);
        assert!(!source.exists());
        assert!(root.join("hosts/demo/hardware-configuration.nix").is_file());

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn flags_legacy_root_hardware_configuration_path() -> Result<()> {
        let root = create_temp_repo()?;
        fs::write(root.join("hardware-configuration.nix"), "{ ... }: { }\n")?;

        let report = audit_repository(&root)?;
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.rule == "legacy-hardware-path")
        );

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
