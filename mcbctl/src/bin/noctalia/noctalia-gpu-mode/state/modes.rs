use super::super::*;

pub(crate) fn mode_file() -> Option<PathBuf> {
    let config_home = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|_| std::env::var("HOME").map(PathBuf::from))
        .ok()?;
    Some(config_home.join("noctalia/gpu-modes"))
}

fn list_modes_from_file() -> Vec<String> {
    let Some(file) = mode_file() else {
        return Vec::new();
    };
    let Ok(text) = fs::read_to_string(file) else {
        return Vec::new();
    };
    text.lines()
        .filter_map(super::current::parse_mode_line)
        .collect()
}

fn list_modes_from_env() -> Vec<String> {
    let Ok(raw) = std::env::var("NOCTALIA_GPU_MODES") else {
        return Vec::new();
    };
    raw.split_whitespace()
        .filter(|s| !s.is_empty())
        .map(normalize_mode)
        .collect()
}

pub(crate) fn list_modes() -> Vec<String> {
    let mut modes = Vec::new();
    if let Ok(entries) = fs::read_dir(MODE_DIR) {
        for entry in entries.flatten() {
            let Ok(ft) = entry.file_type() else {
                continue;
            };
            if !ft.is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("gpu-") {
                modes.push(name);
            }
        }
    }
    if !modes.is_empty() {
        modes.sort();
        modes.dedup();
        return modes;
    }

    let from_file = list_modes_from_file();
    if !from_file.is_empty() {
        return from_file;
    }

    list_modes_from_env()
}
