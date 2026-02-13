#!/usr/bin/env bash
# 一键部署 NixOS 配置（优先本地仓库，必要时再从 GitHub/Gitee 拉取）。

set -euo pipefail

# 仓库地址与分支（仅在未检测到本地仓库时使用）
REPO_URLS=(
  "https://gitee.com/MCB-SMART-BOY/nixos-config.git"
  "https://github.com/MCB-SMART-BOY/nixos-config.git"
)
BRANCH="master"
SOURCE_REF=""
ALLOW_REMOTE_HEAD=false
SOURCE_COMMIT=""
SOURCE_CHOICE_SET=false

# 运行状态（由向导填充）
TARGET_NAME=""
TARGET_USERS=()
TARGET_ADMIN_USERS=()
DEPLOY_MODE="manage-users"   # manage-users | update-existing
DEPLOY_MODE_SET=false
FORCE_REMOTE_SOURCE=false
OVERWRITE_MODE="ask"
OVERWRITE_MODE_SET=false
PER_USER_TUN_ENABLED=false
HOST_PROFILE_KIND="unknown"
# 每用户 TUN 临时映射（用户 -> 接口 / DNS 端口）
declare -A USER_TUN
declare -A USER_DNS
# 服务器软件/虚拟化临时覆盖
SERVER_OVERRIDES_ENABLED=false
SERVER_ENABLE_DEV=""
SERVER_ENABLE_NETWORK_GUI=""
SERVER_ENABLE_BROWSERS_AND_MEDIA=""
SERVER_ENABLE_GEEK_TOOLS=""
SERVER_ENABLE_INSECURE_TOOLS=""
SERVER_ENABLE_DOCKER=""
SERVER_ENABLE_LIBVIRTD=""
# 自动生成的 Home Manager 用户模板
CREATED_HOME_USERS=()
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
# 是否在 nixos-rebuild 时升级上游依赖（默认关闭，保证可复现）
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
# 1) 检查环境 2) 选择主机/用户 3) 准备源代码
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

# 清空管理员用户临时配置。
reset_admin_users() {
  TARGET_ADMIN_USERS=()
}

# 清空服务器软件覆盖配置。
reset_server_overrides() {
  SERVER_OVERRIDES_ENABLED=false
  SERVER_ENABLE_DEV=""
  SERVER_ENABLE_NETWORK_GUI=""
  SERVER_ENABLE_BROWSERS_AND_MEDIA=""
  SERVER_ENABLE_GEEK_TOOLS=""
  SERVER_ENABLE_INSECURE_TOOLS=""
  SERVER_ENABLE_DOCKER=""
  SERVER_ENABLE_LIBVIRTD=""
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

detect_bus_ids_from_lspci() {
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
    fi
  done < <(lspci -D -d ::03xx 2>/dev/null || true)
}

detect_bus_id_from_lspci() {
  local vendor="$1"
  local first=""
  first="$(detect_bus_ids_from_lspci "${vendor}" | head -n1 || true)"
  if [[ -n "${first}" ]]; then
    printf '%s' "${first}"
  fi
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

bus_candidates_for_vendor() {
  local vendor="$1"
  local -A seen=()
  local result=()
  local value=""

  while IFS= read -r value; do
    [[ -n "${value}" ]] || continue
    if [[ -z "${seen[${value}]+x}" ]]; then
      result+=("${value}")
      seen["${value}"]=1
    fi
  done < <(detect_bus_ids_from_lspci "${vendor}" || true)

  local fallback=""
  fallback="$(resolve_bus_id_default "${vendor}" || true)"
  if [[ -n "${fallback}" && -z "${seen[${fallback}]+x}" ]]; then
    result=("${fallback}" "${result[@]}")
  fi

  if [[ ${#result[@]} -gt 0 ]]; then
    printf '%s\n' "${result[@]}"
  fi
}

# 设置部署模式，并同步源策略。
set_deploy_mode() {
  local mode="$1"
  case "${mode}" in
    manage-users|users)
      DEPLOY_MODE="manage-users"
      FORCE_REMOTE_SOURCE=false
      ;;
    update-existing|update)
      DEPLOY_MODE="update-existing"
      FORCE_REMOTE_SOURCE=true
      ;;
    *)
      error "不支持的部署模式：${mode}"
      ;;
  esac
  DEPLOY_MODE_SET=true
}

# 交互式选择部署模式。
prompt_deploy_mode() {
  if [[ "${DEPLOY_MODE_SET}" == "true" || ! -t 0 || ! -t 1 ]]; then
    return 0
  fi
  local pick
  pick="$(menu_prompt "选择部署模式" 1 "新增/调整用户并部署（可修改用户/权限）" "仅更新当前配置（网络仓库最新，不改用户/权限）")"
  case "${pick}" in
    1)
      set_deploy_mode "manage-users"
      ;;
    2)
      set_deploy_mode "update-existing"
      ;;
  esac
}

# 交互式选择覆盖策略（替代命令行参数）。
prompt_overwrite_mode() {
  if [[ "${OVERWRITE_MODE_SET}" == "true" ]]; then
    return 0
  fi
  if ! is_tty; then
    OVERWRITE_MODE="backup"
    OVERWRITE_MODE_SET=true
    return 0
  fi
  local pick
  pick="$(menu_prompt "选择覆盖策略（/etc/nixos 已存在时）" 1 "先备份再覆盖（推荐）" "直接覆盖（不备份）" "执行时再询问")"
  case "${pick}" in
    1) OVERWRITE_MODE="backup" ;;
    2) OVERWRITE_MODE="overwrite" ;;
    3) OVERWRITE_MODE="ask" ;;
  esac
  OVERWRITE_MODE_SET=true
}

