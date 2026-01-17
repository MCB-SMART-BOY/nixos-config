#!/usr/bin/env bash
# Show repository and system status.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./lib.sh
source "${SCRIPT_DIR}/lib.sh"

SHORT=false

usage() {
  cat <<'EOF_USAGE'
Usage: status.sh [options]

Options:
  --short       Show minimal output
  -h, --help    Show this help
EOF_USAGE
}

parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --short)
        SHORT=true
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

show_repo() {
  msg INFO "==> Repo"
  log "Repo root: ${REPO_ROOT}"
  if command -v git >/dev/null 2>&1; then
    if git -C "${REPO_ROOT}" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
      local branch dirty
      branch="$(git -C "${REPO_ROOT}" rev-parse --abbrev-ref HEAD)"
      dirty="$(git -C "${REPO_ROOT}" status --porcelain)"
      ok "Git branch: ${branch}"
      if [[ -n "${dirty}" ]]; then
        warn "Git status: dirty"
      else
        ok "Git status: clean"
      fi
    else
      warn "Not a git repository"
    fi
  else
    warn "git missing"
  fi
}

show_host_vars() {
  msg INFO "==> Host vars"
  if [[ -f "${REPO_ROOT}/host.nix" ]]; then
    local user proxy tun
    user="$(get_host_var "user")"
    proxy="$(get_host_var "proxyUrl")"
    tun="$(get_host_var "tunInterface")"

    if [[ -n "${user}" ]]; then
      ok "vars.user = ${user}"
    else
      warn "vars.user not detected"
    fi

    if [[ -n "${proxy}" ]]; then
      ok "vars.proxyUrl = ${proxy}"
    else
      warn "vars.proxyUrl is empty"
    fi

    if [[ -n "${tun}" ]]; then
      ok "vars.tunInterface = ${tun}"
    else
      warn "vars.tunInterface is empty"
    fi
  else
    warn "host.nix missing"
  fi
}

show_system() {
  msg INFO "==> System"
  if command -v nixos-version >/dev/null 2>&1; then
    ok "NixOS: $(nixos-version)"
  else
    warn "nixos-version missing"
  fi

  if command -v nix >/dev/null 2>&1; then
    ok "nix: $(nix --version)"
  else
    warn "nix missing"
  fi

  if command -v nixos-rebuild >/dev/null 2>&1; then
    ok "nixos-rebuild available"
  else
    warn "nixos-rebuild missing"
  fi

  ok "Kernel: $(uname -r)"
}

show_etc() {
  msg INFO "==> /etc/nixos"
  if [[ -d /etc/nixos ]]; then
    ok "/etc/nixos exists"
    if [[ -f /etc/nixos/flake.nix ]]; then
      ok "flake.nix present"
    else
      warn "flake.nix missing"
    fi

    if [[ -f /etc/nixos/hardware-configuration.nix ]]; then
      ok "hardware-configuration.nix present"
    else
      warn "hardware-configuration.nix missing"
    fi
  else
    warn "/etc/nixos missing"
  fi
}

main() {
  parse_args "$@"

  if [[ "${SHORT}" == true ]]; then
    show_repo
    show_host_vars
    return
  fi

  show_repo
  show_host_vars
  show_system
  show_etc
}

main "$@"
