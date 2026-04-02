use super::*;

impl App {
    pub(super) fn ensure_user_home_entries(&mut self, repo_dir: &Path) -> Result<()> {
        let mut profile_import = "../../profiles/full.nix";
        let extra_imports = vec!["./git.nix", "./packages.nix"];
        let mut include_user_files = true;
        if self.host_profile_kind == HostProfileKind::Server {
            profile_import = "../../profiles/minimal.nix";
            include_user_files = false;
        }

        let template_source = self.resolve_user_template(repo_dir);
        if let Some((template_label, _)) = &template_source {
            self.note(&format!("新用户模板来源：{template_label}"));
        }

        let copy_template_content = std::env::var("MCBCTL_COPY_USER_TEMPLATE")
            .ok()
            .is_some_and(|v| v == "true");
        if copy_template_content {
            self.note("将复制模板用户目录内容（MCBCTL_COPY_USER_TEMPLATE=true）");
        } else {
            self.note(
                "默认仅生成最小用户模板（不复制 config/assets）；如需复制可设置 MCBCTL_COPY_USER_TEMPLATE=true",
            );
        }

        for user in self.target_users.clone() {
            let user_dir = repo_dir.join("home/users").join(&user);
            let user_file = user_dir.join("default.nix");
            let create_default = !user_file.is_file();

            fs::create_dir_all(&user_dir)?;
            if create_default
                && let Some((_, template_dir)) = &template_source
                && user_dir != *template_dir
                && include_user_files
                && copy_template_content
            {
                for item in ["config", "assets"] {
                    let src = template_dir.join(item);
                    let dst = user_dir.join(item);
                    if src.exists() && !dst.exists() {
                        copy_recursively(&src, &dst)?;
                    }
                }
                for template_file in ["files.nix"] {
                    let src = template_dir.join(template_file);
                    let dst = user_dir.join(template_file);
                    if src.is_file() && !dst.exists() {
                        fs::copy(src, dst).ok();
                    }
                }
            }

            let git_file = user_dir.join("git.nix");
            if !git_file.is_file() {
                fs::write(
                    &git_file,
                    r#"# 默认 Git 身份（请按需修改）
{ config, ... }:

{
  programs.git.settings.user = {
    name = config.home.username;
    # email = "you@example.com";
  };
}
"#,
                )?;
            }

            let packages_file = user_dir.join("packages.nix");
            if !packages_file.is_file() {
                if let Some((_, template_dir)) = &template_source {
                    let src = template_dir.join("packages.nix");
                    if src.is_file() && user_dir != *template_dir {
                        fs::copy(src, &packages_file).ok();
                    }
                }
                if !packages_file.is_file() {
                    if self.host_profile_kind == HostProfileKind::Server {
                        fs::write(
                            &packages_file,
                            r#"# 用户个人软件入口（服务器最小模板）
{ pkgs, ... }:

{
  home.packages = with pkgs; [
    # tmux
    # htop
    # rsync
  ];
}
"#,
                        )?;
                    } else {
                        fs::write(
                            &packages_file,
                            r#"# 用户个人软件入口（按需启用，不影响其他用户可见性）
{ pkgs, ... }:

{
  mcb.desktopEntries = {
    enableZed = false;
    enableYesPlayMusic = false;
  };

  # 逐个声明该用户的软件（仅此用户可见）
  home.packages = with pkgs; [
    # firefox
    # helix
    # (callPackage ../../../pkgs/zed { })            # 同时把 enableZed 改为 true
    # (callPackage ../../../pkgs/yesplaymusic { })   # 同时把 enableYesPlayMusic 改为 true
  ];
}
"#,
                        )?;
                    }
                }
            }

            let local_example = user_dir.join("local.nix.example");
            if !local_example.is_file() {
                fs::write(
                    &local_example,
                    r#"# 用户私有覆盖示例（按需复制为 local.nix）
{ ... }:

{
  # 仅当前用户生效的个性化开关示例：
  # home.packages = with pkgs; [ localsend ];
}
"#,
                )?;
            }

            let managed_dir = user_dir.join("managed");
            fs::create_dir_all(&managed_dir)?;

            let managed_default = managed_dir.join("default.nix");
            if !managed_default.is_file() {
                fs::write(
                    &managed_default,
                    r#"# TUI / 自动化工具专用入口。
{ lib, ... }:

{
  imports = [
    ./packages.nix
  ]
  ++ lib.optional (builtins.pathExists ./settings/default.nix) ./settings/default.nix
  ++ lib.optional (
    (!builtins.pathExists ./settings/default.nix) && builtins.pathExists ./settings.nix
  ) ./settings.nix
  ++ lib.optional (builtins.pathExists ./local.nix) ./local.nix;
}
"#,
                )?;
            }

            let managed_packages = managed_dir.join("packages.nix");
            if !managed_packages.is_file() {
                fs::write(
                    &managed_packages,
                    r#"# 机器管理的用户软件入口（由 mcbctl 维护）。
# 说明：真正的软件组会按文件写入 ./packages/*.nix，这里只负责聚合导入。

{ lib, ... }:

let
  packageDir = ./packages;
  packageImports =
    if builtins.pathExists packageDir then
      builtins.map (name: packageDir + "/${name}") (
        lib.sort lib.lessThan (
          lib.filter (name: lib.hasSuffix ".nix" name) (builtins.attrNames (builtins.readDir packageDir))
        )
      )
    else
      [ ];
in
{
  imports = packageImports;
}
"#,
                )?;
            }

            let managed_packages_dir = managed_dir.join("packages");
            fs::create_dir_all(&managed_packages_dir)?;

            let managed_packages_readme = managed_packages_dir.join("README.md");
            if !managed_packages_readme.is_file() {
                fs::write(
                    &managed_packages_readme,
                    r#"# Managed Packages

这个目录给 `mcbctl` 的 Packages 页面使用。

约定：

- 一个软件组对应一个 `.nix` 文件
- `managed/packages.nix` 只做聚合导入
- 这里的文件可以由 TUI 重写，不要放手写长期逻辑
"#,
                )?;
            }

            let managed_settings = managed_dir.join("settings.nix");
            if !managed_settings.is_file() {
                fs::write(
                    &managed_settings,
                    r#"# 兼容旧结构的用户设置入口。
# 现在用户级 managed 设置已经拆成 settings/default.nix + desktop/session/mime 分片。

{ ... }:

{
}
"#,
                )?;
            }

            let managed_settings_dir = managed_dir.join("settings");
            fs::create_dir_all(&managed_settings_dir)?;

            let managed_settings_default = managed_settings_dir.join("default.nix");
            if !managed_settings_default.is_file() {
                fs::write(
                    &managed_settings_default,
                    r#"# 机器管理的用户设置聚合入口。

{ lib, ... }:

let
  splitImports = lib.concatLists [
    (lib.optional (builtins.pathExists ./desktop.nix) ./desktop.nix)
    (lib.optional (builtins.pathExists ./session.nix) ./session.nix)
    (lib.optional (builtins.pathExists ./mime.nix) ./mime.nix)
  ];
  legacySettings =
    lib.optional ((splitImports == [ ]) && builtins.pathExists ../settings.nix) ../settings.nix;
in
{
  imports = splitImports ++ legacySettings;
}
"#,
                )?;
            }

            for (name, body) in [
                (
                    "desktop.nix",
                    r#"# 机器管理的桌面设置分片。

{ ... }:

{ }
"#,
                ),
                (
                    "session.nix",
                    r#"# 机器管理的 session 设置分片。

{ ... }:

{ }
"#,
                ),
                (
                    "mime.nix",
                    r#"# 机器管理的 MIME 设置分片。

{ ... }:

{ }
"#,
                ),
            ] {
                let path = managed_settings_dir.join(name);
                if !path.is_file() {
                    fs::write(path, body)?;
                }
            }

            if !create_default {
                continue;
            }

            let mut import_lines = vec![format!("    {profile_import}")];
            for extra in &extra_imports {
                if user_dir.join(extra).is_file() {
                    import_lines.push(format!("    {extra}"));
                }
            }
            if include_user_files && user_dir.join("files.nix").is_file() {
                import_lines.push("    ./files.nix".to_string());
            }
            if managed_dir.join("default.nix").is_file() {
                import_lines.push("    ./managed/default.nix".to_string());
            }
            let content = format!(
                r#"{{
  lib, ...
}}:

let
  user = "{user}";
in
{{
  imports = [
{}
  ] ++ lib.optional (builtins.pathExists ./local.nix) ./local.nix;

  home.username = user;
  home.homeDirectory = "/home/${{user}}";
  home.stateVersion = "25.11";

  programs.home-manager.enable = true;
  xdg.enable = true;
}}
"#,
                import_lines.join("\n")
            );
            fs::write(&user_file, content)?;
            self.created_home_users.push(user.clone());
            self.warn(&format!(
                "已为新用户自动生成 Home Manager 入口：home/users/{user}/default.nix"
            ));
        }
        Ok(())
    }

    pub(super) fn ensure_host_entry(&mut self, repo_dir: &Path) -> Result<()> {
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

    pub(super) fn preserve_existing_local_override(&self, repo_dir: &Path) -> Result<()> {
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
        Ok(())
    }

    pub(super) fn write_local_override(&mut self, repo_dir: &Path) -> Result<()> {
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
