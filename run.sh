#!/usr/bin/env bash
# 一键部署 NixOS 配置（从 GitHub/Gitee 拉取）。无参数默认流程。

set -euo pipefail

# 仓库地址与分支
REPO_URLS=(
  "https://gitee.com/MCB-SMART-BOY/nixos-config.git"
  "https://github.com/MCB-SMART-BOY/nixos-config.git"
)
BRANCH="master"

# 运行参数（由命令行或向导填充）
TARGET_NAME=""
TARGET_USERS=()
OVERWRITE_MODE="ask"
OVERWRITE_MODE_SET=false
PER_USER_TUN_ENABLED=false
# 每用户 TUN 临时映射（用户 -> 接口 / DNS 端口）
declare -A USER_TUN
declare -A USER_DNS
# GPU 临时覆盖
GPU_OVERRIDE=false
GPU_MODE=""
GPU_IGPU_VENDOR=""
GPU_PRIME_MODE=""
GPU_INTEL_BUS=""
GPU_AMD_BUS=""
GPU_NVIDIA_BUS=""
GPU_NVIDIA_OPEN=""
GPU_SPECIALISATIONS_ENABLED=false
GPU_SPECIALISATION_MODES=()
# nixos-rebuild 模式（switch/test/build）
MODE="switch"
# 是否为 nixos-rebuild 附加 --upgrade（默认关闭，保证可复现）
REBUILD_UPGRADE=false
# 目标配置目录（默认 /etc/nixos）
ETC_DIR="/etc/nixos"
# 临时 DNS 是否已开启
DNS_ENABLED=false
# 临时仓库目录
TMP_DIR=""
# sudo wrapper（root 模式下为空）
SUDO="sudo"
# rootless 模式标记（无 sudo/无法提权）
ROOTLESS=false

# 进度条控制
PROGRESS_TOTAL=7
PROGRESS_CURRENT=0
WIZARD_ACTION=""

# 脚本流程概览：
# 1) 检查环境 2) 选择主机/用户 3) 拉取仓库
# 4) 同步到 /etc/nixos 5) nixos-rebuild 6) 打印摘要

# 检测终端是否支持颜色输出
if command -v tput >/dev/null 2>&1 && [[ -t 1 ]]; then
  COLOR_RESET="$(tput sgr0)"
  COLOR_BOLD="$(tput bold)"
  COLOR_DIM="$(tput dim)"
  COLOR_GREEN="$(tput setaf 2)"
  COLOR_YELLOW="$(tput setaf 3)"
  COLOR_RED="$(tput setaf 1)"
  COLOR_CYAN="$(tput setaf 6)"
else
  COLOR_RESET=""
  COLOR_BOLD=""
  COLOR_DIM=""
  COLOR_GREEN=""
  COLOR_YELLOW=""
  COLOR_RED=""
  COLOR_CYAN=""
fi

# 统一输出带颜色的日志。
msg() {
  local level="$1"
  local label
  shift
  case "${level}" in
    INFO) label="${COLOR_CYAN}信息${COLOR_RESET}" ;;
    OK) label="${COLOR_GREEN}完成${COLOR_RESET}" ;;
    WARN) label="${COLOR_YELLOW}警告${COLOR_RESET}" ;;
    ERROR) label="${COLOR_RED}错误${COLOR_RESET}" ;;
    *) label="${level}" ;;
  esac
  printf '[%s] %s\n' "${label}" "$*"
}

# 输出普通信息。
log() { msg INFO "$*"; }
# 输出成功信息。
success() { msg OK "$*"; }
# 输出警告信息。
warn() { msg WARN "$*"; }
# 输出错误并退出。
error() {
  msg ERROR "$*"
  exit 1
}

# 以 root 执行命令（root 模式下跳过 sudo）。
as_root() {
  if [[ -n "${SUDO}" ]]; then
    sudo "$@"
  else
    "$@"
  fi
}

# 打印脚本标题。
banner() {
  printf '%s\n' "${COLOR_BOLD}==========================================${COLOR_RESET}"
  printf '%s\n' "${COLOR_BOLD}        NixOS 一键部署（run.sh）           ${COLOR_RESET}"
  printf '%s\n' "${COLOR_BOLD}==========================================${COLOR_RESET}"
}

# 打印章节标题。
section() {
  printf '\n%s%s%s\n' "${COLOR_BOLD}" "$*" "${COLOR_RESET}"
}

# 打印灰色提示。
note() {
  printf '%s%s%s\n' "${COLOR_DIM}" "$*" "${COLOR_RESET}"
}

# 读取用户输入，支持默认值。
ask_input() {
  local prompt="$1"
  local default="$2"
  local input=""
  if is_tty; then
    read -r -p "${prompt}（默认 ${default}）： " input
  fi
  if [[ -z "${input}" ]]; then
    printf '%s' "${default}"
  else
    printf '%s' "${input}"
  fi
}

# 更新并显示进度条。
progress_step() {
  local label="$1"
  PROGRESS_CURRENT=$((PROGRESS_CURRENT + 1))
  local width=24
  local filled=$((PROGRESS_CURRENT * width / PROGRESS_TOTAL))
  local empty=$((width - filled))
  local bar
  bar="$(printf "%${filled}s" | tr ' ' '#')"
  bar+=$(printf "%${empty}s" | tr ' ' '-')
  printf '%s进度: [%s] %s/%s %s%s\n' "${COLOR_CYAN}" "${bar}" "${PROGRESS_CURRENT}" "${PROGRESS_TOTAL}" "${label}" "${COLOR_RESET}"
}

# 确认是否继续执行。
confirm_continue() {
  local prompt="$1"
  if ! is_tty; then
    return 0
  fi
  local answer
  read -r -p "${prompt} [Y/n] " answer
  case "${answer}" in
    n|N) error "已退出" ;;
    *) return 0 ;;
  esac
}

