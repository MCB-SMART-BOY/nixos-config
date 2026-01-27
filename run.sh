#!/usr/bin/env bash
# One-step NixOS deploy from GitHub. No arguments.

set -euo pipefail

REPO_URLS=(
  "https://gitee.com/MCB-SMART-BOY/nixos-config.git"
  "https://github.com/MCB-SMART-BOY/nixos-config.git"
)
BRANCH="master"
TARGET_NAME=""
TARGET_USERS=()
OVERWRITE_MODE="overwrite"
PER_USER_TUN_ENABLED=false
declare -A USER_TUN
declare -A USER_DNS
MODE="switch"
ETC_DIR="/etc/nixos"
DNS_ENABLED=false

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
用法: run.sh [选项]

选项:
  -H, --host <name>        选择主机目录名（hosts/<name>）
  -u, --user <name>        指定单个用户名（可重复）
  -U, --users "<a b>"      指定多个用户名（空格或逗号分隔）
  --ask                   遇到 /etc/nixos 已存在时询问备份或覆盖
  --backup                当 /etc/nixos 已存在时先备份再覆盖
  --overwrite             直接覆盖 /etc/nixos（不备份）
  -h, --help              显示帮助
EOF_USAGE
}

parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      -H|--host)
        [[ $# -ge 2 ]] || { usage; error "缺少参数：--host"; }
        TARGET_NAME="$2"
        shift 2
        ;;
      -u|--user)
        [[ $# -ge 2 ]] || { usage; error "缺少参数：--user"; }
        TARGET_USERS+=("$2")
        shift 2
        ;;
      -U|--users)
        [[ $# -ge 2 ]] || { usage; error "缺少参数：--users"; }
        raw_users="$2"
        raw_users="${raw_users//,/ }"
        read -r -a more_users <<< "${raw_users}"
        TARGET_USERS+=("${more_users[@]}")
        shift 2
        ;;
      --backup)
        OVERWRITE_MODE="backup"
        shift
        ;;
      --ask)
        OVERWRITE_MODE="ask"
        shift
        ;;
      --overwrite)
        OVERWRITE_MODE="overwrite"
        shift
        ;;
      -h|--help)
        usage
        exit 0
        ;;
      *)
        usage
        error "不支持的参数：$1"
        ;;
    esac
  done
}

check_env() {
  log "检查环境..."

  if [[ "$(whoami)" == "root" ]]; then
    error "请以普通用户运行（需要时会调用 sudo）。"
  fi

  if ! command -v sudo >/dev/null 2>&1; then
    error "未找到 sudo。"
  fi

  if ! command -v git >/dev/null 2>&1; then
    error "未找到 git。"
  fi

  if ! command -v nixos-rebuild >/dev/null 2>&1; then
    error "未找到 nixos-rebuild。"
  fi

  if [[ ! -f "${ETC_DIR}/hardware-configuration.nix" ]]; then
    error "缺少 ${ETC_DIR}/hardware-configuration.nix；请先运行 nixos-generate-config。"
  fi
}

is_tty() {
  [[ -t 0 && -t 1 ]]
}

