#!/usr/bin/env bash
# Refresh Home Manager systemd service.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./lib.sh
source "${SCRIPT_DIR}/lib.sh"

ASSUME_YES=false
TARGET_USER=""

usage() {
  cat <<'EOF_USAGE'
Usage: home_refresh.sh [options]

Options:
  --user <name>   Override username (default: vars.user or $USER)
  -y, --yes       Skip confirmation
  -h, --help      Show this help
EOF_USAGE
}

parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --user)
        shift
        [[ $# -gt 0 ]] || die "--user requires a value"
        TARGET_USER="$1"
        ;;
      -y|--yes)
        ASSUME_YES=true
        ;;
      -h|--help)
        usage
        exit 0
        ;;
      *)
        die "Unknown option: $1"
        ;;
    esac
    shift
  done
}

main() {
  parse_args "$@"
  require_cmd systemctl
  require_cmd sudo

  local user
  user="${TARGET_USER}"
  if [[ -z "${user}" ]]; then
    user="$(get_host_var "user")"
  fi
  if [[ -z "${user}" ]]; then
    user="${USER:-}"
  fi
  if [[ -z "${user}" ]]; then
    die "Unable to determine user. Use --user <name>."
  fi

  local unit="home-manager-${user}.service"
  if ! systemctl list-unit-files --no-legend "${unit}" 2>/dev/null | grep -q "${unit}"; then
    die "Unit not found: ${unit}"
  fi

  if ! confirm "Start ${unit}?"; then
    warn "Cancelled"
    exit 1
  fi

  if sudo systemctl start "${unit}"; then
    ok "Home Manager refreshed: ${unit}"
  else
    die "Failed to start ${unit}"
  fi
}

main "$@"