# 显示菜单并读取选择。
menu_prompt() {
  local title="$1"
  local default_index="$2"
  shift 2
  local options=("$@")
  local choice=""
  local total=${#options[@]}

  while true; do
    # 打印菜单选项
    printf '\n%s%s%s\n' "${COLOR_BOLD}" "${title}" "${COLOR_RESET}" >&2
    local i=1
    for opt in "${options[@]}"; do
      printf '  %s) %s\n' "${i}" "${opt}" >&2
      i=$((i + 1))
    done
    read -r -p "请选择 [1-${total}]（默认 ${default_index}）： " choice
    if [[ -z "${choice}" ]]; then
      choice="${default_index}"
    fi
    if [[ "${choice}" =~ ^[0-9]+$ ]] && ((choice >= 1 && choice <= total)); then
      printf '%s' "${choice}"
      return 0
    fi
    echo "无效选择，请重试。" >&2
  done
}

# 向导中处理返回/退出。
wizard_back_or_quit() {
  local prompt="$1"
  local answer=""
  # c=继续，b=返回，q=退出
  read -r -p "${prompt} [c继续/b返回/q退出]（默认 c）： " answer
  case "${answer}" in
    b|B) WIZARD_ACTION="back" ;;
    q|Q) error "已退出" ;;
    *) WIZARD_ACTION="continue" ;;
  esac
}

# 清空每用户 TUN 临时配置。
reset_tun_maps() {
  USER_TUN=()
  USER_DNS=()
}

# 清空 GPU 临时配置。
reset_gpu_override() {
  GPU_OVERRIDE=false
  GPU_MODE=""
  GPU_IGPU_VENDOR=""
  GPU_PRIME_MODE=""
  GPU_INTEL_BUS=""
  GPU_AMD_BUS=""
  GPU_NVIDIA_BUS=""
  GPU_NVIDIA_OPEN=""
  GPU_SPECIALISATIONS_ENABLED=false
  GPU_SPECIALISATION_MODES=()
}

# 规整 PCI busId（0000:00:02.0 -> PCI:0:2:0）。
strip_leading_zeros() {
  local value="$1"
  value="$(printf '%s' "${value}" | sed -E 's/^0+//')"
  printf '%s' "${value:-0}"
}

normalize_pci_bus_id() {
  local addr="$1"
  local raw="${addr#0000:}"
  local bus="${raw%%:*}"
  local rest="${raw#*:}"
  local dev="${rest%%.*}"
  local func="${rest#*.}"
  bus="$(strip_leading_zeros "${bus}")"
  dev="$(strip_leading_zeros "${dev}")"
  func="$(strip_leading_zeros "${func}")"
  printf 'PCI:%s:%s:%s' "${bus}" "${dev}" "${func}"
}

detect_bus_id_from_lspci() {
  local vendor="$1"
  if ! command -v lspci >/dev/null 2>&1; then
    return 0
  fi
  local line=""
  while IFS= read -r line; do
    case "${vendor}" in
      intel) [[ "${line}" == *"Intel"* ]] || continue ;;
      amd)
        [[ "${line}" == *"AMD"* || "${line}" == *"Advanced Micro Devices"* ]] || continue
        ;;
      nvidia) [[ "${line}" == *"NVIDIA"* ]] || continue ;;
      *) return 0 ;;
    esac
    local addr="${line%% *}"
    if [[ "${addr}" == *":"*"."* ]]; then
      normalize_pci_bus_id "${addr}"
      return 0
    fi
  done < <(lspci -D -d ::03xx 2>/dev/null || true)
}

extract_bus_id_from_file() {
  local file="$1"
  local key="$2"
  local line=""
  line="$(grep -E "${key}[[:space:]]*=[[:space:]]*\"[^\"]+\"" "${file}" 2>/dev/null | head -n1 || true)"
  if [[ -n "${line}" ]]; then
    printf '%s' "${line}" | sed -E 's/.*"([^"]+)".*/\1/'
  fi
}

resolve_bus_id_default() {
  local vendor="$1"
  local detected=""
  detected="$(detect_bus_id_from_lspci "${vendor}" || true)"
  if [[ -n "${detected}" ]]; then
    printf '%s' "${detected}"
    return 0
  fi

  local key=""
  case "${vendor}" in
    intel) key="intelBusId" ;;
    amd) key="amdgpuBusId" ;;
    nvidia) key="nvidiaBusId" ;;
    *) return 0 ;;
  esac

  local files=()
  if [[ -n "${ETC_DIR}" && -n "${TARGET_NAME}" ]]; then
    files+=("${ETC_DIR}/hosts/${TARGET_NAME}/local.nix")
    files+=("${ETC_DIR}/hosts/${TARGET_NAME}/default.nix")
  fi
  if [[ -n "${TMP_DIR}" && -n "${TARGET_NAME}" ]]; then
    files+=("${TMP_DIR}/hosts/${TARGET_NAME}/local.nix")
    files+=("${TMP_DIR}/hosts/${TARGET_NAME}/default.nix")
  fi

  local file=""
  for file in "${files[@]}"; do
    if [[ -f "${file}" ]]; then
      local value=""
      value="$(extract_bus_id_from_file "${file}" "${key}")"
      if [[ -n "${value}" ]]; then
        printf '%s' "${value}"
        return 0
      fi
    fi
  done
}

# 显示使用帮助。
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
  --upgrade               重建时附加 --upgrade（默认不附加）
  -h, --help              显示帮助
EOF_USAGE
}

# 解析命令行参数。
parse_args() {
  # 逐个解析命令行参数
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
        # 支持逗号或空格分隔
        raw_users="${raw_users//,/ }"
        read -r -a more_users <<< "${raw_users}"
        TARGET_USERS+=("${more_users[@]}")
        shift 2
        ;;
      --backup)
        OVERWRITE_MODE="backup"
        OVERWRITE_MODE_SET=true
        shift
        ;;
      --ask)
        OVERWRITE_MODE="ask"
        OVERWRITE_MODE_SET=true
        shift
        ;;
      --overwrite)
        OVERWRITE_MODE="overwrite"
        OVERWRITE_MODE_SET=true
        shift
        ;;
      --upgrade)
        REBUILD_UPGRADE=true
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

# 检测是否存在硬件配置文件。
has_any_hardware_config() {
  local etc_dir="$1"
  if [[ -f "${etc_dir}/hardware-configuration.nix" ]]; then
    return 0
  fi
  if [[ -n "${TARGET_NAME}" && -f "${etc_dir}/hosts/${TARGET_NAME}/hardware-configuration.nix" ]]; then
    return 0
  fi
  if [[ -d "${etc_dir}/hosts" ]]; then
    if find "${etc_dir}/hosts" -maxdepth 2 -name hardware-configuration.nix -print -quit 2>/dev/null | grep -q .; then
      return 0
    fi
  fi
  return 1
}