list_hosts() {
  local repo_dir="$1"
  local host_dir="${repo_dir}/hosts"
  local hosts=()

  if [[ -d "${host_dir}" ]]; then
    for entry in "${host_dir}"/*; do
      [[ -d "${entry}" ]] || continue
      local name
      name="$(basename "${entry}")"
      [[ "${name}" == "profiles" ]] && continue
      hosts+=("${name}")
    done
  fi

  printf '%s\n' "${hosts[@]}"
}

select_host() {
  local repo_dir="$1"
  if [[ -z "${TARGET_NAME}" ]]; then
    TARGET_NAME="nixos"
  fi
}

validate_host() {
  local repo_dir="$1"
  if [[ -z "${TARGET_NAME}" ]]; then
    error "未指定主机名称。"
  fi
  if [[ ! -d "${repo_dir}/hosts/${TARGET_NAME}" ]]; then
    error "主机不存在：hosts/${TARGET_NAME}"
  fi
}

detect_per_user_tun() {
  local host_file="$1/hosts/${TARGET_NAME}/default.nix"
  local in_block=0
  local line

  if [[ ! -f "${host_file}" ]]; then
    return 1
  fi

  while IFS= read -r line; do
    if [[ "${line}" == *perUserTun* ]]; then
      in_block=1
    fi
    if [[ ${in_block} -eq 1 && "${line}" == *"enable"* && "${line}" == *"true"* ]]; then
      return 0
    fi
    if [[ ${in_block} -eq 1 && "${line}" == *"}"* ]]; then
      in_block=0
    fi
  done < "${host_file}"

  return 1
}

configure_per_user_tun() {
  if [[ "${PER_USER_TUN_ENABLED}" != "true" ]]; then
    return 0
  fi

  local idx=0
  local user
  for user in "${TARGET_USERS[@]}"; do
    local default_iface="tun${idx}"
    local default_dns=$((1053 + idx))
    local iface="${default_iface}"
    local dns_port="${default_dns}"

    if is_tty; then
      read -r -p "用户 ${user} 的 TUN 接口名（默认 ${default_iface}）： " iface_input
      if [[ -n "${iface_input}" ]]; then
        iface="${iface_input}"
      fi
      read -r -p "用户 ${user} 的 DNS 端口（默认 ${default_dns}）： " dns_input
      if [[ -n "${dns_input}" ]]; then
        dns_port="${dns_input}"
      fi
    fi

    if [[ ! "${iface}" =~ ^[A-Za-z0-9_.-]+$ ]]; then
      error "TUN 接口名不合法：${iface}"
    fi
    if [[ ! "${dns_port}" =~ ^[0-9]+$ ]] || ((dns_port < 1 || dns_port > 65535)); then
      error "DNS 端口不合法：${dns_port}"
    fi

    USER_TUN["${user}"]="${iface}"
    USER_DNS["${user}"]="${dns_port}"
    idx=$((idx + 1))
  done
}

prompt_users() {
  if [[ ${#TARGET_USERS[@]} -eq 0 ]]; then
    TARGET_USERS=("mcbnixos")
  fi
}

dedupe_users() {
  local user
  local -A seen=()
  local unique=()
  for user in "${TARGET_USERS[@]}"; do
    if [[ -z "${seen[${user}]+x}" ]]; then
      unique+=("${user}")
      seen["${user}"]=1
    fi
  done
  TARGET_USERS=("${unique[@]}")
}

validate_users() {
  local user
  for user in "${TARGET_USERS[@]}"; do
    if [[ ! "${user}" =~ ^[a-z_][a-z0-9_-]*$ ]]; then
      error "用户名不合法：${user}"
    fi
  done
}

write_local_override() {
  local repo_dir="$1"
  local host_dir="${repo_dir}/hosts/${TARGET_NAME}"
  local file="${host_dir}/local.nix"

  if [[ ${#TARGET_USERS[@]} -eq 0 ]]; then
    return 0
  fi

  if [[ ! -d "${host_dir}" ]]; then
    error "主机目录不存在：${host_dir}"
  fi

  local primary="${TARGET_USERS[0]}"
  local list=""
  local user
  for user in "${TARGET_USERS[@]}"; do
    list="${list} \"${user}\""
  done

  {
    echo "{ ... }:"
    echo ""
    echo "{"
    echo "  mcb.user = \"${primary}\";"
    echo "  mcb.users = [${list} ];"

    if [[ "${PER_USER_TUN_ENABLED}" == "true" && ${#USER_TUN[@]} -gt 0 ]]; then
      echo "  mcb.perUserTun.interfaces = {"
      for user in "${TARGET_USERS[@]}"; do
        echo "    ${user} = \"${USER_TUN[${user}]}\";"
      done
      echo "  };"
      echo "  mcb.perUserTun.dnsPorts = {"
      for user in "${TARGET_USERS[@]}"; do
        echo "    ${user} = ${USER_DNS[${user}]};"
      done
      echo "  };"
    fi

    echo "}"
  } > "${file}"
}

backup_etc() {
  local timestamp
  timestamp="$(date +%Y%m%d-%H%M%S)"
  local backup_dir="${ETC_DIR}.backup-${timestamp}"
  log "备份 ${ETC_DIR} -> ${backup_dir}"
  sudo mkdir -p "${backup_dir}"
  if command -v rsync >/dev/null 2>&1; then
    sudo rsync -a "${ETC_DIR}/" "${backup_dir}/"
  else
    sudo cp -a "${ETC_DIR}/." "${backup_dir}/"
  fi
  success "备份完成"
}

prepare_etc_dir() {
  if [[ -d "${ETC_DIR}" && -n "$(ls -A "${ETC_DIR}" 2>/dev/null)" ]]; then
    case "${OVERWRITE_MODE}" in
      backup)
        backup_etc
        ;;
      overwrite)
        ;;
      ask)
        if is_tty; then
          while true; do
            read -r -p "检测到 ${ETC_DIR} 已存在，选择 [b]备份并覆盖/[o]直接覆盖/[q]退出（默认 b）： " answer
            case "${answer}" in
              b|B|"")
                backup_etc
                break
                ;;
              o|O)
                break
                ;;
              q|Q)
                error "已退出"
                ;;
              *)
                echo "无效选择，请重试。"
                ;;
            esac
          done
        else
          backup_etc
        fi
        ;;
      *)
        ;;
    esac
  fi
}

detect_default_iface() {
  if command -v ip >/dev/null 2>&1; then
    ip route show default 2>/dev/null | awk 'NR==1 {print $5; exit}'
  fi
}

TEMP_DNS_BACKEND=""
TEMP_DNS_BACKUP=""
TEMP_DNS_IFACE=""

temp_dns_enable() {
  local servers=("223.5.5.5" "223.6.6.6")
  local iface=""

  if command -v resolvectl >/dev/null 2>&1 && command -v systemctl >/dev/null 2>&1; then
    if systemctl is-active --quiet systemd-resolved; then
      iface="$(detect_default_iface)"

      if [[ -n "${iface}" ]]; then
        log "临时 DNS（resolvectl ${iface}）：${servers[*]}"
        sudo resolvectl dns "${iface}" "${servers[@]}"
        sudo resolvectl domain "${iface}" "~."
        TEMP_DNS_BACKEND="resolvectl"
        TEMP_DNS_IFACE="${iface}"
        DNS_ENABLED=true
        return 0
      fi
    fi
  fi

  if [[ -f /etc/resolv.conf ]]; then
    TEMP_DNS_BACKUP="$(mktemp)"
    sudo cp -a /etc/resolv.conf "${TEMP_DNS_BACKUP}"
    sudo rm -f /etc/resolv.conf
    printf 'nameserver %s\n' "${servers[@]}" | sudo tee /etc/resolv.conf >/dev/null
    log "临时 DNS（/etc/resolv.conf）：${servers[*]}"
    TEMP_DNS_BACKEND="resolv.conf"
    DNS_ENABLED=true
    return 0
  fi

  error "无法设置临时 DNS（无 resolvectl 且缺少 /etc/resolv.conf）。"
}

temp_dns_disable() {
  if [[ "${TEMP_DNS_BACKEND}" == "resolvectl" ]]; then
    if [[ -n "${TEMP_DNS_IFACE}" ]]; then
      log "恢复 DNS（resolvectl ${TEMP_DNS_IFACE}）"
      sudo resolvectl revert "${TEMP_DNS_IFACE}" || true
      sudo resolvectl flush-caches >/dev/null 2>&1 || true
    fi
  elif [[ "${TEMP_DNS_BACKEND}" == "resolv.conf" ]]; then
    if [[ -n "${TEMP_DNS_BACKUP}" && -f "${TEMP_DNS_BACKUP}" ]]; then
      log "恢复 /etc/resolv.conf"
      sudo cp -a "${TEMP_DNS_BACKUP}" /etc/resolv.conf || true
      rm -f "${TEMP_DNS_BACKUP}"
    fi
  fi
}

clone_repo() {
  local tmp_dir="$1"
  local url="$2"
  log "拉取仓库：${url}（${BRANCH}）"
  if git clone --depth 1 --branch "${BRANCH}" "${url}" "${tmp_dir}"; then
    success "仓库拉取完成"
    return 0
  fi
  warn "仓库拉取失败：${url}"
  return 1
}

clone_repo_any() {
  local tmp_dir="$1"
  local url
  for url in "${REPO_URLS[@]}"; do
    rm -rf "${tmp_dir}"
    mkdir -p "${tmp_dir}"
    if clone_repo "${tmp_dir}" "${url}"; then
      return 0
    fi
  done
  return 1
}

sync_repo_to_etc() {
  local repo_dir="$1"
  log "同步到 ${ETC_DIR}"
  sudo mkdir -p "${ETC_DIR}"

  if command -v rsync >/dev/null 2>&1; then
    sudo rsync -a \
      --exclude '.git/' \
      --exclude 'hardware-configuration.nix' \
      --exclude 'hosts/*/hardware-configuration.nix' \
      "${repo_dir}/" "${ETC_DIR}/"
  else
    (cd "${repo_dir}" && tar --exclude=.git --exclude=hardware-configuration.nix --exclude=hosts/*/hardware-configuration.nix -cf - .) | sudo tar -C "${ETC_DIR}" -xf -
  fi

  success "配置同步完成"
}

rebuild_system() {
  log "重建系统（${MODE}），目标：${TARGET_NAME}"
  local nix_config="experimental-features = nix-command flakes"
  local rebuild_args=("${MODE}" "--show-trace" "--upgrade")
  if [[ -n "${NIX_CONFIG:-}" ]]; then
    nix_config="${NIX_CONFIG}"$'\n'"${nix_config}"
  fi
  if sudo -E env NIX_CONFIG="${nix_config}" nixos-rebuild "${rebuild_args[@]}" --flake "${ETC_DIR}#${TARGET_NAME}"; then
    success "系统重建完成"
    return 0
  fi
  warn "系统重建失败"
  return 1
}

main() {
  printf '==> %s\n' "NixOS 一键部署"
  parse_args "$@"
  check_env

  local tmp_dir
  tmp_dir="$(mktemp -d)"

  cleanup() {
    local status=$?
    temp_dns_disable
    rm -rf "${tmp_dir}"
    exit "${status}"
  }
  trap cleanup EXIT

  if ! clone_repo_any "${tmp_dir}"; then
    log "尝试临时切换阿里云 DNS 后重试"
    temp_dns_enable
    rm -rf "${tmp_dir}"
    tmp_dir="$(mktemp -d)"
    if ! clone_repo_any "${tmp_dir}"; then
      error "仓库拉取失败，请检查网络"
    fi
  fi

  select_host "${tmp_dir}"
  validate_host "${tmp_dir}"
  if detect_per_user_tun "${tmp_dir}"; then
    PER_USER_TUN_ENABLED=true
  fi
  prompt_users
  dedupe_users
  validate_users
  if [[ ${#TARGET_USERS[@]} -gt 0 ]]; then
    configure_per_user_tun
  fi
  write_local_override "${tmp_dir}"
  prepare_etc_dir

  sync_repo_to_etc "${tmp_dir}"
  if ! rebuild_system; then
    if [[ "${DNS_ENABLED}" == false ]]; then
      log "尝试临时切换阿里云 DNS 后重试重建"
      temp_dns_enable
      if ! rebuild_system; then
        error "系统重建失败，请检查日志"
      fi
    else
      error "系统重建失败，请检查日志"
    fi
  fi
}

main "$@"
