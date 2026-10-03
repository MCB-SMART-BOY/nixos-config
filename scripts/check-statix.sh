set -euo pipefail

readonly source_path="$1"

statix check --config "$source_path/statix.toml" "$source_path"
