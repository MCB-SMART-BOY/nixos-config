use crate::write_file_atomic;
use anyhow::{Context, Result, bail};
use flate2::Compression;
use flate2::write::GzEncoder;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use tar::Builder as TarBuilder;
use walkdir::WalkDir;
use zip::CompressionMethod;
use zip::write::SimpleFileOptions;

const RELEASE_BINARIES: &[&str] = &[
    "mcbctl",
    "mcb-deploy",
    "clash-verge-prestart",
    "mcb-tun-route",
    "lock-screen",
    "niri-run",
    "steam-gamescope",
    "wallpaper-random",
    "electron-auto-gpu",
    "zed-auto-gpu",
    "flatpak-setup",
    "musicfox-wrapper",
    "noctalia-bluetooth",
    "noctalia-cpu",
    "noctalia-disk",
    "noctalia-flake-updates",
    "noctalia-gpu-current",
    "noctalia-gpu-mode",
    "noctalia-memory",
    "noctalia-net-speed",
    "noctalia-net-status",
    "noctalia-power",
    "noctalia-proxy-status",
    "noctalia-temperature",
    "update-upstream-apps",
    "update-yesplaymusic-source",
    "update-zed-source",
];

const DEFAULT_RELEASE_REPOSITORY: &str = "MCB-SMART-BOY/nixos-config";
const RELEASE_BUNDLE_TARGETS: &[&str] = &[
    "x86_64-unknown-linux-gnu",
    "x86_64-pc-windows-msvc",
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
];

pub struct ReleaseBundleOptions {
    pub target: String,
    pub version: String,
    pub bin_dir: PathBuf,
    pub out_dir: PathBuf,
}

pub struct ReleaseBundleReport {
    pub archive: PathBuf,
    pub checksum_file: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ReleaseAssetManifest {
    pub target: String,
    pub archive_name: String,
    pub archive_url: String,
    pub checksum_name: String,
    pub checksum_url: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ReleaseManifest {
    pub repository: String,
    pub version: String,
    pub tag: String,
    pub release_url: String,
    pub assets: Vec<ReleaseAssetManifest>,
}

pub fn default_release_repository() -> String {
    std::env::var("GITHUB_REPOSITORY")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_RELEASE_REPOSITORY.to_string())
}

pub fn build_release_manifest(repository: &str, version: &str) -> ReleaseManifest {
    let tag = normalize_release_tag(version);
    let normalized_repository = repository.trim().trim_matches('/').to_string();
    let release_base =
        format!("https://github.com/{normalized_repository}/releases/download/{tag}");
    let assets = RELEASE_BUNDLE_TARGETS
        .iter()
        .map(|target| {
            let archive_name = release_bundle_archive_name(&tag, target);
            let checksum_name = release_bundle_checksum_name(&archive_name);
            ReleaseAssetManifest {
                target: (*target).to_string(),
                archive_url: format!("{release_base}/{archive_name}"),
                checksum_url: format!("{release_base}/{checksum_name}"),
                archive_name,
                checksum_name,
            }
        })
        .collect::<Vec<_>>();

    ReleaseManifest {
        repository: normalized_repository.clone(),
        version: normalized_release_version(&tag),
        tag: tag.clone(),
        release_url: format!("https://github.com/{normalized_repository}/releases/tag/{tag}"),
        assets,
    }
}

pub fn render_release_manifest_json(repository: &str, version: &str) -> Result<String> {
    serde_json::to_string_pretty(&build_release_manifest(repository, version))
        .context("failed to render release manifest")
}

pub fn build_release_bundle(options: &ReleaseBundleOptions) -> Result<ReleaseBundleReport> {
    fs::create_dir_all(&options.out_dir)
        .with_context(|| format!("failed to create {}", options.out_dir.display()))?;

    let suffix = binary_suffix(&options.target);
    let bundle_name = format!("mcbctl-{}-{}", options.version, options.target);
    let staging_dir = std::env::temp_dir().join(format!(
        "mcbctl-release-bundle-{}-{}",
        std::process::id(),
        chrono_like_millis()
    ));
    let bundle_root = staging_dir.join(&bundle_name);
    fs::create_dir_all(&bundle_root)
        .with_context(|| format!("failed to create {}", bundle_root.display()))?;

    for binary in RELEASE_BINARIES {
        let source = options.bin_dir.join(format!("{binary}{suffix}"));
        if !source.is_file() {
            bail!("missing built binary: {}", source.display());
        }
        fs::copy(&source, bundle_root.join(format!("{binary}{suffix}")))
            .with_context(|| format!("failed to stage {}", source.display()))?;
    }

    let deploy_alias = bundle_root.join(format!("deploy{suffix}"));
    fs::copy(
        bundle_root.join(format!("mcb-deploy{suffix}")),
        &deploy_alias,
    )
    .with_context(|| format!("failed to create {}", deploy_alias.display()))?;

    write_file_atomic(
        &bundle_root.join("README.txt"),
        &render_release_readme(&options.version, &options.target),
    )?;

    let archive = if options.target.contains("windows") {
        let path = options.out_dir.join(release_bundle_archive_name(
            &options.version,
            &options.target,
        ));
        write_zip_bundle(&path, &staging_dir, &bundle_root)?;
        path
    } else {
        let path = options.out_dir.join(release_bundle_archive_name(
            &options.version,
            &options.target,
        ));
        write_tar_gz_bundle(&path, &staging_dir, &bundle_root)?;
        path
    };

    let checksum = sha256_file(&archive)?;
    let checksum_file = options.out_dir.join(release_bundle_checksum_name(
        archive
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("mcbctl-release"),
    ));
    write_file_atomic(
        &checksum_file,
        &format!(
            "{}  {}\n",
            checksum,
            archive
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("archive")
        ),
    )?;

    fs::remove_dir_all(&staging_dir).ok();

    Ok(ReleaseBundleReport {
        archive,
        checksum_file,
    })
}

fn binary_suffix(target: &str) -> &'static str {
    if target.contains("windows") {
        ".exe"
    } else {
        ""
    }
}

