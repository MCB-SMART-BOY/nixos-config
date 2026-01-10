#!/usr/bin/env bash
# NixOS flake install script (Home Manager ready)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
SCRIPT_NAME="$(basename "$0")"
DEFAULT_HOST="nixos-dev"
HOST_NAME="${DEFAULT_HOST}"
ASSUME_YES=false
NO_REBUILD=false
NO_SYNC=false
INIT_HOST=false

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
  cat <<EOF
Usage: ${SCRIPT_NAME} [host] [options]

Options:
  -h, --help         Show this help
  -y, --yes          Skip confirmation prompt
  --host <name>      Specify host name (default: ${DEFAULT_HOST})
  --no-sync          Skip hardware-configuration sync
  --no-rebuild       Skip nixos-rebuild
  --init-host        Initialize hosts/<name> from hosts/${DEFAULT_HOST}/default.nix
EOF
}

parse_args() {
  local positional=()

  while [[ $# -gt 0 ]]; do
    case "$1" in
      -h|--help)
        usage
        exit 0
        ;;
      -y|--yes)
        ASSUME_YES=true
        ;;
      --host)
        shift
        [[ $# -gt 0 ]] || error "--host requires a value"
        HOST_NAME="$1"
        ;;
      --no-sync)
        NO_SYNC=true
        ;;
      --no-rebuild)
        NO_REBUILD=true
        ;;
      --init-host)
        INIT_HOST=true
        ;;
      --)
        shift
        break
        ;;
      -*)
        error "Unknown option: $1"
        ;;
      *)
        positional+=("$1")
        ;;
    esac
    shift
  done

  if [[ ${#positional[@]} -gt 1 ]]; then
    error "Too many arguments: ${positional[*]}"
  fi
  if [[ ${#positional[@]} -eq 1 ]]; then
    HOST_NAME="${positional[0]}"
  fi
}

ensure_host_dir() {
  local host_dir="${REPO_ROOT}/hosts/${HOST_NAME}"
  local host_file="${host_dir}/default.nix"
  local template_file="${REPO_ROOT}/hosts/${DEFAULT_HOST}/default.nix"

  if [[ -f "${host_file}" ]]; then
    return
  fi

  if [[ "${INIT_HOST}" != true ]]; then
    error "Host '${HOST_NAME}' not found at ${host_file} (use --init-host to create from template)"
  fi

  if [[ ! -f "${template_file}" ]]; then
    error "Host template not found: ${template_file}"
  fi

  log "Initializing host '${HOST_NAME}' from ${template_file}"
  mkdir -p "${host_dir}"
  cp "${template_file}" "${host_file}"
  success "Host '${HOST_NAME}' initialized (remember to add it in flake.nix)"
}

check_env() {
  log "Checking environment..."

  if [[ "$(whoami)" == "root" ]]; then
    error "Run this script as a normal user (it will use sudo when needed)."
  fi

  if [[ ! -f "${REPO_ROOT}/flake.nix" ]]; then
    error "Missing flake.nix in ${REPO_ROOT}"
  fi

  if ! command -v sudo >/dev/null 2>&1; then
    error "sudo is required but not found."
  fi

  if ! command -v nixos-rebuild >/dev/null 2>&1; then
    error "nixos-rebuild is required but not found."
  fi

  ensure_host_dir
}

sync_hardware_config() {
  local host_dir="${REPO_ROOT}/hosts/${HOST_NAME}"
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
      if git -C "${REPO_ROOT}" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
        git -C "${REPO_ROOT}" add "$target"
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
  sudo -E env NIX_CONFIG="${nix_config}" nixos-rebuild switch --flake "${REPO_ROOT}#${HOST_NAME}"
  success "System rebuild complete"
}

confirm() {
  local steps=()
  if [[ "${NO_SYNC}" != true ]]; then
    steps+=("sync hardware config")
  fi
  if [[ "${NO_REBUILD}" != true ]]; then
    steps+=("rebuild NixOS")
  fi
  if [[ ${#steps[@]} -eq 0 ]]; then
    error "Nothing to do (both --no-sync and --no-rebuild are set)"
  fi

  if [[ "${ASSUME_YES}" == true ]]; then
    return
  fi

  local plan
  plan=$(IFS=", "; echo "${steps[*]}")
  read -r -p "This will ${plan} for ${HOST_NAME}. Continue? [y/N] " -n 1
  echo
  if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Canceled"
    exit 1
  fi
}

main() {
  echo -e "${GREEN}=== NixOS Flake Installer ===${NC}"
  parse_args "$@"
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
