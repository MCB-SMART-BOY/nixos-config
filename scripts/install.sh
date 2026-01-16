#!/usr/bin/env bash
# NixOS flake 部署脚本（单主机）

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
SCRIPT_NAME="$(basename "$0")"
# shellcheck source=./lib.sh
source "${SCRIPT_DIR}/lib.sh"
TARGET_NAME="${TARGET_NAME:-nixos}"
MODE="${MODE:-switch}"
SHOW_TRACE=false
FORCE_SYNC=false
HOST_FILE="${REPO_ROOT}/host.nix"
HARDWARE_FILE="${REPO_ROOT}/hardware-configuration.nix"
ETC_DIR="/etc/nixos"
ASSUME_YES=false
NO_REBUILD=false
NO_SYNC=false
NO_SYNC_ETC=false
SKIP_PREFLIGHT=false
TEMP_DNS=false
DNS_IFACE=""
DNS_SERVERS=()

msg() {
  local level="$1"
  local label
  shift
  case "${level}" in
    INFO) label="信息" ;;
    OK) label="完成" ;;
    WARN) label="警告" ;;
    ERROR) label="错误" ;;
    *) label="${level}" ;;
  esac
  printf '[%s] %s\n' "${label}" "$*"
}

log() { msg INFO "$*"; }
success() { msg OK "$*"; }
warn() { msg WARN "$*"; }
error() {
  msg ERROR "$*"
  exit 1
}

usage() {
  cat <<EOF_USAGE
用法: ${SCRIPT_NAME} [options]

选项:
  -h, --help         显示帮助
  -y, --yes          跳过确认
  --mode <action>    nixos-rebuild 动作: switch|test|build (默认: switch)
  --show-trace       启用 nixos-rebuild --show-trace
  --force-sync       覆盖已有 hardware-configuration.nix
  --no-sync          跳过硬件配置同步
  --no-sync-etc      不同步仓库到 /etc/nixos
  --no-rebuild       跳过 nixos-rebuild
  --skip-preflight   跳过部署前检查
  --temp-dns         部署期间临时指定 DNS（默认 223.5.5.5 223.6.6.6 1.1.1.1 8.8.8.8）
  --dns <ip>         指定临时 DNS（可多次传入）
  --dns-iface <dev>  指定 DNS 绑定网卡（resolvectl）
EOF_USAGE
}

parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      -h|--help)
        usage
        exit 0
        ;;
      -y|--yes)
        ASSUME_YES=true
        ;;
      --mode)
        shift
        [[ $# -gt 0 ]] || error "参数 --mode 需要一个值"
        MODE="$1"
        ;;
      --show-trace)
        SHOW_TRACE=true
        ;;
      --force-sync)
        FORCE_SYNC=true
        ;;
      --no-sync)
        NO_SYNC=true
        ;;
      --no-sync-etc)
        NO_SYNC_ETC=true
        ;;
      --no-rebuild)
        NO_REBUILD=true
        ;;
      --skip-preflight)
        SKIP_PREFLIGHT=true
        ;;
      --temp-dns)
        TEMP_DNS=true
        ;;
      --dns)
        shift
        [[ $# -gt 0 ]] || error "参数 --dns 需要一个值"
        DNS_SERVERS+=("$1")
        TEMP_DNS=true
        ;;
      --dns-iface)
        shift
        [[ $# -gt 0 ]] || error "参数 --dns-iface 需要一个值"
        DNS_IFACE="$1"
        TEMP_DNS=true
        ;;
      --)
        shift
        break
        ;;
      -* )
        error "未知参数: $1"
        ;;
      * )
        error "不支持的参数: $1"
        ;;
    esac
    shift
  done
}

validate_flags() {
  if [[ "${NO_SYNC}" == true && "${FORCE_SYNC}" == true ]]; then
    error "--force-sync 不能与 --no-sync 同时使用"
  fi
}

validate_mode() {
  if [[ "${NO_REBUILD}" == true ]]; then
    return
  fi

  case "${MODE}" in
    switch|test|build)
      ;;
    *)
      error "--mode '${MODE}' 无效（可选: switch|test|build）"
      ;;
  esac
}

check_env() {
  log "检查环境..."

  if [[ "$(whoami)" == "root" ]]; then
    error "请以普通用户运行（需要时会调用 sudo）。"
  fi

  if [[ ! -f "${REPO_ROOT}/flake.nix" ]]; then
    error "未找到 flake.nix: ${REPO_ROOT}"
  fi

  if [[ ! -f "${HOST_FILE}" ]]; then
    error "未找到 host.nix: ${REPO_ROOT}"
  fi

  local needs_sudo=false
  if [[ "${NO_SYNC}" != true || "${NO_REBUILD}" != true || "${NO_SYNC_ETC}" != true ]]; then
    needs_sudo=true
  fi

  if [[ "${needs_sudo}" == true ]] && ! command -v sudo >/dev/null 2>&1; then
    error "需要 sudo，但未找到该命令。"
  fi
  if [[ "${TEMP_DNS}" == true ]] && ! command -v sudo >/dev/null 2>&1; then
    error "临时 DNS 需要 sudo，但未找到该命令。"
  fi

  if [[ "${NO_REBUILD}" != true ]] && ! command -v nixos-rebuild >/dev/null 2>&1; then
    error "需要 nixos-rebuild，但未找到该命令。"
  fi

  if [[ "${NO_SYNC}" == true && "${NO_REBUILD}" != true && ! -f "${HARDWARE_FILE}" ]]; then
    warn "缺少 hardware-configuration.nix；跳过同步可能导致构建失败"
  fi
}