fn render_release_readme(version: &str, target: &str) -> String {
    format!(
        "mcbctl {version}\nTarget: {target}\n\nThis archive contains the prebuilt Rust command suite for this repository.\n\nPrimary entrypoints:\n- mcbctl\n- mcb-deploy\n- deploy\n"
    )
}

fn normalize_release_tag(version: &str) -> String {
    if version.starts_with('v') {
        version.to_string()
    } else {
        format!("v{version}")
    }
}

fn normalized_release_version(tag: &str) -> String {
    tag.strip_prefix('v').unwrap_or(tag).to_string()
}

fn release_bundle_archive_name(version: &str, target: &str) -> String {
    if target.contains("windows") {
        format!("mcbctl-{version}-{target}.zip")
    } else {
        format!("mcbctl-{version}-{target}.tar.gz")
    }
}

fn release_bundle_checksum_name(archive_name: &str) -> String {
    format!("{archive_name}.sha256")
}

fn write_tar_gz_bundle(path: &Path, root: &Path, bundle_root: &Path) -> Result<()> {
    let file =
        fs::File::create(path).with_context(|| format!("failed to create {}", path.display()))?;
    let encoder = GzEncoder::new(file, Compression::default());
    let mut builder = TarBuilder::new(encoder);

    for entry in WalkDir::new(bundle_root).into_iter().flatten() {
        if !entry.file_type().is_file() {
            continue;
        }
        let relative = entry
            .path()
            .strip_prefix(root)
            .unwrap_or(entry.path())
            .to_path_buf();
        builder
            .append_path_with_name(entry.path(), &relative)
            .with_context(|| format!("failed to append {}", entry.path().display()))?;
    }

    builder.finish().context("failed to finish tar.gz archive")
}

fn write_zip_bundle(path: &Path, root: &Path, bundle_root: &Path) -> Result<()> {
    let file =
        fs::File::create(path).with_context(|| format!("failed to create {}", path.display()))?;
    let mut zip = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    for entry in WalkDir::new(bundle_root).into_iter().flatten() {
        if !entry.file_type().is_file() {
            continue;
        }
        let relative = entry
            .path()
            .strip_prefix(root)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .replace('\\', "/");
        zip.start_file(&relative, options)
            .with_context(|| format!("failed to start zip entry {relative}"))?;
        let mut input = fs::File::open(entry.path())
            .with_context(|| format!("failed to open {}", entry.path().display()))?;
        std::io::copy(&mut input, &mut zip)
            .with_context(|| format!("failed to write zip entry {relative}"))?;
    }

    zip.finish().context("failed to finish zip archive")?;
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String> {
    let mut file =
        fs::File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let read = file
            .read(&mut buffer)
            .with_context(|| format!("failed to read {}", path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn chrono_like_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_release_manifest_uses_tagged_asset_urls_for_all_targets() {
        let manifest = build_release_manifest("MCB-SMART-BOY/nixos-config", "6.0.0");

        assert_eq!(manifest.version, "6.0.0");
        assert_eq!(manifest.tag, "v6.0.0");
        assert_eq!(
            manifest.release_url,
            "https://github.com/MCB-SMART-BOY/nixos-config/releases/tag/v6.0.0"
        );
        assert_eq!(manifest.assets.len(), 4);
        assert_eq!(
            manifest.assets[0].archive_name,
            "mcbctl-v6.0.0-x86_64-unknown-linux-gnu.tar.gz"
        );
        assert_eq!(
            manifest.assets[0].checksum_name,
            "mcbctl-v6.0.0-x86_64-unknown-linux-gnu.tar.gz.sha256"
        );
        assert!(
            manifest.assets[0]
                .archive_url
                .ends_with("/download/v6.0.0/mcbctl-v6.0.0-x86_64-unknown-linux-gnu.tar.gz")
        );
    }

    #[test]
    fn render_release_manifest_json_outputs_pretty_json() {
        let json = render_release_manifest_json("MCB-SMART-BOY/nixos-config", "v6.0.0")
            .expect("manifest json should render");

        assert!(json.contains("\"tag\": \"v6.0.0\""));
        assert!(json.contains("\"repository\": \"MCB-SMART-BOY/nixos-config\""));
        assert!(json.contains("x86_64-pc-windows-msvc"));
    }
}