# 选定主机后检查硬件配置是否存在。
ensure_host_hardware_config() {
  if [[ -f "${ETC_DIR}/hardware-configuration.nix" ]]; then
    return 0
  fi
  if [[ -n "${TARGET_NAME}" && -f "${ETC_DIR}/hosts/${TARGET_NAME}/hardware-configuration.nix" ]]; then
    return 0
  fi
  error "缺少硬件配置：${ETC_DIR}/hardware-configuration.nix 或 ${ETC_DIR}/hosts/${TARGET_NAME}/hardware-configuration.nix；请先运行 nixos-generate-config。"
}

# 检查环境依赖与权限。
check_env() {
  log "检查环境..."

  # root 直接运行；普通用户依赖 sudo（若不可用则进入 rootless）
  if [[ "$(whoami)" == "root" ]]; then
    warn "检测到 root，将跳过 sudo。"
    SUDO=""
  else
    if ! command -v sudo >/dev/null 2>&1; then
      warn "未找到 sudo，进入 rootless 模式。"
      SUDO=""
      ROOTLESS=true
    fi
  fi

  if ! command -v git >/dev/null 2>&1; then
    error "未找到 git。"
  fi

  # 确保当前环境是 NixOS
  if ! command -v nixos-rebuild >/dev/null 2>&1; then
    error "未找到 nixos-rebuild。"
  fi

  # sudo 可用性检查（避免容器 no_new_privileges）
  if [[ -n "${SUDO}" ]]; then
    if ! sudo -n true 2>/tmp/mcb-sudo-check.$$; then
      if grep -qi "no new privileges" /tmp/mcb-sudo-check.$$ 2>/dev/null; then
        warn "sudo 无法提权（no new privileges），进入 rootless 模式。"
        SUDO=""
        ROOTLESS=true
      else
        warn "sudo 需要交互输入密码，将在需要时提示。"
      fi
    fi
    rm -f /tmp/mcb-sudo-check.$$ 2>/dev/null || true
  fi

  # /etc/nixos 必须包含硬件配置（避免覆盖后无法启动）
  if ! has_any_hardware_config "${ETC_DIR}"; then
    error "缺少硬件配置：${ETC_DIR}/hardware-configuration.nix 或 ${ETC_DIR}/hosts/<hostname>/hardware-configuration.nix；请先运行 nixos-generate-config。"
  fi

  # rootless 模式下校验写入路径与 rebuild 模式
  if [[ "${ROOTLESS}" == "true" ]]; then
    if [[ ! -w "${ETC_DIR}" ]]; then
      if is_tty; then
        local alt_dir="${HOME}/.nixos"
        read -r -p "无权限写入 ${ETC_DIR}，改用 ${alt_dir}？ [Y/n] " ans
        case "${ans}" in
          n|N)
            error "无法写入 ${ETC_DIR}，请使用 root 运行或修改权限。"
            ;;
          *)
            ETC_DIR="${alt_dir}"
            ;;
        esac
      else
        ETC_DIR="${HOME}/.nixos"
      fi
      log "rootless 模式使用目录：${ETC_DIR}"
    fi

    if [[ "${MODE}" == "switch" || "${MODE}" == "test" ]]; then
      warn "rootless 模式无法切换系统，将自动改为 build。"
      MODE="build"
    fi
  fi
}

# 检测脚本的 shebang shell。
script_shebang_shell() {
  # 只允许 bash/sh 作为 shebang，避免执行失败
  local line="$1"
  case "${line}" in
    *"/bash"*|*"env bash"*|*"/sh"*|*"env sh"*) return 0 ;;
    *) return 1 ;;
  esac
}

