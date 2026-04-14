use crate::domain::tui::{HomeManagedSettings, ManagedBarProfile, ManagedToggle};
use crate::{ensure_existing_managed_file, write_managed_file};
use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub fn load_home_user_settings(
    repo_root: &Path,
    users: &[String],
) -> BTreeMap<String, HomeManagedSettings> {
    let mut settings = BTreeMap::new();

    for user in users {
        settings.insert(user.clone(), load_user_managed_settings(repo_root, user));
    }

    settings
}

pub fn load_user_managed_settings(repo_root: &Path, user: &str) -> HomeManagedSettings {
    let split_path = managed_home_desktop_path(repo_root, user);
    let content = if let Ok(content) = fs::read_to_string(&split_path) {
        content
    } else {
        return HomeManagedSettings::default();
    };

    HomeManagedSettings {
        bar_profile: parse_bar_profile_marker(&content, "noctalia.barProfile"),
        enable_zed_entry: parse_toggle_marker(&content, "desktop.enableZed"),
        enable_yesplaymusic_entry: parse_toggle_marker(&content, "desktop.enableYesPlayMusic"),
    }
}

pub fn ensure_managed_settings_layout(managed_dir: &Path) -> Result<()> {
    fs::create_dir_all(managed_dir)
        .with_context(|| format!("failed to create {}", managed_dir.display()))?;

    let settings_dir = managed_dir.join("settings");
    fs::create_dir_all(&settings_dir)
        .with_context(|| format!("failed to create {}", settings_dir.display()))?;

    write_managed_file(
        &settings_dir.join("default.nix"),
        "home-settings-default",
        &render_settings_default_file(),
        &["# 机器管理的用户设置聚合入口"],
    )?;

    for (name, content) in [
        ("desktop.nix", render_settings_placeholder_file("desktop")),
        ("session.nix", render_settings_placeholder_file("session")),
        ("mime.nix", render_settings_placeholder_file("mime")),
    ] {
        let path = settings_dir.join(name);
        let kind = match name {
            "desktop.nix" => "home-settings-desktop",
            "session.nix" => "home-settings-session",
            "mime.nix" => "home-settings-mime",
            _ => unreachable!("unexpected managed settings placeholder"),
        };
        ensure_existing_managed_file(&path, kind)?;
        if !path.exists() {
            write_managed_file(&path, kind, &content, &["# 机器管理的"])?;
        }
    }

    Ok(())
}

pub fn managed_home_desktop_path(repo_root: &Path, user: &str) -> PathBuf {
    repo_root
        .join("home/users")
        .join(user)
        .join("managed/settings/desktop.nix")
}

pub fn user_noctalia_override_path(repo_root: &Path, user: &str) -> PathBuf {
    repo_root.join("home/users").join(user).join("noctalia.nix")
}

pub fn user_has_custom_noctalia_layout(repo_root: &Path, user: &str) -> bool {
    user_noctalia_override_path(repo_root, user).is_file()
}

pub fn render_managed_desktop_file(settings: &HomeManagedSettings) -> String {
    let mut lines = vec![
        "# 机器管理的桌面设置分片（由 mcbctl 维护）。".to_string(),
        "# 适合放桌面入口、Noctalia 配置等结构化 UI 开关。".to_string(),
        format!(
            "# managed-setting: noctalia.barProfile={}",
            settings.bar_profile.marker()
        ),
        format!(
            "# managed-setting: desktop.enableZed={}",
            settings.enable_zed_entry.marker()
        ),
        format!(
            "# managed-setting: desktop.enableYesPlayMusic={}",
            settings.enable_yesplaymusic_entry.marker()
        ),
        "".to_string(),
        "{ lib, ... }:".to_string(),
        "".to_string(),
        "{".to_string(),
        "  # 由 TUI / 自动化工具维护".to_string(),
    ];

    match settings.bar_profile {
        ManagedBarProfile::Inherit => {}
        ManagedBarProfile::Default => {
            lines.push("  mcb.noctalia.barProfile = \"default\";".to_string());
        }
        ManagedBarProfile::None => {
            lines.push("  mcb.noctalia.barProfile = \"none\";".to_string());
        }
    }

    append_toggle_assignment(
        &mut lines,
        "mcb.desktopEntries.enableZed",
        settings.enable_zed_entry,
    );
    append_toggle_assignment(
        &mut lines,
        "mcb.desktopEntries.enableYesPlayMusic",
        settings.enable_yesplaymusic_entry,
    );

    lines.push("}".to_string());
    lines.push("".to_string());
    lines.join("\n")
}

