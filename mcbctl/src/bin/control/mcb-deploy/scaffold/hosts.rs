use super::*;

impl App {
    pub(crate) fn ensure_host_entry(&mut self, repo_dir: &Path) -> Result<()> {
        if self.target_name.is_empty() {
            bail!("未指定主机名称。");
        }

        let host_dir = repo_dir.join("hosts").join(&self.target_name);
        if host_dir.join("default.nix").is_file() && host_dir.join("system.nix").is_file() {
            return Ok(());
        }

        let (template_label, template_dir) = self
            .resolve_host_template(repo_dir)
            .context("未找到可用主机模板，无法创建新主机目录")?;
        self.note(&format!("主机模板来源：{template_label}"));

        copy_recursively_if_missing(&template_dir, &host_dir)?;

        let primary_user = self
            .target_users
            .first()
            .cloned()
            .unwrap_or_else(|| self.resolve_default_user());
        let default_file = host_dir.join("default.nix");
        if default_file.is_file() {
            let content = fs::read_to_string(&default_file)
                .with_context(|| format!("读取主机模板失败：{}", default_file.display()))?;
            let rendered = content
                .replace("your-host", &self.target_name)
                .replace("your-user", &primary_user);
            fs::write(&default_file, rendered)
                .with_context(|| format!("写入主机入口失败：{}", default_file.display()))?;
        }

        self.warn(&format!(
            "已为新主机生成模板目录：hosts/{}",
            self.target_name
        ));
        Ok(())
    }

    pub(crate) fn preserve_existing_local_override(&self, repo_dir: &Path) -> Result<()> {
        if self.deploy_mode != DeployMode::UpdateExisting {
            return Ok(());
        }
        if self.target_name.is_empty() {
            return Ok(());
        }
        let src = self
            .etc_dir
            .join("hosts")
            .join(&self.target_name)
            .join("local.nix");
        let dst = repo_dir
            .join("hosts")
            .join(&self.target_name)
            .join("local.nix");
        if src.is_file() {
            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&src, &dst).with_context(|| {
                format!(
                    "仅更新模式：复制现有 local.nix 失败：{} -> {}",
                    src.display(),
                    dst.display()
                )
            })?;
            self.note(&format!(
                "仅更新模式：已保留现有 hosts/{}/local.nix",
                self.target_name
            ));
        } else {
            self.note("仅更新模式：未发现现有 hosts/<host>/local.nix，将按仓库默认配置更新。");
        }

