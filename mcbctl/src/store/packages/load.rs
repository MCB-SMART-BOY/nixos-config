use crate::domain::tui::CatalogEntry;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

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
            if !path.is_file() || path.extension().is_none_or(|ext| ext != "nix") {
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

    selected
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
            workflow_tags: Vec::new(),
            lifecycle: None,
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