fn render_settings_default_file() -> String {
    [
        "# 机器管理的用户设置聚合入口（由 mcbctl 维护）。",
        "",
        "{ lib, ... }:",
        "",
        "let",
        "  splitImports = lib.concatLists [",
        "    (lib.optional (builtins.pathExists ./desktop.nix) ./desktop.nix)",
        "    (lib.optional (builtins.pathExists ./session.nix) ./session.nix)",
        "    (lib.optional (builtins.pathExists ./mime.nix) ./mime.nix)",
        "  ];",
        "in",
        "{",
        "  imports = splitImports;",
        "}",
        "",
    ]
    .join("\n")
}

fn render_settings_placeholder_file(kind: &str) -> String {
    let title = kind;

    [
        format!("# 机器管理的 {title} 设置分片。"),
        "# 当前为空；当 mcbctl 后续接入对应页面时，会写入这里。".to_string(),
        "".to_string(),
        "{ ... }:".to_string(),
        "".to_string(),
        "{ }".to_string(),
        "".to_string(),
    ]
    .join("\n")
}

fn append_toggle_assignment(lines: &mut Vec<String>, key: &str, value: ManagedToggle) {
    match value {
        ManagedToggle::Inherit => {}
        ManagedToggle::Enabled => lines.push(format!("  {key} = lib.mkForce true;")),
        ManagedToggle::Disabled => lines.push(format!("  {key} = lib.mkForce false;")),
    }
}

fn parse_setting_marker<'a>(content: &'a str, key: &str) -> Option<&'a str> {
    let prefix = format!("# managed-setting: {key}=");
    content
        .lines()
        .find_map(|line| line.trim().strip_prefix(&prefix))
        .map(str::trim)
}

fn parse_toggle_marker(content: &str, key: &str) -> ManagedToggle {
    match parse_setting_marker(content, key) {
        Some("enabled") | Some("true") | Some("on") => ManagedToggle::Enabled,
        Some("disabled") | Some("false") | Some("off") => ManagedToggle::Disabled,
        _ => ManagedToggle::Inherit,
    }
}

fn parse_bar_profile_marker(content: &str, key: &str) -> ManagedBarProfile {
    match parse_setting_marker(content, key) {
        Some("default") => ManagedBarProfile::Default,
        Some("none") => ManagedBarProfile::None,
        _ => ManagedBarProfile::Inherit,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render_managed_file;
    use anyhow::Result;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn load_user_managed_settings_reads_markers_from_managed_file() -> Result<()> {
        let root = create_temp_repo()?;
        let path = managed_home_desktop_path(&root, "demo");
        fs::create_dir_all(path.parent().expect("desktop parent"))?;
        let content = render_managed_file(
            "home-settings-desktop",
            &render_managed_desktop_file(&HomeManagedSettings {
                bar_profile: ManagedBarProfile::Default,
                enable_zed_entry: ManagedToggle::Enabled,
                enable_yesplaymusic_entry: ManagedToggle::Disabled,
            }),
        );
        fs::write(&path, content)?;

        let settings = load_user_managed_settings(&root, "demo");
        assert_eq!(settings.bar_profile, ManagedBarProfile::Default);
        assert_eq!(settings.enable_zed_entry, ManagedToggle::Enabled);
        assert_eq!(settings.enable_yesplaymusic_entry, ManagedToggle::Disabled);

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn custom_noctalia_layout_detection_uses_user_override_path() -> Result<()> {
        let root = create_temp_repo()?;
        let override_path = user_noctalia_override_path(&root, "demo");
        fs::create_dir_all(override_path.parent().expect("override parent"))?;
        fs::write(&override_path, "{ ... }: { }\n")?;

        assert!(user_has_custom_noctalia_layout(&root, "demo"));
        assert!(!user_has_custom_noctalia_layout(&root, "other"));

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn render_managed_desktop_file_omits_inherit_bar_profile_assignment() {
        let rendered = render_managed_desktop_file(&HomeManagedSettings {
            bar_profile: ManagedBarProfile::Inherit,
            enable_zed_entry: ManagedToggle::Enabled,
            enable_yesplaymusic_entry: ManagedToggle::Inherit,
        });

        assert!(!rendered.contains("mcb.noctalia.barProfile"));
        assert!(rendered.contains("mcb.desktopEntries.enableZed = lib.mkForce true;"));
    }

    fn create_temp_repo() -> Result<PathBuf> {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let root =
            std::env::temp_dir().join(format!("mcbctl-home-store-{}-{unique}", std::process::id()));
        fs::create_dir_all(&root)?;
        Ok(root)
    }
}
