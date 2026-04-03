use crate::domain::tui::CatalogEntry;
use crate::write_file_atomic;
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

fn selected_entries_by_group<'a>(
    catalog: &'a [CatalogEntry],
    selected: &BTreeMap<String, String>,
) -> BTreeMap<String, Vec<&'a CatalogEntry>> {
    let mut grouped = BTreeMap::<String, Vec<&CatalogEntry>>::new();

    for entry in catalog.iter().filter(|entry| selected.contains_key(&entry.id)) {
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