sync_hardware_config() {
  local target="${HARDWARE_FILE}"

  if [[ -f "${target}" && "${FORCE_SYNC}" != true ]]; then
    success "hardware-configuration.nix 已存在"
    return
  fi

  if [[ -f "${target}" && "${FORCE_SYNC}" == true ]]; then
    warn "将覆盖已有 hardware-configuration.nix"
  fi

  if [[ -f /etc/nixos/hardware-configuration.nix ]]; then
    log "同步 /etc/nixos/hardware-configuration.nix -> ${target}"
    sudo cp /etc/nixos/hardware-configuration.nix "${target}"
    success "hardware-configuration.nix 已同步"
    if command -v git >/dev/null 2>&1; then
      if git -C "${REPO_ROOT}" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
        git -C "${REPO_ROOT}" add "${target}"
        success "hardware-configuration.nix 已暂存"
      fi
    fi
  else
    error "未找到 /etc/nixos/hardware-configuration.nix；请在目标机运行 nixos-generate-config"
  fi
}

sync_repo_to_etc() {
  local target="${ETC_DIR}"
  log "同步仓库到 ${target}"
  sudo mkdir -p "${target}"
  if command -v rsync >/dev/null 2>&1; then
    sudo rsync -a --exclude '.git/' "${REPO_ROOT}/" "${target}/"
  else
    (cd "${REPO_ROOT}" && tar --exclude=.git -cf - .) | sudo tar -C "${target}" -xf -
  fi
  success "配置已同步到 ${target}"
}

rebuild_system() {
  log "重建系统 (${MODE})，目标: ${TARGET_NAME}"
  local nix_config="experimental-features = nix-command flakes"
  local rebuild_args=("${MODE}")
  if [[ -n "${NIX_CONFIG:-}" ]]; then
    nix_config="${NIX_CONFIG}"$'\n'"${nix_config}"
  fi
  if [[ "${SHOW_TRACE}" == true ]]; then
    rebuild_args+=("--show-trace")
  fi
  sudo -E env NIX_CONFIG="${nix_config}" nixos-rebuild "${rebuild_args[@]}" --flake "${REPO_ROOT}#${TARGET_NAME}"
  success "系统重建完成"
}

confirm() {
  local steps=()
  if [[ "${NO_SYNC}" != true ]]; then
    if [[ "${FORCE_SYNC}" == true ]]; then
      steps+=("同步硬件配置（覆盖）")
    else
      steps+=("同步硬件配置")
    fi
  fi
  if [[ "${NO_SYNC_ETC}" != true ]]; then
    steps+=("同步配置到 ${ETC_DIR}")
  fi
  if [[ "${NO_REBUILD}" != true ]]; then
    steps+=("重建系统 (${MODE})")
  fi
  if [[ ${#steps[@]} -eq 0 ]]; then
    error "无需执行任何操作（同时设置了 --no-sync 与 --no-rebuild）"
  fi

  if [[ "${ASSUME_YES}" == true ]]; then
    return
  fi

  local plan
  plan=$(IFS=", "; echo "${steps[*]}")
  read -r -p "将执行 ${plan}，目标 ${TARGET_NAME}。是否继续? [y/N] " -n 1
  echo
  if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    warn "已取消"
    exit 1
  fi
}

main() {
  printf '==> %s\n' "NixOS Flake 安装器"
  parse_args "$@"
  validate_flags
  validate_mode
  check_env

  if [[ "${TEMP_DNS}" == true ]]; then
    TEMP_DNS_IFACE="${DNS_IFACE}"
    log "启用临时 DNS..."
    temp_dns_enable "${DNS_SERVERS[@]}"
    trap temp_dns_disable EXIT
  fi

  if [[ "${SKIP_PREFLIGHT}" != true && -x "${REPO_ROOT}/scripts/preflight.sh" ]]; then
    log "运行部署前检查 (preflight)"
    "${REPO_ROOT}/scripts/preflight.sh"
  fi

  confirm

  if [[ "${NO_SYNC}" != true ]]; then
    sync_hardware_config
  else
    warn "跳过硬件配置同步"
  fi

  if [[ "${NO_SYNC_ETC}" != true ]]; then
    sync_repo_to_etc
  else
    warn "跳过 /etc/nixos 同步"
  fi

  if [[ "${NO_REBUILD}" != true ]]; then
    rebuild_system
  else
    warn "跳过 nixos-rebuild"
  fi
}

main "$@"
