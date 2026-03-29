use anyhow::{Result, anyhow, bail};
use scripts_rs::run_capture;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

#[derive(Default)]
struct Args {
    action: String,
    user: String,
    iface: String,
    table_id: String,
    priority: String,
    dns_port: String,
    redirect_dns: bool,
}

fn parse_args() -> Result<Args> {
    let mut out = Args::default();
    let mut args = std::env::args().skip(1);
    out.action = args.next().ok_or_else(|| anyhow!("missing action"))?;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--redirect-dns" => out.redirect_dns = true,
            "--user" => out.user = args.next().ok_or_else(|| anyhow!("missing user"))?,
            "--iface" => out.iface = args.next().ok_or_else(|| anyhow!("missing iface"))?,
            "--table-id" => {
                out.table_id = args.next().ok_or_else(|| anyhow!("missing table-id"))?
            }
            "--priority" => {
                out.priority = args.next().ok_or_else(|| anyhow!("missing priority"))?
            }
            "--dns-port" => {
                out.dns_port = args.next().ok_or_else(|| anyhow!("missing dns-port"))?
            }
            _ => bail!("unknown argument: {arg}"),
        }
    }
    if out.user.is_empty()
        || out.iface.is_empty()
        || out.table_id.is_empty()
        || out.priority.is_empty()
    {
        bail!("missing required arguments");
    }
    Ok(out)
}

fn uid_of(user: &str) -> Result<String> {
    Ok(run_capture("id", &["-u", user])?.trim().to_string())
}

fn iptables_rule_exists(uid: &str, proto: &str, dns_port: &str) -> bool {
    Command::new("iptables")
        .args([
            "-t",
            "nat",
            "-C",
            "OUTPUT",
            "-p",
            proto,
            "--dport",
            "53",
            "-m",
            "owner",
            "--uid-owner",
            uid,
            "-j",
            "REDIRECT",
            "--to-ports",
            dns_port,
        ])
        .status()
        .ok()
        .is_some_and(|s| s.success())
}

fn ss_output_has_port(output: &str, port: &str) -> bool {
    let suffix = format!(":{port}");
    output
        .lines()
        .flat_map(|line| line.split_whitespace())
        .any(|token| token.ends_with(&suffix))
}

fn delete_dns_rules(uid: &str, dns_port: &str) {
    for proto in ["udp", "tcp"] {
        let _ = Command::new("iptables")
            .args([
                "-t",
                "nat",
                "-D",
                "OUTPUT",
                "-p",
                proto,
                "--dport",
                "53",
                "-m",
                "owner",
                "--uid-owner",
                uid,
                "-j",
                "REDIRECT",
                "--to-ports",
                dns_port,
            ])
            .status();
    }
}

fn start(args: &Args) -> Result<()> {
    let uid = uid_of(&args.user)?;

    let mut ready = false;
    for _ in 0..150 {
        let exists = Command::new("ip")
            .args(["link", "show", "dev", &args.iface])
            .status()
            .ok()
            .is_some_and(|s| s.success());
        if exists {
            let operstate =
                std::fs::read_to_string(format!("/sys/class/net/{}/operstate", args.iface))
                    .unwrap_or_default();
            let operstate = operstate.trim();
            if !operstate.is_empty() && operstate != "down" {
                ready = true;
                break;
            }
        }
        sleep(Duration::from_millis(200));
    }
    if !ready {
        bail!("interface {} not ready", args.iface);
    }

    let rules = run_capture("ip", &["rule", "show"]).unwrap_or_default();
    let needle = format!("uidrange {uid}-{uid}");
    let table_needle = format!("lookup {}", args.table_id);
    let has_rule = rules
        .lines()
        .any(|line| line.contains(&needle) && line.contains(&table_needle));
    if !has_rule {
        let status = Command::new("ip")
            .args([
                "rule",
                "add",
                "priority",
                &args.priority,
                "uidrange",
                &format!("{uid}-{uid}"),
                "lookup",
                &args.table_id,
            ])
            .status()?;
        if !status.success() {
            bail!("failed to add ip rule");
        }
    }

    let status = Command::new("ip")
        .args([
            "route",
            "replace",
            "default",
            "dev",
            &args.iface,
            "table",
            &args.table_id,
        ])
        .status()?;
    if !status.success() {
        bail!("failed to replace default route");
    }

    if args.redirect_dns {
        if args.dns_port == "0" || args.dns_port.is_empty() {
            bail!("dns redirect enabled but no dns port configured");
        }
        delete_dns_rules(&uid, &args.dns_port);

        let mut dns_ready = false;
        for _ in 0..60 {
            let udp = run_capture("ss", &["-lun"]).unwrap_or_default();
            let tcp = run_capture("ss", &["-ltn"]).unwrap_or_default();
            if ss_output_has_port(&udp, &args.dns_port) || ss_output_has_port(&tcp, &args.dns_port)
            {
                dns_ready = true;
                break;
            }
            sleep(Duration::from_millis(500));
        }
        if !dns_ready {
            bail!("dns port {} not listening", args.dns_port);
        }

        for proto in ["udp", "tcp"] {
            if !iptables_rule_exists(&uid, proto, &args.dns_port) {
                let status = Command::new("iptables")
                    .args([
                        "-t",
                        "nat",
                        "-A",
                        "OUTPUT",
                        "-p",
                        proto,
                        "--dport",
                        "53",
                        "-m",
                        "owner",
                        "--uid-owner",
                        &uid,
                        "-j",
                        "REDIRECT",
                        "--to-ports",
                        &args.dns_port,
                    ])
                    .status()?;
                if !status.success() {
                    bail!("failed to add iptables rule for {proto}");
                }
            }
        }
    }

    Ok(())
}

fn stop(args: &Args) -> Result<()> {
    let uid = uid_of(&args.user).unwrap_or_default();
    let _ = Command::new("ip")
        .args([
            "route",
            "del",
            "default",
            "dev",
            &args.iface,
            "table",
            &args.table_id,
        ])
        .status();
    if !uid.is_empty() {
        let _ = Command::new("ip")
            .args([
                "rule",
                "del",
                "uidrange",
                &format!("{uid}-{uid}"),
                "lookup",
                &args.table_id,
            ])
            .status();
        if args.redirect_dns && !args.dns_port.is_empty() {
            delete_dns_rules(&uid, &args.dns_port);
        }
    }
    Ok(())
}

fn main() {
    let result = (|| -> Result<()> {
        let args = parse_args()?;
        match args.action.as_str() {
            "start" => start(&args),
            "stop" => stop(&args),
            other => bail!("unsupported action: {other}"),
        }
    })();

    if let Err(err) = result {
        eprintln!("mcb-tun-route-rs: {err:#}");
        std::process::exit(1);
    }
}
