#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
CHECK_ARGS=()

usage() {
  cat <<'EOF'
Usage:
  update-upstream-apps.sh          # update source pins
  update-upstream-apps.sh --check  # check source pins are up-to-date
EOF
}

if [[ "${1:-}" == "--check" ]]; then
  CHECK_ARGS=(--check)
  shift
fi

if [[ $# -ne 0 ]]; then
  usage >&2
  exit 2
fi

"${SCRIPT_DIR}/../zed/scripts/update-source.sh" "${CHECK_ARGS[@]}"
"${SCRIPT_DIR}/../yesplaymusic/scripts/update-source.sh" "${CHECK_ARGS[@]}"

if [[ ${#CHECK_ARGS[@]} -gt 0 ]]; then
  echo "done: upstream app source pins are up-to-date (zed, yesplaymusic)"
else
  echo "done: updated upstream app source pins (zed, yesplaymusic)"
fi
