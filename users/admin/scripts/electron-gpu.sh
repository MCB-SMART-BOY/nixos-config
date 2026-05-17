# Electron 自适应 GPU 启动器。
# dgpu 模式强制使用稳定的 X11 渲染路径。
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "Usage: electron-auto-gpu <command> [args...]" >&2
  exit 2
fi

app="$1"
shift

if ! command -v "$app" >/dev/null 2>&1; then
  echo "electron-auto-gpu: command not found: $app" >&2
  exit 127
fi

mode="$(noctalia-gpu-current 2>/dev/null || printf '%s' base)"
if [[ "$mode" == "dgpu" ]]; then
  export NIXOS_OZONE_WL="0"
  export ELECTRON_OZONE_PLATFORM_HINT="x11"
  export OZONE_PLATFORM="x11"
fi

exec "$app" "$@"
