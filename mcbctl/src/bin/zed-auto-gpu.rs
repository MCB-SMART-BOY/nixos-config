use mcbctl::current_gpu_mode_label;
use std::os::unix::process::CommandExt;
use std::process::Command;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mode = current_gpu_mode_label();

    let mut cmd = Command::new("zeditor");
    cmd.args(&args);

    if mode == "dgpu" {
        cmd.env(
            "WGPU_BACKEND",
            std::env::var("WGPU_BACKEND").unwrap_or_else(|_| "gl".to_string()),
        );
        cmd.env("__GLX_VENDOR_LIBRARY_NAME", "nvidia");
        cmd.env("__VK_LAYER_NV_optimus", "NVIDIA_only");
    } else {
        cmd.env_remove("__NV_PRIME_RENDER_OFFLOAD");
        cmd.env_remove("__NV_PRIME_RENDER_OFFLOAD_PROVIDER");
        cmd.env_remove("__GLX_VENDOR_LIBRARY_NAME");
        cmd.env_remove("__VK_LAYER_NV_optimus");
        cmd.env_remove("DRI_PRIME");
        cmd.env_remove("WGPU_BACKEND");
    }

    let err = cmd.exec();
    eprintln!("zed-auto-gpu: {err}");
    std::process::exit(1);
}
