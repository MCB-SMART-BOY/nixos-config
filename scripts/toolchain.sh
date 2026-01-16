#!/usr/bin/env bash
# Bootstrap development toolchains (rustup).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./lib.sh
source "${SCRIPT_DIR}/lib.sh"

ASSUME_YES=false
ENABLE_RUST=true
RUST_TOOLCHAIN="stable"
RUST_COMPONENTS=("rust-analyzer" "rustfmt" "clippy")

usage() {
  cat <<'EOF_USAGE'
Usage: toolchain.sh [options]

Options:
  --no-rust                 Skip rustup toolchain setup
  --rust-toolchain <name>   Rust toolchain (default: stable)
  --rust-components <list>  Comma-separated components (default: rust-analyzer,rustfmt,clippy)
  -y, --yes                 Skip confirmation
  -h, --help                Show this help
EOF_USAGE
}

parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --no-rust)
        ENABLE_RUST=false
        ;;
      --rust-toolchain)
        shift
        [[ $# -gt 0 ]] || die "--rust-toolchain requires a value"
        RUST_TOOLCHAIN="$1"
        ;;
      --rust-components)
        shift
        [[ $# -gt 0 ]] || die "--rust-components requires a value"
        IFS=',' read -r -a RUST_COMPONENTS <<< "$1"
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

find_rustup() {
  local rustup_bin
  rustup_bin="$(command -v rustup || true)"
  if [[ -z "${rustup_bin}" && -x "${HOME}/.nix-profile/bin/rustup" ]]; then
    rustup_bin="${HOME}/.nix-profile/bin/rustup"
  fi
  if [[ -z "${rustup_bin}" && -x "/run/current-system/sw/bin/rustup" ]]; then
    rustup_bin="/run/current-system/sw/bin/rustup"
  fi
  printf '%s' "${rustup_bin}"
}

setup_rust() {
  local rustup_bin failed
  rustup_bin="$(find_rustup)"
  if [[ -z "${rustup_bin}" ]]; then
    warn "rustup not found; skip Rust toolchain"
    return 1
  fi

  log "Installing Rust toolchain: ${RUST_TOOLCHAIN}"
  "${rustup_bin}" toolchain install "${RUST_TOOLCHAIN}"
  "${rustup_bin}" default "${RUST_TOOLCHAIN}"

  failed=0
  for component in "${RUST_COMPONENTS[@]}"; do
    if [[ -z "${component}" ]]; then
      continue
    fi
    if "${rustup_bin}" component add "${component}" --toolchain "${RUST_TOOLCHAIN}"; then
      ok "Rust component installed: ${component}"
    else
      warn "Rust component failed: ${component}"
      failed=1
    fi
  done

  return "${failed}"
}

main() {
  parse_args "$@"
  ensure_not_root

  if ! confirm "Install development toolchains now?"; then
    warn "Cancelled"
    exit 1
  fi

  local failed=0
  if [[ "${ENABLE_RUST}" == true ]]; then
    if ! setup_rust; then
      failed=1
    fi
  fi

  if [[ "${failed}" -ne 0 ]]; then
    die "Toolchain setup failed"
  fi

  ok "Toolchain setup completed"
}

main "$@"
