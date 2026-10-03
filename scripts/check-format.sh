set -euo pipefail

readonly source_path="$1"

find "$source_path" -type f -name '*.nix' -print0 \
  | sort -z \
  | xargs -0 -r nixfmt --check
