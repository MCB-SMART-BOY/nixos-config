use super::current::current_mode_inner;
use super::topology::{HostGpuTopology, effective_mode, host_topology};

pub(crate) fn emit_status() {
    let raw_mode = current_mode_inner();
    let effective = effective_mode(&raw_mode);
    let topology = host_topology();
    if topology != HostGpuTopology::MultiGpu {
        let json = serde_json::json!({
            "text": "",
            "alt": effective,
            "class": ["gpu-mode", "hidden", topology.id()],
            "tooltip": format!(
                "Host topology: {}\n当前主机不是多显卡机器，GPU 模式切换入口已隐藏。",
                topology.summary()
            )
        });
        println!("{json}");
        return;
    }

    let specialisation = if raw_mode == "base" {
        format!("base (default: {effective})")
    } else {
        raw_mode.clone()
    };
    let class = if raw_mode == "base" {
        vec![
            "gpu-mode".to_string(),
            "gpu-base".to_string(),
            format!("gpu-{effective}"),
        ]
    } else {
        vec!["gpu-mode".to_string(), raw_mode.clone()]
    };
    let json = serde_json::json!({
        "text": format!("GPU:{effective}"),
        "alt": effective,
        "class": class,
        "tooltip": format!(
            "Host topology: {}\nGPU specialisation: {specialisation}\nEffective mode: {effective}\n切换后 Waybar/Noctalia 会自动刷新，但已打开的图形应用通常需要手动重启。",
            topology.summary()
        )
    });
    println!("{json}");
}
