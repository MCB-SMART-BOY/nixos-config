#!/usr/bin/env bash
# One-step NixOS deploy from GitHub. No arguments.

set -euo pipefail

REPO_URL="https://github.com/MCB-SMART-BOY/nixos-config.git"
BRANCH="master"
TARGET_NAME="nixos"
MODE="switch"
ETC_DIR="/etc/nixos"

msg() {
  local level="$1"
  shift
  printf '[%s] %s\n' "${level}" "$*"
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
Usage: run.sh

This script takes no arguments.
EOF_USAGE
}

parse_args() {
  if [[ $# -gt 0 ]]; then
    usage
    error "Unexpected arguments: $*"
  fi
}

check_env() {
  log "Checking environment..."

  if [[ "$(whoami)" == "root" ]]; then
    error "Run as a normal user (sudo will be used when needed)."
  fi

  if ! command -v sudo >/dev/null 2>&1; then
    error "sudo not found."
  fi

  if ! command -v git >/dev/null 2>&1; then
    error "git not found."
  fi

  if ! command -v nixos-rebuild >/dev/null 2>&1; then
    error "nixos-rebuild not found."
  fi

  if [[ ! -f "${ETC_DIR}/hardware-configuration.nix" ]]; then
    error "Missing ${ETC_DIR}/hardware-configuration.nix; run nixos-generate-config first."
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
        log "Temporary DNS (resolvectl ${iface}): ${servers[*]}"
        sudo resolvectl dns "${iface}" "${servers[@]}"
        sudo resolvectl domain "${iface}" "~."
        TEMP_DNS_BACKEND="resolvectl"
        TEMP_DNS_IFACE="${iface}"
        return 0
      fi
    fi
  fi

  if [[ -f /etc/resolv.conf ]]; then
    TEMP_DNS_BACKUP="$(mktemp)"
    sudo cp -a /etc/resolv.conf "${TEMP_DNS_BACKUP}"
    sudo rm -f /etc/resolv.conf
    printf 'nameserver %s\n' "${servers[@]}" | sudo tee /etc/resolv.conf >/dev/null
    log "Temporary DNS (/etc/resolv.conf): ${servers[*]}"
    TEMP_DNS_BACKEND="resolv.conf"
    return 0
  fi

  error "Failed to set temporary DNS (no resolvectl and no /etc/resolv.conf)."
}

temp_dns_disable() {
  if [[ "${TEMP_DNS_BACKEND}" == "resolvectl" ]]; then
    if [[ -n "${TEMP_DNS_IFACE}" ]]; then
      log "Reverting DNS (resolvectl ${TEMP_DNS_IFACE})"
      sudo resolvectl revert "${TEMP_DNS_IFACE}" || true
      sudo resolvectl flush-caches >/dev/null 2>&1 || true
    fi
  elif [[ "${TEMP_DNS_BACKEND}" == "resolv.conf" ]]; then
    if [[ -n "${TEMP_DNS_BACKUP}" && -f "${TEMP_DNS_BACKUP}" ]]; then
      log "Restoring /etc/resolv.conf"
      sudo cp -a "${TEMP_DNS_BACKUP}" /etc/resolv.conf || true
      rm -f "${TEMP_DNS_BACKUP}"
    fi
  fi
}

clone_repo() {
  local tmp_dir="$1"
  log "Cloning: ${REPO_URL} (${BRANCH})"
  git clone --depth 1 --branch "${BRANCH}" "${REPO_URL}" "${tmp_dir}"
  success "Repository fetched"
}

sync_repo_to_etc() {
  local repo_dir="$1"
  log "Syncing to ${ETC_DIR}"
  sudo mkdir -p "${ETC_DIR}"

  if command -v rsync >/dev/null 2>&1; then
    sudo rsync -a --exclude '.git/' --exclude 'hardware-configuration.nix' "${repo_dir}/" "${ETC_DIR}/"
  else
    (cd "${repo_dir}" && tar --exclude=.git --exclude=hardware-configuration.nix -cf - .) | sudo tar -C "${ETC_DIR}" -xf -
  fi

  success "Config synced"
}

rebuild_system() {
  log "Rebuilding system (${MODE}), target: ${TARGET_NAME}"
  local nix_config="experimental-features = nix-command flakes"
  local rebuild_args=("${MODE}" "--show-trace" "--upgrade")
  if [[ -n "${NIX_CONFIG:-}" ]]; then
    nix_config="${NIX_CONFIG}"$'\n'"${nix_config}"
  fi
  sudo -E env NIX_CONFIG="${nix_config}" nixos-rebuild "${rebuild_args[@]}" --flake "${ETC_DIR}#${TARGET_NAME}"
  success "Rebuild complete"
}

main() {
  printf '==> %s\n' "NixOS one-step deploy"
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

  temp_dns_enable

  clone_repo "${tmp_dir}"
  sync_repo_to_etc "${tmp_dir}"
  rebuild_system
}

main "$@"