# 交互式选择是否在重建时升级上游依赖。
prompt_rebuild_upgrade() {
  if ! is_tty; then
    REBUILD_UPGRADE=false
    return 0
  fi
  REBUILD_UPGRADE="$(ask_bool "重建时升级上游依赖？" "false")"
}

# 交互式选择源代码来源与版本策略。
prompt_source_strategy() {
  if [[ "${SOURCE_CHOICE_SET}" == "true" ]]; then
    return 0
  fi

  local local_repo=""
  local_repo="$(detect_local_repo_dir || true)"

  if ! is_tty; then
    if [[ "${DEPLOY_MODE}" == "update-existing" ]]; then
      FORCE_REMOTE_SOURCE=true
      ALLOW_REMOTE_HEAD=true
      SOURCE_REF=""
    else
      if [[ -n "${local_repo}" ]]; then
        FORCE_REMOTE_SOURCE=false
        ALLOW_REMOTE_HEAD=false
        SOURCE_REF=""
      else
        FORCE_REMOTE_SOURCE=true
        ALLOW_REMOTE_HEAD=false
      fi
    fi
    SOURCE_CHOICE_SET=true
    return 0
  fi

  local options=()
  local default_index=1
  if [[ -n "${local_repo}" ]]; then
    options+=("使用本地仓库（推荐）: ${local_repo}")
  fi
  options+=("使用网络仓库固定版本（输入 commit/tag）")
  options+=("使用网络仓库最新版本（HEAD）")

  if [[ "${DEPLOY_MODE}" == "update-existing" ]]; then
    default_index=${#options[@]}
  fi

  local pick
  pick="$(menu_prompt "选择配置来源" "${default_index}" "${options[@]}")"

  if [[ -n "${local_repo}" && "${pick}" == "1" ]]; then
    FORCE_REMOTE_SOURCE=false
    ALLOW_REMOTE_HEAD=false
    SOURCE_REF=""
  else
    local remote_pick="${pick}"
    if [[ -n "${local_repo}" ]]; then
      remote_pick=$((pick - 1))
    fi
    case "${remote_pick}" in
      1)
        FORCE_REMOTE_SOURCE=true
        ALLOW_REMOTE_HEAD=false
        while true; do
          read -r -p "请输入远端固定版本（commit/tag）： " SOURCE_REF
          if [[ -n "${SOURCE_REF}" ]]; then
            break
          fi
          echo "版本不能为空，请重试。"
        done
        ;;
      2)
        FORCE_REMOTE_SOURCE=true
        ALLOW_REMOTE_HEAD=true
        SOURCE_REF=""
        ;;
    esac
  fi

  SOURCE_CHOICE_SET=true
}

