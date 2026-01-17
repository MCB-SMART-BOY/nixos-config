#!/usr/bin/env bash
# Wrapper for nixos-rebuild using this flake.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./lib.sh
source "${SCRIPT_DIR}/lib.sh"

TARGET_NAME="${TARGET_NAME:-nixos}"
MODE="${MODE:-switch}"
FLAKE_PATH="${REPO_ROOT}"
SHOW_TRACE=false
ASSUME_YES=false

usage() {
  cat <<'EOF_USAGE'
Usage: rebuild.sh [options]

Options:
  --mode <action>    nixos-rebuild action: switch|test|build (default: switch)
  --target <name>    Flake target name (default: nixos)
  --flake <path>     Override flake path (default: repo root)
  --show-trace       Enable nixos-rebuild --show-trace
  -y, --yes          Skip confirmation
  -h, --help         Show this help
EOF_USAGE
}

parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --mode)
        shift
        [[ $# -gt 0 ]] || die "--mode requires a value"
        MODE="$1"
        ;;
      --target)
        shift
        [[ $# -gt 0 ]] || die "--target requires a value"
        TARGET_NAME="$1"
        ;;
      --flake)
        shift
        [[ $# -gt 0 ]] || die "--flake requires a value"
        FLAKE_PATH="$1"
        ;;
      --show-trace)
        SHOW_TRACE=true
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

validate_mode() {
  case "${MODE}" in
    switch|test|build)
      ;;
    *)
      die "Invalid mode: ${MODE}"
      ;;
  esac
}

main() {
  parse_args "$@"
  validate_mode
  ensure_not_root
  ensure_nixos
  require_cmd sudo
  require_cmd nixos-rebuild

  if [[ ! -f "${FLAKE_PATH}/flake.nix" ]]; then
    die "flake.nix not found in ${FLAKE_PATH}"
  fi

  if ! confirm "Run nixos-rebuild ${MODE} for ${TARGET_NAME}?"; then
    warn "Cancelled"
    exit 1
  fi

  local nix_config="experimental-features = nix-command flakes"
  if [[ -n "${NIX_CONFIG:-}" ]]; then
    nix_config="${NIX_CONFIG}"$'\n'"${nix_config}"
  fi

  local args=("${MODE}")
  if [[ "${SHOW_TRACE}" == true ]]; then
    args+=("--show-trace")
  fi

  sudo -E env NIX_CONFIG="${nix_config}" nixos-rebuild "${args[@]}" --flake "${FLAKE_PATH}#${TARGET_NAME}"
  ok "nixos-rebuild completed"
}

main "$@"
