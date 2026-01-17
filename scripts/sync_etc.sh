#!/usr/bin/env bash
# Sync repository files into /etc/nixos.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./lib.sh
source "${SCRIPT_DIR}/lib.sh"

ASSUME_YES=false
FORCE_HARDWARE=false
DRY_RUN=false
REPO_PATH="${REPO_ROOT}"
ETC_DIR="/etc/nixos"

usage() {
  cat <<'EOF_USAGE'
Usage: sync_etc.sh [options]

Options:
  --repo <path>       Override repository path
  --force-hardware    Include hardware-configuration.nix
  --dry-run           Show what would be synced (rsync only)
  -y, --yes           Skip confirmation
  -h, --help          Show this help
EOF_USAGE
}

parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --repo)
        shift
        [[ $# -gt 0 ]] || die "--repo requires a path"
        REPO_PATH="$1"
        ;;
      --force-hardware)
        FORCE_HARDWARE=true
        ;;
      --dry-run)
        DRY_RUN=true
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

sync_repo_to_etc() {
  log "Syncing ${REPO_PATH} -> ${ETC_DIR}"
  sudo mkdir -p "${ETC_DIR}"

  local excludes=(--exclude '.git/')
  if [[ "${FORCE_HARDWARE}" != true ]]; then
    excludes+=(--exclude 'hardware-configuration.nix')
  fi

  if command -v rsync >/dev/null 2>&1; then
    local rsync_args=(-a)
    if [[ "${DRY_RUN}" == true ]]; then
      rsync_args+=(--dry-run --itemize-changes)
    fi
    sudo rsync "${rsync_args[@]}" "${excludes[@]}" "${REPO_PATH}/" "${ETC_DIR}/"
  else
    if [[ "${DRY_RUN}" == true ]]; then
      warn "rsync missing; dry-run unavailable"
      return 0
    fi
    local tar_excludes=(--exclude=.git)
    if [[ "${FORCE_HARDWARE}" != true ]]; then
      tar_excludes+=(--exclude=hardware-configuration.nix)
    fi
    (cd "${REPO_PATH}" && tar "${tar_excludes[@]}" -cf - .) | sudo tar -C "${ETC_DIR}" -xf -
  fi
}

main() {
  parse_args "$@"
  ensure_not_root
  require_cmd sudo

  if [[ ! -f "${REPO_PATH}/flake.nix" ]]; then
    die "flake.nix not found in ${REPO_PATH}"
  fi

  if [[ "${DRY_RUN}" == true ]]; then
    log "Dry-run enabled; no files will be modified"
  fi

  if ! confirm "Sync ${REPO_PATH} to ${ETC_DIR}?"; then
    warn "Cancelled"
    exit 1
  fi

  sync_repo_to_etc
  ok "Sync completed"
}

main "$@"
