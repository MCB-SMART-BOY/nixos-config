#!/usr/bin/env bash
# Sync hardware-configuration.nix from /etc/nixos into the repo.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./lib.sh
source "${SCRIPT_DIR}/lib.sh"

ASSUME_YES=false
FORCE=false
SRC_PATH="/etc/nixos/hardware-configuration.nix"
DEST_PATH="${REPO_ROOT}/hardware-configuration.nix"

usage() {
  cat <<'EOF_USAGE'
Usage: sync_hardware.sh [options]

Options:
  --from <path>     Source hardware config (default: /etc/nixos/hardware-configuration.nix)
  --to <path>       Destination path (default: repo root hardware-configuration.nix)
  --force           Overwrite destination if it exists
  -y, --yes         Skip confirmation
  -h, --help        Show this help
EOF_USAGE
}

parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --from)
        shift
        [[ $# -gt 0 ]] || die "--from requires a path"
        SRC_PATH="$1"
        ;;
      --to)
        shift
        [[ $# -gt 0 ]] || die "--to requires a path"
        DEST_PATH="$1"
        ;;
      --force)
        FORCE=true
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
  ensure_not_root
  require_cmd sudo

  if [[ ! -f "${SRC_PATH}" ]]; then
    die "Source file not found: ${SRC_PATH}"
  fi

  if [[ -f "${DEST_PATH}" && "${FORCE}" != true ]]; then
    die "Destination exists: ${DEST_PATH} (use --force to overwrite)"
  fi

  if ! confirm "Copy ${SRC_PATH} to ${DEST_PATH}?"; then
    warn "Cancelled"
    exit 1
  fi

  sudo cp "${SRC_PATH}" "${DEST_PATH}"
  ok "Hardware config synced to ${DEST_PATH}"
}

main "$@"
