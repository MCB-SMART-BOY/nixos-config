#!/usr/bin/env bash
# Update flake.lock for this repository.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./lib.sh
source "${SCRIPT_DIR}/lib.sh"

REPO_PATH="${REPO_ROOT}"
UPDATE_INPUT=""
COMMIT=false
ASSUME_YES=false

usage() {
  cat <<'EOF_USAGE'
Usage: flake_update.sh [options]

Options:
  --repo <path>     Override repository path
  --input <name>    Update a single flake input
  --commit          Commit flake.lock after update (git required)
  -y, --yes         Skip confirmation
  -h, --help        Show this help
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
      --input)
        shift
        [[ $# -gt 0 ]] || die "--input requires a name"
        UPDATE_INPUT="$1"
        ;;
      --commit)
        COMMIT=true
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
  require_cmd nix

  if [[ ! -f "${REPO_PATH}/flake.nix" ]]; then
    die "flake.nix not found in ${REPO_PATH}"
  fi

  local cmd=(nix flake update)
  if [[ -n "${UPDATE_INPUT}" ]]; then
    cmd+=("--update-input" "${UPDATE_INPUT}")
  fi
  if [[ "${COMMIT}" == true ]]; then
    cmd+=("--commit-lock-file")
  fi

  if ! confirm "Run '${cmd[*]}' in ${REPO_PATH}?"; then
    warn "Cancelled"
    exit 1
  fi

  (cd "${REPO_PATH}" && "${cmd[@]}")
  ok "flake.lock updated"
}

main "$@"
