use mcbctl::{emit_json, xdg_cache_home};
use std::fs;

fn main() {
    let cache_dir = xdg_cache_home();
    let state_file = cache_dir.join("noctalia-cpu.state");
    let _ = fs::create_dir_all(&cache_dir);

    let stat = fs::read_to_string("/proc/stat").unwrap_or_default();
    let mut parts = stat
        .lines()
        .next()
        .unwrap_or_default()
        .split_whitespace()
        .skip(1)
        .filter_map(|s| s.parse::<u64>().ok());
    let user = parts.next().unwrap_or(0);
    let nice = parts.next().unwrap_or(0);
    let system = parts.next().unwrap_or(0);
    let idle = parts.next().unwrap_or(0);
    let iowait = parts.next().unwrap_or(0);
    let irq = parts.next().unwrap_or(0);
    let softirq = parts.next().unwrap_or(0);
    let steal = parts.next().unwrap_or(0);

    let idle_all = idle + iowait;
    let total = user + nice + system + idle + iowait + irq + softirq + steal;

    let (prev_total, prev_idle) = fs::read_to_string(&state_file)
        .ok()
        .and_then(|s| {
            let mut it = s.split_whitespace().filter_map(|x| x.parse::<u64>().ok());
            Some((it.next()?, it.next()?))
        })
        .unwrap_or((total, idle_all));

    let diff_total = total.saturating_sub(prev_total);
    let diff_idle = idle_all.saturating_sub(prev_idle);
    let usage = ((100 * diff_total.saturating_sub(diff_idle)) / diff_total.max(1)) as u32;

    let _ = fs::write(&state_file, format!("{total} {idle_all}\n"));
    emit_json(&format!("{usage}%"), &format!("CPU {usage}%"), "cpu");
}
