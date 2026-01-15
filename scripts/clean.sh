#!/usr/bin/env bash
# Garbage-collect old Nix generations.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./lib.sh
source "${SCRIPT_DIR}/lib.sh"

ASSUME_YES=false
RUN=false
SYSTEM=false
USER=true
DELETE_OLDER_THAN=""
FULL=false

usage() {
  cat <<'EOF_USAGE'
Usage: clean.sh [options]

Options:
  --system                Run system GC with sudo
  --no-user               Skip user GC
  --delete-older-than <d> Delete generations older than duration (e.g. 7d)
  --full                  Delete all old generations (-d)
  --run                   Execute GC (default is dry-run)
  -y, --yes               Skip confirmation
  -h, --help              Show this help
EOF_USAGE
}

parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --system)
        SYSTEM=true
        ;;
      --no-user)
        USER=false
        ;;
      --delete-older-than)
        shift
        [[ $# -gt 0 ]] || die "--delete-older-than requires a value"
        DELETE_OLDER_THAN="$1"
        ;;
      --full)
        FULL=true
        ;;
      --run)
        RUN=true
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

build_args() {
  local args=()
  if [[ -n "${DELETE_OLDER_THAN}" ]]; then
    args+=("--delete-older-than" "${DELETE_OLDER_THAN}")
  elif [[ "${FULL}" == true ]]; then
    args+=("-d")
  fi
  printf '%s\n' "${args[@]}"
}

main() {
  parse_args "$@"
  require_cmd nix-collect-garbage

  local args
  mapfile -t args < <(build_args)

  if [[ "${RUN}" != true ]]; then
    log "Dry-run mode. Use --run to execute."
    log "User GC: nix-collect-garbage ${args[*]:-}" 
    if [[ "${SYSTEM}" == true ]]; then
      log "System GC: sudo nix-collect-garbage ${args[*]:-}"
    fi
    exit 0
  fi

  if ! confirm "Run nix garbage collection now?"; then
    warn "Cancelled"
    exit 1
  fi

  if [[ "${USER}" == true ]]; then
    nix-collect-garbage "${args[@]}"
    ok "User GC completed"
  fi

  if [[ "${SYSTEM}" == true ]]; then
    require_cmd sudo
    sudo nix-collect-garbage "${args[@]}"
    ok "System GC completed"
  fi
}

main "$@"
