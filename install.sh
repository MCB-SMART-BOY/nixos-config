#!/usr/bin/env bash
# NixOS 一键部署脚本 (优化版)
# 功能：部署 dotfiles 并重建系统

set -e # 遇到错误立即停止

# --- 变量定义 ---
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DOTFILES_DIR="${SCRIPT_DIR}/dotfiles"
NIXOS_DIR="/etc/nixos"
CONFIG_DIR="${HOME}/.config"
BACKUP_DATE="$(date +%Y%m%d-%H%M%S)"
BACKUP_DIR="${HOME}/.config-backup-${BACKUP_DATE}"

# 颜色
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# --- 辅助函数 ---
log() { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[OK]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() {
  echo -e "${RED}[ERROR]${NC} $1"
  exit 1
}

# --- 1. 环境检查 ---
check_env() {
  log "正在检查环境..."

  # 确保不是 root 用户直接运行（某些步骤需要非 root 权限）
  if [[ "$(whoami)" == "root" ]]; then
    error "请使用普通用户运行此脚本 (脚本内部会按需请求 sudo 密码)"
  fi

  # 检查 dotfiles 目录是否存在
  if [[ ! -d "$DOTFILES_DIR" ]]; then
    error "未找到 dotfiles 目录！请确保脚本位于正确的文件结构中。\n当前路径: $SCRIPT_DIR"
  fi

  # 检查 configuration.nix 是否存在
  if [[ ! -f "${SCRIPT_DIR}/configuration.nix" ]]; then
    error "当前目录下缺少 configuration.nix 文件"
  fi
}

# --- 2. 部署系统配置 ---
deploy_system() {
  log "开始部署 NixOS 系统配置..."

  # 备份原配置
  if [[ -f "${NIXOS_DIR}/configuration.nix" ]]; then
    log "备份系统配置到 ${NIXOS_DIR}/configuration.nix.${BACKUP_DATE}.bak"
    sudo cp "${NIXOS_DIR}/configuration.nix" "${NIXOS_DIR}/configuration.nix.${BACKUP_DATE}.bak"
  fi

  # 复制新配置 (使用 sudo)
  sudo cp "${SCRIPT_DIR}/configuration.nix" "${NIXOS_DIR}/"

  # ⚠️ 重要：保留系统自动生成的 hardware-configuration.nix
  if [[ ! -f "${NIXOS_DIR}/hardware-configuration.nix" ]]; then
    warn "未找到 hardware-configuration.nix，正在生成..."
    sudo nixos-generate-config --root /
  else
    success "保留现有的 hardware-configuration.nix (不覆盖)"
  fi

  success "系统配置部署完成"
}

# --- 3. 部署用户配置 (软链接模式) ---
deploy_user() {
  log "开始部署用户配置 (Dotfiles)..."
  mkdir -p "$BACKUP_DIR"

  # 需要处理的应用列表 (对应 dotfiles 文件夹下的名字)
  apps=("helix" "niri" "waybar" "alacritty" "fuzzel" "starship" "zsh" "mako" "swaylock" "gtk3.0" "gtk4.0")

  for app in "${apps[@]}"; do
    # 源路径
    if [[ "$app" == "starship" ]]; then
      source_path="${DOTFILES_DIR}/starship/starship.toml"
      target_path="${CONFIG_DIR}/starship.toml"
    elif [[ "$app" == "zsh" ]]; then
      source_path="${DOTFILES_DIR}/zsh/.zshrc"
      target_path="${HOME}/.zshrc"
    else
      source_path="${DOTFILES_DIR}/${app}"
      target_path="${CONFIG_DIR}/${app}"
    fi

    # 检查源文件是否存在
    if [[ ! -e "$source_path" ]]; then
      warn "跳过 $app: 源文件不存在 ($source_path)"
      continue
    fi

    # 备份现有配置
    if [[ -e "$target_path" || -L "$target_path" ]]; then
      mv "$target_path" "$BACKUP_DIR/"
      log "已备份旧 $app 配置"
    fi

    # 创建父目录 (针对 starship 这种单文件的情况)
    mkdir -p "$(dirname "$target_path")"

    # 创建软链接
    ln -sf "$source_path" "$target_path"
    success "已链接: $app"
  done
}

# --- 4. 杂项设置 ---
setup_misc() {
  log "创建必要目录..."
  mkdir -p "${HOME}/Pictures/Screenshots"
  mkdir -p "${HOME}/Projects"

  # 如果安装了 Rust，初始化环境
  if command -v rustup &>/dev/null; then
    log "检测到 Rust，更新工具链..."
    rustup default stable
  fi
}

# --- 主程序 ---
main() {
  echo -e "${GREEN}=== NixOS 一键部署脚本 ===${NC}"
  check_env

  read -p "即将覆盖系统配置并链接用户配置文件，确定继续吗? [y/N] " -n 1 -r
  echo
  if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "操作已取消"
    exit 1
  fi

  deploy_system
  deploy_user
  setup_misc

  echo ""
  echo -e "${GREEN}配置已就绪！${NC}"
  read -p "是否立即重建系统 (nixos-rebuild switch)? [y/N] " -n 1 -r
  echo
  if [[ $REPLY =~ ^[Yy]$ ]]; then
    log "正在重建系统 (这可能需要几分钟，取决于网速)..."
    # 使用 sudo -E 保留当前用户的环境变量 (如代理设置)
    if sudo -E nixos-rebuild switch; then
      success "🎉 系统重建成功！建议重启电脑。"
    else
      error "系统重建失败，请检查上方错误日志。"
    fi
  else
    echo "已跳过重建。稍后请手动运行: sudo nixos-rebuild switch"
  fi
}

main
