use fs2::FileExt;
use scripts_rs::{emit_json, find_repo_root, xdg_cache_home};
use serde_json::Value;
use std::env;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const TTL_SECONDS: u64 = 900;

fn resolve_repo_root() -> Option<PathBuf> {
    let etc_root = PathBuf::from("/etc/nixos");
    if etc_root.join("flake.lock").is_file() {
        return Some(etc_root);
    }

    if let Ok(root) = env::var("MCB_FLAKE_ROOT") {
        let p = PathBuf::from(root);
        if p.join("flake.lock").is_file() {
            return Some(p);
        }
    }

    if let Ok(nixos_flake) = env::var("NIXOS_FLAKE") {
        let raw = nixos_flake.split('#').next().unwrap_or("");
        if !raw.is_empty() {
            let p = PathBuf::from(raw);
            if p.join("flake.lock").is_file() {
                return Some(p);
            }
        }
    }

    let home = env::var("HOME").unwrap_or_default();
    for dir in [
        format!("{home}/nixos-config"),
        format!("{home}/.dotfiles/nixos-config"),
        format!("{home}/dev/nixos-config"),
        format!("{home}/.config/nixos"),
    ] {
        let p = PathBuf::from(dir);
        if p.join("flake.lock").is_file() {
            return Some(p);
        }
    }

    if let Ok(cwd) = env::current_dir()
        && cwd.join("flake.lock").is_file()
    {
        return Some(cwd);
    }

    find_repo_root().ok()
}

fn cache_age(path: &Path) -> Option<u64> {
    let mtime = fs::metadata(path).ok()?.modified().ok()?;
    let now = SystemTime::now();
    let now_secs = now.duration_since(UNIX_EPOCH).ok()?.as_secs();
    let mtime_secs = mtime.duration_since(UNIX_EPOCH).ok()?.as_secs();
    Some(now_secs.saturating_sub(mtime_secs))
}

fn git_ls_remote(owner: &str, repo: &str, r#ref: &str) -> Option<String> {
    let url = format!("https://github.com/{owner}/{repo}.git");
    let out = std::process::Command::new("git")
        .env("GIT_TERMINAL_PROMPT", "0")
        .args(["ls-remote", &url, r#ref])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout);
    s.lines()
        .next()
        .and_then(|l| l.split_whitespace().next())
        .map(ToOwned::to_owned)
}

fn write_cache(path: &Path, text: &str, tooltip: &str, class: &str) {
    let payload = serde_json::json!({
        "text": text,
        "tooltip": tooltip,
        "class": class
    });
    let _ = fs::write(path, format!("{payload}\n"));
}

fn main() {
    let Some(repo_root) = resolve_repo_root() else {
        emit_json("", "flake.lock not found", "error");
        return;
    };
    let lock_file = repo_root.join("flake.lock");
    let cache_dir = xdg_cache_home();
    let cache_file = cache_dir.join("noctalia-flake-updates.json");
    let lock_path = cache_dir.join("noctalia-flake-updates.lock");
    let _ = fs::create_dir_all(&cache_dir);

    if cache_file.is_file()
        && cache_age(&cache_file).is_some_and(|age| age < TTL_SECONDS)
        && let Ok(cached) = fs::read_to_string(&cache_file)
    {
        print!("{cached}");
        return;
    }

    let Ok(lock_fd) = File::create(&lock_path) else {
        if let Ok(cached) = fs::read_to_string(&cache_file) {
            print!("{cached}");
            return;
        }
        emit_json("", "Update check in progress", "pending");
        return;
    };
    if lock_fd.try_lock_exclusive().is_err() {
        if let Ok(cached) = fs::read_to_string(&cache_file) {
            print!("{cached}");
            return;
        }
        emit_json("", "Update check in progress", "pending");
        return;
    }

    if !lock_file.is_file() {
        write_cache(&cache_file, "", "flake.lock not found", "error");
        if let Ok(cached) = fs::read_to_string(&cache_file) {
            print!("{cached}");
        }
        return;
    }
    if !scripts_rs::command_exists("git") {
        write_cache(&cache_file, "", "git not installed", "error");
        if let Ok(cached) = fs::read_to_string(&cache_file) {
            print!("{cached}");
        }
        return;
    }

    let Ok(raw) = fs::read_to_string(&lock_file) else {
        write_cache(&cache_file, "", "failed to read flake.lock", "error");
        if let Ok(cached) = fs::read_to_string(&cache_file) {
            print!("{cached}");
        }
        return;
    };
    let Ok(parsed) = serde_json::from_str::<Value>(&raw) else {
        write_cache(&cache_file, "", "invalid flake.lock", "error");
        if let Ok(cached) = fs::read_to_string(&cache_file) {
            print!("{cached}");
        }
        return;
    };

    let mut updates = 0usize;
    let mut names = Vec::<String>::new();

    if let Some(nodes) = parsed.get("nodes").and_then(|n| n.as_object()) {
        for (name, node) in nodes {
            let locked = node.get("locked").and_then(|x| x.as_object());
            let original = node.get("original").and_then(|x| x.as_object());
            let Some(locked) = locked else { continue };

            let ty = locked.get("type").and_then(|x| x.as_str()).unwrap_or("");
            if ty != "github" {
                continue;
            }

            let owner = locked.get("owner").and_then(|x| x.as_str()).unwrap_or("");
            let repo = locked.get("repo").and_then(|x| x.as_str()).unwrap_or("");
            let rev = locked.get("rev").and_then(|x| x.as_str()).unwrap_or("");
            if owner.is_empty() || repo.is_empty() || rev.is_empty() {
                continue;
            }
            let r#ref = original
                .and_then(|o| o.get("ref"))
                .and_then(|x| x.as_str())
                .filter(|s| !s.is_empty())
                .unwrap_or("HEAD");

            let remote_rev = git_ls_remote(owner, repo, r#ref).unwrap_or_default();
            if !remote_rev.is_empty() && remote_rev != rev {
                updates += 1;
                names.push(name.to_string());
            }
        }
    }

    if updates > 0 {
        let tooltip = format!("Updates available: {updates}\\n{}", names.join(" "));
        write_cache(&cache_file, &updates.to_string(), &tooltip, "updates");
    } else {
        write_cache(&cache_file, "", "Up to date", "up-to-date");
    }

    if let Ok(cached) = fs::read_to_string(&cache_file) {
        print!("{cached}");
    }
}
