#!/usr/bin/env bash
# 一键部署 NixOS 配置（优先本地仓库，必要时再从 GitHub/Gitee 拉取）。

set -euo pipefail

# 版本号（优先读取仓库 VERSION 文件）。
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
VERSION_FILE="${SCRIPT_DIR}/VERSION"
RUN_SH_VERSION="v2026.03.20"
if [[ -r "${VERSION_FILE}" ]]; then
  version_from_file="$(tr -d '[:space:]' < "${VERSION_FILE}")"
  if [[ -n "${version_from_file}" ]]; then
    RUN_SH_VERSION="${version_from_file}"
  fi
fi

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
# 远端拉取超时（秒），用于避免某镜像无响应导致“卡住”。
GIT_CLONE_TIMEOUT_SEC="${GIT_CLONE_TIMEOUT_SEC:-90}"

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
SERVER_ENABLE_NETWORK_CLI=""
SERVER_ENABLE_NETWORK_GUI=""
SERVER_ENABLE_SHELL_TOOLS=""
SERVER_ENABLE_WAYLAND_TOOLS=""
SERVER_ENABLE_SYSTEM_TOOLS=""
SERVER_ENABLE_GEEK_TOOLS=""
SERVER_ENABLE_GAMING=""
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
GPU_SPECIALISATIONS_SET=false
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
# 运行模式：deploy | release
RUN_ACTION="deploy"

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

# 加载 run.sh 分层库（UI/交互 + 状态工具）。
SCRIPT_LIB_DIR="${SCRIPT_DIR}/scripts/run/lib"
# shellcheck source=/dev/null
source "${SCRIPT_LIB_DIR}/ui.sh"
# shellcheck source=/dev/null
source "${SCRIPT_LIB_DIR}/state.sh"
# shellcheck source=/dev/null
source "${SCRIPT_LIB_DIR}/env.sh"
# shellcheck source=/dev/null
source "${SCRIPT_LIB_DIR}/targets.sh"
# shellcheck source=/dev/null
source "${SCRIPT_LIB_DIR}/pipeline.sh"
# shellcheck source=/dev/null
source "${SCRIPT_LIB_DIR}/wizard.sh"
# shellcheck source=/dev/null
source "${SCRIPT_DIR}/scripts/run/cmd/release.sh"
# shellcheck source=/dev/null
source "${SCRIPT_DIR}/scripts/run/cmd/deploy.sh"

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
  ./run.sh release

说明:
  默认模式为全交互部署向导，不需要任何命令行参数。
  所有配置项（部署模式、来源、覆盖策略、用户、权限、GPU、TUN 等）
  均在执行过程中通过菜单选择。

  release 模式用于发布新版本：更新 VERSION、创建 tag，并发布 GitHub Release。
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
  if [[ $# -eq 1 && ( "$1" == "release" || "$1" == "--release" ) ]]; then
    RUN_ACTION="release"
    return 0
  fi
  usage
  error "此脚本已改为全交互模式，请直接运行 ./run.sh（不需要参数）。"
}

# 脚本主入口。
main() {
  parse_args "$@"
  case "${RUN_ACTION}" in
    release)
      release_flow
      ;;
    deploy)
      deploy_flow
      ;;
    *)
      error "未知运行模式：${RUN_ACTION}"
      ;;
  esac
}

main "$@"