# 校验部署模式与运行时状态是否冲突。
validate_mode_conflicts() {
  if [[ "${DEPLOY_MODE}" == "update-existing" ]]; then
    if [[ ${#TARGET_USERS[@]} -gt 0 ]]; then
      error "仅更新模式不允许修改用户列表；该模式会保留现有用户与权限。"
    fi
  fi
}

# 显示使用帮助。
usage() {
  cat <<EOF_USAGE
用法:
  ./run.sh

说明:
  本脚本为全交互式向导，不需要任何命令行参数。
  所有配置项（部署模式、来源、覆盖策略、用户、权限、GPU、TUN 等）
  均在执行过程中通过菜单选择。
EOF_USAGE
}

# 解析命令行参数。
parse_args() {
  if [[ $# -eq 0 ]]; then
    return 0
  fi
  if [[ $# -eq 1 && ( "$1" == "-h" || "$1" == "--help" ) ]]; then
    usage
    exit 0
  fi
  usage
  error "此脚本已改为全交互模式，请直接运行 ./run.sh（不需要参数）。"
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

should_require_hardware_config() {
  # rootless + build 仅做构建/评估，不强制要求目标目录存在硬件文件
  if [[ "${ROOTLESS}" == "true" && "${MODE}" == "build" ]]; then
    return 1
  fi
  return 0
}

# 选定主机后检查硬件配置是否存在。
ensure_host_hardware_config() {
  if ! should_require_hardware_config; then
    return 0
  fi
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

  # 仅在可切换系统场景强制要求硬件配置；rootless+build 可依赖 host fallback 做评估/构建
  if should_require_hardware_config; then
    if ! has_any_hardware_config "${ETC_DIR}"; then
      error "缺少硬件配置：${ETC_DIR}/hardware-configuration.nix 或 ${ETC_DIR}/hosts/<hostname>/hardware-configuration.nix；请先运行 nixos-generate-config。"
    fi
  else
    note "rootless + build 模式：跳过硬件配置强制检查（仅构建/评估）。"
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

  if [[ ${#hosts[@]} -gt 0 ]]; then
    printf '%s\n' "${hosts[@]}"
  fi
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

# 检测主机 profile 类型（server/desktop/unknown）。
detect_host_profile_kind() {
  local repo_dir="$1"
  local host_file="${repo_dir}/hosts/${TARGET_NAME}/default.nix"
  HOST_PROFILE_KIND="unknown"
  if [[ ! -f "${host_file}" ]]; then
    return 0
  fi
  if grep -qE '\.\./profiles/server\.nix' "${host_file}"; then
    HOST_PROFILE_KIND="server"
  elif grep -qE '\.\./profiles/desktop\.nix' "${host_file}"; then
    HOST_PROFILE_KIND="desktop"
  fi
}

# 询问布尔开关（返回 true/false）。
ask_bool() {
  local prompt="$1"
  local default="${2:-false}"
  if ! is_tty; then
    printf '%s' "${default}"
    return 0
  fi

  local default_index=2
  if [[ "${default}" == "true" ]]; then
    default_index=1
  fi
  local pick
  pick="$(menu_prompt "${prompt}" "${default_index}" "是 (true)" "否 (false)")"
  case "${pick}" in
    1) printf '%s' "true" ;;
    2) printf '%s' "false" ;;
    *) printf '%s' "${default}" ;;
  esac
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

# 列出仓库中已存在的 Home Manager 用户目录。
list_existing_home_users() {
  local repo_dir="$1"
  local users_dir="${repo_dir}/home/users"
  local users=()
  if [[ -d "${users_dir}" ]]; then
    local entry=""
    for entry in "${users_dir}"/*; do
      [[ -d "${entry}" ]] || continue
      local name
      name="$(basename "${entry}")"
      if [[ "${name}" =~ ^[a-z_][a-z0-9_-]*$ ]]; then
        users+=("${name}")
      fi
    done
  fi
  if [[ ${#users[@]} -gt 0 ]]; then
    printf '%s\n' "${users[@]}"
  fi
}

# 交互式配置每用户 TUN。
configure_per_user_tun() {
  if [[ "${PER_USER_TUN_ENABLED}" != "true" ]]; then
    return 0
  fi

  if is_tty; then
    # 让用户选择配置方式
    local pick
    pick="$(menu_prompt "TUN 配置方式" 1 "沿用主机配置" "使用默认接口/端口 (tun0/tun1 + 1053..)" "使用常见接口名 (Meta/Mihomo/clash0)" "返回")"
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
        local idx=0
        local user
        local common_ifaces=("Meta" "Mihomo" "clash0" "tun0" "tun1" "tun2")
        for user in "${TARGET_USERS[@]}"; do
          local iface="tun${idx}"
          if (( idx < ${#common_ifaces[@]} )); then
            iface="${common_ifaces[$idx]}"
          fi
          USER_TUN["${user}"]="${iface}"
          USER_DNS["${user}"]=$((1053 + idx))
          idx=$((idx + 1))
        done
        return 0
        ;;
    esac
  else
    reset_tun_maps
    return 0
  fi
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

    # iGPU busId（从检测结果中选择）
    if [[ "${GPU_IGPU_VENDOR}" == "intel" ]]; then
      local intel_candidates=()
      mapfile -t intel_candidates < <(bus_candidates_for_vendor intel)
      if [[ ${#intel_candidates[@]} -eq 0 ]]; then
        pick="$(menu_prompt "未检测到 Intel iGPU busId" 1 "沿用主机配置" "返回")"
        case "${pick}" in
          1)
            reset_gpu_override
            return 0
            ;;
          2)
            WIZARD_ACTION="back"
            return 0
            ;;
        esac
      fi
      local intel_options=("${intel_candidates[@]}" "返回")
      pick="$(menu_prompt "选择 Intel iGPU busId" 1 "${intel_options[@]}")"
      if (( pick == ${#intel_options[@]} )); then
        WIZARD_ACTION="back"
        return 0
      fi
      GPU_INTEL_BUS="${intel_options[$((pick - 1))]}"
    else
      local amd_candidates=()
      mapfile -t amd_candidates < <(bus_candidates_for_vendor amd)
      if [[ ${#amd_candidates[@]} -eq 0 ]]; then
        pick="$(menu_prompt "未检测到 AMD iGPU busId" 1 "沿用主机配置" "返回")"
        case "${pick}" in
          1)
            reset_gpu_override
            return 0
            ;;
          2)
            WIZARD_ACTION="back"
            return 0
            ;;
        esac
      fi
      local amd_options=("${amd_candidates[@]}" "返回")
      pick="$(menu_prompt "选择 AMD iGPU busId" 1 "${amd_options[@]}")"
      if (( pick == ${#amd_options[@]} )); then
        WIZARD_ACTION="back"
        return 0
      fi
      GPU_AMD_BUS="${amd_options[$((pick - 1))]}"
    fi

    # NVIDIA busId（从检测结果中选择）
    local nvidia_candidates=()
    mapfile -t nvidia_candidates < <(bus_candidates_for_vendor nvidia)
    if [[ ${#nvidia_candidates[@]} -eq 0 ]]; then
      pick="$(menu_prompt "未检测到 NVIDIA dGPU busId" 1 "沿用主机配置" "返回")"
      case "${pick}" in
        1)
          reset_gpu_override
          return 0
          ;;
        2)
          WIZARD_ACTION="back"
          return 0
          ;;
      esac
    fi
    local nvidia_options=("${nvidia_candidates[@]}" "返回")
    pick="$(menu_prompt "选择 NVIDIA dGPU busId" 1 "${nvidia_options[@]}")"
    if (( pick == ${#nvidia_options[@]} )); then
      WIZARD_ACTION="back"
      return 0
    fi
    GPU_NVIDIA_BUS="${nvidia_options[$((pick - 1))]}"
  fi

  if [[ "${GPU_MODE}" == "hybrid" || "${GPU_MODE}" == "dgpu" ]]; then
    pick="$(menu_prompt "NVIDIA 使用开源内核模块？" 1 "是（open=true）" "否（open=false）" "返回")"
    case "${pick}" in
      1) GPU_NVIDIA_OPEN="true" ;;
      2) GPU_NVIDIA_OPEN="false" ;;
      3)
        WIZARD_ACTION="back"
        return 0
        ;;
    esac
  fi

  if [[ "${GPU_MODE}" == "hybrid" ]]; then
    pick="$(menu_prompt "生成 GPU specialisation（igpu/hybrid/dgpu）以便切换？" 1 "是" "否" "返回")"
    case "${pick}" in
      1) GPU_SPECIALISATIONS_ENABLED=true ;;
      2) GPU_SPECIALISATIONS_ENABLED=false ;;
      3)
        WIZARD_ACTION="back"
        return 0
        ;;
    esac
    if [[ "${GPU_SPECIALISATIONS_ENABLED}" == "true" ]]; then
      GPU_SPECIALISATION_MODES=("igpu" "hybrid" "dgpu")
    fi
  fi
}

# 判断用户是否在列表中。
user_in_list() {
  local needle="$1"
  shift
  local item
  for item in "$@"; do
    if [[ "${item}" == "${needle}" ]]; then
      return 0
    fi
  done
  return 1
}

# 向 TARGET_USERS 添加用户（去重）。
add_target_user() {
  local user="$1"
  if ! user_in_list "${user}" "${TARGET_USERS[@]}"; then
    TARGET_USERS+=("${user}")
  fi
}

# 从 TARGET_USERS 移除用户。
remove_target_user() {
  local user="$1"
  local kept=()
  local item
  for item in "${TARGET_USERS[@]}"; do
    if [[ "${item}" != "${user}" ]]; then
      kept+=("${item}")
    fi
  done
  TARGET_USERS=("${kept[@]}")
}

# 切换 TARGET_USERS 中用户选中状态。
toggle_target_user() {
  local user="$1"
  if user_in_list "${user}" "${TARGET_USERS[@]}"; then
    remove_target_user "${user}"
  else
    add_target_user "${user}"
  fi
}

# 向 TARGET_ADMIN_USERS 添加用户（去重）。
add_admin_user() {
  local user="$1"
  if ! user_in_list "${user}" "${TARGET_ADMIN_USERS[@]}"; then
    TARGET_ADMIN_USERS+=("${user}")
  fi
}

# 从 TARGET_ADMIN_USERS 移除用户。
remove_admin_user() {
  local user="$1"
  local kept=()
  local item
  for item in "${TARGET_ADMIN_USERS[@]}"; do
    if [[ "${item}" != "${user}" ]]; then
      kept+=("${item}")
    fi
  done
  TARGET_ADMIN_USERS=("${kept[@]}")
}

# 切换 TARGET_ADMIN_USERS 中用户选中状态。
toggle_admin_user() {
  local user="$1"
  if user_in_list "${user}" "${TARGET_ADMIN_USERS[@]}"; then
    remove_admin_user "${user}"
  else
    add_admin_user "${user}"
  fi
}

# 从已存在用户中勾选目标用户。
select_existing_users_menu() {
  local users=("$@")
  local pick
  while true; do
    local options=()
    local user=""
    for user in "${users[@]}"; do
      if user_in_list "${user}" "${TARGET_USERS[@]}"; then
        options+=("[x] ${user}")
      else
        options+=("[ ] ${user}")
      fi
    done
    options+=("完成")
    options+=("返回")

    pick="$(menu_prompt "勾选已有用户（可重复切换）" 1 "${options[@]}")"
    if (( pick >= 1 && pick <= ${#users[@]} )); then
      toggle_target_user "${users[$((pick - 1))]}"
      continue
    fi
    if (( pick == ${#users[@]} + 1 )); then
      return 0
    fi
    return 1
  done
}

# 从已选用户中勾选管理员。
select_admin_users_menu() {
  local pick
  while true; do
    local options=()
    local user=""
    for user in "${TARGET_USERS[@]}"; do
      if user_in_list "${user}" "${TARGET_ADMIN_USERS[@]}"; then
        options+=("[x] ${user}")
      else
        options+=("[ ] ${user}")
      fi
    done
    options+=("完成")
    options+=("返回")

    pick="$(menu_prompt "勾选管理员用户（可重复切换）" 1 "${options[@]}")"
    if (( pick >= 1 && pick <= ${#TARGET_USERS[@]} )); then
      toggle_admin_user "${TARGET_USERS[$((pick - 1))]}"
      continue
    fi
    if (( pick == ${#TARGET_USERS[@]} + 1 )); then
      return 0
    fi
    return 1
  done
}

# 交互式输入用户列表。
prompt_users() {
  local default_user=""
  default_user="$(resolve_default_user)"

  if ! is_tty; then
    if [[ ${#TARGET_USERS[@]} -eq 0 ]]; then
      TARGET_USERS=("${default_user}")
    fi
    return 0
  fi

  if [[ ${#TARGET_USERS[@]} -eq 0 ]]; then
    TARGET_USERS=("${default_user}")
  fi

  while true; do
    local current_users="未选择"
    if [[ ${#TARGET_USERS[@]} -gt 0 ]]; then
      current_users="${TARGET_USERS[*]}"
    fi

    local pick
    pick="$(menu_prompt "选择用户（当前：${current_users}）" 1 "仅使用默认用户 (${default_user})" "从已有 Home 用户中选择" "新增用户（手写用户名）" "清空已选用户" "完成" "返回" "退出")"
    case "${pick}" in
      1)
        TARGET_USERS=("${default_user}")
        ;;
      2)
        local existing_users=()
        mapfile -t existing_users < <(list_existing_home_users "${TMP_DIR}" | sort -u)
        if [[ ${#existing_users[@]} -eq 0 ]]; then
          warn "未发现可选的已有 Home 用户目录。"
          continue
        fi
        select_existing_users_menu "${existing_users[@]}" || true
        ;;
      3)
        local input=""
        read -r -p "输入新增用户名（留空取消）： " input
        if [[ -z "${input}" ]]; then
          continue
        fi
        if [[ ! "${input}" =~ ^[a-z_][a-z0-9_-]*$ ]]; then
          warn "用户名不合法：${input}"
          continue
        fi
        add_target_user "${input}"
        ;;
      4)
        TARGET_USERS=()
        ;;
      5)
        if [[ ${#TARGET_USERS[@]} -eq 0 ]]; then
          warn "请至少选择一个用户。"
          continue
        fi
        return 0
        ;;
      6)
        WIZARD_ACTION="back"
        return 0
        ;;
      7)
        error "已退出"
        ;;
    esac
  done
}

# 交互式输入管理员用户列表（wheel）。
prompt_admin_users() {
  local default_admin="${TARGET_USERS[0]}"
  if [[ ${#TARGET_USERS[@]} -eq 0 ]]; then
    error "用户列表为空，无法选择管理员。"
  fi

  if ! is_tty; then
    if [[ ${#TARGET_ADMIN_USERS[@]} -eq 0 ]]; then
      TARGET_ADMIN_USERS=("${default_admin}")
    fi
    return 0
  fi

  if [[ ${#TARGET_ADMIN_USERS[@]} -eq 0 ]]; then
    TARGET_ADMIN_USERS=("${default_admin}")
  fi

  while true; do
    local current_admins="未选择"
    if [[ ${#TARGET_ADMIN_USERS[@]} -gt 0 ]]; then
      current_admins="${TARGET_ADMIN_USERS[*]}"
    fi

    local pick
    pick="$(menu_prompt "管理员权限（wheel，当前：${current_admins}）" 1 "仅主用户 (${default_admin})" "所有用户" "自定义勾选管理员" "清空管理员" "完成" "返回" "退出")"
    case "${pick}" in
      1)
        TARGET_ADMIN_USERS=("${default_admin}")
        ;;
      2)
        TARGET_ADMIN_USERS=("${TARGET_USERS[@]}")
        ;;
      3)
        select_admin_users_menu || true
        ;;
      4)
        TARGET_ADMIN_USERS=()
        ;;
      5)
        if [[ ${#TARGET_ADMIN_USERS[@]} -eq 0 ]]; then
          warn "至少需要一个管理员用户。"
          continue
        fi
        return 0
        ;;
      6)
        WIZARD_ACTION="back"
        return 0
        ;;
      7)
        error "已退出"
        ;;
    esac
  done
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

# 管理员列表去重并保持顺序。
dedupe_admin_users() {
  local user
  local -A seen=()
  local unique=()
  for user in "${TARGET_ADMIN_USERS[@]}"; do
    if [[ -z "${seen[${user}]+x}" ]]; then
      unique+=("${user}")
      seen["${user}"]=1
    fi
  done
  TARGET_ADMIN_USERS=("${unique[@]}")
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

# 校验管理员列表：格式合法且必须是用户子集。
validate_admin_users() {
  local user
  if [[ ${#TARGET_ADMIN_USERS[@]} -eq 0 ]]; then
    TARGET_ADMIN_USERS=("${TARGET_USERS[0]}")
  fi
  for user in "${TARGET_ADMIN_USERS[@]}"; do
    if [[ ! "${user}" =~ ^[a-z_][a-z0-9_-]*$ ]]; then
      error "管理员用户名不合法：${user}"
    fi
    if [[ ! " ${TARGET_USERS[*]} " =~ [[:space:]]${user}[[:space:]] ]]; then
      error "管理员用户必须包含在用户列表中：${user}"
    fi
  done
}

# 交互式配置服务器软件/虚拟化覆盖项。
configure_server_overrides() {
  if [[ "${HOST_PROFILE_KIND}" != "server" ]]; then
    reset_server_overrides
    return 0
  fi

  if ! is_tty; then
    reset_server_overrides
    return 0
  fi

  local pick
  pick="$(menu_prompt "服务器软件配置" 1 "沿用主机配置" "开发服务器预设（Dev + Geek + Docker）" "自定义开关" "返回")"
  case "${pick}" in
    1)
      reset_server_overrides
      return 0
      ;;
    2)
      SERVER_OVERRIDES_ENABLED=true
      SERVER_ENABLE_DEV="true"
      SERVER_ENABLE_NETWORK_GUI="false"
      SERVER_ENABLE_BROWSERS_AND_MEDIA="false"
      SERVER_ENABLE_GEEK_TOOLS="true"
      SERVER_ENABLE_INSECURE_TOOLS="false"
      SERVER_ENABLE_DOCKER="true"
      SERVER_ENABLE_LIBVIRTD="false"
      return 0
      ;;
    3)
      SERVER_OVERRIDES_ENABLED=true
      SERVER_ENABLE_DEV="$(ask_bool "启用开发工具（mcb.packages.enableDev）？" "false")"
      SERVER_ENABLE_NETWORK_GUI="$(ask_bool "启用网络图形工具（mcb.packages.enableNetworkGui）？" "false")"
      SERVER_ENABLE_BROWSERS_AND_MEDIA="$(ask_bool "启用浏览器/媒体应用（mcb.packages.enableBrowsersAndMedia）？" "false")"
      SERVER_ENABLE_GEEK_TOOLS="$(ask_bool "启用调试/诊断工具（mcb.packages.enableGeekTools）？" "false")"
      SERVER_ENABLE_INSECURE_TOOLS="$(ask_bool "启用不安全软件组（mcb.packages.enableInsecureTools）？" "false")"
      SERVER_ENABLE_DOCKER="$(ask_bool "启用 Docker（mcb.virtualisation.docker.enable）？" "false")"
      SERVER_ENABLE_LIBVIRTD="$(ask_bool "启用 Libvirt/KVM（mcb.virtualisation.libvirtd.enable）？" "false")"
      return 0
      ;;
    4)
      WIZARD_ACTION="back"
      return 0
      ;;
  esac
}

# 为缺失用户自动生成 Home Manager 最小入口模板。
ensure_user_home_entries() {
  local repo_dir="$1"
  local profile_import="../../profiles/full.nix"
  if [[ "${HOST_PROFILE_KIND}" == "server" ]]; then
    profile_import="../../profiles/minimal.nix"
  fi

  local user=""
  for user in "${TARGET_USERS[@]}"; do
    local user_dir="${repo_dir}/home/users/${user}"
    local user_file="${user_dir}/default.nix"
    if [[ -f "${user_file}" ]]; then
      continue
    fi

    mkdir -p "${user_dir}"
    cat > "${user_file}" <<EOF_USER
{ ... }:

let
  user = "${user}";
in
{
  imports = [
    ${profile_import}
  ];

  home.username = user;
  home.homeDirectory = "/home/\${user}";
  home.stateVersion = "25.11";

  programs.home-manager.enable = true;
  xdg.enable = true;
}
EOF_USER
    CREATED_HOME_USERS+=("${user}")
    warn "已为新用户自动生成 Home Manager 入口：home/users/${user}/default.nix"
  done
}

# 仅更新模式下保留当前主机 local.nix，避免覆盖现有用户/权限。
preserve_existing_local_override() {
  local repo_dir="$1"
  if [[ "${DEPLOY_MODE}" != "update-existing" ]]; then
    return 0
  fi
  if [[ -z "${TARGET_NAME}" ]]; then
    return 0
  fi
  local src="${ETC_DIR}/hosts/${TARGET_NAME}/local.nix"
  local dst="${repo_dir}/hosts/${TARGET_NAME}/local.nix"
  if [[ -f "${src}" ]]; then
    mkdir -p "$(dirname "${dst}")"
    if cp -a "${src}" "${dst}"; then
      note "仅更新模式：已保留现有 hosts/${TARGET_NAME}/local.nix"
    else
      warn "仅更新模式：复制现有 local.nix 失败，将继续使用仓库版本。"
    fi
  else
    note "仅更新模式：未发现现有 hosts/${TARGET_NAME}/local.nix，将按仓库默认配置更新。"
  fi
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
  local admin_list=""
  local user
  if [[ ${#TARGET_ADMIN_USERS[@]} -eq 0 ]]; then
    TARGET_ADMIN_USERS=("${primary}")
  fi
  # 生成用户列表字符串
  for user in "${TARGET_USERS[@]}"; do
    list="${list} \"${user}\""
  done
  for user in "${TARGET_ADMIN_USERS[@]}"; do
    admin_list="${admin_list} \"${user}\""
  done

  {
    # local.nix 会覆盖 mcb.user/mcb.users 等配置
    echo "{ lib, ... }:"
    echo ""
    echo "{"
    echo "  mcb.user = lib.mkForce \"${primary}\";"
    echo "  mcb.users = lib.mkForce [${list} ];"
    echo "  mcb.adminUsers = lib.mkForce [${admin_list} ];"

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

    if [[ "${SERVER_OVERRIDES_ENABLED}" == "true" ]]; then
      echo "  mcb.packages.enableDev = lib.mkForce ${SERVER_ENABLE_DEV};"
      echo "  mcb.packages.enableNetworkGui = lib.mkForce ${SERVER_ENABLE_NETWORK_GUI};"
      echo "  mcb.packages.enableBrowsersAndMedia = lib.mkForce ${SERVER_ENABLE_BROWSERS_AND_MEDIA};"
      echo "  mcb.packages.enableGeekTools = lib.mkForce ${SERVER_ENABLE_GEEK_TOOLS};"
      echo "  mcb.packages.enableInsecureTools = lib.mkForce ${SERVER_ENABLE_INSECURE_TOOLS};"
      echo "  mcb.virtualisation.docker.enable = lib.mkForce ${SERVER_ENABLE_DOCKER};"
      echo "  mcb.virtualisation.libvirtd.enable = lib.mkForce ${SERVER_ENABLE_LIBVIRTD};"
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

# 检测本地仓库目录（优先当前目录，其次脚本所在目录）。
detect_local_repo_dir() {
  local script_dir=""
  script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  local candidates=(
    "$(pwd)"
    "${script_dir}"
  )
  local dir
  for dir in "${candidates[@]}"; do
    if [[ -f "${dir}/flake.nix" && -d "${dir}/hosts" && -d "${dir}/modules" && -d "${dir}/home" ]]; then
      printf '%s' "${dir}"
      return 0
    fi
  done
  return 1
}

# 未使用本地仓库时，要求固定远端来源版本（除非显式允许跟随远端 HEAD）。
require_remote_source_pin() {
  if [[ "${ALLOW_REMOTE_HEAD}" == "true" ]]; then
    warn "当前将跟随远端分支最新提交（存在供应链风险）。"
    return 0
  fi
  if [[ -z "${SOURCE_REF}" ]]; then
    error "未检测到本地仓库，且未选择远端固定版本；请在向导中选择固定版本或明确选择远端最新版本。"
  fi
}

# 使用本地仓库作为部署源，避免依赖远端浮动分支。
prepare_local_source() {
  local tmp_dir="$1"
  local source_dir="$2"
  log "使用本地仓库：${source_dir}"
  rm -rf "${tmp_dir}"
  mkdir -p "${tmp_dir}"
  if command -v rsync >/dev/null 2>&1; then
    rsync -a --exclude '.git/' "${source_dir}/" "${tmp_dir}/"
  else
    (cd "${source_dir}" && tar --exclude=.git -cf - .) | tar -C "${tmp_dir}" -xf -
  fi
  if git -C "${source_dir}" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    SOURCE_COMMIT="$(git -C "${source_dir}" rev-parse HEAD 2>/dev/null || true)"
  fi
  if [[ -n "${SOURCE_COMMIT}" ]]; then
    note "本地源提交：${SOURCE_COMMIT}"
  fi
}

# 克隆配置仓库。
clone_repo() {
  local tmp_dir="$1"
  local url="$2"

  if [[ -n "${SOURCE_REF}" ]]; then
    log "拉取仓库：${url}（固定 ref: ${SOURCE_REF}）"
    # 固定 ref 模式：完整克隆后切到指定提交/标签，避免跟随浮动分支。
    if git clone "${url}" "${tmp_dir}" \
      && git -C "${tmp_dir}" checkout --detach "${SOURCE_REF}" >/dev/null 2>&1; then
      SOURCE_COMMIT="$(git -C "${tmp_dir}" rev-parse HEAD 2>/dev/null || true)"
      success "仓库拉取完成（${SOURCE_COMMIT}）"
      return 0
    fi
    warn "仓库拉取或 checkout 失败：${url}（ref: ${SOURCE_REF}）"
    return 1
  fi

  log "拉取仓库：${url}（${BRANCH}）"
  # 仅在显式允许时使用远端分支 HEAD。
  if git clone --depth 1 --branch "${BRANCH}" "${url}" "${tmp_dir}"; then
    SOURCE_COMMIT="$(git -C "${tmp_dir}" rev-parse HEAD 2>/dev/null || true)"
    success "仓库拉取完成（${SOURCE_COMMIT}）"
    return 0
  fi
  warn "仓库拉取失败：${url}"
  return 1
}

# 尝试多个镜像地址克隆。
clone_repo_any() {
  local tmp_dir="$1"
  local url
  SOURCE_COMMIT=""
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
  if [[ "${DEPLOY_MODE}" == "update-existing" ]]; then
    printf '%s部署模式：%s%s\n' "${COLOR_BOLD}" "仅更新当前配置（保留用户/权限）" "${COLOR_RESET}"
  else
    printf '%s部署模式：%s%s\n' "${COLOR_BOLD}" "新增/调整用户并部署" "${COLOR_RESET}"
  fi
  printf '%s主机：%s%s\n' "${COLOR_BOLD}" "${TARGET_NAME}" "${COLOR_RESET}"
  if [[ "${DEPLOY_MODE}" == "update-existing" ]]; then
    if [[ -n "${SOURCE_REF}" ]]; then
      printf '%s源策略：%s%s\n' "${COLOR_BOLD}" "网络仓库固定版本 (${SOURCE_REF})" "${COLOR_RESET}"
    else
      printf '%s源策略：%s%s\n' "${COLOR_BOLD}" "网络仓库最新 HEAD" "${COLOR_RESET}"
    fi
    printf '%s用户/权限：%s%s\n' "${COLOR_BOLD}" "保持当前主机 local.nix" "${COLOR_RESET}"
  else
    printf '%s用户：%s%s\n' "${COLOR_BOLD}" "${TARGET_USERS[*]}" "${COLOR_RESET}"
    printf '%s管理员：%s%s\n' "${COLOR_BOLD}" "${TARGET_ADMIN_USERS[*]}" "${COLOR_RESET}"
  fi
  if [[ -n "${SOURCE_COMMIT}" ]]; then
    printf '%s源提交：%s%s\n' "${COLOR_BOLD}" "${SOURCE_COMMIT}" "${COLOR_RESET}"
  fi
  printf '%s覆盖策略：%s%s\n' "${COLOR_BOLD}" "${OVERWRITE_MODE}" "${COLOR_RESET}"
  if [[ "${REBUILD_UPGRADE}" == "true" ]]; then
    printf '%s依赖升级：%s%s\n' "${COLOR_BOLD}" "启用" "${COLOR_RESET}"
  else
    printf '%s依赖升级：%s%s\n' "${COLOR_BOLD}" "关闭" "${COLOR_RESET}"
  fi
  if [[ "${DEPLOY_MODE}" == "update-existing" ]]; then
    return 0
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

  if [[ "${SERVER_OVERRIDES_ENABLED}" == "true" ]]; then
    printf '%s服务器软件覆盖：%s%s\n' "${COLOR_BOLD}" "已启用" "${COLOR_RESET}"
    printf '  - enableDev=%s\n' "${SERVER_ENABLE_DEV}"
    printf '  - enableNetworkGui=%s\n' "${SERVER_ENABLE_NETWORK_GUI}"
    printf '  - enableBrowsersAndMedia=%s\n' "${SERVER_ENABLE_BROWSERS_AND_MEDIA}"
    printf '  - enableGeekTools=%s\n' "${SERVER_ENABLE_GEEK_TOOLS}"
    printf '  - enableInsecureTools=%s\n' "${SERVER_ENABLE_INSECURE_TOOLS}"
    printf '  - docker.enable=%s\n' "${SERVER_ENABLE_DOCKER}"
    printf '  - libvirtd.enable=%s\n' "${SERVER_ENABLE_LIBVIRTD}"
  fi
}

# 交互式向导主流程。
wizard_flow() {
  local step=1
  WIZARD_ACTION=""

  if [[ "${DEPLOY_MODE}" == "update-existing" ]]; then
    while true; do
      case "${step}" in
        1)
          select_host "${TMP_DIR}"
          validate_host "${TMP_DIR}"
          detect_host_profile_kind "${TMP_DIR}"
          step=2
          ;;
        2)
          print_summary
          if is_tty; then
            wizard_back_or_quit "确认仅更新当前配置并继续？"
            case "${WIZARD_ACTION}" in
              back)
                TARGET_NAME=""
                step=1
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
  fi

  while true; do
    case "${step}" in
      1)
        # 选择主机
        select_host "${TMP_DIR}"
        validate_host "${TMP_DIR}"
        detect_host_profile_kind "${TMP_DIR}"
        step=2
        ;;
      2)
        # 选择用户列表
        WIZARD_ACTION=""
        prompt_users
        if [[ "${WIZARD_ACTION}" == "back" ]]; then
          TARGET_USERS=()
          reset_admin_users
          reset_tun_maps
          reset_gpu_override
          reset_server_overrides
          TARGET_NAME=""
          step=1
          continue
        fi
        dedupe_users
        validate_users
        reset_admin_users
        reset_tun_maps
        reset_gpu_override
        reset_server_overrides
        step=3
        ;;
      3)
        # 选择管理员用户（wheel）
        WIZARD_ACTION=""
        prompt_admin_users
        if [[ "${WIZARD_ACTION}" == "back" ]]; then
          reset_admin_users
          step=2
          continue
        fi
        dedupe_admin_users
        validate_admin_users
        step=4
        ;;
      4)
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
            step=3
            continue
          fi
        else
          reset_tun_maps
        fi
        step=5
        ;;
      5)
        # 配置 GPU 覆盖（可选，server 主机默认跳过）
        if [[ "${HOST_PROFILE_KIND}" == "server" ]]; then
          reset_gpu_override
          step=6
          continue
        fi
        WIZARD_ACTION=""
        configure_gpu
        if [[ "${WIZARD_ACTION}" == "back" ]]; then
          reset_gpu_override
          step=4
          continue
        fi
        step=6
        ;;
      6)
        # 服务器软件/虚拟化配置（仅 server profile）
        if [[ "${HOST_PROFILE_KIND}" != "server" ]]; then
          reset_server_overrides
          step=7
          continue
        fi
        WIZARD_ACTION=""
        configure_server_overrides
        if [[ "${WIZARD_ACTION}" == "back" ]]; then
          reset_server_overrides
          step=5
          continue
        fi
        step=7
        ;;
      7)
        # 最终确认
        print_summary
        if is_tty; then
          wizard_back_or_quit "确认以上配置"
          case "${WIZARD_ACTION}" in
            back)
              if [[ "${HOST_PROFILE_KIND}" == "server" ]]; then
                step=6
              else
                step=5
              fi
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
  prompt_deploy_mode
  validate_mode_conflicts
  prompt_overwrite_mode
  prompt_rebuild_upgrade
  prompt_source_strategy

  if [[ -n "${SOURCE_REF}" && "${ALLOW_REMOTE_HEAD}" == "true" ]]; then
    warn "检测到来源策略冲突，将优先使用固定版本。"
    ALLOW_REMOTE_HEAD=false
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

  # 按模式准备源代码：默认优先本地；仅更新模式强制走远端。
  section "准备源代码"
  if [[ "${FORCE_REMOTE_SOURCE}" == "true" ]]; then
    require_remote_source_pin
    if ! clone_repo_any "${TMP_DIR}"; then
      log "尝试临时切换阿里云 DNS 后重试"
      temp_dns_enable
      rm -rf "${TMP_DIR}"
      TMP_DIR="$(mktemp -d)"
      if ! clone_repo_any "${TMP_DIR}"; then
        error "仓库拉取失败，请检查网络"
      fi
    fi
  else
    local source_dir=""
    source_dir="$(detect_local_repo_dir || true)"
    if [[ -n "${source_dir}" ]]; then
      prepare_local_source "${TMP_DIR}" "${source_dir}"
    else
      require_remote_source_pin
      if ! clone_repo_any "${TMP_DIR}"; then
        log "尝试临时切换阿里云 DNS 后重试"
        temp_dns_enable
        rm -rf "${TMP_DIR}"
        TMP_DIR="$(mktemp -d)"
        if ! clone_repo_any "${TMP_DIR}"; then
          error "仓库拉取失败，请检查网络"
        fi
      fi
    fi
  fi
  progress_step "准备源代码"

  section "脚本自检"
  self_check_scripts "${TMP_DIR}"
  progress_step "脚本自检"

  # 交互式向导：选择主机/用户/TUN
  wizard_flow
  if [[ "${DEPLOY_MODE}" == "update-existing" ]]; then
    preserve_existing_local_override "${TMP_DIR}"
  else
    ensure_user_home_entries "${TMP_DIR}"
    if [[ ${#CREATED_HOME_USERS[@]} -gt 0 ]]; then
      warn "已自动创建用户 Home Manager 模板：${CREATED_HOME_USERS[*]}"
    fi
    write_local_override "${TMP_DIR}"
  fi
  ensure_host_hardware_config
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
