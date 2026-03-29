use mcbctl::{command_exists, exit_from_status, run_status};

fn main() {
    if !command_exists("noctalia-shell") {
        eprintln!("lock-screen: noctalia-shell not found in PATH");
        std::process::exit(1);
    }
    match run_status("noctalia-shell", &["ipc", "call", "lockScreen", "lock"]) {
        Ok(status) => exit_from_status(status),
        Err(err) => {
            eprintln!("lock-screen: {err:#}");
            std::process::exit(1);
        }
    }
}
