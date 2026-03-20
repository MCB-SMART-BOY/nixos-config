#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../../.." && pwd -P)"
SOURCE_FILE="${REPO_ROOT}/pkgs/zed/source.nix"

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

asset_field() {
  local release_json="$1"
  local asset_name="$2"
  local field="$3"
  jq -r --arg name "${asset_name}" --arg field "${field}" '
    [ .assets[] | select(.name == $name) ][0][$field] // empty
  ' <<<"${release_json}"
}

require_cmd curl
require_cmd jq
require_cmd nix

release_json="$(curl -fsSL https://api.github.com/repos/zed-industries/zed/releases/latest)"
tag="$(jq -r '.tag_name' <<<"${release_json}")"
version="${tag#v}"

x86_asset="zed-linux-x86_64.tar.gz"
arm_asset="zed-linux-aarch64.tar.gz"

x86_url="$(asset_field "${release_json}" "${x86_asset}" "browser_download_url")"
x86_digest="$(asset_field "${release_json}" "${x86_asset}" "digest")"
arm_url="$(asset_field "${release_json}" "${arm_asset}" "browser_download_url")"
arm_digest="$(asset_field "${release_json}" "${arm_asset}" "digest")"

if [[ -z "${x86_url}" || -z "${arm_url}" ]]; then
  echo "failed to locate expected zed assets in latest release ${tag}" >&2
  exit 1
fi

x86_hash="$(to_sri_hash "${x86_url}" "${x86_digest}")"
arm_hash="$(to_sri_hash "${arm_url}" "${arm_digest}")"

cat > "${SOURCE_FILE}" <<EOF_ZED
{
  x86_64-linux = {
    version = "${version}";
    url = "${x86_url}";
    hash = "${x86_hash}";
  };

  aarch64-linux = {
    version = "${version}";
    url = "${arm_url}";
    hash = "${arm_hash}";
  };
}
EOF_ZED

echo "updated ${SOURCE_FILE}"
echo "zed official stable -> ${tag}"
