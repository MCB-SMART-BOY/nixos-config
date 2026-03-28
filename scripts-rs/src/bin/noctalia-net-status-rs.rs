use scripts_rs::{command_exists, emit_json, run_capture_allow_fail};

fn first_ipv4_of_iface(iface: &str) -> Option<String> {
    let out = run_capture_allow_fail("ip", &["-4", "addr", "show", "dev", iface])?;
    out.lines().find_map(|line| {
        let t = line.trim();
        if !t.starts_with("inet ") {
            return None;
        }
        t.split_whitespace().nth(1).map(ToOwned::to_owned)
    })
}

fn main() {
    if command_exists("nmcli") {
        let line = run_capture_allow_fail(
            "nmcli",
            &["-t", "-f", "DEVICE,TYPE,STATE,CONNECTION", "dev", "status"],
        )
        .unwrap_or_default()
        .lines()
        .find(|l| l.split(':').nth(2) == Some("connected"))
        .map(ToOwned::to_owned);

        if let Some(line) = line {
            let cols: Vec<&str> = line.split(':').collect();
            let iface = cols.first().copied().unwrap_or_default();
            let kind = cols.get(1).copied().unwrap_or_default();
            let ip = first_ipv4_of_iface(iface).unwrap_or_else(|| "no ip".to_string());

            if kind == "wifi" {
                let signal = run_capture_allow_fail(
                    "nmcli",
                    &["-t", "-f", "IN-USE,SIGNAL", "dev", "wifi", "list"],
                )
                .unwrap_or_default()
                .lines()
                .find_map(|l| {
                    let c: Vec<&str> = l.split(':').collect();
                    if c.first() == Some(&"*") {
                        Some(c.get(1).copied().unwrap_or("?").to_string())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "?".to_string());
                let ssid = run_capture_allow_fail(
                    "nmcli",
                    &["-t", "-f", "IN-USE,SSID", "dev", "wifi", "list"],
                )
                .unwrap_or_default()
                .lines()
                .find_map(|l| {
                    let c: Vec<&str> = l.split(':').collect();
                    if c.first() == Some(&"*") {
                        Some(c.get(1).copied().unwrap_or("unknown").to_string())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "unknown".to_string());
                emit_json(
                    &format!("{signal}%"),
                    &format!("WiFi {ssid}\\n{ip}\\n{iface}"),
                    "wifi",
                );
                return;
            }

            if !iface.is_empty() {
                emit_json(iface, &format!("Ethernet\\n{ip}"), "ethernet");
                return;
            }
        }
    }

    let iface = run_capture_allow_fail("ip", &["-4", "route", "list", "default"])
        .unwrap_or_default()
        .lines()
        .next()
        .and_then(|line| {
            let cols: Vec<&str> = line.split_whitespace().collect();
            cols.iter()
                .position(|x| *x == "dev")
                .and_then(|i| cols.get(i + 1).copied())
        })
        .unwrap_or("")
        .to_string();

    if !iface.is_empty() {
        let ip = first_ipv4_of_iface(&iface).unwrap_or_else(|| "unknown".to_string());
        emit_json(&iface, &format!("IP {ip}"), "connected");
        return;
    }

    emit_json("", "disconnected", "disconnected");
}
