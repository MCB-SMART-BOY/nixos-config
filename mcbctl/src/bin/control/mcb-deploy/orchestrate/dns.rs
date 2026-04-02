use super::super::*;

impl App {
    fn detect_default_iface(&self) -> Option<String> {
        let out = run_capture_allow_fail("ip", &["route", "show", "default"])?;
        let line = out.lines().next()?;
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() >= 5 {
            Some(cols[4].to_string())
        } else {
            None
        }
    }

    pub(crate) fn temp_dns_enable(&mut self) -> Result<bool> {
        let servers = vec!["223.5.5.5".to_string(), "223.6.6.6".to_string()];
        if self.rootless {
            self.warn("rootless 模式无法临时设置 DNS，跳过。");
            return Ok(false);
        }

        if command_exists("resolvectl") && command_exists("systemctl") {
            let active = Command::new("systemctl")
                .args(["is-active", "--quiet", "systemd-resolved"])
                .status()
                .ok()
                .is_some_and(|s| s.success());
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

    pub(crate) fn temp_dns_disable(&mut self) {
        if self.temp_dns_backend == "resolvectl" {
            if !self.temp_dns_iface.is_empty() {
                self.log(&format!("恢复 DNS（resolvectl {}）", self.temp_dns_iface));
                let _ = self.run_as_root_inherit(
                    "resolvectl",
                    &["revert".to_string(), self.temp_dns_iface.clone()],
                );
                let _ = self.run_as_root_inherit("resolvectl", &["flush-caches".to_string()]);
            }
        } else if self.temp_dns_backend == "resolv.conf"
            && let Some(backup) = &self.temp_dns_backup
            && backup.is_file()
        {
            self.log("恢复 /etc/resolv.conf");
            let _ = self.run_as_root_inherit(
                "cp",
                &[
                    "-a".to_string(),
                    backup.display().to_string(),
                    "/etc/resolv.conf".to_string(),
                ],
            );
            fs::remove_file(backup).ok();
        }
        self.temp_dns_backend.clear();
        self.temp_dns_iface.clear();
        self.temp_dns_backup = None;
    }
}
