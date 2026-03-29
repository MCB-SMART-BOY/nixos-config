use crate::domain::tui::CatalogEntry;
use crate::write_file_atomic;
use anyhow::{Context, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

pub fn load_package_user_selections(
    repo_root: &Path,
    users: &[String],
    catalog: &[CatalogEntry],
) -> BTreeMap<String, BTreeMap<String, String>> {
    let mut selections = BTreeMap::new();

    for user in users {
        selections.insert(
            user.clone(),
            load_user_managed_package_ids(repo_root, user, catalog),
        );
    }

    selections
}

pub fn load_managed_package_entries(
    repo_root: &Path,
    users: &[String],
    catalog: &[CatalogEntry],
) -> Vec<CatalogEntry> {
    let known_ids = catalog
        .iter()
        .map(|entry| entry.id.clone())
        .collect::<BTreeSet<_>>();
    let mut discovered = BTreeMap::<String, CatalogEntry>::new();

    for user in users {
        let managed_dir = repo_root
            .join("home/users")
            .join(user)
            .join("managed/packages");
        let Ok(entries) = fs::read_dir(managed_dir) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() || !path.extension().is_some_and(|ext| ext == "nix") {
                continue;
            }
            let Some(group) = path.file_stem().and_then(|stem| stem.to_str()) else {
                continue;
            };

            for managed in parse_managed_entries_from_group_file(&path, group) {
                if !known_ids.contains(&managed.id) {
                    discovered.entry(managed.id.clone()).or_insert(managed);
                }
            }
        }
    }

    discovered.into_values().collect()
}

pub fn managed_package_group_path(repo_root: &Path, user: &str, group: &str) -> PathBuf {
    repo_root
        .join("home/users")
        .join(user)
        .join("managed/packages")
        .join(format!("{group}.nix"))
}

pub fn ensure_managed_packages_layout(managed_dir: &Path) -> Result<()> {
    fs::create_dir_all(managed_dir)
        .with_context(|| format!("failed to create {}", managed_dir.display()))?;

    let aggregator = managed_dir.join("packages.nix");
    write_file_atomic(&aggregator, &render_managed_packages_aggregator_file())?;

    let grouped_dir = managed_dir.join("packages");
    fs::create_dir_all(&grouped_dir)
        .with_context(|| format!("failed to create {}", grouped_dir.display()))?;

    let readme = grouped_dir.join("README.md");
    if !readme.exists() {
        write_file_atomic(&readme, &render_managed_packages_readme())?;
    }

    Ok(())
}

pub fn write_grouped_managed_packages(
    managed_dir: &Path,
    catalog: &[CatalogEntry],
    selected: &BTreeMap<String, String>,
) -> Result<()> {
    let grouped_dir = managed_dir.join("packages");
    let grouped_entries = selected_entries_by_group(catalog, selected);

    let mut stale_files = fs::read_dir(&grouped_dir)
        .into_iter()
        .flat_map(|entries| entries.flatten())
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && path.extension().is_some_and(|ext| ext == "nix"))
        .collect::<Vec<_>>();
    stale_files.sort();

    let active_files = grouped_entries
        .keys()
        .map(|group| grouped_dir.join(format!("{group}.nix")))
        .collect::<BTreeSet<_>>();

    for path in stale_files {
        if !active_files.contains(&path) {
            fs::remove_file(&path)
                .with_context(|| format!("failed to remove stale {}", path.display()))?;
        }
    }

    for (group, entries) in grouped_entries {
        let path = grouped_dir.join(format!("{group}.nix"));
        let content = render_managed_package_group_file(&group, &entries);
        write_file_atomic(&path, &content)?;
    }

    Ok(())
}

