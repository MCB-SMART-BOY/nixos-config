#!/usr/bin/env bash
# 上游应用版本更新共享辅助函数。
# 供 pkgs/*/scripts/update-source.sh 引用，避免每个包重复 GitHub API 解析逻辑。
set -euo pipefail

# 获取 GitHub 最新 release tag
# 用法: latest_github_tag "owner/repo"
latest_github_tag() {
  local repo="$1"
  local url="https://api.github.com/repos/${repo}/releases/latest"

  if command -v curl >/dev/null 2>&1; then
    curl -sfL "${url}" | grep '"tag_name":' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/' | head -n 1
  elif command -v wget >/dev/null 2>&1; then
    wget -qO- "${url}" | grep '"tag_name":' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/' | head -n 1
  else
    echo "ERROR: curl or wget required" >&2
    return 1
  fi
}

# 去除版本号前缀 v（如 v1.2.3 → 1.2.3）
strip_v_prefix() {
  local version="$1"
  echo "${version#v}"
}

# 生成 SRI hash（nix hash to-sri 风格）
# 用法: nix_prefetch_url_hash "https://example.com/file.tar.gz"
nix_prefetch_url_hash() {
  local url="$1"
  nix hash to-sri --type sha256 "$(nix-prefetch-url "${url}")"
}

# 计算已下载文件的 SRI hash
# 用法: nix_hash_file "path/to/file"
nix_hash_file() {
  local file="$1"
  nix hash to-sri --type sha256 "$(nix hash file --type sha256 "${file}")"
}