# 自检脚本的 shebang 兼容性。
self_check_scripts() {
  # 遍历仓库脚本，确保第一行 shebang 合法
  local repo_dir="$1"
  local search_dir="${repo_dir}/home/users"
  local scripts=()

  # 收集 home/users/*/scripts 下的脚本
  if [[ -d "${search_dir}" ]]; then
    mapfile -d '' -t scripts < <(
      find "${search_dir}" -type f -path "*/scripts/*" -print0 2>/dev/null
    )
  fi

  # 也检查本脚本本身
  if [[ -f "${repo_dir}/run.sh" ]]; then
    scripts+=("${repo_dir}/run.sh")
  fi

  if [[ ${#scripts[@]} -eq 0 ]]; then
    warn "未找到可自检脚本（home/users/*/scripts 或 run.sh）"
    return 0
  fi

  log "脚本自检..."

  local errors=0
  local warnings=0
  local file
  local shellcheck_available=false

  if command -v shellcheck >/dev/null 2>&1; then
    shellcheck_available=true
  else
    warn "未检测到 shellcheck，跳过 Lint 检查"
  fi

  for file in "${scripts[@]}"; do
    local rel="${file#"${repo_dir}"/}"
    local shebang=""

    if [[ ! -s "${file}" ]]; then
      warn "脚本为空：${rel}"
      warnings=$((warnings + 1))
      continue
    fi

    if [[ ! -x "${file}" ]]; then
      warn "脚本缺少可执行权限：${rel}"
      errors=$((errors + 1))
    fi

    if LC_ALL=C grep -q $'\r' "${file}"; then
      warn "检测到 CRLF：${rel}"
      errors=$((errors + 1))
    fi

    shebang="$(head -n1 "${file}" 2>/dev/null || true)"
    if [[ "${shebang}" != "#!"* ]]; then
      warn "缺少 shebang：${rel}"
      errors=$((errors + 1))
      continue
    fi

    if script_shebang_shell "${shebang}"; then
      if ! bash -n "${file}" 2>/tmp/mcb-script-check.$$; then
        warn "语法检查失败：${rel}"
        sed 's/^/  /' /tmp/mcb-script-check.$$ >&2 || true
        errors=$((errors + 1))
      fi
      rm -f /tmp/mcb-script-check.$$ 2>/dev/null || true

      if [[ "${shellcheck_available}" == "true" ]]; then
        if ! shellcheck -x "${file}" >/tmp/mcb-shellcheck.$$ 2>&1; then
          warn "shellcheck 警告：${rel}"
          sed 's/^/  /' /tmp/mcb-shellcheck.$$ >&2 || true
          warnings=$((warnings + 1))
        fi
        rm -f /tmp/mcb-shellcheck.$$ 2>/dev/null || true
      fi
    else
      warn "非 bash/sh 脚本，跳过语法检查：${rel}"
      warnings=$((warnings + 1))
    fi
  done

  if (( errors > 0 )); then
    error "脚本自检失败：${errors} 个错误（请修复后再继续）"
  fi

  success "脚本自检完成（${warnings} 个警告）"
}

# 判断是否为交互式终端。
is_tty() {
  [[ -t 0 && -t 1 ]]
}

# 列出可用主机。
list_hosts() {
  local repo_dir="$1"
  local host_dir="${repo_dir}/hosts"
  local hosts=()

  # hosts/ 下每个目录都是一个主机（profiles 除外）
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

# 选择目标主机。
select_host() {
  local repo_dir="$1"
  if [[ -z "${TARGET_NAME}" ]]; then
    # 交互式终端优先走菜单
    if is_tty; then
      local hosts=()
      mapfile -t hosts < <(list_hosts "${repo_dir}")
      if [[ ${#hosts[@]} -eq 0 ]]; then
        error "未找到可用的 hosts 目录。"
      fi
      local default_index=1
      local i=1
      for h in "${hosts[@]}"; do
        if [[ "${h}" == "nixos" ]]; then
          default_index="${i}"
          break
        fi
        i=$((i + 1))
      done
      local pick
      pick="$(menu_prompt "选择主机" "${default_index}" "${hosts[@]}")"
      TARGET_NAME="${hosts[$((pick - 1))]}"
    else
      # 非交互式则默认使用 nixos
      TARGET_NAME="nixos"
    fi
  fi
}

# 校验主机名合法性。
validate_host() {
  local repo_dir="$1"
  if [[ -z "${TARGET_NAME}" ]]; then
    error "未指定主机名称。"
  fi
  # 确保 hosts/<name> 存在
  if [[ ! -d "${repo_dir}/hosts/${TARGET_NAME}" ]]; then
    error "主机不存在：hosts/${TARGET_NAME}"
  fi
}

# 检测每用户 TUN 配置是否完整。
detect_per_user_tun() {
  local host_file="$1/hosts/${TARGET_NAME}/default.nix"
  local in_block=0
  local line

  if [[ ! -f "${host_file}" ]]; then
    return 1
  fi

  # 简单扫描 perUserTun.enable = true
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

extract_user_from_file() {
  local file="$1"
  local line=""
  line="$(grep -E 'mcb\.user[[:space:]]*=[[:space:]]*.*"[^"]+"' "${file}" 2>/dev/null | head -n1 || true)"
  if [[ -z "${line}" ]]; then
    line="$(grep -E '^[[:space:]]*user[[:space:]]*=[[:space:]]*"[^"]+"' "${file}" 2>/dev/null | head -n1 || true)"
  fi
  if [[ -n "${line}" ]]; then
    printf '%s' "${line}" | sed -E 's/.*"([^"]+)".*/\1/'
  fi
}

resolve_default_user() {
  local files=()
  local file=""
  local value=""

  if [[ -n "${TMP_DIR}" && -n "${TARGET_NAME}" ]]; then
    files+=("${TMP_DIR}/hosts/${TARGET_NAME}/local.nix")
    files+=("${TMP_DIR}/hosts/${TARGET_NAME}/default.nix")
  fi
  if [[ -n "${ETC_DIR}" && -n "${TARGET_NAME}" ]]; then
    files+=("${ETC_DIR}/hosts/${TARGET_NAME}/local.nix")
    files+=("${ETC_DIR}/hosts/${TARGET_NAME}/default.nix")
  fi

  for file in "${files[@]}"; do
    if [[ -f "${file}" ]]; then
      value="$(extract_user_from_file "${file}")"
      if [[ -n "${value}" ]]; then
        printf '%s' "${value}"
        return 0
      fi
    fi
  done
  printf '%s' "mcbnixos"
}

# 交互式配置每用户 TUN。
configure_per_user_tun() {
  if [[ "${PER_USER_TUN_ENABLED}" != "true" ]]; then
    return 0
  fi

  if is_tty; then
    # 让用户选择配置方式
    local pick
    pick="$(menu_prompt "TUN 配置方式" 1 "沿用主机配置" "使用默认接口/端口" "逐个设置" "返回")"
    case "${pick}" in
      4)
        WIZARD_ACTION="back"
        return 0
        ;;
      1)
        reset_tun_maps
        return 0
        ;;
      2)
        # 自动分配 tun0/tun1 + 1053/1054 ...
        reset_tun_maps
        local idx=0
        local user
        for user in "${TARGET_USERS[@]}"; do
          USER_TUN["${user}"]="tun${idx}"
          USER_DNS["${user}"]=$((1053 + idx))
          idx=$((idx + 1))
        done
        return 0
        ;;
      3)
        reset_tun_maps
        ;;
    esac
  else
    reset_tun_maps
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
      # 逐个询问每个用户的接口/端口
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

# 交互式配置 GPU。
configure_gpu() {
  if ! is_tty; then
    reset_gpu_override
    return 0
  fi

  local pick
  pick="$(menu_prompt "GPU 配置方式" 1 "沿用主机配置" "选择 GPU 模式" "返回")"
  case "${pick}" in
    1)
      reset_gpu_override
      return 0
      ;;
    2)
      GPU_OVERRIDE=true
      ;;
    3)
      WIZARD_ACTION="back"
      return 0
      ;;
  esac

  pick="$(menu_prompt "选择 GPU 模式" 2 "核显 (igpu)" "混合 (hybrid)" "独显 (dgpu)" "返回")"
  case "${pick}" in
    1) GPU_MODE="igpu" ;;
    2) GPU_MODE="hybrid" ;;
    3) GPU_MODE="dgpu" ;;
    4)
      WIZARD_ACTION="back"
      return 0
      ;;
  esac

  if [[ "${GPU_MODE}" == "igpu" || "${GPU_MODE}" == "hybrid" ]]; then
    pick="$(menu_prompt "核显厂商" 1 "Intel" "AMD" "返回")"
    case "${pick}" in
      1) GPU_IGPU_VENDOR="intel" ;;
      2) GPU_IGPU_VENDOR="amd" ;;
      3)
        WIZARD_ACTION="back"
        return 0
        ;;
    esac
  fi

  if [[ "${GPU_MODE}" == "hybrid" ]]; then
    pick="$(menu_prompt "PRIME 模式" 1 "offload（推荐，Wayland）" "sync（偏向 X11）" "reverseSync（偏向 X11）" "返回")"
    case "${pick}" in
      1) GPU_PRIME_MODE="offload" ;;
      2) GPU_PRIME_MODE="sync" ;;
      3) GPU_PRIME_MODE="reverseSync" ;;
      4)
        WIZARD_ACTION="back"
        return 0
        ;;
    esac

    # iGPU busId
    if [[ "${GPU_IGPU_VENDOR}" == "intel" ]]; then
      local default_intel=""
      default_intel="$(resolve_bus_id_default intel || true)"
      while true; do
        if [[ -n "${default_intel}" ]]; then
          read -r -p "Intel iGPU busId（如 PCI:0:2:0，默认 ${default_intel}）： " GPU_INTEL_BUS
          if [[ -z "${GPU_INTEL_BUS}" ]]; then
            GPU_INTEL_BUS="${default_intel}"
          fi
        else
          read -r -p "Intel iGPU busId（如 PCI:0:2:0）： " GPU_INTEL_BUS
        fi
        [[ -n "${GPU_INTEL_BUS}" ]] && break
        echo "不能为空，请重试。"
      done
    else
      local default_amd=""
      default_amd="$(resolve_bus_id_default amd || true)"
      while true; do
        if [[ -n "${default_amd}" ]]; then
          read -r -p "AMD iGPU busId（如 PCI:4:0:0，默认 ${default_amd}）： " GPU_AMD_BUS
          if [[ -z "${GPU_AMD_BUS}" ]]; then
            GPU_AMD_BUS="${default_amd}"
          fi
        else
          read -r -p "AMD iGPU busId（如 PCI:4:0:0）： " GPU_AMD_BUS
        fi
        [[ -n "${GPU_AMD_BUS}" ]] && break
        echo "不能为空，请重试。"
      done
    fi

    # NVIDIA busId
    local default_nvidia=""
    default_nvidia="$(resolve_bus_id_default nvidia || true)"
    while true; do
      if [[ -n "${default_nvidia}" ]]; then
        read -r -p "NVIDIA dGPU busId（如 PCI:1:0:0，默认 ${default_nvidia}）： " GPU_NVIDIA_BUS
        if [[ -z "${GPU_NVIDIA_BUS}" ]]; then
          GPU_NVIDIA_BUS="${default_nvidia}"
        fi
      else
        read -r -p "NVIDIA dGPU busId（如 PCI:1:0:0）： " GPU_NVIDIA_BUS
      fi
      [[ -n "${GPU_NVIDIA_BUS}" ]] && break
      echo "不能为空，请重试。"
    done
  fi

  if [[ "${GPU_MODE}" == "hybrid" || "${GPU_MODE}" == "dgpu" ]]; then
    local answer=""
    read -r -p "NVIDIA 使用开源内核模块？ [Y/n] " answer
    case "${answer}" in
      n|N) GPU_NVIDIA_OPEN="false" ;;
      *) GPU_NVIDIA_OPEN="true" ;;
    esac
  fi

  if [[ "${GPU_MODE}" == "hybrid" ]]; then
    local answer=""
    read -r -p "生成 GPU specialisation（igpu/hybrid/dgpu）以便切换？ [Y/n] " answer
    case "${answer}" in
      n|N) GPU_SPECIALISATIONS_ENABLED=false ;;
      *) GPU_SPECIALISATIONS_ENABLED=true ;;
    esac
    if [[ "${GPU_SPECIALISATIONS_ENABLED}" == "true" ]]; then
      GPU_SPECIALISATION_MODES=("igpu" "hybrid" "dgpu")
    fi
  fi
}