fn load_user_managed_package_ids(
    repo_root: &Path,
    user: &str,
    catalog: &[CatalogEntry],
) -> BTreeMap<String, String> {
    let managed_dir = repo_root.join("home/users").join(user).join("managed");
    let grouped_dir = managed_dir.join("packages");
    let mut selected = BTreeMap::new();

    if grouped_dir.is_dir() {
        let mut files = fs::read_dir(&grouped_dir)
            .into_iter()
            .flat_map(|entries| entries.flatten())
            .map(|entry| entry.path())
            .filter(|path| path.is_file() && path.extension().is_some_and(|ext| ext == "nix"))
            .collect::<Vec<_>>();
        files.sort();
        for path in files {
            let Some(group) = path.file_stem().and_then(|stem| stem.to_str()) else {
                continue;
            };
            for id in load_managed_package_ids(&path, catalog) {
                selected.insert(id, group.to_string());
            }
        }
    }

    if !selected.is_empty() {
        return selected;
    }

    let legacy_path = managed_dir.join("packages.nix");
    let mut legacy_selected = BTreeMap::new();
    for id in load_managed_package_ids(&legacy_path, catalog) {
        let group = catalog
            .iter()
            .find(|entry| entry.id == id)
            .map(|entry| entry.group_key().to_string())
            .unwrap_or_else(|| "misc".to_string());
        legacy_selected.insert(id, group);
    }
    legacy_selected
}

fn load_managed_package_ids(path: &Path, catalog: &[CatalogEntry]) -> BTreeSet<String> {
    let Ok(content) = fs::read_to_string(path) else {
        return BTreeSet::new();
    };

    let mut selected = BTreeSet::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(id) = trimmed.strip_prefix("# managed-id: ") {
            selected.insert(id.trim().to_string());
        }
    }

    if !selected.is_empty() {
        return selected;
    }

    for entry in catalog {
        if content.contains(&entry.expr) {
            selected.insert(entry.id.clone());
        }
    }
    selected
}

fn selected_entries_by_group<'a>(
    catalog: &'a [CatalogEntry],
    selected: &BTreeMap<String, String>,
) -> BTreeMap<String, Vec<&'a CatalogEntry>> {
    let mut grouped = BTreeMap::<String, Vec<&CatalogEntry>>::new();

    for entry in catalog
        .iter()
        .filter(|entry| selected.contains_key(&entry.id))
    {
        grouped
            .entry(
                selected
                    .get(&entry.id)
                    .cloned()
                    .unwrap_or_else(|| entry.group_key().to_string()),
            )
            .or_default()
            .push(entry);
    }

    for entries in grouped.values_mut() {
        entries.sort_by(|left, right| {
            left.category
                .cmp(&right.category)
                .then_with(|| left.name.cmp(&right.name))
        });
    }

    grouped
}

fn render_managed_packages_aggregator_file() -> String {
    [
        "# 机器管理的用户软件入口（由 mcbctl 维护）。",
        "# 说明：真正的软件组会按文件写入 ./packages/*.nix，这里只负责聚合导入。",
        "",
        "{ lib, ... }:",
        "",
        "let",
        "  packageDir = ./packages;",
        "  packageImports =",
        "    if builtins.pathExists packageDir then",
        "      builtins.map (name: packageDir + \"/${name}\") (",
        "        lib.sort lib.lessThan (",
        "          lib.filter (name: lib.hasSuffix \".nix\" name) (builtins.attrNames (builtins.readDir packageDir))",
        "        )",
        "      )",
        "    else",
        "      [ ];",
        "in",
        "{",
        "  imports = packageImports;",
        "}",
        "",
    ]
    .join("\n")
}

fn render_managed_packages_readme() -> String {
    [
        "# Managed Packages",
        "",
        "这个目录给 `mcbctl` 的 Packages 页面使用。",
        "",
        "约定：",
        "",
        "- 一个软件组对应一个 `.nix` 文件",
        "- `managed/packages.nix` 只做聚合导入",
        "- 这里的文件可以由 TUI 重写，不要放手写长期逻辑",
        "",
    ]
    .join("\n")
}

