set -euo pipefail

readonly source_path="$1"

deadnix --fail "$source_path"
