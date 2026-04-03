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

    pub(crate) fn extract_user_from_file(file: &Path) -> Option<String> {
        let text = fs::read_to_string(file).ok()?;
        for line in text.lines() {
            let l = strip_comment(line);
            if l.contains("mcb.user")
                && l.contains('=')
                && l.contains('"')
                && let Some(v) = first_quoted(l)
            {
                return Some(v);
            }
            if l.trim_start().starts_with("user")
                && l.contains('=')
                && l.contains('"')
                && let Some(v) = first_quoted(l)
            {
                return Some(v);
            }
        }
        None
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
            if let Some(v) = Self::extract_user_from_file(&file) {
                return v;
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
        "mcbnixos".to_string()
    }

    pub(crate) fn list_existing_home_users(&self, repo_dir: &Path) -> Vec<String> {
        let mut users = Vec::new();
        let users_dir = repo_dir.join("home/users");
        if users_dir.is_dir()
            && let Ok(entries) = fs::read_dir(users_dir)
        {
            for entry in entries.flatten() {
                if !entry.path().is_dir() {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().to_string();
                if is_valid_username(&name) {
                    users.push(name);
                }
            }
        }
        users.sort();
        users
    }
}
