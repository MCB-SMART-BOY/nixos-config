use crate::domain::tui::{CatalogEntry, GroupMeta, HomeOptionMeta, WorkflowMeta};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Default, Deserialize)]
struct PackageCatalog {
    #[serde(default, rename = "package")]
    packages: Vec<CatalogEntry>,
}

#[derive(Debug, Default, Deserialize)]
struct GroupCatalog {
    #[serde(default, rename = "group")]
    groups: Vec<GroupMeta>,
}

#[derive(Debug, Default, Deserialize)]
struct HomeOptionsCatalog {
    #[serde(default, rename = "option")]
    options: Vec<HomeOptionMeta>,
}

#[derive(Debug, Default, Deserialize)]
struct WorkflowCatalog {
    #[serde(default, rename = "workflow")]
    workflows: Vec<WorkflowMeta>,
}

pub fn load_catalog(path: &Path) -> (Vec<CatalogEntry>, Vec<String>, Vec<String>) {
    let packages = if path.is_dir() {
        load_catalog_directory(path)
    } else if let Some(dir) = path.parent().map(|parent| parent.join("packages"))
        && dir.is_dir()
    {
        load_catalog_directory(&dir)
    } else {
        load_catalog_file(path)
    };

    let mut categories = BTreeSet::new();
    let mut sources = BTreeSet::new();
    for package in &packages {
        categories.insert(package.category.clone());
        sources.insert(package.source_label().to_string());
    }

    (
        packages,
        categories.into_iter().collect(),
        sources.into_iter().collect(),
    )
}

fn load_catalog_directory(path: &Path) -> Vec<CatalogEntry> {
    let Ok(entries) = fs::read_dir(path) else {
        return Vec::new();
    };

    let mut files = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && path.extension().is_some_and(|ext| ext == "toml"))
        .collect::<Vec<_>>();
    files.sort();

    let mut packages = Vec::new();
    for file in files {
        packages.extend(load_catalog_file(&file));
    }
    packages
}

fn load_catalog_file(path: &Path) -> Vec<CatalogEntry> {
    let Ok(content) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let Ok(parsed) = toml::from_str::<PackageCatalog>(&content) else {
        return Vec::new();
    };
    parsed.packages
}

pub fn load_group_catalog(path: &Path) -> BTreeMap<String, GroupMeta> {
    let Ok(content) = fs::read_to_string(path) else {
        return BTreeMap::new();
    };
    let Ok(parsed) = toml::from_str::<GroupCatalog>(&content) else {
        return BTreeMap::new();
    };

    parsed
        .groups
        .into_iter()
        .map(|group| (group.id.clone(), group))
        .collect()
}

pub fn load_home_options_catalog(path: &Path) -> Vec<HomeOptionMeta> {
    let Ok(content) = fs::read_to_string(path) else {
        return default_home_options();
    };
    let Ok(parsed) = toml::from_str::<HomeOptionsCatalog>(&content) else {
        return default_home_options();
    };

    let mut options = parsed.options;
    if options.is_empty() {
        options = default_home_options();
    }
    options.sort_by(|left, right| {
        left.order
            .cmp(&right.order)
            .then_with(|| left.label.cmp(&right.label))
            .then_with(|| left.id.cmp(&right.id))
    });
    options
}

pub fn load_workflow_catalog(path: &Path) -> BTreeMap<String, WorkflowMeta> {
    let Ok(content) = fs::read_to_string(path) else {
        return BTreeMap::new();
    };
    let Ok(parsed) = toml::from_str::<WorkflowCatalog>(&content) else {
        return BTreeMap::new();
    };

    parsed
        .workflows
        .into_iter()
        .map(|workflow| (workflow.id.clone(), workflow))
        .collect()
}

fn default_home_options() -> Vec<HomeOptionMeta> {
    vec![
        HomeOptionMeta {
            id: "noctalia.barProfile".to_string(),
            label: "Noctalia 顶栏".to_string(),
            description: Some("控制默认顶栏 profile。".to_string()),
            area: "desktop".to_string(),
            order: 10,
        },
        HomeOptionMeta {
            id: "desktop.enableZed".to_string(),
            label: "Zed 桌面入口".to_string(),
            description: Some("控制 Zed 桌面入口开关。".to_string()),
            area: "desktop".to_string(),
            order: 20,
        },
        HomeOptionMeta {
            id: "desktop.enableYesPlayMusic".to_string(),
            label: "YesPlayMusic 桌面入口".to_string(),
            description: Some("控制 YesPlayMusic 桌面入口开关。".to_string()),
            area: "desktop".to_string(),
            order: 30,
        },
    ]
}
