use mcbctl::emit_json;
use std::fs;

fn main() {
    let mut max_temp = 0i64;
    let mut found = false;

    if let Ok(entries) = fs::read_dir("/sys/class/thermal") {
        for ent in entries.flatten() {
            let path = ent.path().join("temp");
            let Ok(raw) = fs::read_to_string(path) else {
                continue;
            };
            let Ok(value) = raw.trim().parse::<i64>() else {
                continue;
            };
            if value > max_temp {
                max_temp = value;
                found = true;
            }
        }
    }

    if !found {
        emit_json("", "Temperature unavailable", "temp");
        return;
    }

    let temp_c = max_temp / 1000;
    emit_json(&format!("{temp_c}C"), &format!("Temp {temp_c}C"), "temp");
}