# 交互式输入用户列表。
prompt_users() {
  local default_user=""
  default_user="$(resolve_default_user)"
  if [[ ${#TARGET_USERS[@]} -eq 0 ]]; then
    if is_tty; then
      # 提供默认用户或手动输入
      local pick
      pick="$(menu_prompt "选择用户" 1 "使用默认用户 (${default_user})" "输入用户列表" "返回" "退出")"
      case "${pick}" in
        1)
          TARGET_USERS=("${default_user}")
          ;;
        2)
          # 支持空格或逗号分隔
          local input
          read -r -p "用户名列表（空格或逗号分隔）： " input
          input="${input//,/ }"
          if [[ -n "${input}" ]]; then
            read -r -a TARGET_USERS <<< "${input}"
          else
            TARGET_USERS=("${default_user}")
          fi
          ;;
        3)
          WIZARD_ACTION="back"
          return 0
          ;;
        4)
          error "已退出"
          ;;
      esac
    else
      # 非交互模式默认使用主机配置中的 mcb.user
      TARGET_USERS=("${default_user}")
    fi
  fi
}

# 用户列表去重并保持顺序。
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

# 校验用户列表与格式。
validate_users() {
  local user
  for user in "${TARGET_USERS[@]}"; do
    # 只允许 linux 用户名格式
    if [[ ! "${user}" =~ ^[a-z_][a-z0-9_-]*$ ]]; then
      error "用户名不合法：${user}"
    fi
  done
}

