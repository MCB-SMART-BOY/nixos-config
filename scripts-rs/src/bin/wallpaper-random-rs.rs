use rand::prelude::IndexedRandom;
use scripts_rs::{command_exists, home_dir, prepend_paths};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;
use walkdir::WalkDir;

fn wait_for_wayland() -> bool {
    let attempts = 50;
    for _ in 0..attempts {
        if let (Ok(display), Ok(runtime)) = (
            std::env::var("WAYLAND_DISPLAY"),
            std::env::var("XDG_RUNTIME_DIR"),
        ) {
            let sock = Path::new(&runtime).join(display);
            if sock.exists() {
                return true;
            }
        }

        if let Ok(runtime) = std::env::var("XDG_RUNTIME_DIR") {
            let wayland0 = Path::new(&runtime).join("wayland-0");
            if wayland0.exists() {
                // SAFETY: single-threaded CLI, environment mutation is local to process startup.
                unsafe {
                    std::env::set_var("WAYLAND_DISPLAY", "wayland-0");
                }
                return true;
            }
            if let Ok(entries) = fs::read_dir(&runtime) {
                for ent in entries.flatten() {
                    let name = ent.file_name().to_string_lossy().to_string();
                    if name.starts_with("wayland-") {
                        // SAFETY: single-threaded CLI, environment mutation is local to process startup.
                        unsafe {
                            std::env::set_var("WAYLAND_DISPLAY", name);
                        }
                        return true;
                    }
                }
            }
        }
        thread::sleep(Duration::from_millis(200));
    }
    false
}

fn collect_wallpapers(candidates: &[PathBuf]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for dir in candidates {
        if !dir.is_dir() {
            continue;
        }
        let found: Vec<PathBuf> = WalkDir::new(dir)
            .follow_links(true)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
            .map(|e| e.into_path())
            .filter(|p| {
                p.extension()
                    .and_then(|s| s.to_str())
                    .map(|ext| {
                        matches!(
                            ext.to_ascii_lowercase().as_str(),
                            "png" | "jpg" | "jpeg" | "webp"
                        )
                    })
                    .unwrap_or(false)
            })
            .collect();
        if !found.is_empty() {
            out = found;
            break;
        }
    }
    out
}

fn main() {
    let user = std::env::var("USER").unwrap_or_default();
    let home = home_dir();
    prepend_paths(&[
        PathBuf::from("/run/wrappers/bin"),
        home.join(".local/bin"),
        home.join(".nix-profile/bin"),
        PathBuf::from(format!("/etc/profiles/per-user/{user}/bin")),
        PathBuf::from("/run/current-system/sw/bin"),
    ]);

    if !wait_for_wayland() {
        eprintln!("wallpaper-random-rs: Wayland socket not ready");
        std::process::exit(1);
    }

    let mut candidates = Vec::<PathBuf>::new();
    if let Ok(v) = std::env::var("WALLPAPER_DIR") {
        candidates.extend(v.split(':').filter(|s| !s.is_empty()).map(PathBuf::from));
    }
    candidates.push(home.join("Pictures/Wallpapers"));

    let mut wallpapers = collect_wallpapers(&candidates);
    if wallpapers.is_empty() {
        eprintln!("wallpaper-random-rs: no wallpapers found");
        std::process::exit(1);
    }

    let cache_dir = home.join(".cache");
    let lock_image = cache_dir.join("wallpaper-lock.png");
    let current_image = cache_dir.join("wallpaper-current");
    let _ = fs::create_dir_all(&cache_dir);

    let current_target = fs::read_link(&current_image)
        .ok()
        .and_then(|p| p.canonicalize().ok());
    if let Some(current) = current_target
        && wallpapers.len() > 1
    {
        wallpapers.retain(|p| p.canonicalize().ok().is_none_or(|c| c != current));
    }

    let mut rng = rand::rng();
    let Some(choice) = wallpapers.choose(&mut rng).cloned() else {
        eprintln!("wallpaper-random-rs: no wallpaper selected");
        std::process::exit(1);
    };

    let _ = fs::remove_file(&lock_image);
    let _ = fs::remove_file(&current_image);
    if std::os::unix::fs::symlink(&choice, &lock_image).is_err()
        || std::os::unix::fs::symlink(&choice, &current_image).is_err()
    {
        eprintln!(
            "wallpaper-random-rs: failed to write symlink in {}",
            cache_dir.display()
        );
        std::process::exit(1);
    }

    if !command_exists("noctalia-shell") {
        eprintln!("wallpaper-random-rs: noctalia-shell not found in PATH");
        std::process::exit(1);
    }

    let status = Command::new("noctalia-shell")
        .args(["ipc", "call", "wallpaper", "set"])
        .arg(choice.to_string_lossy().to_string())
        .arg("")
        .status();
    match status {
        Ok(s) if s.success() => {}
        Ok(s) => std::process::exit(s.code().unwrap_or(1)),
        Err(err) => {
            eprintln!("wallpaper-random-rs: {err}");
            std::process::exit(1);
        }
    }
}
