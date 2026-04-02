use anyhow::{Result, anyhow};
use mcbctl::{command_exists, home_dir, prepend_paths, xdg_state_home};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

const MODE_DIR: &str = "/run/current-system/specialisation";

#[derive(Debug)]
enum MenuPickError {
    NoMenuBackend,
    Cancelled,
}

#[path = "noctalia-gpu-mode/apply.rs"]
mod apply;
#[path = "noctalia-gpu-mode/menu.rs"]
mod menu;
#[path = "noctalia-gpu-mode/state.rs"]
mod state;

fn normalize_mode(mode: &str) -> String {
    if mode.starts_with("gpu-") {
        mode.to_string()
    } else {
        format!("gpu-{mode}")
    }
}

fn init_path() {
    let mut extra = Vec::new();
    extra.push(PathBuf::from("/run/wrappers/bin"));
    extra.push(PathBuf::from("/run/current-system/sw/bin"));
    if let Ok(user) = std::env::var("USER") {
        extra.push(PathBuf::from(format!("/etc/profiles/per-user/{user}/bin")));
    }
    extra.push(home_dir().join(".nix-profile/bin"));
    extra.push(home_dir().join(".local/bin"));
    prepend_paths(&extra);
}

fn real_main() -> Result<()> {
    init_path();

    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("--menu") => menu::menu_flow(),
        Some("--menu-cli") => menu::menu_flow_cli(),
        Some("--apply") => {
            let target = args.next().unwrap_or_default();
            apply::apply_mode(&target)
        }
        Some("--set-state") => {
            if let Some(mode) = args.next() {
                if mode == "base" {
                    let _ = state::write_state_mode("base");
                } else {
                    let _ = state::write_state_mode(&normalize_mode(&mode));
                }
            }
            Ok(())
        }
        _ => {
            state::emit_status();
            Ok(())
        }
    }
}

fn main() {
    if let Err(err) = real_main() {
        eprintln!("noctalia-gpu-mode: {err:#}");
        std::process::exit(1);
    }
}