        let src_hw = host_hardware_config_path(&self.etc_dir, &self.target_name);
        let dst_hw = host_hardware_config_path(repo_dir, &self.target_name);
        if src_hw.is_file() {
            if let Some(parent) = dst_hw.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&src_hw, &dst_hw).with_context(|| {
                format!(
                    "仅更新模式：复制现有 hardware-configuration.nix 失败：{} -> {}",
                    src_hw.display(),
                    dst_hw.display()
                )
            })?;
            self.note(&format!(
                "仅更新模式：已保留现有 hosts/{}/hardware-configuration.nix",
                self.target_name
            ));
        } else {
            self.note(
                "仅更新模式：未发现现有 hosts/<host>/hardware-configuration.nix，将使用仓库内已有文件或评估 fallback。",
            );
        }
        Ok(())
    }

    pub(crate) fn write_local_override(&mut self, repo_dir: &Path) -> Result<()> {
        if self.target_users.is_empty() {
            return Ok(());
        }
        let host_dir = repo_dir.join("hosts").join(&self.target_name);
        if !host_dir.is_dir() {
            bail!("主机目录不存在：{}", host_dir.display());
        }
        let file = host_dir.join("local.nix");

        if self.target_admin_users.is_empty() {
            self.target_admin_users = vec![self.target_users[0].clone()];
        }
        let primary = &self.target_users[0];
        let users_list = self
            .target_users
            .iter()
            .map(|u| format!(" \"{u}\""))
            .collect::<String>();
        let admins_list = self
            .target_admin_users
            .iter()
            .map(|u| format!(" \"{u}\""))
            .collect::<String>();

        let mut out = String::new();
        out.push_str("{ lib, ... }:\n\n{\n");
        out.push_str(&format!("  mcb.user = lib.mkForce \"{primary}\";\n"));
        out.push_str(&format!("  mcb.users = lib.mkForce [{users_list} ];\n"));
        out.push_str(&format!(
            "  mcb.adminUsers = lib.mkForce [{admins_list} ];\n"
        ));

        if self.per_user_tun_enabled && !self.user_tun.is_empty() {
            out.push_str("  mcb.perUserTun.interfaces = lib.mkForce {\n");
            for user in &self.target_users {
                if let Some(v) = self.user_tun.get(user) {
                    out.push_str(&format!("    {user} = \"{v}\";\n"));
                }
            }
            out.push_str("  };\n");
            out.push_str("  mcb.perUserTun.dnsPorts = lib.mkForce {\n");
            for user in &self.target_users {
                if let Some(v) = self.user_dns.get(user) {
                    out.push_str(&format!("    {user} = {v};\n"));
                }
            }
            out.push_str("  };\n");
        }

        if self.gpu_override {
            out.push_str(&format!(
                "  mcb.hardware.gpu.mode = lib.mkForce \"{}\";\n",
                self.gpu_mode
            ));
            if !self.gpu_igpu_vendor.is_empty() {
                out.push_str(&format!(
                    "  mcb.hardware.gpu.igpuVendor = lib.mkForce \"{}\";\n",
                    self.gpu_igpu_vendor
                ));
            }
            if !self.gpu_nvidia_open.is_empty() {
                out.push_str(&format!(
                    "  mcb.hardware.gpu.nvidia.open = lib.mkForce {};\n",
                    self.gpu_nvidia_open
                ));
            }
            if !self.gpu_prime_mode.is_empty()
                || !self.gpu_intel_bus.is_empty()
                || !self.gpu_amd_bus.is_empty()
                || !self.gpu_nvidia_bus.is_empty()
            {
                out.push_str("  mcb.hardware.gpu.prime = lib.mkForce {\n");
                if !self.gpu_prime_mode.is_empty() {
                    out.push_str(&format!("    mode = \"{}\";\n", self.gpu_prime_mode));
                }
                if !self.gpu_intel_bus.is_empty() {
                    out.push_str(&format!("    intelBusId = \"{}\";\n", self.gpu_intel_bus));
                }
                if !self.gpu_amd_bus.is_empty() {
                    out.push_str(&format!("    amdgpuBusId = \"{}\";\n", self.gpu_amd_bus));
                }
                if !self.gpu_nvidia_bus.is_empty() {
                    out.push_str(&format!("    nvidiaBusId = \"{}\";\n", self.gpu_nvidia_bus));
                }
                out.push_str("  };\n");
            }
            if self.gpu_specialisations_set {
                out.push_str(&format!(
                    "  mcb.hardware.gpu.specialisations.enable = lib.mkForce {};\n",
                    self.gpu_specialisations_enabled
                ));
                if self.gpu_specialisations_enabled && !self.gpu_specialisation_modes.is_empty() {
                    let mode_list = self
                        .gpu_specialisation_modes
                        .iter()
                        .map(|m| format!(" \"{m}\""))
                        .collect::<String>();
                    out.push_str(&format!(
                        "  mcb.hardware.gpu.specialisations.modes = lib.mkForce [{mode_list} ];\n"
                    ));
                }
            }
        }

        if self.server_overrides_enabled {
            out.push_str(&format!(
                "  mcb.packages.enableNetworkCli = lib.mkForce {};\n",
                self.server_enable_network_cli
            ));
            out.push_str(&format!(
                "  mcb.packages.enableNetworkGui = lib.mkForce {};\n",
                self.server_enable_network_gui
            ));
            out.push_str(&format!(
                "  mcb.packages.enableShellTools = lib.mkForce {};\n",
                self.server_enable_shell_tools
            ));
            out.push_str(&format!(
                "  mcb.packages.enableWaylandTools = lib.mkForce {};\n",
                self.server_enable_wayland_tools
            ));
            out.push_str(&format!(
                "  mcb.packages.enableSystemTools = lib.mkForce {};\n",
                self.server_enable_system_tools
            ));
            out.push_str(&format!(
                "  mcb.packages.enableGeekTools = lib.mkForce {};\n",
                self.server_enable_geek_tools
            ));
            out.push_str(&format!(
                "  mcb.packages.enableGaming = lib.mkForce {};\n",
                self.server_enable_gaming
            ));
            out.push_str(&format!(
                "  mcb.packages.enableInsecureTools = lib.mkForce {};\n",
                self.server_enable_insecure_tools
            ));
            out.push_str(&format!(
                "  mcb.virtualisation.docker.enable = lib.mkForce {};\n",
                self.server_enable_docker
            ));
            out.push_str(&format!(
                "  mcb.virtualisation.libvirtd.enable = lib.mkForce {};\n",
                self.server_enable_libvirtd
            ));
        }
        out.push_str("}\n");
        fs::write(file, out)?;
        Ok(())
    }
}
