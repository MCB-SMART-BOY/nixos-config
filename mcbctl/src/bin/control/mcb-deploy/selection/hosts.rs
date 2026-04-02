use super::*;

impl App {
    pub(crate) fn list_hosts(&self, repo_dir: &Path) -> Vec<String> {
        let mut hosts = Vec::new();
        let host_dir = repo_dir.join("hosts");
        if host_dir.is_dir()
            && let Ok(entries) = fs::read_dir(host_dir)
        {
            for entry in entries.flatten() {
                if !entry.path().is_dir() {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().to_string();
                if name != "profiles" && name != "templates" {
                    hosts.push(name);
                }
            }
        }
        hosts.sort();
        hosts
    }

    pub(crate) fn host_exists(&self, repo_dir: &Path) -> bool {
        !self.target_name.is_empty() && repo_dir.join("hosts").join(&self.target_name).is_dir()
    }

    pub(crate) fn resolve_host_template(&self, repo_dir: &Path) -> Option<(String, PathBuf)> {
        let template_name = match self.host_profile_kind {
            HostProfileKind::Server => "server",
            HostProfileKind::Desktop => "laptop",
            HostProfileKind::Unknown => return None,
        };
        let template_dir = repo_dir.join("hosts/templates").join(template_name);
        if template_dir.is_dir() {
            return Some((format!("hosts/templates/{template_name}"), template_dir));
        }
        None
    }

    pub(crate) fn prompt_new_host_name(
        &self,
        repo_dir: &Path,
        template_label: &str,
    ) -> Result<Option<String>> {
        loop {
            print!("输入新主机名（模板：{template_label}，留空取消）： ");
            io::stdout().flush().ok();
            let mut input = String::new();
            io::stdin().read_line(&mut input).ok();
            let input = input.trim();
            if input.is_empty() {
                return Ok(None);
            }
            if !is_valid_host_name(input) {
                self.warn(&format!("主机名不合法：{input}"));
                continue;
            }
            let reserved = ["profiles", "templates"];
            if reserved.contains(&input) {
                self.warn(&format!("主机名保留不可用：{input}"));
                continue;
            }
            if repo_dir.join("hosts").join(input).exists() {
                self.warn(&format!("主机已存在：hosts/{input}"));
                continue;
            }
            return Ok(Some(input.to_string()));
        }
    }

    pub(crate) fn select_host(&mut self, repo_dir: &Path) -> Result<()> {
        if !self.target_name.is_empty() {
            return Ok(());
        }
        self.host_profile_kind = HostProfileKind::Unknown;
        if self.is_tty() {
            loop {
                let hosts = self.list_hosts(repo_dir);
                let mut options = Vec::<String>::new();
                let has_existing_hosts = !hosts.is_empty();
                if has_existing_hosts {
                    options.push("使用已有主机".to_string());
                }
                if self.deploy_mode != DeployMode::UpdateExisting {
                    options.push("新建桌面主机（从模板）".to_string());
                    options.push("新建服务器主机（从模板）".to_string());
                }
                options.push("退出".to_string());
                let pick = self.menu_prompt("选择主机来源", 1, &options)?;

                let mut cursor = 1usize;
                if has_existing_hosts {
                    if pick == cursor {
                        let mut default_index = 1usize;
                        for (i, h) in hosts.iter().enumerate() {
                            if h == "nixos" {
                                default_index = i + 1;
                                break;
                            }
                        }
                        let host_pick = self.menu_prompt("选择已有主机", default_index, &hosts)?;
                        self.target_name = hosts[host_pick - 1].clone();
                        return Ok(());
                    }
                    cursor += 1;
                }
                if self.deploy_mode != DeployMode::UpdateExisting {
                    if pick == cursor {
                        self.host_profile_kind = HostProfileKind::Desktop;
                        if let Some(name) = self.prompt_new_host_name(repo_dir, "desktop")? {
                            self.target_name = name;
                            return Ok(());
                        }
                        self.host_profile_kind = HostProfileKind::Unknown;
                        continue;
                    }
                    cursor += 1;
                    if pick == cursor {
                        self.host_profile_kind = HostProfileKind::Server;
                        if let Some(name) = self.prompt_new_host_name(repo_dir, "server")? {
                            self.target_name = name;
                            return Ok(());
                        }
                        self.host_profile_kind = HostProfileKind::Unknown;
                        continue;
                    }
                }
                bail!("已退出");
            }
        } else {
            self.target_name = "nixos".to_string();
        }
        Ok(())
    }

    pub(crate) fn validate_host(&self, repo_dir: &Path) -> Result<()> {
        if self.target_name.is_empty() {
            bail!("未指定主机名称。");
        }
        if self.host_exists(repo_dir) {
            return Ok(());
        }
        if self.deploy_mode == DeployMode::UpdateExisting {
            bail!("仅更新模式不允许创建新主机：hosts/{}", self.target_name);
        }
        if self.resolve_host_template(repo_dir).is_none() {
            bail!(
                "主机不存在：hosts/{}，且未找到可用的主机模板。",
                self.target_name
            );
        }
        Ok(())
    }

    pub(crate) fn detect_host_profile_kind(&mut self, repo_dir: &Path) {
        self.host_profile_kind = HostProfileKind::Unknown;
        let host_file = repo_dir
            .join("hosts")
            .join(&self.target_name)
            .join("default.nix");
        if let Ok(text) = fs::read_to_string(host_file) {
            if text.contains("../profiles/server.nix") {
                self.host_profile_kind = HostProfileKind::Server;
            } else if text.contains("../profiles/desktop.nix") {
                self.host_profile_kind = HostProfileKind::Desktop;
            }
        }
    }

    pub(crate) fn detect_per_user_tun(&self, repo_dir: &Path) -> bool {
        if command_exists("nix") {
            let mut nix_config = "experimental-features = nix-command flakes".to_string();
            if let Ok(extra) = std::env::var("NIX_CONFIG")
                && !extra.trim().is_empty()
            {
                nix_config = format!("{extra}\n{nix_config}");
            }
            let target = format!(
                "{}#nixosConfigurations.{}.config.mcb.perUserTun.enable",
                repo_dir.display(),
                self.target_name
            );
            let out = Command::new("nix")
                .env("NIX_CONFIG", nix_config)
                .args(["eval", "--raw", &target])
                .output();
            if let Ok(out) = out
                && out.status.success()
            {
                let v = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if v == "true" {
                    return true;
                }
                if v == "false" {
                    return false;
                }
            }
        }

        let files = vec![
            repo_dir
                .join("hosts")
                .join(&self.target_name)
                .join("local.nix"),
            repo_dir
                .join("hosts")
                .join(&self.target_name)
                .join("default.nix"),
        ];
        for file in files {
            let Ok(text) = fs::read_to_string(file) else {
                continue;
            };
            if text
                .lines()
                .map(strip_comment)
                .any(|l| l.contains("mcb.perUserTun.enable") && l.contains("true"))
            {
                return true;
            }
            let mut in_block = false;
            for line in text.lines().map(strip_comment) {
                if line.contains("perUserTun") && line.contains('{') {
                    in_block = true;
                }
                if in_block && line.contains("enable") && line.contains("true") {
                    return true;
                }
                if in_block && line.contains('}') {
                    in_block = false;
                }
            }
        }
        false
    }
}