# 写入 hosts/<host>/local.nix 覆盖项。
write_local_override() {
  local repo_dir="$1"
  local host_dir="${repo_dir}/hosts/${TARGET_NAME}"
  local file="${host_dir}/local.nix"

  if [[ ${#TARGET_USERS[@]} -eq 0 ]]; then
    return 0
  fi

  # 只在需要时生成 local.nix（不会覆盖已有文件）

  if [[ ! -d "${host_dir}" ]]; then
    error "主机目录不存在：${host_dir}"
  fi

  local primary="${TARGET_USERS[0]}"
  local list=""
  local user
  # 生成用户列表字符串
  for user in "${TARGET_USERS[@]}"; do
    list="${list} \"${user}\""
  done

  {
    # local.nix 会覆盖 mcb.user/mcb.users 等配置
    echo "{ lib, ... }:"
    echo ""
    echo "{"
    echo "  mcb.user = lib.mkForce \"${primary}\";"
    echo "  mcb.users = lib.mkForce [${list} ];"

    if [[ "${PER_USER_TUN_ENABLED}" == "true" && ${#USER_TUN[@]} -gt 0 ]]; then
      echo "  mcb.perUserTun.interfaces = lib.mkForce {"
      for user in "${TARGET_USERS[@]}"; do
        echo "    ${user} = \"${USER_TUN[${user}]}\";"
      done
      echo "  };"
      echo "  mcb.perUserTun.dnsPorts = lib.mkForce {"
      for user in "${TARGET_USERS[@]}"; do
        echo "    ${user} = ${USER_DNS[${user}]};"
      done
      echo "  };"
    fi

    if [[ "${GPU_OVERRIDE}" == "true" ]]; then
      echo "  mcb.hardware.gpu.mode = lib.mkForce \"${GPU_MODE}\";"
      if [[ -n "${GPU_IGPU_VENDOR}" ]]; then
        echo "  mcb.hardware.gpu.igpuVendor = lib.mkForce \"${GPU_IGPU_VENDOR}\";"
      fi
      if [[ -n "${GPU_NVIDIA_OPEN}" ]]; then
        echo "  mcb.hardware.gpu.nvidia.open = lib.mkForce ${GPU_NVIDIA_OPEN};"
      fi
      if [[ -n "${GPU_PRIME_MODE}" || -n "${GPU_INTEL_BUS}" || -n "${GPU_AMD_BUS}" || -n "${GPU_NVIDIA_BUS}" ]]; then
        echo "  mcb.hardware.gpu.prime = lib.mkForce {"
        if [[ -n "${GPU_PRIME_MODE}" ]]; then
          echo "    mode = \"${GPU_PRIME_MODE}\";"
        fi
        if [[ -n "${GPU_INTEL_BUS}" ]]; then
          echo "    intelBusId = \"${GPU_INTEL_BUS}\";"
        fi
        if [[ -n "${GPU_AMD_BUS}" ]]; then
          echo "    amdgpuBusId = \"${GPU_AMD_BUS}\";"
        fi
        if [[ -n "${GPU_NVIDIA_BUS}" ]]; then
          echo "    nvidiaBusId = \"${GPU_NVIDIA_BUS}\";"
        fi
        echo "  };"
      fi
      if [[ "${GPU_SPECIALISATIONS_ENABLED}" == "true" ]]; then
        echo "  mcb.hardware.gpu.specialisations.enable = lib.mkForce true;"
        if [[ ${#GPU_SPECIALISATION_MODES[@]} -gt 0 ]]; then
          local mode_list=""
          local mode
          for mode in "${GPU_SPECIALISATION_MODES[@]}"; do
            mode_list+=" \"${mode}\""
          done
          echo "  mcb.hardware.gpu.specialisations.modes = lib.mkForce [${mode_list} ];"
        fi
      fi
    fi

    echo "}"
  } > "${file}"
}

# 备份 /etc/nixos 到时间戳目录。
backup_etc() {
  # 备份目录按时间戳命名，便于回滚
  local timestamp
  timestamp="$(date +%Y%m%d-%H%M%S)"
  local backup_dir="${ETC_DIR}.backup-${timestamp}"
  log "备份 ${ETC_DIR} -> ${backup_dir}"
  as_root mkdir -p "${backup_dir}"
  if command -v rsync >/dev/null 2>&1; then
    as_root rsync -a "${ETC_DIR}/" "${backup_dir}/"
  else
    as_root cp -a "${ETC_DIR}/." "${backup_dir}/"
  fi
  success "备份完成"
}

# 准备 /etc/nixos 目录。
prepare_etc_dir() {
  # 当目录已存在时，根据策略决定是否备份/覆盖
  if [[ -d "${ETC_DIR}" && -n "$(ls -A "${ETC_DIR}" 2>/dev/null)" ]]; then
    case "${OVERWRITE_MODE}" in
      backup)
        backup_etc
        ;;
      overwrite)
        note "将覆盖 ${ETC_DIR}（未启用备份）"
        ;;
      ask)
        if is_tty; then
          while true; do
            read -r -p "检测到 ${ETC_DIR} 已存在，选择 [b]备份并覆盖/[o]直接覆盖/[q]退出（默认 b）： " answer
            case "${answer}" in
              b|B|"")
                backup_etc
                OVERWRITE_MODE="backup"
                break
                ;;
              o|O)
                OVERWRITE_MODE="overwrite"
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
          OVERWRITE_MODE="backup"
        fi
        ;;
      *)
        error "不支持的覆盖策略：${OVERWRITE_MODE}"
        ;;
    esac
  fi
}

# 清理 /etc/nixos，保留硬件配置文件。
clean_etc_dir_keep_hardware() {
  if [[ -z "${ETC_DIR}" || "${ETC_DIR}" == "/" ]]; then
    error "ETC_DIR 无效，拒绝清理：${ETC_DIR}"
  fi
  if [[ ! -d "${ETC_DIR}" ]]; then
    return 0
  fi

  local preserve_dir
  preserve_dir="$(mktemp -d)"

  if [[ -f "${ETC_DIR}/hardware-configuration.nix" ]]; then
    as_root cp -a "${ETC_DIR}/hardware-configuration.nix" "${preserve_dir}/"
  fi

  if [[ -d "${ETC_DIR}/hosts" ]]; then
    while IFS= read -r -d '' file; do
      local rel="${file#"${ETC_DIR}"/}"
      as_root mkdir -p "${preserve_dir}/$(dirname "${rel}")"
      as_root cp -a "${file}" "${preserve_dir}/${rel}"
    done < <(find "${ETC_DIR}/hosts" -maxdepth 2 -name hardware-configuration.nix -print0 2>/dev/null)
  fi

  as_root find "${ETC_DIR}" -mindepth 1 -maxdepth 1 -exec rm -rf {} +

  if [[ -f "${preserve_dir}/hardware-configuration.nix" ]]; then
    as_root cp -a "${preserve_dir}/hardware-configuration.nix" "${ETC_DIR}/"
  fi

  if [[ -d "${preserve_dir}/hosts" ]]; then
    as_root mkdir -p "${ETC_DIR}/hosts"
    as_root cp -a "${preserve_dir}/hosts/." "${ETC_DIR}/hosts/"
  fi

  rm -rf "${preserve_dir}"
}

# 检测默认网卡名称。
detect_default_iface() {
  # 读取默认路由对应的网卡
  if command -v ip >/dev/null 2>&1; then
    ip route show default 2>/dev/null | awk 'NR==1 {print $5; exit}'
  fi
}

TEMP_DNS_BACKEND=""
TEMP_DNS_BACKUP=""
TEMP_DNS_IFACE=""

# 临时启用 DNS 以修复网络。
temp_dns_enable() {
  local servers=("223.5.5.5" "223.6.6.6")
  local iface=""

  if [[ "${ROOTLESS}" == "true" ]]; then
    warn "rootless 模式无法临时设置 DNS，跳过。"
    return 1
  fi

  # 优先通过 systemd-resolved 临时设置 DNS
  if command -v resolvectl >/dev/null 2>&1 && command -v systemctl >/dev/null 2>&1; then
    if systemctl is-active --quiet systemd-resolved; then
      iface="$(detect_default_iface)"

      if [[ -n "${iface}" ]]; then
        log "临时 DNS（resolvectl ${iface}）：${servers[*]}"
        as_root resolvectl dns "${iface}" "${servers[@]}"
        as_root resolvectl domain "${iface}" "~."
        TEMP_DNS_BACKEND="resolvectl"
        TEMP_DNS_IFACE="${iface}"
        DNS_ENABLED=true
        return 0
      fi
    fi
  fi

  # 兜底方案：直接写 /etc/resolv.conf
  if [[ -f /etc/resolv.conf ]]; then
    TEMP_DNS_BACKUP="$(mktemp)"
    as_root cp -a /etc/resolv.conf "${TEMP_DNS_BACKUP}"
    as_root rm -f /etc/resolv.conf
    printf 'nameserver %s\n' "${servers[@]}" | as_root tee /etc/resolv.conf >/dev/null
    log "临时 DNS（/etc/resolv.conf）：${servers[*]}"
    TEMP_DNS_BACKEND="resolv.conf"
    DNS_ENABLED=true
    return 0
  fi

  error "无法设置临时 DNS（无 resolvectl 且缺少 /etc/resolv.conf）。"
}

# 恢复系统 DNS 设置。
temp_dns_disable() {
  if [[ "${TEMP_DNS_BACKEND}" == "resolvectl" ]]; then
    if [[ -n "${TEMP_DNS_IFACE}" ]]; then
      log "恢复 DNS（resolvectl ${TEMP_DNS_IFACE}）"
      as_root resolvectl revert "${TEMP_DNS_IFACE}" || true
      as_root resolvectl flush-caches >/dev/null 2>&1 || true
    fi
  elif [[ "${TEMP_DNS_BACKEND}" == "resolv.conf" ]]; then
    if [[ -n "${TEMP_DNS_BACKUP}" && -f "${TEMP_DNS_BACKUP}" ]]; then
      log "恢复 /etc/resolv.conf"
      as_root cp -a "${TEMP_DNS_BACKUP}" /etc/resolv.conf || true
      rm -f "${TEMP_DNS_BACKUP}"
    fi
  fi
}

# 克隆配置仓库。
clone_repo() {
  local tmp_dir="$1"
  local url="$2"
  log "拉取仓库：${url}（${BRANCH}）"
  # 使用浅克隆加快速度
  if git clone --depth 1 --branch "${BRANCH}" "${url}" "${tmp_dir}"; then
    success "仓库拉取完成"
    return 0
  fi
  warn "仓库拉取失败：${url}"
  return 1
}

# 尝试多个镜像地址克隆。
clone_repo_any() {
  local tmp_dir="$1"
  local url
  # 依次尝试 Gitee / GitHub
  for url in "${REPO_URLS[@]}"; do
    rm -rf "${tmp_dir}"
    mkdir -p "${tmp_dir}"
    if clone_repo "${tmp_dir}" "${url}"; then
      return 0
    fi
  done
  return 1
}

# 同步仓库到 /etc/nixos。
sync_repo_to_etc() {
  local repo_dir="$1"
  local delete_flags=()
  if [[ "${OVERWRITE_MODE}" == "overwrite" || "${OVERWRITE_MODE}" == "backup" ]]; then
    delete_flags=(--delete)
  fi
  log "同步到 ${ETC_DIR}"
  as_root mkdir -p "${ETC_DIR}"

  # 同步时排除 .git 与硬件配置，避免覆盖本机硬件配置
  if command -v rsync >/dev/null 2>&1; then
    as_root rsync -a \
      "${delete_flags[@]}" \
      --exclude '.git/' \
      --exclude 'hardware-configuration.nix' \
      --exclude 'hosts/*/hardware-configuration.nix' \
      "${repo_dir}/" "${ETC_DIR}/"
  else
    if [[ ${#delete_flags[@]} -gt 0 ]]; then
      clean_etc_dir_keep_hardware
    fi
    (cd "${repo_dir}" && tar --exclude=.git --exclude=hardware-configuration.nix --exclude=hosts/*/hardware-configuration.nix -cf - .) | as_root tar -C "${ETC_DIR}" -xf -
  fi

  success "配置同步完成"
}

# 执行 nixos-rebuild（switch/test/build）。
rebuild_system() {
  log "重建系统（${MODE}），目标：${TARGET_NAME}"
  local nix_config="experimental-features = nix-command flakes"
  # 默认不带 --upgrade，显式请求时再升级上游依赖
  local rebuild_args=("${MODE}" "--show-trace")
  if [[ "${REBUILD_UPGRADE}" == "true" ]]; then
    rebuild_args+=("--upgrade")
  fi
  # 合并外部 NIX_CONFIG（如用户自定义缓存）
  if [[ -n "${NIX_CONFIG:-}" ]]; then
    nix_config="${NIX_CONFIG}"$'\n'"${nix_config}"
  fi
  if [[ -n "${SUDO}" ]]; then
    if sudo -E env NIX_CONFIG="${nix_config}" nixos-rebuild "${rebuild_args[@]}" --flake "${ETC_DIR}#${TARGET_NAME}"; then
      success "系统重建完成"
      return 0
    fi
  else
    if env NIX_CONFIG="${nix_config}" nixos-rebuild "${rebuild_args[@]}" --flake "${ETC_DIR}#${TARGET_NAME}"; then
      success "系统重建完成"
      return 0
    fi
  fi
  warn "系统重建失败"
  return 1
}

# 打印部署摘要与提示。
print_summary() {
  section "部署概要"
  printf '%s主机：%s%s\n' "${COLOR_BOLD}" "${TARGET_NAME}" "${COLOR_RESET}"
  printf '%s用户：%s%s\n' "${COLOR_BOLD}" "${TARGET_USERS[*]}" "${COLOR_RESET}"
  printf '%s覆盖策略：%s%s\n' "${COLOR_BOLD}" "${OVERWRITE_MODE}" "${COLOR_RESET}"
  if [[ "${REBUILD_UPGRADE}" == "true" ]]; then
    printf '%s依赖升级：%s%s\n' "${COLOR_BOLD}" "启用 (--upgrade)" "${COLOR_RESET}"
  else
    printf '%s依赖升级：%s%s\n' "${COLOR_BOLD}" "关闭" "${COLOR_RESET}"
  fi
  if [[ "${PER_USER_TUN_ENABLED}" == "true" ]]; then
    if [[ ${#USER_TUN[@]} -gt 0 ]]; then
      printf '%sPer-user TUN：%s%s\n' "${COLOR_BOLD}" "已启用" "${COLOR_RESET}"
      local user
      for user in "${TARGET_USERS[@]}"; do
        printf '  - %s -> %s (DNS %s)\n' "${user}" "${USER_TUN[${user}]}" "${USER_DNS[${user}]}"
      done
    else
      printf '%sPer-user TUN：%s%s\n' "${COLOR_BOLD}" "已启用（沿用主机配置）" "${COLOR_RESET}"
    fi
  else
    printf '%sPer-user TUN：%s%s\n' "${COLOR_BOLD}" "未启用" "${COLOR_RESET}"
  fi

  if [[ "${GPU_OVERRIDE}" == "true" ]]; then
    printf '%sGPU：%s%s\n' "${COLOR_BOLD}" "${GPU_MODE}" "${COLOR_RESET}"
    if [[ -n "${GPU_IGPU_VENDOR}" ]]; then
      printf '  - iGPU 厂商：%s\n' "${GPU_IGPU_VENDOR}"
    fi
    if [[ -n "${GPU_PRIME_MODE}" ]]; then
      printf '  - PRIME：%s\n' "${GPU_PRIME_MODE}"
    fi
    if [[ -n "${GPU_INTEL_BUS}" ]]; then
      printf '  - Intel busId：%s\n' "${GPU_INTEL_BUS}"
    fi
    if [[ -n "${GPU_AMD_BUS}" ]]; then
      printf '  - AMD busId：%s\n' "${GPU_AMD_BUS}"
    fi
    if [[ -n "${GPU_NVIDIA_BUS}" ]]; then
      printf '  - NVIDIA busId：%s\n' "${GPU_NVIDIA_BUS}"
    fi
    if [[ -n "${GPU_NVIDIA_OPEN}" ]]; then
      printf '  - NVIDIA open：%s\n' "${GPU_NVIDIA_OPEN}"
    fi
    if [[ "${GPU_SPECIALISATIONS_ENABLED}" == "true" ]]; then
      printf '  - specialisation：启用 (%s)\n' "${GPU_SPECIALISATION_MODES[*]}"
    fi
  else
    printf '%sGPU：%s%s\n' "${COLOR_BOLD}" "沿用主机配置" "${COLOR_RESET}"
  fi
}

# 交互式向导主流程。
wizard_flow() {
  local step=1
  WIZARD_ACTION=""

  while true; do
    case "${step}" in
      1)
        # 选择主机
        select_host "${TMP_DIR}"
        validate_host "${TMP_DIR}"
        step=2
        ;;
      2)
        # 选择用户列表
        WIZARD_ACTION=""
        prompt_users
        if [[ "${WIZARD_ACTION}" == "back" ]]; then
          TARGET_USERS=()
          reset_tun_maps
          TARGET_NAME=""
          step=1
          continue
        fi
        dedupe_users
        validate_users
        step=3
        ;;
      3)
        # 检测并配置 per-user TUN
        if detect_per_user_tun "${TMP_DIR}"; then
          PER_USER_TUN_ENABLED=true
        else
          PER_USER_TUN_ENABLED=false
        fi
        WIZARD_ACTION=""
        if [[ "${PER_USER_TUN_ENABLED}" == "true" ]]; then
          configure_per_user_tun
          if [[ "${WIZARD_ACTION}" == "back" ]]; then
            reset_tun_maps
            step=2
            continue
          fi
        else
          reset_tun_maps
        fi
        step=4
        ;;
      4)
        # 配置 GPU 覆盖（可选）
        WIZARD_ACTION=""
        configure_gpu
        if [[ "${WIZARD_ACTION}" == "back" ]]; then
          reset_gpu_override
          step=3
          continue
        fi
        step=5
        ;;
      5)
        # 最终确认
        print_summary
        if is_tty; then
          wizard_back_or_quit "确认以上配置"
          case "${WIZARD_ACTION}" in
            back)
              step=4
              ;;
            continue)
              return 0
              ;;
            *)
              return 0
              ;;
          esac
        else
          return 0
        fi
        ;;
    esac
  done
}

# 脚本主入口。
main() {
  banner
  parse_args "$@"
  if [[ "${OVERWRITE_MODE_SET}" == "false" ]]; then
    if is_tty; then
      OVERWRITE_MODE="ask"
      note "未指定覆盖策略，交互模式默认询问（--ask）"
    else
      OVERWRITE_MODE="backup"
      note "未指定覆盖策略，非交互模式默认备份并覆盖（--backup）"
    fi
    OVERWRITE_MODE_SET=true
  fi
  section "环境检查"
  check_env
  progress_step "环境检查"

  TMP_DIR="$(mktemp -d)"

  cleanup() {
    local status=$?
    temp_dns_disable
    if [[ -n "${TMP_DIR}" ]]; then
      rm -rf "${TMP_DIR}"
    fi
    exit "${status}"
  }
  trap cleanup EXIT

  # 先拉取仓库（失败时尝试临时 DNS）
  section "拉取仓库"
  if ! clone_repo_any "${TMP_DIR}"; then
    log "尝试临时切换阿里云 DNS 后重试"
    temp_dns_enable
    rm -rf "${TMP_DIR}"
    TMP_DIR="$(mktemp -d)"
    if ! clone_repo_any "${TMP_DIR}"; then
      error "仓库拉取失败，请检查网络"
    fi
  fi
  progress_step "拉取仓库"

  section "脚本自检"
  self_check_scripts "${TMP_DIR}"
  progress_step "脚本自检"

  # 交互式向导：选择主机/用户/TUN
  wizard_flow
  ensure_host_hardware_config
  write_local_override "${TMP_DIR}"
  progress_step "收集配置"
  confirm_continue "确认以上配置并继续同步？"
  section "同步与构建"
  prepare_etc_dir
  progress_step "准备覆盖策略"

  sync_repo_to_etc "${TMP_DIR}"
  progress_step "同步配置"
  confirm_continue "配置已同步，继续重建系统？"
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
  progress_step "系统重建"
}

main "$@"
