use super::*;
use mcbctl::store::home::ensure_managed_settings_layout;
use mcbctl::store::packages::ensure_managed_packages_layout;

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

            ensure_managed_packages_layout(&managed_dir)?;
            ensure_managed_settings_layout(&managed_dir)?;

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
