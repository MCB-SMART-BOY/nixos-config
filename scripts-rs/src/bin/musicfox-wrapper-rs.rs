use anyhow::{Context, Result, anyhow};
use scripts_rs::{command_exists, run_status_inherit, xdg_config_home};
use std::fs;
use std::path::PathBuf;

fn ensure_ini_key(content: &str, section: &str, key: &str, value: &str) -> String {
    let mut out = Vec::<String>::new();
    let mut in_section = false;
    let mut section_seen = false;
    let mut found_key = false;

    for line in content.lines() {
        let trimmed = line.trim();
        let is_section = trimmed.starts_with('[') && trimmed.ends_with(']');
        if is_section {
            if in_section && !found_key {
                out.push(format!("{key}={value}"));
            }
            let current = trimmed.trim_start_matches('[').trim_end_matches(']');
            in_section = current == section;
            if in_section {
                section_seen = true;
                found_key = false;
            }
            out.push(line.to_string());
            continue;
        }

        if in_section {
            let key_match = trimmed
                .split_once('=')
                .map(|(lhs, _)| lhs.trim() == key)
                .unwrap_or(false);
            if key_match {
                if !found_key {
                    out.push(format!("{key}={value}"));
                    found_key = true;
                }
                continue;
            }
        }

        out.push(line.to_string());
    }

    if in_section && !found_key {
        out.push(format!("{key}={value}"));
    }

    if !section_seen {
        if !out.is_empty() && !out.last().is_some_and(|l| l.is_empty()) {
            out.push(String::new());
        }
        out.push(format!("[{section}]"));
        out.push(format!("{key}={value}"));
    }

    out.join("\n") + "\n"
}

fn resolve_real_bin() -> Option<String> {
    if let Ok(v) = std::env::var("MUSICFOX_REAL_BIN")
        && !v.trim().is_empty()
    {
        return Some(v);
    }
    for candidate in ["go-musicfox-real", "musicfox-real"] {
        if command_exists(candidate) {
            return Some(candidate.to_string());
        }
    }
    None
}

fn real_main() -> Result<()> {
    let root = std::env::var("MUSICFOX_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| xdg_config_home().join("go-musicfox"));
    let cfg_file = root.join("go-musicfox.ini");
    fs::create_dir_all(&root).with_context(|| format!("failed to create {}", root.display()))?;

    if !cfg_file.is_file() {
        fs::write(
            &cfg_file,
            "[player]\nengine=mpv\nmpvBin=mpv\n\n[unm]\nswitch=true\nsources=kuwo,kugou,migu,qq\nsearchLimit=3\nskipInvalidTracks=true\n",
        )?;
    }

    let mut content = fs::read_to_string(&cfg_file).unwrap_or_default();
    content = ensure_ini_key(&content, "player", "engine", "mpv");
    content = ensure_ini_key(&content, "player", "mpvBin", "mpv");
    content = ensure_ini_key(&content, "unm", "switch", "true");
    content = ensure_ini_key(&content, "unm", "sources", "kuwo,kugou,migu,qq");
    content = ensure_ini_key(&content, "unm", "searchLimit", "3");
    content = ensure_ini_key(&content, "unm", "skipInvalidTracks", "true");
    fs::write(&cfg_file, content)?;

    let real_bin = resolve_real_bin().ok_or_else(|| anyhow!("musicfox real binary not found"))?;
    // SAFETY: CLI process updates env before spawning child and does not use threads here.
    unsafe {
        std::env::set_var("MUSICFOX_ROOT", &root);
    }
    let args: Vec<String> = std::env::args().skip(1).collect();
    let status = run_status_inherit(&real_bin, &args)?;
    std::process::exit(status.code().unwrap_or(1));
}

fn main() {
    if let Err(err) = real_main() {
        eprintln!("musicfox-wrapper-rs: {err:#}");
        std::process::exit(1);
    }
}
