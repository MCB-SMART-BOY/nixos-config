use super::*;

impl App {
    pub(crate) fn resolve_user_template(&self, repo_dir: &Path) -> Option<(String, PathBuf)> {
        let template_name = if self.host_profile_kind == HostProfileKind::Server {
            "server"
        } else {
            "laptop"
        };
        let template_dir = repo_dir.join("home/templates/users").join(template_name);
        if template_dir.is_dir() {
            return Some((
                format!("home/templates/users/{template_name}"),
                template_dir,
            ));
        }

        None
    }

    pub(crate) fn extract_user_from_file(file: &Path) -> Result<Option<String>> {
        let text = match fs::read_to_string(file) {
            Ok(text) => text,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(err) => {
                return Err(err)
                    .with_context(|| format!("读取默认用户候选文件 {} 失败", file.display()));
            }
        };
        for line in text.lines() {
            let l = strip_comment(line);
            if l.contains("mcb.user")
                && l.contains('=')
                && l.contains('"')
                && let Some(v) = first_quoted(l)
            {
                return Ok(Some(v));
            }
            if l.trim_start().starts_with("user")
                && l.contains('=')
                && l.contains('"')
                && let Some(v) = first_quoted(l)
            {
                return Ok(Some(v));
            }
        }
        Ok(None)
    }

    pub(crate) fn resolve_default_user(&self) -> String {
        let mut files = Vec::new();
        if let Some(tmp_dir) = &self.tmp_dir {
            files.push(
                tmp_dir
                    .join("hosts")
                    .join(&self.target_name)
                    .join("local.nix"),
            );
            files.push(
                tmp_dir
                    .join("hosts")
                    .join(&self.target_name)
                    .join("default.nix"),
            );
        }
        files.push(
            self.etc_dir
                .join("hosts")
                .join(&self.target_name)
                .join("local.nix"),
        );
        files.push(
            self.etc_dir
                .join("hosts")
                .join(&self.target_name)
                .join("default.nix"),
        );
        for file in files {
            match Self::extract_user_from_file(&file) {
                Ok(Some(v)) => return v,
                Ok(None) => {}
                Err(err) => self.warn(&err.to_string()),
            }
        }
        for key in ["SUDO_USER", "USER", "LOGNAME"] {
            if let Ok(v) = std::env::var(key)
                && is_valid_username(&v)
                && v != "root"
            {
                return v;
            }
        }

        let mut repo_candidates = self.list_existing_home_users(&self.repo_dir);
        if self.etc_dir != self.repo_dir {
            repo_candidates.extend(self.list_existing_home_users(&self.etc_dir));
        }
        repo_candidates.sort();
        repo_candidates.dedup();
        if let Some(user) = repo_candidates.into_iter().next() {
            return user;
        }

        "user".to_string()
    }

    fn list_existing_home_users_in(repo_dir: &Path) -> Result<Vec<String>> {
        let users_dir = repo_dir.join("home/users");
        if !users_dir.exists() {
            return Ok(Vec::new());
        }
        if !users_dir.is_dir() {
            bail!("现有 Home 用户目录不是目录：{}", users_dir.display());
        }

        let entries = fs::read_dir(&users_dir)
            .with_context(|| format!("读取 Home 用户目录 {} 失败", users_dir.display()))?;
        let mut users = Vec::new();
        for entry in entries {
            let entry = entry
                .with_context(|| format!("遍历 Home 用户目录 {} 失败", users_dir.display()))?;
            let file_type = entry.file_type().with_context(|| {
                format!("读取 Home 用户条目类型 {} 失败", entry.path().display())
            })?;
            if !file_type.is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if is_valid_username(&name) {
                users.push(name);
            }
        }
        users.sort();
        Ok(users)
    }

    pub(crate) fn list_existing_home_users(&self, repo_dir: &Path) -> Vec<String> {
        match Self::list_existing_home_users_in(repo_dir) {
            Ok(users) => users,
            Err(err) => {
                self.warn(&err.to_string());
                Vec::new()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_user_from_file_returns_none_for_missing_file() -> Result<()> {
        let temp_root = create_temp_dir("mcbctl-default-user-missing")?;
        let missing = temp_root.join("missing.nix");

        assert_eq!(App::extract_user_from_file(&missing)?, None);
        Ok(())
    }

    #[test]
    fn extract_user_from_file_reports_unreadable_path() -> Result<()> {
        let temp_root = create_temp_dir("mcbctl-default-user-unreadable")?;
        let directory = temp_root.join("candidate.nix");
        fs::create_dir_all(&directory)?;

        let err = App::extract_user_from_file(&directory)
            .expect_err("directories should not be treated as missing files");

        assert!(err.to_string().contains("读取默认用户候选文件"));
        Ok(())
    }

    #[test]
    fn resolve_default_user_skips_unreadable_candidate_files() -> Result<()> {
        let temp_root = create_temp_dir("mcbctl-default-user-fallback")?;
        let host_dir = temp_root.join("hosts/demo");
        fs::create_dir_all(host_dir.join("local.nix"))?;
        fs::write(host_dir.join("default.nix"), r#"{ mcb.user = "alice"; }"#)?;
        let app = test_app(temp_root.clone());

        assert_eq!(app.resolve_default_user(), "alice");
        Ok(())
    }

    #[test]
    fn list_existing_home_users_in_reports_invalid_home_users_root() -> Result<()> {
        let temp_root = create_temp_dir("mcbctl-home-users-invalid-root")?;
        let home_dir = temp_root.join("home");
        fs::create_dir_all(&home_dir)?;
        fs::write(home_dir.join("users"), "not-a-directory")?;

        let err = App::list_existing_home_users_in(&temp_root)
            .expect_err("a file at home/users should be reported as invalid");

        assert!(err.to_string().contains("现有 Home 用户目录不是目录"));
        Ok(())
    }

    fn create_temp_dir(prefix: &str) -> Result<PathBuf> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let root = std::env::temp_dir().join(format!("{prefix}-{}-{unique}", std::process::id()));
        fs::create_dir_all(&root)?;
        Ok(root)
    }

    fn test_app(tmp_dir: PathBuf) -> App {
        App {
            repo_dir: tmp_dir.clone(),
            repo_urls: Vec::new(),
            branch: "rust脚本分支".to_string(),
            source_ref: String::new(),
            allow_remote_head: false,
            source_commit: String::new(),
            source_choice_set: false,
            target_name: "demo".to_string(),
            target_users: Vec::new(),
            target_admin_users: Vec::new(),
            deploy_mode: DeployMode::ManageUsers,
            deploy_mode_set: false,
            force_remote_source: false,
            overwrite_mode: OverwriteMode::Ask,
            overwrite_mode_set: false,
            per_user_tun_enabled: false,
            host_profile_kind: HostProfileKind::Desktop,
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
            tmp_dir: Some(tmp_dir),
            sudo_cmd: None,
            rootless: false,
            run_action: RunAction::Deploy,
            progress_total: 7,
            progress_current: 0,
            git_clone_timeout_sec: 90,
        }
    }
}