fn render_managed_package_group_file(group: &str, entries: &[&CatalogEntry]) -> String {
    let mut lines = vec![
        format!("# 机器管理的软件组：{group}（由 mcbctl 维护）。"),
        "# 注意：这个文件只维护当前组，适合通过 TUI 修改。".to_string(),
        "".to_string(),
        "{ lib, pkgs, ... }:".to_string(),
        "".to_string(),
        "{".to_string(),
    ];

    let mut desktop_flags = entries
        .iter()
        .filter_map(|entry| entry.desktop_entry_flag.as_deref())
        .collect::<Vec<_>>();
    desktop_flags.sort_unstable();
    desktop_flags.dedup();

    if !desktop_flags.is_empty() {
        lines.push("  mcb.desktopEntries = {".to_string());
        for flag in desktop_flags {
            lines.push(format!("    {flag} = lib.mkDefault true;"));
        }
        lines.push("  };".to_string());
        lines.push("".to_string());
    }

    lines.push("  home.packages = [".to_string());
    for entry in entries {
        lines.push(format!("    # managed-id: {}", entry.id));
        lines.push(format!("    # managed-name: {}", entry.name));
        lines.push(format!("    # managed-category: {}", entry.category));
        lines.push(format!("    # managed-source: {}", entry.source_label()));
        lines.push(format!("    # managed-expr: {}", entry.expr));
        lines.push(format!("    {}", entry.expr));
        lines.push("".to_string());
    }
    if lines.last().is_some_and(|line| line.is_empty()) {
        lines.pop();
    }
    lines.push("  ];".to_string());
    lines.push("}".to_string());
    lines.push("".to_string());

    lines.join("\n")
}

fn parse_managed_entries_from_group_file(path: &Path, group: &str) -> Vec<CatalogEntry> {
    let Ok(content) = fs::read_to_string(path) else {
        return Vec::new();
    };

    let mut entries = Vec::new();
    let mut pending_id: Option<String> = None;
    let mut pending_name: Option<String> = None;
    let mut pending_category: Option<String> = None;
    let mut pending_source: Option<String> = None;
    let mut pending_expr: Option<String> = None;

    let push_pending = |entries: &mut Vec<CatalogEntry>,
                        pending_id: &mut Option<String>,
                        pending_name: &mut Option<String>,
                        pending_category: &mut Option<String>,
                        pending_source: &mut Option<String>,
                        pending_expr: &mut Option<String>| {
        let (Some(id), Some(name), Some(category), Some(source), Some(expr)) = (
            pending_id.take(),
            pending_name.take(),
            pending_category.take(),
            pending_source.take(),
            pending_expr.take(),
        ) else {
            pending_id.take();
            pending_name.take();
            pending_category.take();
            pending_source.take();
            pending_expr.take();
            return;
        };

        entries.push(CatalogEntry {
            id,
            name,
            category,
            group: Some(group.to_string()),
            expr,
            description: None,
            keywords: Vec::new(),
            source: Some(source),
            platforms: Vec::new(),
            desktop_entry_flag: None,
        });
    };

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("# managed-id: ") {
            push_pending(
                &mut entries,
                &mut pending_id,
                &mut pending_name,
                &mut pending_category,
                &mut pending_source,
                &mut pending_expr,
            );
            pending_id = Some(value.trim().to_string());
        } else if let Some(value) = trimmed.strip_prefix("# managed-name: ") {
            pending_name = Some(value.trim().to_string());
        } else if let Some(value) = trimmed.strip_prefix("# managed-category: ") {
            pending_category = Some(value.trim().to_string());
        } else if let Some(value) = trimmed.strip_prefix("# managed-source: ") {
            pending_source = Some(value.trim().to_string());
        } else if let Some(value) = trimmed.strip_prefix("# managed-expr: ") {
            pending_expr = Some(value.trim().to_string());
        }
    }

    push_pending(
        &mut entries,
        &mut pending_id,
        &mut pending_name,
        &mut pending_category,
        &mut pending_source,
        &mut pending_expr,
    );

    entries
}
