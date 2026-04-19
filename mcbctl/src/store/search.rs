use crate::domain::tui::CatalogEntry;
use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::process::Command;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchPackageRecord {
    pub attr_path: String,
    pub expr: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
    pub source: String,
}

#[derive(Debug, Default, Deserialize)]
struct SearchPackageMeta {
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    pname: Option<String>,
    #[serde(default)]
    version: Option<String>,
}

pub fn search_packages(
    installable: &str,
    query: &str,
    system: &str,
) -> Result<Vec<SearchPackageRecord>> {
    let output = Command::new("nix")
        .args([
            "--extra-experimental-features",
            "nix-command flakes",
            "search",
            installable,
            query,
            "--json",
            "--no-update-lock-file",
        ])
        .output()
        .with_context(|| format!("failed to run nix search for {installable}"))?;

    if !output.status.success() {
        bail!(
            "nix search failed with {}",
            output.status.code().unwrap_or_default()
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let Some(json_start) = stdout.find('{') else {
        return Ok(Vec::new());
    };

    let parsed = serde_json::from_str::<BTreeMap<String, SearchPackageMeta>>(&stdout[json_start..])
        .context("failed to parse nix search JSON output")?;

    let mut results = parsed
        .into_iter()
        .map(|(attr_path, meta)| SearchPackageRecord {
            expr: nixpkgs_expr_from_attr_path(&attr_path, system),
            name: meta.pname.clone().unwrap_or_else(|| {
                attr_path
                    .rsplit('.')
                    .next()
                    .unwrap_or(&attr_path)
                    .to_string()
            }),
            description: meta.description,
            version: meta.version,
            source: installable.to_string(),
            attr_path,
        })
        .collect::<Vec<_>>();

    results.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| left.attr_path.cmp(&right.attr_path))
    });
    Ok(results)
}

pub fn search_catalog_entries(
    installable: &str,
    query: &str,
    system: &str,
) -> Result<Vec<CatalogEntry>> {
    Ok(search_packages(installable, query, system)?
        .into_iter()
        .filter_map(|record| {
            let expr = record.expr?;
            Some(CatalogEntry {
                id: format!("search:{}", record.attr_path),
                name: record.name,
                category: "search".to_string(),
                group: Some("search".to_string()),
                expr,
                description: combine_description(record.description, record.version),
                keywords: vec![record.attr_path],
                workflow_tags: Vec::new(),
                lifecycle: None,
                source: Some(format!("search/{}", record.source)),
                platforms: vec![system.to_string()],
                desktop_entry_flag: None,
            })
        })
        .collect())
}

pub fn nixpkgs_expr_from_attr_path(attr_path: &str, system: &str) -> Option<String> {
    let legacy_prefix = format!("legacyPackages.{system}.");
    let packages_prefix = format!("packages.{system}.");

    if let Some(rest) = attr_path.strip_prefix(&legacy_prefix) {
        return Some(attr_path_to_expr(rest));
    }
    if let Some(rest) = attr_path.strip_prefix(&packages_prefix) {
        return Some(attr_path_to_expr(rest));
    }
    None
}

fn combine_description(description: Option<String>, version: Option<String>) -> Option<String> {
    match (description, version) {
        (Some(description), Some(version)) if !description.is_empty() => {
            Some(format!("{description} (v{version})"))
        }
        (Some(description), None) => Some(description),
        (None, Some(version)) => Some(format!("version {version}")),
        (None, None) => None,
        (Some(description), Some(_)) => Some(description),
    }
}

fn attr_path_to_expr(rest: &str) -> String {
    if !rest.contains('.') {
        return format!("pkgs.{rest}");
    }

    let parts = rest
        .split('.')
        .map(|part| format!("\"{}\"", part.replace('\\', "\\\\").replace('"', "\\\"")))
        .collect::<Vec<_>>()
        .join(" ");
    format!("(lib.getAttrFromPath [ {parts} ] pkgs)")
}

#[cfg(test)]
mod tests {
    use super::nixpkgs_expr_from_attr_path;

    #[test]
    fn converts_legacy_packages_attr_path() {
        assert_eq!(
            nixpkgs_expr_from_attr_path("legacyPackages.x86_64-linux.hello", "x86_64-linux"),
            Some("pkgs.hello".to_string())
        );
    }

    #[test]
    fn converts_nested_attr_path() {
        assert_eq!(
            nixpkgs_expr_from_attr_path(
                "legacyPackages.x86_64-linux.gnomeExtensions.blur-my-shell",
                "x86_64-linux"
            ),
            Some(
                "(lib.getAttrFromPath [ \"gnomeExtensions\" \"blur-my-shell\" ] pkgs)".to_string()
            )
        );
    }

    #[test]
    fn rejects_other_prefixes() {
        assert_eq!(
            nixpkgs_expr_from_attr_path("apps.x86_64-linux.hello", "x86_64-linux"),
            None
        );
    }
}
