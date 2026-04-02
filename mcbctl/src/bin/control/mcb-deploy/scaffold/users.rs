use super::*;

impl App {
    pub(crate) fn ensure_user_home_entries(&mut self, repo_dir: &Path) -> Result<()> {
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
in
{
  imports = splitImports;
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
}
