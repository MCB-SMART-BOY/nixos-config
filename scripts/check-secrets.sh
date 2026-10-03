set -euo pipefail

readonly source_path="$1"

gitleaks detect --no-git --source "$source_path" --no-banner --redact
