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

usage() {
  cat <<EOF_USAGE
Usage: ${SCRIPT_NAME} <command> [args]

Commands:
  list                 List available scripts
  help [command]       Show help for a script

Examples:
  ${SCRIPT_NAME} list
  ${SCRIPT_NAME} preflight --no-network
  ${SCRIPT_NAME} install --mode test
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
  if [[ $# -lt 1 ]]; then
    usage
    list_scripts
    exit 1
  fi

  local cmd="$1"
  shift

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
    *)
      local script
      script="$(resolve_script "${cmd}")" || err "Unknown command: ${cmd}"
      exec "${script}" "$@"
      ;;
  esac
}

main "$@"
