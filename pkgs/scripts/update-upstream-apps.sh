#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"

"${SCRIPT_DIR}/../zed/scripts/update-source.sh"
"${SCRIPT_DIR}/../yesplaymusic/scripts/update-source.sh"

echo "done: updated upstream app source pins (zed, yesplaymusic)"
