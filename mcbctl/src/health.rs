use crate::repo::{audit_repository, preferred_remote_branch};
use crate::store::context::detect_hostname;
use crate::store::deploy::host_hardware_config_path;
use crate::{command_exists, run_capture_allow_fail};
use anyhow::{Result, bail};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DoctorToolStatus {
    pub git: bool,
    pub nix: bool,
    pub nixos_rebuild: bool,
    pub cargo: bool,
}

impl DoctorToolStatus {
    pub fn detect() -> Self {
        Self {
            git: command_exists("git"),
            nix: command_exists("nix"),
            nixos_rebuild: command_exists("nixos-rebuild"),
            cargo: command_exists("cargo"),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DoctorAssessment {
    pub blocking_issues: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct DoctorReport {
    pub repo_root: PathBuf,
    pub remote_branch: String,
    pub tools: DoctorToolStatus,
    pub repo_hardware: String,
    pub legacy_root_hardware: bool,
    pub current_user: String,
    pub current_uid: String,
    pub layout_error: Option<String>,
    pub integrity_clean: bool,
    pub integrity_lines: Vec<String>,
    pub assessment: DoctorAssessment,
}

impl DoctorReport {
    pub fn is_healthy(&self) -> bool {
        self.integrity_clean
            && self.layout_error.is_none()
            && self.assessment.blocking_issues.is_empty()
    }

    pub fn failure_lines(&self) -> Vec<String> {
        let mut failures = Vec::new();
        if !self.integrity_clean {
            if self.integrity_lines.len() > 1 {
                failures.extend(self.integrity_lines.iter().skip(1).cloned());
            } else {
                failures.push("repo integrity: failed".to_string());
            }
        }
        if let Some(err) = &self.layout_error {
            failures.push(format!("repo layout: {err}"));
        }
        if !self.assessment.blocking_issues.is_empty() {
            failures.push(format!(
                "deployment environment: {}",
                self.assessment.blocking_issues.join("；")
            ));
        }
        failures
    }
}

pub fn ensure_required_layout(root: &Path) -> Result<()> {
    for rel in [
        "flake.nix",
        "mcbctl/Cargo.toml",
        "hosts",
        "hosts/templates",
        "modules",
        "home",
        "home/templates/users",
        "catalog",
        "pkgs",
        "pkgs/mcbctl/default.nix",
    ] {
        let path = root.join(rel);
        if !path.exists() {
            bail!("缺少必需的仓库边界：{}", path.display());
        }
    }
    Ok(())
}

pub fn assess_doctor_environment(tools: DoctorToolStatus) -> DoctorAssessment {
    let mut assessment = DoctorAssessment::default();
    if !tools.nix {
        assessment
            .blocking_issues
            .push("缺少 nix，无法求值或构建 flake。".to_string());
    }
    if !tools.nixos_rebuild {
        assessment
            .warnings
            .push("缺少 nixos-rebuild，本机无法直接部署或重建系统。".to_string());
    }
    if !tools.git {
        assessment
            .warnings
            .push("缺少 git，远端来源、release 与部分更新流程不可用。".to_string());
    }
    if !tools.cargo {
        assessment
            .warnings
            .push("缺少 cargo，本机无法直接运行 Rust 开发检查。".to_string());
    }
    assessment
}

pub fn collect_doctor_report(root: &Path) -> Result<DoctorReport> {
    let integrity_report = audit_repository(root)?;
    let layout_error = ensure_required_layout(root)
        .err()
        .map(|err| err.to_string());
    let tools = DoctorToolStatus::detect();
    let mut assessment = assess_doctor_environment(tools);
    let current_host = detect_hostname();
    let repo_hardware = if !current_host.is_empty()
        && root.join("hosts").join(&current_host).is_dir()
    {
        let path = host_hardware_config_path(root, &current_host);
        if path.is_file() {
            format!("present ({})", path.display())
        } else {
            assessment.warnings.push(format!(
                    "缺少 {current_host} 的 hardware-configuration.nix；switch/test/boot 不可用，当前仅允许 build 和求值。"
                ));
            format!("missing for {current_host} (eval fallback active)")
        }
    } else {
        "unknown (current host not mapped into repo)".to_string()
    };

    Ok(DoctorReport {
        repo_root: root.to_path_buf(),
        remote_branch: preferred_remote_branch(root),
        tools,
        repo_hardware,
        legacy_root_hardware: root.join("hardware-configuration.nix").is_file(),
        current_user: run_capture_allow_fail("id", &["-un"])
            .map(|user| user.trim().to_string())
            .filter(|user| !user.is_empty())
            .unwrap_or_else(|| "unknown".to_string()),
        current_uid: run_capture_allow_fail("id", &["-u"])
            .map(|uid| uid.trim().to_string())
            .filter(|uid| !uid.is_empty())
            .unwrap_or_else(|| "unknown".to_string()),
        layout_error,
        integrity_clean: integrity_report.is_clean(),
        integrity_lines: integrity_report.render_lines(),
        assessment,
    })
}

pub fn tool_status_label(is_present: bool) -> &'static str {
    if is_present { "ok" } else { "missing" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assess_doctor_environment_requires_nix_and_warns_for_nixos_rebuild() {
        let assessment = assess_doctor_environment(DoctorToolStatus {
            git: true,
            nix: false,
            nixos_rebuild: false,
            cargo: true,
        });

        assert_eq!(assessment.blocking_issues.len(), 1);
        assert!(
            assessment
                .blocking_issues
                .iter()
                .any(|issue| issue.contains("缺少 nix"))
        );
        assert!(
            assessment
                .warnings
                .iter()
                .any(|warning| warning.contains("缺少 nixos-rebuild"))
        );
        assert_eq!(assessment.warnings.len(), 1);
    }

    #[test]
    fn assess_doctor_environment_only_warns_for_git_and_cargo() {
        let assessment = assess_doctor_environment(DoctorToolStatus {
            git: false,
            nix: true,
            nixos_rebuild: true,
            cargo: false,
        });

        assert!(assessment.blocking_issues.is_empty());
        assert_eq!(assessment.warnings.len(), 2);
        assert!(
            assessment
                .warnings
                .iter()
                .any(|warning| warning.contains("缺少 git"))
        );
        assert!(
            assessment
                .warnings
                .iter()
                .any(|warning| warning.contains("缺少 cargo"))
        );
    }

    #[test]
    fn doctor_report_failure_lines_include_integrity_layout_and_environment() {
        let report = DoctorReport {
            repo_root: PathBuf::from("/repo"),
            remote_branch: "main".to_string(),
            tools: DoctorToolStatus::default(),
            repo_hardware: "missing".to_string(),
            legacy_root_hardware: false,
            current_user: "alice".to_string(),
            current_uid: "1000".to_string(),
            layout_error: Some("layout broken".to_string()),
            integrity_clean: false,
            integrity_lines: vec![
                "repository integrity check failed".to_string(),
                "- [rule] path: detail".to_string(),
            ],
            assessment: DoctorAssessment {
                blocking_issues: vec!["缺少 nix".to_string()],
                warnings: Vec::new(),
            },
        };

        assert_eq!(
            report.failure_lines(),
            vec![
                "- [rule] path: detail".to_string(),
                "repo layout: layout broken".to_string(),
                "deployment environment: 缺少 nix".to_string(),
            ]
        );
        assert!(!report.is_healthy());
    }
}
