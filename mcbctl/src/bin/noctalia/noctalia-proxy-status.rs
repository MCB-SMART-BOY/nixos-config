use mcbctl::{command_exists, emit_json, run_status};
use std::env;

fn is_active(args: &[&str]) -> bool {
    run_status("systemctl", args)
        .map(|s| s.success())
        .unwrap_or(false)
}

fn main() {
    if !command_exists("systemctl") {
        emit_json("", "Proxy: off", "off");
        return;
    }

    if is_active(&["--user", "is-active", "--quiet", "clash-verge-service"]) {
        emit_json("ON", "Proxy: clash-verge-service (user)", "on");
        return;
    }

    let user = env::var("USER").unwrap_or_default();
    let templated = format!("clash-verge-service@{user}");
    if is_active(&["is-active", "--quiet", &templated]) {
        emit_json("ON", &format!("Proxy: {templated}"), "on");
        return;
    }

    if is_active(&["is-active", "--quiet", "clash-verge-service"]) {
        emit_json("ON", "Proxy: clash-verge-service", "on");
        return;
    }

    if is_active(&["is-active", "--quiet", "mihomo"]) {
        emit_json("ON", "Proxy: mihomo", "on");
        return;
    }

    emit_json("", "Proxy: off", "off");
}
