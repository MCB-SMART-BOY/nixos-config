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
