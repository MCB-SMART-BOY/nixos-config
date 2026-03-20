#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../../.." && pwd -P)"
SOURCE_FILE="${REPO_ROOT}/pkgs/yesplaymusic/source.nix"
CHECK_ONLY=false

usage() {
  cat <<'EOF'
Usage:
  update-source.sh          # update source.nix to latest upstream stable release
  update-source.sh --check  # check whether source.nix is up-to-date
EOF
}

if [[ "${1:-}" == "--check" ]]; then
  CHECK_ONLY=true
  shift
fi

if [[ $# -ne 0 ]]; then
  usage >&2
  exit 2
fi

require_cmd() {
  local cmd="$1"
  if ! command -v "${cmd}" >/dev/null 2>&1; then
    echo "missing command: ${cmd}" >&2
    exit 1
  fi
}

to_sri_hash() {
  local url="$1"
  local digest="$2"
  if [[ -n "${digest}" && "${digest}" != "null" && "${digest}" == sha256:* ]]; then
    local hex="${digest#sha256:}"
    nix --extra-experimental-features 'nix-command' hash convert --hash-algo sha256 --to sri "${hex}"
    return 0
  fi
  nix --extra-experimental-features 'nix-command' store prefetch-file --json "${url}" | jq -r '.hash'
}

require_cmd curl
require_cmd jq
require_cmd nix

release_json="$(curl -fsSL https://api.github.com/repos/qier222/YesPlayMusic/releases/latest)"
tag="$(jq -r '.tag_name' <<<"${release_json}")"
version="${tag#v}"

asset_json="$(
  jq -r '
    [ .assets[]
      | select((.name | startswith("YesPlayMusic-")) and (.name | endswith(".AppImage")))
    ][0] // empty
  ' <<<"${release_json}"
)"

if [[ -z "${asset_json}" ]]; then
  echo "failed to locate AppImage asset in latest release ${tag}" >&2
  exit 1
fi

url="$(jq -r '.browser_download_url // empty' <<<"${asset_json}")"
digest="$(jq -r '.digest // empty' <<<"${asset_json}")"
if [[ -z "${url}" ]]; then
  echo "missing browser_download_url for AppImage in release ${tag}" >&2
  exit 1
fi

hash="$(to_sri_hash "${url}" "${digest}")"

tmp_file="$(mktemp)"
trap 'rm -f "${tmp_file}"' EXIT

cat > "${tmp_file}" <<EOF_YES
{
  version = "${version}";
  url = "${url}";
  hash = "${hash}";
}
EOF_YES

if [[ "${CHECK_ONLY}" == "true" ]]; then
  if [[ -f "${SOURCE_FILE}" ]] && cmp -s "${tmp_file}" "${SOURCE_FILE}"; then
    echo "up-to-date: ${SOURCE_FILE} (${tag})"
    exit 0
  fi

  echo "outdated: ${SOURCE_FILE} (latest ${tag})" >&2
  if command -v diff >/dev/null 2>&1 && [[ -f "${SOURCE_FILE}" ]]; then
    diff -u "${SOURCE_FILE}" "${tmp_file}" || true
  fi
  exit 1
fi

mv "${tmp_file}" "${SOURCE_FILE}"
trap - EXIT
echo "updated ${SOURCE_FILE}"
echo "yesplaymusic official stable -> ${tag}"
