#!/usr/bin/env bash
# NixOS flake 云端同步脚本（单主机）

set -euo pipefail

SCRIPT_NAME="$(basename "$0")"
REPO_URL="${REPO_URL:-https://github.com/MCB-SMART-BOY/nixos-config.git}"
BRANCH="${BRANCH:-master}"
TARGET_NAME="${TARGET_NAME:-nixos}"
MODE="${MODE:-switch}"
ETC_DIR="/etc/nixos"
ASSUME_YES=false
SHOW_TRACE=false
NO_REBUILD=false
FORCE_HARDWARE=false
SKIP_PREFLIGHT=false

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
  -h, --help           显示帮助
  -y, --yes            跳过确认
  --repo <url>         仓库地址（默认: ${REPO_URL}）
  --branch <name>      分支名（默认: ${BRANCH}）
  --target <name>      flake 目标名（默认: ${TARGET_NAME}）
  --mode <action>      nixos-rebuild 动作: switch|test|build (默认: switch)
  --show-trace         启用 nixos-rebuild --show-trace
  --force-hardware     允许覆盖 /etc/nixos/hardware-configuration.nix
  --no-rebuild         跳过 nixos-rebuild
  --skip-preflight     跳过部署前检查
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
      --repo)
        shift
        [[ $# -gt 0 ]] || error "参数 --repo 需要一个值"
        REPO_URL="$1"
        ;;
      --branch)
        shift
        [[ $# -gt 0 ]] || error "参数 --branch 需要一个值"
        BRANCH="$1"
        ;;
      --target)
        shift
        [[ $# -gt 0 ]] || error "参数 --target 需要一个值"
        TARGET_NAME="$1"
        ;;
      --mode)
        shift
        [[ $# -gt 0 ]] || error "参数 --mode 需要一个值"
        MODE="$1"
        ;;
      --show-trace)
        SHOW_TRACE=true
        ;;
      --force-hardware)
        FORCE_HARDWARE=true
        ;;
      --no-rebuild)
        NO_REBUILD=true
        ;;
      --skip-preflight)
        SKIP_PREFLIGHT=true
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

  if ! command -v sudo >/dev/null 2>&1; then
    error "需要 sudo，但未找到该命令。"
  fi

  if ! command -v git >/dev/null 2>&1; then
    error "需要 git，但未找到该命令。"
  fi

  if [[ "${NO_REBUILD}" != true ]] && ! command -v nixos-rebuild >/dev/null 2>&1; then
    error "需要 nixos-rebuild，但未找到该命令。"
  fi

  if [[ "${FORCE_HARDWARE}" != true && ! -f "${ETC_DIR}/hardware-configuration.nix" ]]; then
    error "未找到 ${ETC_DIR}/hardware-configuration.nix；请先运行 nixos-generate-config 或使用 --force-hardware"
  fi
}

clone_repo() {
  local tmp_dir="$1"
  log "拉取仓库: ${REPO_URL} (${BRANCH})"
  git clone --branch "${BRANCH}" "${REPO_URL}" "${tmp_dir}"
  success "仓库拉取完成"
}

sync_repo_to_etc() {
  local repo_dir="$1"
  local target="${ETC_DIR}"
  log "同步仓库到 ${target}"
  sudo mkdir -p "${target}"

  local excludes=(--exclude '.git/')
  if [[ "${FORCE_HARDWARE}" != true ]]; then
    excludes+=(--exclude 'hardware-configuration.nix')
  fi

  if command -v rsync >/dev/null 2>&1; then
    sudo rsync -a "${excludes[@]}" "${repo_dir}/" "${target}/"
  else
    local tar_excludes=(--exclude=.git)
    if [[ "${FORCE_HARDWARE}" != true ]]; then
      tar_excludes+=(--exclude=hardware-configuration.nix)
    fi
    (cd "${repo_dir}" && tar "${tar_excludes[@]}" -cf - .) | sudo tar -C "${target}" -xf -
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
  sudo -E env NIX_CONFIG="${nix_config}" nixos-rebuild "${rebuild_args[@]}" --flake "${ETC_DIR}#${TARGET_NAME}"
  success "系统重建完成"
}

detect_user() {
  local host_file="${ETC_DIR}/host.nix"
  if [[ ! -f "${host_file}" ]]; then
    return 1
  fi

  awk -F'"' '/user[[:space:]]*=[[:space:]]*"/ {print $2; exit}' "${host_file}"
}

activate_home_manager() {
  local user
  user="$(detect_user || true)"
  if [[ -z "${user}" ]]; then
    warn "未检测到用户配置，跳过 Home Manager 刷新"
    return
  fi

  local unit="home-manager-${user}.service"
  if systemctl list-unit-files --no-legend "${unit}" 2>/dev/null | grep -q "${unit}"; then
    log "刷新 Home Manager 配置 (${user})"
    if sudo systemctl start "${unit}" >/dev/null 2>&1; then
      success "Home Manager 配置已刷新"
    else
      warn "Home Manager 刷新失败，请检查 ${unit}"
    fi
  else
    warn "未找到 ${unit}，请确认 Home Manager 是否启用"
  fi
}

confirm() {
  local steps=(
    "拉取仓库 ${REPO_URL} (${BRANCH})"
    "运行部署前自检 (preflight)"
    "同步配置到 ${ETC_DIR}"
  )
  if [[ "${SKIP_PREFLIGHT}" == true ]]; then
    steps=(
      "拉取仓库 ${REPO_URL} (${BRANCH})"
      "同步配置到 ${ETC_DIR}"
    )
  fi
  if [[ "${NO_REBUILD}" != true ]]; then
    steps+=("重建系统 (${MODE})")
    steps+=("刷新 Home Manager 配置")
  fi

  if [[ "${ASSUME_YES}" == true ]]; then
    return
  fi

  local plan
  plan=$(IFS=", "; echo "${steps[*]}")
  read -r -p "将执行 ${plan}。是否继续? [y/N] " -n 1
  echo
  if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    warn "已取消"
    exit 1
  fi
}

main() {
  printf '==> %s\n' "NixOS Flake 云端安装器"
  parse_args "$@"
  validate_mode
  check_env
  confirm

  local tmp_dir
  tmp_dir="$(mktemp -d)"
  trap 'rm -rf "${tmp_dir}"' EXIT

  clone_repo "${tmp_dir}"
  if [[ "${SKIP_PREFLIGHT}" != true && -x "${tmp_dir}/scripts/preflight.sh" ]]; then
    log "运行部署前检查 (preflight)"
    (cd "${tmp_dir}" && "${tmp_dir}/scripts/preflight.sh")
  fi
  sync_repo_to_etc "${tmp_dir}"

  if [[ "${NO_REBUILD}" != true ]]; then
    rebuild_system
    activate_home_manager
  else
    warn "跳过 nixos-rebuild"
    warn "未执行重建，~/.config 不会更新为 Home Manager 链接"
  fi
}

main "$@"
