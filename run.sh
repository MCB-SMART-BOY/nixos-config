#!/usr/bin/env bash
# Repository script runner.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCRIPTS_DIR="${ROOT_DIR}/scripts"
SCRIPT_NAME="$(basename "$0")"

err() {
  printf '[ERROR] %s\n' "$*" >&2
  exit 1
}

warn() {
  printf '[WARN] %s\n' "$*" >&2
}

usage() {
  cat <<EOF_USAGE
Usage: ${SCRIPT_NAME} [command] [args]

Commands:
  auto                 Run preflight + install (default)
  cloud                Sync local repo (if available) + install_from_github
  sync                 Sync current repo with remote (safe git pull)
  list                 List available scripts
  help [command]       Show help for a script

Examples:
  ${SCRIPT_NAME}
  ${SCRIPT_NAME} list
  ${SCRIPT_NAME} preflight --no-network
  ${SCRIPT_NAME} install --mode test
  ${SCRIPT_NAME} cloud

Environment:
  RUN_PREFLIGHT_ARGS   Extra args for preflight.sh
  RUN_INSTALL_ARGS     Extra args for install.sh
  RUN_CLOUD_ARGS       Extra args for install_from_github.sh
  RUN_SYNC_ARGS        Extra args for sync_cloud.sh
EOF_USAGE
}

script_desc() {
  local file="$1"
  awk 'NR==1 {next} /^#/ {
    if ($0 ~ /^# shellcheck/) {next}
    sub(/^# ?/, "")
    print
    exit
  }' "${file}"
}

list_scripts() {
  if [[ ! -d "${SCRIPTS_DIR}" ]]; then
    err "scripts directory not found: ${SCRIPTS_DIR}"
  fi

  shopt -s nullglob
  printf 'Available scripts:\n'
  for file in "${SCRIPTS_DIR}"/*.sh; do
    local base
    base="$(basename "${file}")"
    if [[ "${base}" == "lib.sh" ]]; then
      continue
    fi
    local name desc
    name="${base%.sh}"
    desc="$(script_desc "${file}")"
    if [[ -z "${desc}" ]]; then
      desc="(no description)"
    fi
    printf '  %-20s %s\n' "${name}" "${desc}"
  done
  shopt -u nullglob

  printf '\nBuilt-in workflows:\n'
  printf '  %-20s %s\n' "auto" "Run preflight + install"
  printf '  %-20s %s\n' "cloud" "Sync local repo (if available) + install_from_github"
  printf '  %-20s %s\n' "sync" "Sync current repo with remote (safe git pull)"
}

resolve_script() {
  local name="$1"
  local candidate

  if [[ -z "${name}" ]]; then
    return 1
  fi

  for candidate in "${SCRIPTS_DIR}/${name}" "${SCRIPTS_DIR}/${name}.sh"; do
    if [[ -f "${candidate}" ]]; then
      printf '%s' "${candidate}"
      return 0
    fi
  done

  return 1
}

run_script() {
  local name="$1"
  shift
  local script
  script="$(resolve_script "${name}")" || err "Unknown command: ${name}"
  exec "${script}" "$@"
}

split_env_args() {
  local env_value="$1"
  local -n out_ref="$2"

  if [[ -n "${env_value}" ]]; then
    read -r -a out_ref <<< "${env_value}"
  fi
}

run_auto() {
  local preflight_args=()
  local install_args=()

  split_env_args "${RUN_PREFLIGHT_ARGS:-}" preflight_args
  split_env_args "${RUN_INSTALL_ARGS:-}" install_args

  if [[ -x "${SCRIPTS_DIR}/preflight.sh" ]]; then
    "${SCRIPTS_DIR}/preflight.sh" "${preflight_args[@]}"
  else
    err "preflight.sh not found or not executable"
  fi

  if [[ -x "${SCRIPTS_DIR}/install.sh" ]]; then
    "${SCRIPTS_DIR}/install.sh" "${install_args[@]}"
  else
    err "install.sh not found or not executable"
  fi
}

run_cloud() {
  local cloud_args=()
  local sync_args=()
  split_env_args "${RUN_CLOUD_ARGS:-}" cloud_args
  split_env_args "${RUN_SYNC_ARGS:-}" sync_args

  if [[ -x "${SCRIPTS_DIR}/sync_cloud.sh" ]]; then
    if command -v git >/dev/null 2>&1; then
      if git -C "${ROOT_DIR}" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
        if ! "${SCRIPTS_DIR}/sync_cloud.sh" "${sync_args[@]}"; then
          warn "Local sync failed; continuing with cloud install"
        fi
      else
        warn "Current directory is not a git repo; skipping local sync"
      fi
    else
      warn "git not found; skipping local sync"
    fi
  fi

  if [[ -x "${SCRIPTS_DIR}/install_from_github.sh" ]]; then
    "${SCRIPTS_DIR}/install_from_github.sh" "${cloud_args[@]}"
  else
    err "install_from_github.sh not found or not executable"
  fi
}

run_sync() {
  local sync_args=()
  split_env_args "${RUN_SYNC_ARGS:-}" sync_args

  if [[ -x "${SCRIPTS_DIR}/sync_cloud.sh" ]]; then
    "${SCRIPTS_DIR}/sync_cloud.sh" "${sync_args[@]}"
  else
    err "sync_cloud.sh not found or not executable"
  fi
}

show_script_help() {
  local target="$1"
  local script
  script="$(resolve_script "${target}")" || err "Unknown command: ${target}"

  if "${script}" --help 2>/dev/null; then
    return 0
  fi

  printf 'No --help available for %s.\n' "${target}"
  local desc
  desc="$(script_desc "${script}")"
  if [[ -n "${desc}" ]]; then
    printf 'Description: %s\n' "${desc}"
  fi
}

main() {
  local cmd="auto"
  if [[ $# -ge 1 ]]; then
    cmd="$1"
    shift
  fi

  case "${cmd}" in
    -h|--help|help)
      if [[ $# -gt 0 ]]; then
        show_script_help "$1"
      else
        usage
        list_scripts
      fi
      ;;
    list|ls)
      list_scripts
      ;;
    auto|local)
      run_auto
      ;;
    cloud|remote)
      run_cloud
      ;;
    sync|update)
      run_sync
      ;;
    *)
      run_script "${cmd}" "$@"
      ;;
  esac
}

main "$@"
