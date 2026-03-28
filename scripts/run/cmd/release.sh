# run.sh release 命令链路

default_release_version() {
  local today
  today="$(date +%Y.%m.%d)"
  local base="v${today}"
  local max=-1
  local tag=""
  for tag in $(git tag --list "${base}" "${base}.*" 2>/dev/null); do
    if [[ "${tag}" == "${base}" ]]; then
      max=0
    elif [[ "${tag}" =~ ^${base}\.([0-9]+)$ ]]; then
      local num="${BASH_REMATCH[1]}"
      if (( num > max )); then
        max="${num}"
      fi
    fi
  done
  if (( max >= 0 )); then
    printf '%s' "${base}.$((max + 1))"
  else
    printf '%s' "${base}"
  fi
}

resolve_release_version() {
  local version="${RELEASE_VERSION:-}"
  local default_version=""
  default_version="$(default_release_version)"
  if [[ -z "${version}" ]] && is_tty; then
    read -r -p "请输入发布版本（默认 ${default_version}）： " version
  fi
  if [[ -z "${version}" ]]; then
    version="${default_version}"
  fi
  if [[ "${version}" != v* ]]; then
    version="v${version}"
  fi
  printf '%s' "${version}"
}

find_last_release_tag() {
  git describe --tags --abbrev=0 2>/dev/null || true
}

generate_release_notes() {
  local last_tag="$1"
  local range="HEAD"
  if [[ -n "${last_tag}" ]]; then
    range="${last_tag}..HEAD"
  fi
  local notes=""
  local lines=()
  mapfile -t lines < <(git log --oneline --no-merges "${range}" 2>/dev/null || true)
  if [[ ${#lines[@]} -eq 0 ]]; then
    if [[ -n "${last_tag}" ]]; then
      notes="No code changes since ${last_tag}."
    else
      notes="No code changes found."
    fi
  else
    local header="Changes"
    if [[ -n "${last_tag}" ]]; then
      header="Changes since ${last_tag}"
    fi
    notes="## ${header}\n"
    local line=""
    for line in "${lines[@]}"; do
      notes+="- ${line}\n"
    done
  fi
  printf '%b' "${notes}"
}

update_version_files() {
  local version="$1"
  printf '%s\n' "${version}" > "${SCRIPT_DIR}/VERSION"
  if grep -q '^RUN_SH_VERSION=' "${SCRIPT_DIR}/run.sh"; then
    sed -i "s/^RUN_SH_VERSION=.*/RUN_SH_VERSION=\"${version}\"/" "${SCRIPT_DIR}/run.sh"
  fi
}

release_flow() {
  banner

  if ! command -v git >/dev/null 2>&1; then
    error "未找到 git。"
  fi
  if ! command -v gh >/dev/null 2>&1; then
    error "未找到 GitHub CLI (gh)。"
  fi
  if ! gh auth status >/dev/null 2>&1; then
    error "gh 未登录，请先执行 gh auth login。"
  fi

  cd "${SCRIPT_DIR}" || error "无法进入仓库目录：${SCRIPT_DIR}"
  if [[ ! -d "${SCRIPT_DIR}/.git" ]]; then
    error "当前目录不是 git 仓库：${SCRIPT_DIR}"
  fi

  local dirty
  dirty="$(git status --porcelain)"
  if [[ -n "${dirty}" && "${RELEASE_ALLOW_DIRTY:-false}" != "true" ]]; then
    error "工作区存在未提交变更，发布前请先提交或设置 RELEASE_ALLOW_DIRTY=true。"
  fi

  local version
  version="$(resolve_release_version)"
  if git rev-parse "${version}" >/dev/null 2>&1; then
    error "标签已存在：${version}"
  fi

  local last_tag
  last_tag="$(find_last_release_tag)"
  local notes="${RELEASE_NOTES:-}"
  if [[ -z "${notes}" ]]; then
    notes="$(generate_release_notes "${last_tag}")"
  fi

  if is_tty; then
    printf '\n将发布版本：%s\n' "${version}"
    if [[ -n "${last_tag}" ]]; then
      printf '上一个版本：%s\n' "${last_tag}"
    fi
    printf '\nRelease Notes 预览：\n%s\n' "${notes}"
    confirm_continue "确认发布 ${version}？"
  fi

  update_version_files "${version}"

  git add VERSION run.sh
  if git diff --cached --quiet; then
    warn "VERSION 未变化，跳过版本提交。"
  else
    git commit -m "release: ${version}"
  fi

  git tag -a "${version}" -m "${version}"
  git push origin HEAD
  git push origin "${version}"

  local notes_file=""
  notes_file="$(mktemp)"
  printf '%s\n' "${notes}" > "${notes_file}"
  gh release create "${version}" --title "${version}" --notes-file "${notes_file}"
  rm -f "${notes_file}"

  success "Release 已发布：${version}"
}
