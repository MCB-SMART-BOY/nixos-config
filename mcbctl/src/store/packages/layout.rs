use crate::{write_file_atomic, write_managed_file};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

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
    write_managed_file(
        &aggregator,
        "home-packages-aggregator",
        &render_managed_packages_aggregator_file(),
        &["# 机器管理的用户软件入口"],
    )?;

    let grouped_dir = managed_dir.join("packages");
    fs::create_dir_all(&grouped_dir)
        .with_context(|| format!("failed to create {}", grouped_dir.display()))?;

    let readme = grouped_dir.join("README.md");
    if !readme.exists() {
        write_file_atomic(&readme, &render_managed_packages_readme())?;
    }

    Ok(())
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
