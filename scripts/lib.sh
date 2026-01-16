#!/usr/bin/env bash
# Common helpers for scripts in this repository.

set -euo pipefail

if [[ -n "${NIXOS_CONFIG_LIB_SOURCED:-}" ]]; then
  return 0
fi
NIXOS_CONFIG_LIB_SOURCED=1

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

msg() {
  local level="$1"
  shift
  printf '[%s] %s\n' "${level}" "$*"
}

log() { msg INFO "$*"; }
ok() { msg OK "$*"; }
warn() { msg WARN "$*"; }
err() { msg ERROR "$*"; }

die() {
  err "$*"
  exit 1
}

require_cmd() {
  local cmd="$1"
  command -v "${cmd}" >/dev/null 2>&1 || die "Missing command: ${cmd}"
}

warn_if_missing_cmd() {
  local cmd="$1"
  local label="$2"
  if ! command -v "${cmd}" >/dev/null 2>&1; then
    warn "${label} missing: ${cmd}"
  else
    ok "${label} available: ${cmd}"
  fi
}

ensure_not_root() {
  if [[ "$(id -u)" -eq 0 ]]; then
    die "Please run as a normal user (sudo will be used when needed)."
  fi
}

is_nixos() {
  if [[ -f /etc/os-release ]]; then
    grep -q '^ID=nixos' /etc/os-release
  else
    return 1
  fi
}

ensure_nixos() {
  if ! is_nixos; then
    die "This command should be run on NixOS."
  fi
}

detect_default_iface() {
  if command -v ip >/dev/null 2>&1; then
    ip route show default 2>/dev/null | awk 'NR==1 {print $5; exit}'
  fi
}

temp_dns_enable() {
  local servers=("$@")
  local iface=""

  if [[ ${#servers[@]} -eq 0 ]]; then
    servers=("223.5.5.5" "223.6.6.6" "1.1.1.1" "8.8.8.8")
  fi

  TEMP_DNS_SERVERS=("${servers[@]}")
  TEMP_DNS_BACKEND=""
  TEMP_DNS_BACKUP=""

  if command -v resolvectl >/dev/null 2>&1 && command -v systemctl >/dev/null 2>&1; then
    if systemctl is-active --quiet systemd-resolved; then
      iface="${TEMP_DNS_IFACE:-}"
      if [[ -z "${iface}" ]]; then
        iface="$(detect_default_iface)"
      fi

      if [[ -z "${iface}" ]]; then
        warn "Cannot detect default interface; falling back to /etc/resolv.conf"
      else
        log "Setting temporary DNS via resolvectl on ${iface}: ${servers[*]}"
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
    for server in "${servers[@]}"; do
      printf 'nameserver %s\n' "${server}"
    done | sudo tee /etc/resolv.conf >/dev/null
    log "Setting temporary DNS via /etc/resolv.conf: ${servers[*]}"
    TEMP_DNS_BACKEND="resolv.conf"
    return 0
  fi

  die "Failed to set temporary DNS (no resolvectl and no /etc/resolv.conf)"
}

temp_dns_disable() {
  if [[ "${TEMP_DNS_BACKEND:-}" == "resolvectl" ]]; then
    if [[ -n "${TEMP_DNS_IFACE:-}" ]]; then
      log "Reverting DNS via resolvectl on ${TEMP_DNS_IFACE}"
      sudo resolvectl revert "${TEMP_DNS_IFACE}" || true
      sudo resolvectl flush-caches >/dev/null 2>&1 || true
    fi
  elif [[ "${TEMP_DNS_BACKEND:-}" == "resolv.conf" ]]; then
    if [[ -n "${TEMP_DNS_BACKUP:-}" && -f "${TEMP_DNS_BACKUP}" ]]; then
      log "Restoring /etc/resolv.conf"
      sudo cp -a "${TEMP_DNS_BACKUP}" /etc/resolv.conf || true
      rm -f "${TEMP_DNS_BACKUP}"
    fi
  fi
}

confirm() {
  local prompt="${1:-Continue?}"
  if [[ "${ASSUME_YES:-false}" == true ]]; then
    return 0
  fi

  read -r -p "${prompt} [y/N] " -n 1
  echo
  [[ $REPLY =~ ^[Yy]$ ]]
}

get_host_var() {
  local key="$1"
  local file="${2:-${REPO_ROOT}/host.nix}"
  awk -F'"' -v k="${key}" '$0 ~ k"[[:space:]]*=" {print $2; exit}' "${file}" 2>/dev/null || true
}
