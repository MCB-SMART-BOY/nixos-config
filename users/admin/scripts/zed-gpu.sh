# Zed 自适应 GPU 启动器。
# dgpu 模式使用 OpenGL 后端避免 niri+NVIDIA Wayland 残帧问题。
set -euo pipefail

mode="$(noctalia-gpu-current 2>/dev/null || printf '%s' base)"
case "$mode" in
  dgpu)
    export WGPU_BACKEND="${WGPU_BACKEND:-gl}"
    export __GLX_VENDOR_LIBRARY_NAME="nvidia"
    export __VK_LAYER_NV_optimus="NVIDIA_only"
    ;;
  *)
    unset __NV_PRIME_RENDER_OFFLOAD
    unset __NV_PRIME_RENDER_OFFLOAD_PROVIDER
    unset __GLX_VENDOR_LIBRARY_NAME
    unset __VK_LAYER_NV_optimus
    unset DRI_PRIME
    unset WGPU_BACKEND
    ;;
esac

exec zeditor "$@"
