#!/usr/bin/env bash
# NixOS flake install script (Home Manager ready)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HOST_NAME="${1:-nixos-dev}"

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

log() { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[OK]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() {
  echo -e "${RED}[ERROR]${NC} $1"
  exit 1
}

check_env() {
  log "Checking environment..."

  if [[ "$(whoami)" == "root" ]]; then
    error "Run this script as a normal user (it will use sudo when needed)."
  fi

  if [[ ! -f "${SCRIPT_DIR}/flake.nix" ]]; then
    error "Missing flake.nix in ${SCRIPT_DIR}"
  fi

  if ! command -v sudo >/dev/null 2>&1; then
    error "sudo is required but not found."
  fi

  if ! command -v nixos-rebuild >/dev/null 2>&1; then
    error "nixos-rebuild is required but not found."
  fi

  local host_file="${SCRIPT_DIR}/hosts/${HOST_NAME}/default.nix"
  if [[ ! -f "${host_file}" ]]; then
    error "Host '${HOST_NAME}' not found at ${host_file}"
  fi
}

sync_hardware_config() {
  local host_dir="${SCRIPT_DIR}/hosts/${HOST_NAME}"
  local target="${host_dir}/hardware-configuration.nix"

  if [[ ! -d "${host_dir}" ]]; then
    error "Host directory not found: ${host_dir}"
  fi

  if [[ -f "$target" ]]; then
    success "hardware-configuration.nix already exists for ${HOST_NAME}"
    return
  fi

  if [[ -f /etc/nixos/hardware-configuration.nix ]]; then
    log "Copying /etc/nixos/hardware-configuration.nix into ${target}"
    sudo cp /etc/nixos/hardware-configuration.nix "$target"
    success "hardware-configuration.nix synced"
    if command -v git >/dev/null 2>&1; then
      if git -C "${SCRIPT_DIR}" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
        git -C "${SCRIPT_DIR}" add "$target"
        success "hardware-configuration.nix staged for flake builds"
      fi
    fi
  else
    error "hardware-configuration.nix not found; run nixos-generate-config on target system"
  fi
}

rebuild_system() {
  log "Rebuilding system with flake: ${HOST_NAME}"
  local nix_config="experimental-features = nix-command flakes"
  if [[ -n "${NIX_CONFIG:-}" ]]; then
    nix_config="${NIX_CONFIG}"$'\n'"${nix_config}"
  fi
  sudo -E env NIX_CONFIG="${nix_config}" nixos-rebuild switch --flake "${SCRIPT_DIR}#${HOST_NAME}"
  success "System rebuild complete"
}

main() {
  echo -e "${GREEN}=== NixOS Flake Installer ===${NC}"
  check_env

  read -p "This will rebuild NixOS using ${HOST_NAME}. Continue? [y/N] " -n 1 -r
  echo
  if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Canceled"
    exit 1
  fi

  sync_hardware_config
  rebuild_system
}

main
