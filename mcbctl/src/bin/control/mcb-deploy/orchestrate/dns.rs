use super::super::*;

fn summarize_cleanup_failures(context: &str, failures: &[String]) -> String {
    if failures.is_empty() {
        return context.to_string();
    }

    format!("{context}: {}", failures.join(" | "))
}

fn systemd_resolved_is_active() -> Result<bool> {
    let status = Command::new("systemctl")
        .args(["is-active", "--quiet", "systemd-resolved"])
        .status()
        .context("failed to run systemctl is-active systemd-resolved")?;
    Ok(status.success())
}

impl App {
    fn detect_default_iface(&self) -> Option<String> {
        let out = run_capture_allow_fail("ip", &["route", "show", "default"])?;
        parse_default_iface_from_route_output(&out)
    }

    pub(crate) fn temp_dns_enable(&mut self) -> Result<bool> {
        let servers = vec!["223.5.5.5".to_string(), "223.6.6.6".to_string()];
        if self.rootless {
            self.warn("rootless 模式无法临时设置 DNS，跳过。");
            return Ok(false);
        }

        if command_exists("resolvectl") && command_exists("systemctl") {
            let active = systemd_resolved_is_active()?;
            if active && let Some(iface) = self.detect_default_iface() {
                self.log(&format!(
                    "临时 DNS（resolvectl {}）：{}",
                    iface,
                    servers.join(" ")
                ));
                let mut dns_args = vec!["dns".to_string(), iface.clone()];
                dns_args.extend(servers.clone());
                self.run_as_root_ok("resolvectl", &dns_args)?;
                self.run_as_root_ok(
                    "resolvectl",
                    &["domain".to_string(), iface.clone(), "~.".to_string()],
                )?;
                self.temp_dns_backend = "resolvectl".to_string();
                self.temp_dns_iface = iface;
                self.dns_enabled = true;
                return Ok(true);
            }
        }

        let resolv = PathBuf::from("/etc/resolv.conf");
        if resolv.exists() {
            let backup = create_temp_path("mcbctl-resolv", "conf")?;
            self.run_as_root_ok(
                "cp",
                &[
                    "-a".to_string(),
                    "/etc/resolv.conf".to_string(),
                    backup.display().to_string(),
                ],
            )?;
            self.run_as_root_ok("rm", &["-f".to_string(), "/etc/resolv.conf".to_string()])?;

            let content_file = create_temp_path("mcbctl-resolv-new", "conf")?;
            let content = servers
                .iter()
                .map(|s| format!("nameserver {s}"))
                .collect::<Vec<_>>()
                .join("\n")
                + "\n";
            fs::write(&content_file, content)?;
            self.run_as_root_ok(
                "cp",
                &[
                    "-a".to_string(),
                    content_file.display().to_string(),
                    "/etc/resolv.conf".to_string(),
                ],
            )?;
            fs::remove_file(content_file).ok();

            self.log(&format!(
                "临时 DNS（/etc/resolv.conf）：{}",
                servers.join(" ")
            ));
            self.temp_dns_backend = "resolv.conf".to_string();
            self.temp_dns_backup = Some(backup);
            self.dns_enabled = true;
            return Ok(true);
        }

        bail!("无法设置临时 DNS（无 resolvectl 且缺少 /etc/resolv.conf）。")
    }

    pub(crate) fn temp_dns_disable(&mut self) -> Result<()> {
        let mut failures = Vec::new();
        if self.temp_dns_backend == "resolvectl" {
            if !self.temp_dns_iface.is_empty() {
                self.log(&format!("恢复 DNS（resolvectl {}）", self.temp_dns_iface));
                match self.run_as_root_inherit(
                    "resolvectl",
                    &["revert".to_string(), self.temp_dns_iface.clone()],
                ) {
                    Ok(status) if status.success() => {}
                    Ok(status) => failures.push(format!(
                        "resolvectl revert {} failed with {}",
                        self.temp_dns_iface,
                        status.code().unwrap_or(1)
                    )),
                    Err(err) => failures.push(format!(
                        "resolvectl revert {} failed: {err}",
                        self.temp_dns_iface
                    )),
                }
                match self.run_as_root_inherit("resolvectl", &["flush-caches".to_string()]) {
                    Ok(status) if status.success() => {}
                    Ok(status) => failures.push(format!(
                        "resolvectl flush-caches failed with {}",
                        status.code().unwrap_or(1)
                    )),
                    Err(err) => failures.push(format!("resolvectl flush-caches failed: {err}")),
                }
            }
        } else if self.temp_dns_backend == "resolv.conf"
            && let Some(backup) = &self.temp_dns_backup
            && backup.is_file()
        {
            self.log("恢复 /etc/resolv.conf");
            match self.run_as_root_inherit(
                "cp",
                &[
                    "-a".to_string(),
                    backup.display().to_string(),
                    "/etc/resolv.conf".to_string(),
                ],
            ) {
                Ok(status) if status.success() => {}
                Ok(status) => failures.push(format!(
                    "restore /etc/resolv.conf failed with {}",
                    status.code().unwrap_or(1)
                )),
                Err(err) => failures.push(format!("restore /etc/resolv.conf failed: {err}")),
            }
            if let Err(err) = fs::remove_file(backup) {
                failures.push(format!(
                    "remove DNS backup {} failed: {err}",
                    backup.display()
                ));
            }
        }
        self.temp_dns_backend.clear();
        self.temp_dns_iface.clear();
        self.temp_dns_backup = None;
        if failures.is_empty() {
            Ok(())
        } else {
            bail!(
                "{}",
                summarize_cleanup_failures("恢复临时 DNS 失败", &failures)
            )
        }
    }
}

fn parse_default_iface_from_route_output(output: &str) -> Option<String> {
    let line = output.lines().next()?;
    let cols: Vec<&str> = line.split_whitespace().collect();
    if cols.len() >= 5 && cols.get(3) == Some(&"dev") {
        Some(cols[4].to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_default_iface_from_route_output_extracts_device() {
        let iface = parse_default_iface_from_route_output(
            "default via 192.168.1.1 dev wlan0 proto dhcp src 192.168.1.9 metric 600\n",
        );
        assert_eq!(iface.as_deref(), Some("wlan0"));
    }

    #[test]
    fn parse_default_iface_from_route_output_rejects_malformed_lines() {
        assert_eq!(parse_default_iface_from_route_output(""), None);
        assert_eq!(
            parse_default_iface_from_route_output("default via 192.168.1.1 proto dhcp"),
            None
        );
    }

    #[test]
    fn summarize_cleanup_failures_joins_messages() {
        let summary = summarize_cleanup_failures(
            "恢复临时 DNS 失败",
            &[
                "resolvectl revert failed".to_string(),
                "remove backup failed".to_string(),
            ],
        );

        assert!(summary.contains("恢复临时 DNS 失败"));
        assert!(summary.contains("resolvectl revert failed"));
        assert!(summary.contains("remove backup failed"));
    }

    #[test]
    fn summarize_cleanup_failures_without_details_returns_context() {
        assert_eq!(
            summarize_cleanup_failures("恢复临时 DNS 失败", &[]),
            "恢复临时 DNS 失败"
        );
    }
}
