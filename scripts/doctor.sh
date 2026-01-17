#!/usr/bin/env bash
# Extended checks for this NixOS config repository.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./lib.sh
source "${SCRIPT_DIR}/lib.sh"

NO_NETWORK=false
NO_PORTS=false
STRICT=false
SKIP_PREFLIGHT=false
SKIP_SCRIPTS=false
FLAKE_CHECK=false

failures=0

usage() {
  cat <<'EOF_USAGE'
Usage: doctor.sh [options]

Options:
  --no-network      Skip network reachability checks
  --no-ports        Skip local port checks
  --strict          Treat warnings as errors in preflight
  --skip-preflight  Skip running scripts/preflight.sh
  --skip-scripts    Skip bash syntax checks for scripts
  --flake-check     Run `nix flake check` in repo root
  -h, --help        Show this help
EOF_USAGE
}

parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --no-network)
        NO_NETWORK=true
        ;;
      --no-ports)
        NO_PORTS=true
        ;;
      --strict)
        STRICT=true
        ;;
      --skip-preflight)
        SKIP_PREFLIGHT=true
        ;;
      --skip-scripts)
        SKIP_SCRIPTS=true
        ;;
      --flake-check)
        FLAKE_CHECK=true
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

run_preflight() {
  if [[ "${SKIP_PREFLIGHT}" == true ]]; then
    warn "Skip preflight"
    return 0
  fi

  if [[ ! -x "${REPO_ROOT}/scripts/preflight.sh" ]]; then
    warn "preflight.sh missing or not executable"
    failures=$((failures + 1))
    return 1
  fi

  local args=()
  [[ "${NO_NETWORK}" == true ]] && args+=(--no-network)
  [[ "${NO_PORTS}" == true ]] && args+=(--no-ports)
  [[ "${STRICT}" == true ]] && args+=(--strict)

  msg INFO "==> Running preflight"
  if ! "${REPO_ROOT}/scripts/preflight.sh" "${args[@]}"; then
    failures=$((failures + 1))
    warn "Preflight reported failures"
  fi
}

check_scripts() {
  if [[ "${SKIP_SCRIPTS}" == true ]]; then
    warn "Skip script syntax checks"
    return 0
  fi

  msg INFO "==> Bash syntax checks"
  local failed=0
  local file
  for file in "${REPO_ROOT}/run.sh" "${REPO_ROOT}/scripts"/*.sh; do
    if [[ -f "${file}" ]]; then
      if bash -n "${file}"; then
        ok "Syntax OK: ${file}"
      else
        warn "Syntax error: ${file}"
        failed=1
      fi
    fi
  done

  if [[ "${failed}" -ne 0 ]]; then
    failures=$((failures + 1))
  fi
}

flake_check() {
  if [[ "${FLAKE_CHECK}" != true ]]; then
    return 0
  fi

  msg INFO "==> nix flake check"
  if ! command -v nix >/dev/null 2>&1; then
    warn "nix missing; cannot run flake check"
    failures=$((failures + 1))
    return 1
  fi

  if (cd "${REPO_ROOT}" && nix flake check); then
    ok "flake check passed"
  else
    warn "flake check failed"
    failures=$((failures + 1))
  fi
}

main() {
  parse_args "$@"

  run_preflight
  check_scripts
  flake_check

  msg INFO "==> Doctor summary"
  if [[ "${failures}" -eq 0 ]]; then
    ok "All checks passed"
  else
    err "${failures} check(s) failed"
    exit 1
  fi
}

main "$@"
