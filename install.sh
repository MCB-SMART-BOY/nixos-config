#!/usr/bin/env bash
# NixOS flake install script (single host)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="${SCRIPT_DIR}"
SCRIPT_NAME="$(basename "$0")"
TARGET_NAME="${TARGET_NAME:-nixos}"
MODE="${MODE:-switch}"
SHOW_TRACE=false
FORCE_SYNC=false
HOST_FILE="${REPO_ROOT}/host.nix"
HARDWARE_FILE="${REPO_ROOT}/hardware-configuration.nix"
ASSUME_YES=false
NO_REBUILD=false
NO_SYNC=false

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

usage() {
  cat <<EOF_USAGE
Usage: ${SCRIPT_NAME} [options]

Options:
  -h, --help         Show this help
  -y, --yes          Skip confirmation prompt
  --mode <action>    nixos-rebuild action: switch|test|build (default: switch)
  --show-trace       Enable nixos-rebuild --show-trace
  --force-sync       Overwrite existing hardware-configuration.nix
  --no-sync          Skip hardware-configuration sync
  --no-rebuild       Skip nixos-rebuild
EOF_USAGE
}

parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      -h|--help)
        usage
        exit 0
        ;;
      -y|--yes)
        ASSUME_YES=true
        ;;
      --mode)
        shift
        [[ $# -gt 0 ]] || error "--mode requires a value"
        MODE="$1"
        ;;
      --show-trace)
        SHOW_TRACE=true
        ;;
      --force-sync)
        FORCE_SYNC=true
        ;;
      --no-sync)
        NO_SYNC=true
        ;;
      --no-rebuild)
        NO_REBUILD=true
        ;;
      --)
        shift
        break
        ;;
      -* )
        error "Unknown option: $1"
        ;;
      * )
        error "Unexpected argument: $1"
        ;;
    esac
    shift
  done
}

validate_flags() {
  if [[ "${NO_SYNC}" == true && "${FORCE_SYNC}" == true ]]; then
    error "--force-sync cannot be used with --no-sync"
  fi
}

validate_mode() {
  if [[ "${NO_REBUILD}" == true ]]; then
    return
  fi

  case "${MODE}" in
    switch|test|build)
      ;;
    *)
      error "Invalid --mode '${MODE}' (use switch|test|build)"
      ;;
  esac
}

check_env() {
  log "Checking environment..."

  if [[ "$(whoami)" == "root" ]]; then
    error "Run this script as a normal user (it will use sudo when needed)."
  fi

  if [[ ! -f "${REPO_ROOT}/flake.nix" ]]; then
    error "Missing flake.nix in ${REPO_ROOT}"
  fi

  if [[ ! -f "${HOST_FILE}" ]]; then
    error "Missing host.nix in ${REPO_ROOT}"
  fi

  local needs_sudo=false
  if [[ "${NO_SYNC}" != true || "${NO_REBUILD}" != true ]]; then
    needs_sudo=true
  fi

  if [[ "${needs_sudo}" == true ]] && ! command -v sudo >/dev/null 2>&1; then
    error "sudo is required but not found."
  fi

  if [[ "${NO_REBUILD}" != true ]] && ! command -v nixos-rebuild >/dev/null 2>&1; then
    error "nixos-rebuild is required but not found."
  fi

  if [[ "${NO_SYNC}" == true && "${NO_REBUILD}" != true && ! -f "${HARDWARE_FILE}" ]]; then
    warn "hardware-configuration.nix is missing; rebuild will likely fail without sync"
  fi
}

sync_hardware_config() {
  local target="${HARDWARE_FILE}"

  if [[ -f "${target}" && "${FORCE_SYNC}" != true ]]; then
    success "hardware-configuration.nix already exists"
    return
  fi

  if [[ -f "${target}" && "${FORCE_SYNC}" == true ]]; then
    warn "Overwriting existing hardware-configuration.nix"
  fi

  if [[ -f /etc/nixos/hardware-configuration.nix ]]; then
    log "Copying /etc/nixos/hardware-configuration.nix into ${target}"
    sudo cp /etc/nixos/hardware-configuration.nix "${target}"
    success "hardware-configuration.nix synced"
    if command -v git >/dev/null 2>&1; then
      if git -C "${REPO_ROOT}" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
        git -C "${REPO_ROOT}" add "${target}"
        success "hardware-configuration.nix staged for flake builds"
      fi
    fi
  else
    error "hardware-configuration.nix not found; run nixos-generate-config on target system"
  fi
}

rebuild_system() {
  log "Rebuilding system (${MODE}) with flake: ${TARGET_NAME}"
  local nix_config="experimental-features = nix-command flakes"
  local rebuild_args=("${MODE}")
  if [[ -n "${NIX_CONFIG:-}" ]]; then
    nix_config="${NIX_CONFIG}"$'\n'"${nix_config}"
  fi
  if [[ "${SHOW_TRACE}" == true ]]; then
    rebuild_args+=("--show-trace")
  fi
  sudo -E env NIX_CONFIG="${nix_config}" nixos-rebuild "${rebuild_args[@]}" --flake "${REPO_ROOT}#${TARGET_NAME}"
  success "System rebuild complete"
}

confirm() {
  local steps=()
  if [[ "${NO_SYNC}" != true ]]; then
    if [[ "${FORCE_SYNC}" == true ]]; then
      steps+=("sync hardware config (overwrite)")
    else
      steps+=("sync hardware config")
    fi
  fi
  if [[ "${NO_REBUILD}" != true ]]; then
    steps+=("rebuild NixOS (${MODE})")
  fi
  if [[ ${#steps[@]} -eq 0 ]]; then
    error "Nothing to do (both --no-sync and --no-rebuild are set)"
  fi

  if [[ "${ASSUME_YES}" == true ]]; then
    return
  fi

  local plan
  plan=$(IFS=", "; echo "${steps[*]}")
  read -r -p "This will ${plan} for ${TARGET_NAME}. Continue? [y/N] " -n 1
  echo
  if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Canceled"
    exit 1
  fi
}

main() {
  echo -e "${GREEN}=== NixOS Flake Installer ===${NC}"
  parse_args "$@"
  validate_flags
  validate_mode
  check_env
  confirm

  if [[ "${NO_SYNC}" != true ]]; then
    sync_hardware_config
  else
    warn "Skipping hardware-configuration sync"
  fi

  if [[ "${NO_REBUILD}" != true ]]; then
    rebuild_system
  else
    warn "Skipping nixos-rebuild"
  fi
}

main "$@"
