use mcbctl::{emit_json, format_rate, run_capture_allow_fail, xdg_cache_home};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

fn default_iface() -> Option<String> {
    let out = run_capture_allow_fail("ip", &["-4", "route", "list", "default"])?;
    let line = out.lines().next()?;
    let cols: Vec<&str> = line.split_whitespace().collect();
    cols.iter()
        .position(|x| *x == "dev")
        .and_then(|i| cols.get(i + 1).copied())
        .map(ToOwned::to_owned)
}

fn fallback_iface() -> Option<String> {
    let mut picked = None::<String>;
    if let Ok(entries) = fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name != "lo" {
                picked = Some(name);
                break;
            }
        }
    }
    picked
}

fn read_u64(path: &str) -> Option<u64> {
    fs::read_to_string(path).ok()?.trim().parse::<u64>().ok()
}

fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn main() {
    let cache_dir = xdg_cache_home();
    let state_file = cache_dir.join("noctalia-net-speed.state");
    let _ = fs::create_dir_all(&cache_dir);

    let iface = default_iface().or_else(fallback_iface).unwrap_or_default();
    if iface.is_empty() {
        emit_json("", "No network interface", "disconnected");
        return;
    }

    let rx_path = format!("/sys/class/net/{iface}/statistics/rx_bytes");
    let tx_path = format!("/sys/class/net/{iface}/statistics/tx_bytes");
    let state_path = format!("/sys/class/net/{iface}/operstate");

    let Some(rx_now) = read_u64(&rx_path) else {
        emit_json("", &format!("No stats for {iface}"), "disconnected");
        return;
    };
    let Some(tx_now) = read_u64(&tx_path) else {
        emit_json("", &format!("No stats for {iface}"), "disconnected");
        return;
    };

    let operstate = fs::read_to_string(state_path)
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "up".to_string());
    let now = now_epoch();

    let (rx_prev, tx_prev, time_prev) = fs::read_to_string(&state_file)
        .ok()
        .and_then(|s| {
            let mut it = s.split_whitespace().filter_map(|x| x.parse::<u64>().ok());
            Some((it.next()?, it.next()?, it.next()?))
        })
        .unwrap_or((rx_now, tx_now, now));

    let dt = now.saturating_sub(time_prev).max(1);
    let rx_rate = rx_now.saturating_sub(rx_prev) / dt;
    let tx_rate = tx_now.saturating_sub(tx_prev) / dt;
    let _ = fs::write(&state_file, format!("{rx_now} {tx_now} {now}\n"));

    let rx_human = format_rate(rx_rate);
    let tx_human = format_rate(tx_rate);
    let text = format!("U:{tx_human} D:{rx_human}");
    let tooltip = format!("Iface {iface}\\nUp {tx_human}\\nDown {rx_human}");

    if operstate != "up" {
        emit_json(&text, &tooltip, "disconnected");
    } else {
        emit_json(&text, &tooltip, "connected");
    }
}
