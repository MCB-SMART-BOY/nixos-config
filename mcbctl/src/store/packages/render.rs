use crate::domain::tui::CatalogEntry;
use crate::{managed_file_is_valid, managed_file_kind, write_managed_file};
use anyhow::{Context, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

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
            ensure_stale_package_file_is_managed(&path)?;
            fs::remove_file(&path)
                .with_context(|| format!("failed to remove stale {}", path.display()))?;
        }
    }

    for (group, entries) in grouped_entries {
        let path = grouped_dir.join(format!("{group}.nix"));
        let content = render_managed_package_group_file(&group, &entries);
        write_managed_file(
            &path,
            &format!("package-group:{group}"),
            &content,
            &["# 机器管理的软件组："],
        )?;
    }

    Ok(())
}

fn ensure_stale_package_file_is_managed(path: &Path) -> Result<()> {
    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let has_valid_marker = managed_file_kind(&content)
        .is_some_and(|kind| kind.starts_with("package-group:") && managed_file_is_valid(&content));
    let legacy_group_file = content.trim_start().starts_with("# 机器管理的软件组：");
    if has_valid_marker || legacy_group_file {
        return Ok(());
    }

    anyhow::bail!(
        "refusing to remove stale unmanaged package file {}; move manual content out of managed/packages first",
        path.display()
    )
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

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn write_grouped_managed_packages_rejects_unmanaged_stale_files() -> Result<()> {
        let unique = format!("{}-{}", std::process::id(), rand::random::<u64>());
        let dir = std::env::temp_dir().join(format!("mcbctl-packages-{unique}"));
        let managed_dir = dir.join("managed");
        let grouped_dir = managed_dir.join("packages");
        fs::create_dir_all(&grouped_dir)?;
        fs::write(
            grouped_dir.join("manual.nix"),
            "{ pkgs, ... }: { home.packages = [ pkgs.hello ]; }\n",
        )?;

        let catalog = vec![CatalogEntry {
            id: "hello".to_string(),
            name: "Hello".to_string(),
            category: "cli".to_string(),
            group: Some("misc".to_string()),
            expr: "pkgs.hello".to_string(),
            description: None,
            keywords: Vec::new(),
            source: Some("nixpkgs".to_string()),
            platforms: Vec::new(),
            desktop_entry_flag: None,
        }];

        let err = write_grouped_managed_packages(&managed_dir, &catalog, &BTreeMap::new())
            .expect_err("unmanaged stale file should block package save");
        assert!(
            err.to_string()
                .contains("refusing to remove stale unmanaged package file")
        );

        fs::remove_dir_all(&dir)?;
        Ok(())
    }
}
