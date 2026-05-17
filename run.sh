#!/usr/bin/env bash
# NixOS 配置管理入口（子命令架构）。
#
# 用法:
#   run.sh                        # 默认 = deploy 向导
#   run.sh deploy [选项]          # 交互式/声明式部署
#   run.sh add-user <name> [选项] # 增量添加用户
#   run.sh switch [选项]          # 快速重建（日常最高频）
#   run.sh update [选项]          # flake update + 可选重建
#   run.sh check [选项]           # flake check
#   run.sh clean [选项]           # 清理旧备份
#   run.sh release                # 发布新版本
#   run.sh --help                 # 显示此帮助

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
SCRIPT_LIB_DIR="${SCRIPT_DIR}/scripts/run/lib"

# 读取仓库版本号
VERSION_FILE="${SCRIPT_DIR}/VERSION"
if [[ -r "${VERSION_FILE}" ]]; then
  version_from_file="$(tr -d '[:space:]' < "${VERSION_FILE}")"
  if [[ -n "${version_from_file}" ]]; then
    RUN_SH_VERSION="${version_from_file}"
  fi
fi

# ── 加载共享状态 ──
# shellcheck source=/dev/null
source "${SCRIPT_LIB_DIR}/vars.sh"
init_colors

# ── 加载库文件 ──
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

# ── 加载命令模块 ──
# shellcheck source=/dev/null
source "${SCRIPT_DIR}/scripts/run/cmd/deploy.sh"
# shellcheck source=/dev/null
source "${SCRIPT_DIR}/scripts/run/cmd/add-user.sh"
# shellcheck source=/dev/null
source "${SCRIPT_DIR}/scripts/run/cmd/switch.sh"
# shellcheck source=/dev/null
source "${SCRIPT_DIR}/scripts/run/cmd/update.sh"
# shellcheck source=/dev/null
source "${SCRIPT_DIR}/scripts/run/cmd/check.sh"
# shellcheck source=/dev/null
source "${SCRIPT_DIR}/scripts/run/cmd/clean.sh"
# shellcheck source=/dev/null
source "${SCRIPT_DIR}/scripts/run/cmd/release.sh"

# ── 全局帮助 ──
global_usage() {
  cat <<'EOF'
NixOS 配置管理入口

用法:
  run.sh [命令] [选项]

命令（选择你要做什么）:
  deploy      交互式或声明式部署向导（默认命令）
  add-user    增量添加用户到已有系统
  switch      快速重建系统 ── 改了配置就跑这个
  update      更新 flake 输入，可选重建
  check       运行 flake check 检查
  clean       清理 /etc/nixos 旧备份
  release     发布新版本到 GitHub

全局选项:
  --help, -h  显示此帮助

示例:
  run.sh                              # 启动部署向导
  run.sh deploy                       # 同上（显式）
  run.sh add-user alice --admin       # 添加管理员用户
  run.sh add-user bob --copy-from admin
  run.sh switch                       # 快速重建当前主机
  run.sh switch --host desktop --test # 指定主机试跑
  run.sh update --rebuild             # 更新依赖并重建
  run.sh check                        # flake check
  run.sh clean --keep 5               # 保留 5 个最新备份
  run.sh release                      # 发布

详细帮助:
  run.sh <命令> --help                # 查看命令的详细选项
EOF
}

# ── 设置部署模式 ──
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

# ── 主入口：子命令路由 ──
main() {
  local cmd="${1:-}"

  # 无参数 / --help / -h → 全局帮助或默认 deploy
  if [[ $# -eq 0 ]]; then
    cmd="deploy"
  elif [[ "$1" == "--help" || "$1" == "-h" ]]; then
    global_usage
    exit 0
  fi

  shift || true

  case "${cmd}" in
    deploy)
      deploy_flow "$@"
      ;;
    add-user)
      add_user_flow "$@"
      ;;
    switch)
      switch_flow "$@"
      ;;
    update)
      update_flow "$@"
      ;;
    check)
      check_flow "$@"
      ;;
    clean)
      clean_flow "$@"
      ;;
    release|--release)
      release_flow "$@"
      ;;
    *)
      warn "未知命令：${cmd}"
      global_usage
      exit 2
      ;;
  esac
}

main "$@"
