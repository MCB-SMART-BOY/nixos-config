set -euo pipefail

readonly check_script="$1"

printf '>>> [source-%s] checking %s\n' "$checkName" "$checkSource"
if ! bash "$check_script" "$checkSource"; then
  printf 'error: source-%s failed; see diagnostics above\n' "$checkName" >&2
  exit 1
fi

touch "$out"
