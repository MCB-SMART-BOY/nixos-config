use super::*;

const RESERVED_HOST_DIRS: &[&str] = &["profiles", "templates", "_support"];

fn is_visible_host_dir_name(name: &str) -> bool {
    !RESERVED_HOST_DIRS.contains(&name)
}

fn default_existing_host_name(hosts: &[String]) -> Option<&str> {
    hosts
        .iter()
        .find(|host| host.as_str() == "nixos")
        .map(String::as_str)
        .or_else(|| hosts.first().map(String::as_str))
}

fn default_existing_host_index(hosts: &[String]) -> usize {
    hosts
        .iter()
        .position(|host| host == "nixos")
        .map_or(1, |index| index + 1)
}

fn list_visible_hosts(repo_dir: &Path) -> Vec<String> {
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
            if is_visible_host_dir_name(&name) {
                hosts.push(name);
            }
        }
    }
    hosts.sort();
    hosts
}

impl App {
    pub(crate) fn list_hosts(&self, repo_dir: &Path) -> Vec<String> {
        list_visible_hosts(repo_dir)
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
            let input = self.prompt_line(&format!(
                "输入新主机名（模板：{template_label}，留空取消）： "
            ))?;
            let input = input.trim();
            if input.is_empty() {
                return Ok(None);
            }
            if !is_valid_host_name(input) {
                self.warn(&format!("主机名不合法：{input}"));
                continue;
            }
            if RESERVED_HOST_DIRS.contains(&input) {
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
                        let default_index = default_existing_host_index(&hosts);
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
            self.target_name = default_existing_host_name(&self.list_hosts(repo_dir))
                .map(str::to_string)
                .context("非交互模式下无法推断目标主机；请先准备至少一个 hosts/<name> 目录")?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn default_existing_host_name_prefers_nixos_then_first_visible() {
        let hosts = vec!["alpha".to_string(), "nixos".to_string(), "zeta".to_string()];
        assert_eq!(default_existing_host_name(&hosts), Some("nixos"));

        let hosts = vec!["alpha".to_string(), "zeta".to_string()];
        assert_eq!(default_existing_host_name(&hosts), Some("alpha"));

        let hosts = Vec::<String>::new();
        assert_eq!(default_existing_host_name(&hosts), None);
    }

    #[test]
    fn default_existing_host_index_prefers_nixos() {
        let hosts = vec!["alpha".to_string(), "nixos".to_string(), "zeta".to_string()];
        assert_eq!(default_existing_host_index(&hosts), 2);

        let hosts = vec!["alpha".to_string(), "zeta".to_string()];
        assert_eq!(default_existing_host_index(&hosts), 1);
    }

    #[test]
    fn list_hosts_excludes_reserved_directories() -> Result<()> {
        let repo_dir = create_temp_repo_dir("mcbctl-deploy-host-list")?;
        for name in ["nixos", "alpha", "_support", "profiles", "templates"] {
            fs::create_dir_all(repo_dir.join("hosts").join(name))?;
        }

        let hosts = list_visible_hosts(&repo_dir);

        assert_eq!(hosts, vec!["alpha".to_string(), "nixos".to_string()]);

        fs::remove_dir_all(repo_dir)?;
        Ok(())
    }

    fn create_temp_repo_dir(prefix: &str) -> Result<PathBuf> {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let root = std::env::temp_dir().join(format!("{prefix}-{}-{unique}", std::process::id()));
        fs::create_dir_all(root.join("hosts"))?;
        Ok(root)
    }
}
