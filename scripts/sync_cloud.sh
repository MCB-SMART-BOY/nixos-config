#!/usr/bin/env bash
# Sync the current repository with the remote (safe fast-forward only by default).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCRIPT_NAME="$(basename "$0")"
# shellcheck source=./lib.sh
source "${SCRIPT_DIR}/lib.sh"

REPO_URL="${REPO_URL:-https://github.com/MCB-SMART-BOY/nixos-config.git}"
BRANCH="${BRANCH:-master}"
ASSUME_YES=false
AUTO_STASH=false
REPLACE=false

usage() {
  cat <<EOF_USAGE
用法: ${SCRIPT_NAME} [options]

选项:
  -h, --help        显示帮助
  -y, --yes         跳过确认（仅对 --replace 生效）
  --repo <url>      远端仓库地址（默认: ${REPO_URL}）
  --branch <name>   分支名（默认: 当前分支或 ${BRANCH}）
  --autostash       工作区有改动时自动 stash + rebase
  --replace         用云端快照覆盖当前目录（会删除本地多余文件）
EOF_USAGE
}

parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      -h|--help)
        usage
        exit 0
        ;;
      -y|--yes)
        ASSUME_YES=true
        ;;
      --repo)
        shift
        [[ $# -gt 0 ]] || die "参数 --repo 需要一个值"
        REPO_URL="$1"
        ;;
      --branch)
        shift
        [[ $# -gt 0 ]] || die "参数 --branch 需要一个值"
        BRANCH="$1"
        ;;
      --autostash)
        AUTO_STASH=true
        ;;
      --replace)
        REPLACE=true
        ;;
      --)
        shift
        break
        ;;
      -* )
        die "未知参数: $1"
        ;;
      * )
        die "不支持的参数: $1"
        ;;
    esac
    shift
  done
}

resolve_remote() {
  local remote=""
  if git -C "${REPO_ROOT}" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    remote="$(git -C "${REPO_ROOT}" remote get-url origin 2>/dev/null || true)"
  fi
  if [[ -n "${remote}" ]]; then
    printf '%s' "${remote}"
    return 0
  fi
  printf '%s' "${REPO_URL}"
}

resolve_branch() {
  local branch=""
  if git -C "${REPO_ROOT}" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    branch="$(git -C "${REPO_ROOT}" symbolic-ref --quiet --short HEAD 2>/dev/null || true)"
  fi
  if [[ -n "${branch}" ]]; then
    printf '%s' "${branch}"
    return 0
  fi
  printf '%s' "${BRANCH}"
}

sync_git_repo() {
  local remote branch dirty
  remote="$(resolve_remote)"
  branch="$(resolve_branch)"

  dirty="$(git -C "${REPO_ROOT}" status --porcelain)"
  if [[ -n "${dirty}" && "${AUTO_STASH}" != true ]]; then
    die "仓库有未提交改动，请先提交/暂存，或使用 --autostash"
  fi

  if [[ "${AUTO_STASH}" == true ]]; then
    log "拉取更新（rebase + autostash）：${remote} ${branch}"
    git -C "${REPO_ROOT}" pull --rebase --autostash "${remote}" "${branch}"
    ok "本地仓库已更新"
    return
  fi

  log "拉取更新（fast-forward）：${remote} ${branch}"
  git -C "${REPO_ROOT}" fetch --prune "${remote}" "${branch}"
  git -C "${REPO_ROOT}" merge --ff-only FETCH_HEAD
  ok "本地仓库已更新"
}

replace_from_remote() {
  local tmp_dir
  tmp_dir="$(mktemp -d)"
  trap 'rm -rf "${tmp_dir}"' EXIT

  log "拉取仓库: ${REPO_URL} (${BRANCH})"
  git clone --branch "${BRANCH}" "${REPO_URL}" "${tmp_dir}"

  if ! confirm "将使用云端覆盖当前目录（会删除本地未提交内容），是否继续?"; then
    warn "已取消"
    exit 1
  fi

  log "同步云端快照到 ${REPO_ROOT}"
  if command -v rsync >/dev/null 2>&1; then
    rsync -a --delete --exclude '.git/' "${tmp_dir}/" "${REPO_ROOT}/"
  else
    (cd "${tmp_dir}" && tar --exclude=.git -cf - .) | (cd "${REPO_ROOT}" && tar -xf -)
  fi
  ok "当前目录已覆盖为云端版本"
}

main() {
  parse_args "$@"
  ensure_not_root
  require_cmd git

  if [[ "${REPLACE}" == true ]]; then
    replace_from_remote
    return
  fi

  if git -C "${REPO_ROOT}" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    sync_git_repo
    return
  fi

  die "当前目录不是 git 仓库。需要覆盖同步请使用 --replace"
}

main "$@"
