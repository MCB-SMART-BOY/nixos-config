use scripts_rs::emit_json;
use std::fs;

fn main() {
    let meminfo = fs::read_to_string("/proc/meminfo").unwrap_or_default();
    let mut total_kb = None::<u64>;
    let mut avail_kb = None::<u64>;

    for line in meminfo.lines() {
        if let Some(v) = line.strip_prefix("MemTotal:") {
            total_kb = v
                .split_whitespace()
                .next()
                .and_then(|x| x.parse::<u64>().ok());
        } else if let Some(v) = line.strip_prefix("MemAvailable:") {
            avail_kb = v
                .split_whitespace()
                .next()
                .and_then(|x| x.parse::<u64>().ok());
        }
    }

    let (total_kb, avail_kb) = match (total_kb, avail_kb) {
        (Some(t), Some(a)) if t > 0 => (t, a),
        _ => {
            emit_json("", "Memory unavailable", "memory");
            return;
        }
    };

    let used_kb = total_kb.saturating_sub(avail_kb);
    let percent = (used_kb * 100) / total_kb;

    let used_gib = used_kb as f64 / 1_048_576.0;
    let total_gib = total_kb as f64 / 1_048_576.0;
    let tooltip = format!("RAM {:.1}/{:.1}GiB", used_gib, total_gib);
    emit_json(&format!("{percent}%"), &tooltip, "memory");
}
