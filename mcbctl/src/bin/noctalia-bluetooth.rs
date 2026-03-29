use mcbctl::{command_exists, emit_json, run_capture_allow_fail};

fn main() {
    if !command_exists("bluetoothctl") {
        emit_json("", "Bluetooth unavailable", "off");
        return;
    }

    let powered = run_capture_allow_fail("bluetoothctl", &["show"])
        .and_then(|s| {
            s.lines()
                .find(|l| l.trim_start().starts_with("Powered:"))
                .map(|l| l.split(':').nth(1).unwrap_or("").trim().to_string())
        })
        .unwrap_or_default();
    if powered != "yes" {
        emit_json("", "Bluetooth off", "off");
        return;
    }

    let devices = run_capture_allow_fail("bluetoothctl", &["devices", "Connected"])
        .unwrap_or_default()
        .lines()
        .filter_map(|line| {
            let mut it = line.split_whitespace();
            let _ = it.next()?;
            let _ = it.next()?;
            Some(it.collect::<Vec<_>>().join(" "))
        })
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();

    if devices.is_empty() {
        emit_json("", "Bluetooth on", "on");
        return;
    }
    let names = devices.join(", ");
    emit_json(
        &devices.len().to_string(),
        &format!("Connected: {names}"),
        "connected",
    );
}
