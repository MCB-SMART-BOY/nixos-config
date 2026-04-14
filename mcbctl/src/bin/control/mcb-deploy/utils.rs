use super::*;

pub(crate) fn strip_comment(line: &str) -> &str {
    line.split('#').next().unwrap_or("")
}

pub(crate) fn first_quoted(line: &str) -> Option<String> {
    let start = line.find('"')?;
    let rest = &line[start + 1..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

pub(crate) fn is_valid_username(v: &str) -> bool {
    let mut chars = v.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first.is_ascii_lowercase()) {
        return false;
    }
    chars.all(|c| c == '_' || c == '-' || c.is_ascii_lowercase() || c.is_ascii_digit())
}

fn can_write_dir_with_probe_cleanup<F>(path: &Path, cleanup_probe: F) -> bool
where
    F: FnOnce(&Path) -> Result<()>,
{
    if fs::create_dir_all(path).is_err() {
        return false;
    }
    let probe = path.join(format!(".mcbctl-write-{}", std::process::id()));
    match fs::write(&probe, b"ok") {
        Ok(_) => {
            // This is only a rootless writeability probe; cleanup stays best-effort.
            let _ = cleanup_probe(&probe);
            true
        }
        Err(_) => false,
    }
}

pub(crate) fn can_write_dir(path: &Path) -> bool {
    can_write_dir_with_probe_cleanup(path, |probe| {
        fs::remove_file(probe)
            .with_context(|| format!("failed to remove write probe {}", probe.display()))
    })
}

pub(crate) fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/"))
}

pub(crate) fn path_looks_repo(dir: &Path) -> bool {
    dir.join("flake.nix").is_file()
        && dir.join("hosts").is_dir()
        && dir.join("modules").is_dir()
        && dir.join("home").is_dir()
}

pub(crate) fn detect_repo_dir() -> Result<PathBuf> {
    if let Ok(root) = find_repo_root() {
        return Ok(root);
    }
    if let Ok(exe) = std::env::current_exe() {
        let mut cur = exe.parent().map(|p| p.to_path_buf());
        while let Some(dir) = cur {
            if path_looks_repo(&dir) {
                return Ok(dir);
            }
            cur = dir.parent().map(|p| p.to_path_buf());
        }
    }
    let cwd = std::env::current_dir()?;
    if path_looks_repo(&cwd) {
        return Ok(cwd);
    }
    bail!("mcbctl: cannot locate repository root");
}

pub(crate) fn create_temp_dir(prefix: &str) -> Result<PathBuf> {
    let base = std::env::temp_dir();
    for n in 0..2048u32 {
        let p = base.join(format!(
            "{prefix}-{}-{}-{n}",
            std::process::id(),
            chrono_like_millis()
        ));
        if fs::create_dir(&p).is_ok() {
            return Ok(p);
        }
    }
    bail!("failed to create temporary directory");
}

pub(crate) fn create_temp_path(prefix: &str, ext: &str) -> Result<PathBuf> {
    let base = std::env::temp_dir();
    for n in 0..2048u32 {
        let p = base.join(format!(
            "{prefix}-{}-{}-{n}.{ext}",
            std::process::id(),
            chrono_like_millis()
        ));
        if !p.exists() {
            return Ok(p);
        }
    }
    bail!("failed to allocate temporary path");
}

fn chrono_like_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

pub(crate) fn copy_recursively(src: &Path, dst: &Path) -> Result<()> {
    if src.is_file() {
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(src, dst)?;
        return Ok(());
    }
    fs::create_dir_all(dst)?;
    for entry in WalkDir::new(src).into_iter().flatten() {
        let path = entry.path();
        let rel = path.strip_prefix(src).unwrap_or(path);
        let target = dst.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(path, target)?;
        }
    }
    Ok(())
}

pub(crate) fn copy_recursively_if_missing(src: &Path, dst: &Path) -> Result<()> {
    if src.is_file() {
        if dst.exists() {
            return Ok(());
        }
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(src, dst)?;
        return Ok(());
    }

    fs::create_dir_all(dst)?;
    for entry in WalkDir::new(src).into_iter().flatten() {
        let path = entry.path();
        let rel = path.strip_prefix(src).unwrap_or(path);
        let target = dst.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)?;
        } else if entry.file_type().is_file() {
            if target.exists() {
                continue;
            }
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(path, target)?;
        }
    }
    Ok(())
}

pub(crate) fn is_valid_host_name(v: &str) -> bool {
    if v.is_empty() || v.len() > 63 {
        return false;
    }
    if v.starts_with('-') || v.ends_with('-') {
        return false;
    }
    v.bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_write_dir_accepts_writable_directory_and_cleans_probe() -> Result<()> {
        let root = create_temp_dir("mcbctl-utils-can-write")?;
        let target = root.join("target");

        assert!(can_write_dir(&target));

        let leftover = fs::read_dir(&target)?.flatten().any(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with(".mcbctl-write-")
        });
        assert!(!leftover);

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn can_write_dir_keeps_success_when_probe_cleanup_fails() -> Result<()> {
        let root = create_temp_dir("mcbctl-utils-probe-cleanup")?;
        let target = root.join("target");

        let ok = can_write_dir_with_probe_cleanup(&target, |_probe| {
            Err(anyhow::anyhow!("probe cleanup failed"))
        });

        assert!(ok);
        let leftover = fs::read_dir(&target)?.flatten().any(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with(".mcbctl-write-")
        });
        assert!(leftover);

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn can_write_dir_rejects_uncreatable_target_path() -> Result<()> {
        let root = create_temp_dir("mcbctl-utils-can-write-blocked")?;
        let blocked = root.join("blocked");
        fs::write(&blocked, "file blocks nested dir creation")?;

        assert!(!can_write_dir(&blocked.join("nested")));

        fs::remove_dir_all(root)?;
        Ok(())
    }
}
