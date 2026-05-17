# run.sh 主机选择与目标探测（单主机架构）

# 列出可用主机（单主机：固定为 host）。
list_hosts() {
  local repo_dir="$1"
  if [[ -d "${repo_dir}" ]]; then
    echo "host"
  fi
}

# 选择目标主机（单主机：固定为 host）。
select_host() {
  local repo_dir="$1"
  if [[ -z "${TARGET_NAME}" ]]; then
    if is_tty; then
      if [[ -d "${repo_dir}" ]]; then
        TARGET_NAME="host"
        note "单主机模式：目标 = ${TARGET_NAME}"
      else
        error "未找到 host/ 目录。"
      fi
    else
      TARGET_NAME="host"
    fi
  fi
}

# 校验主机名合法性。
validate_host() {
  local repo_dir="$1"
  if [[ -z "${TARGET_NAME}" ]]; then
    error "未指定主机名称。"
  fi
  if [[ ! -d "${repo_dir}" ]]; then
    error "主机目录不存在：host/"
  fi
}

# 检测主机 profile 类型（单主机：从 host/default.nix 检测 hostRole）。
detect_host_profile_kind() {
  local repo_dir="$1"
  local host_file="${repo_dir}/modules/default.nix"
  HOST_PROFILE_KIND="unknown"
  if [[ ! -f "${host_file}" ]]; then
    return 0
  fi
  if grep -qE 'hostRole[[:space:]]*=[[:space:]]*"server"' "${host_file}"; then
    HOST_PROFILE_KIND="server"
  elif grep -qE 'hostRole[[:space:]]*=[[:space:]]*"desktop"' "${host_file}"; then
    HOST_PROFILE_KIND="desktop"
  fi
}

# 询问布尔开关（返回 true/false）。
ask_bool() {
  local prompt="$1"
  local default="${2:-false}"
  if ! is_tty; then
    printf '%s' "${default}"
    return 0
  fi

  local default_index=2
  if [[ "${default}" == "true" ]]; then
    default_index=1
  fi
  local pick
  pick="$(menu_prompt "${prompt}" "${default_index}" "是 (true)" "否 (false)")"
  case "${pick}" in
    1) printf '%s' "true" ;;
    2) printf '%s' "false" ;;
    *) printf '%s' "${default}" ;;
  esac
}


extract_user_from_file() {
  local file="$1"
  local line=""
  line="$(grep -E 'mcb\.user[[:space:]]*=[[:space:]]*.*"[^"]+"' "${file}" 2>/dev/null | head -n1 || true)"
  if [[ -z "${line}" ]]; then
    line="$(grep -E '^[[:space:]]*user[[:space:]]*=[[:space:]]*"[^"]+"' "${file}" 2>/dev/null | head -n1 || true)"
  fi
  if [[ -n "${line}" ]]; then
    printf '%s' "${line}" | sed -E 's/.*"([^"]+)".*/\1/'
  fi
}

resolve_default_user() {
  local files=()
  local file=""
  local value=""

  if [[ -n "${TMP_DIR}" && -n "${TARGET_NAME}" ]]; then
    files+=("${TMP_DIR}/local.nix")
    files+=("${TMP_DIR}/modules/default.nix")
  fi
  if [[ -n "${ETC_DIR}" && -n "${TARGET_NAME}" ]]; then
    files+=("${ETC_DIR}/local.nix")
    files+=("${ETC_DIR}/modules/default.nix")
  fi

  for file in "${files[@]}"; do
    if [[ -f "${file}" ]]; then
      value="$(extract_user_from_file "${file}")"
      if [[ -n "${value}" ]]; then
        printf '%s' "${value}"
        return 0
      fi
    fi
  done
  printf '%s' "admin"
}

# 列出仓库中已存在的 Home Manager 用户目录。
list_existing_home_users() {
  local repo_dir="$1"
  local users_dir="${repo_dir}/users"
  local users=()
  if [[ -d "${users_dir}" ]]; then
    local entry=""
    for entry in "${users_dir}"/*; do
      [[ -d "${entry}" ]] || continue
      local name
      name="$(basename "${entry}")"
      if [[ "${name}" =~ ^[a-z_][a-z0-9_-]*$ ]]; then
        users+=("${name}")
      fi
    done
  fi
  if [[ ${#users[@]} -gt 0 ]]; then
    printf '%s\n' "${users[@]}"
  fi
}
