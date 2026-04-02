use super::*;

fn pick_menu(lines: &[String]) -> std::result::Result<String, MenuPickError> {
    let mut candidates: Vec<(&str, Vec<&str>)> = Vec::new();
    if command_exists("fuzzel") {
        candidates.push(("fuzzel", vec!["--dmenu", "--prompt", "GPU mode: "]));
    }
    if command_exists("wofi") {
        candidates.push(("wofi", vec!["--dmenu", "-p", "GPU mode"]));
    }
    if command_exists("rofi") {
        candidates.push(("rofi", vec!["-dmenu", "-p", "GPU mode"]));
    }
    let Some((bin, args)) = candidates.into_iter().next() else {
        return Err(MenuPickError::NoMenuBackend);
    };

    let mut child = Command::new(bin)
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| MenuPickError::Cancelled)?;

    if let Some(stdin) = child.stdin.as_mut() {
        let input = format!("{}\n", lines.join("\n"));
        let _ = stdin.write_all(input.as_bytes());
    }

    let out = child
        .wait_with_output()
        .map_err(|_| MenuPickError::Cancelled)?;
    if !out.status.success() {
        return Err(MenuPickError::Cancelled);
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn select_self_command() -> String {
    std::env::current_exe()
        .ok()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "noctalia-gpu-mode".to_string())
}

fn quote_shell(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\"'\"'"))
}

pub(super) fn menu_flow() -> Result<()> {
    let modes = state::list_modes();
    if modes.is_empty() {
        if command_exists("notify-send") {
            let mut hint = "No GPU specialisations found.".to_string();
            if let Some(path) = state::mode_file() {
                hint.push_str(&format!(
                    " Provide modes via {} or NOCTALIA_GPU_MODES.",
                    path.display()
                ));
            }
            let _ = Command::new("notify-send")
                .args(["GPU specialisation", &hint])
                .status();
        }
        return Ok(());
    }

    let mut labels = vec!["base".to_string()];
    let mut label_to_mode = HashMap::new();
    for m in &modes {
        let label = m.strip_prefix("gpu-").unwrap_or(m).to_string();
        labels.push(label.clone());
        label_to_mode.insert(label, m.clone());
    }

    let selection = match pick_menu(&labels) {
        Ok(v) => v,
        Err(MenuPickError::NoMenuBackend) => {
            let cmd_path = select_self_command();
            let shell_cmd = format!("{} --menu-cli", quote_shell(&cmd_path));
            let cmd = vec!["bash".to_string(), "-lc".to_string(), shell_cmd];
            let _ = apply::launch_in_terminal(&cmd);
            return Ok(());
        }
        Err(MenuPickError::Cancelled) => return Ok(()),
    };

    if selection.is_empty() || selection == "cancel" {
        return Ok(());
    }

    let target = if selection == "base" {
        "base".to_string()
    } else {
        match label_to_mode.get(&selection) {
            Some(v) => v.clone(),
            None => return Ok(()),
        }
    };

    let cmd_path = select_self_command();
    let cmd = vec![
        cmd_path,
        "--apply".to_string(),
        if target.is_empty() {
            "base".to_string()
        } else {
            target
        },
    ];
    let _ = apply::launch_in_terminal(&cmd);
    Ok(())
}

fn read_choice(max: usize) -> Option<usize> {
    let mut line = String::new();
    loop {
        line.clear();
        print!("GPU mode: ");
        let _ = io::stdout().flush();
        if io::stdin().read_line(&mut line).is_err() {
            return None;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(idx) = trimmed.parse::<usize>()
            && idx >= 1
            && idx <= max
        {
            return Some(idx);
        }
    }
}

pub(super) fn menu_flow_cli() -> Result<()> {
    let modes = state::list_modes();
    if modes.is_empty() {
        let mut hint = "No GPU specialisations found.".to_string();
        if let Some(path) = state::mode_file() {
            hint.push_str(&format!(
                " Provide modes via {} or NOCTALIA_GPU_MODES.",
                path.display()
            ));
        }
        println!("{hint}");
        return Ok(());
    }

    let mut labels = vec!["base".to_string()];
    let mut label_to_mode = HashMap::new();
    for m in &modes {
        let label = m.strip_prefix("gpu-").unwrap_or(m).to_string();
        labels.push(label.clone());
        label_to_mode.insert(label, m.clone());
    }

    println!("Select GPU mode:");
    for (idx, label) in labels.iter().enumerate() {
        println!("  {}) {}", idx + 1, label);
    }
    let Some(choice) = read_choice(labels.len()) else {
        return Ok(());
    };
    let selection = &labels[choice - 1];

    if selection == "base" {
        apply::apply_mode("base")
    } else if let Some(mode) = label_to_mode.get(selection) {
        apply::apply_mode(mode)
    } else {
        Ok(())
    }
}
