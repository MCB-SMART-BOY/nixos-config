# 检测当前 GPU 模式（igpu/hybrid/dgpu/base）。
# 优先级：MCB_GPU_MODE 环境变量 > /run/current-system 路径 > /proc/cmdline > noctalia-gpu-mode 命令。
set -euo pipefail

normalize_mode() {
  case "$1" in
    gpu-dgpu|dgpu) printf '%s\n' "dgpu" ;;
    gpu-hybrid|hybrid) printf '%s\n' "hybrid" ;;
    gpu-igpu|igpu) printf '%s\n' "igpu" ;;
    *) printf '%s\n' "base" ;;
  esac
}

mode_from_path() {
  local path="$1"
  local mode=""

  if [[ "$path" == */specialisation/gpu-* ]]; then
    mode="${path##*/specialisation/}"
    mode="${mode%%/*}"
  elif [[ "$path" == *specialisation-gpu-* ]]; then
    mode="${path##*specialisation-}"
    mode="${mode%%/*}"
  fi

  if [[ -n "$mode" ]]; then
    normalize_mode "$mode"
    return 0
  fi
  return 1
}

if [[ -n "${MCB_GPU_MODE-}" ]]; then
  normalize_mode "${MCB_GPU_MODE}"
  exit 0
fi

path="$(readlink -f /run/current-system 2>/dev/null || true)"
mode="$(mode_from_path "$path" 2>/dev/null || true)"
if [[ -n "$mode" ]]; then
  printf '%s\n' "$mode"
  exit 0
fi

path="$(readlink -f /run/booted-system 2>/dev/null || true)"
mode="$(mode_from_path "$path" 2>/dev/null || true)"
if [[ -n "$mode" ]]; then
  printf '%s\n' "$mode"
  exit 0
fi

if [[ -r /proc/cmdline ]]; then
  for token in $(</proc/cmdline); do
    case "$token" in
      init=*|systemConfig=*)
        cmd_path="${token#*=}"
        ;;
      *)
        cmd_path=""
        ;;
    esac

    if [[ -n "$cmd_path" ]]; then
      mode="$(mode_from_path "$cmd_path" 2>/dev/null || true)"
      if [[ -n "$mode" ]]; then
        printf '%s\n' "$mode"
        exit 0
      fi
    fi
  done
fi

if command -v noctalia-gpu-mode >/dev/null 2>&1; then
  mode="$(noctalia-gpu-mode 2>/dev/null | sed -n 's/.*specialisation: \([^"]*\).*/\1/p' | head -n 1 || true)"
  if [[ -n "$mode" ]]; then
    normalize_mode "$mode"
    exit 0
  fi
fi

printf '%s\n' "base"
