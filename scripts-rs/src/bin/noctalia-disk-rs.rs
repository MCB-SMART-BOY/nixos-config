use scripts_rs::{emit_json, parse_df_root};

fn main() {
    let Some((size, used, usep)) = parse_df_root() else {
        emit_json("", "Disk unavailable", "disk");
        return;
    };
    emit_json(&usep, &format!("Disk {used}/{size}"), "disk");
}
