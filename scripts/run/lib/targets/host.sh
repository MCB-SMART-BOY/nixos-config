# run.sh 主机选择与目标探测

# 列出可用主机。
list_hosts() {
  local repo_dir="$1"
  local host_dir="${repo_dir}/hosts"
  local hosts=()

  # hosts/ 下每个目录都是一个主机（profiles 除外）
  if [[ -d "${host_dir}" ]]; then
    for entry in "${host_dir}"/*; do
      [[ -d "${entry}" ]] || continue
      local name
      name="$(basename "${entry}")"
      [[ "${name}" == "profiles" ]] && continue
      hosts+=("${name}")
    done
  fi

  if [[ ${#hosts[@]} -gt 0 ]]; then
    printf '%s\n' "${hosts[@]}"
  fi
}

# 选择目标主机。
select_host() {
  local repo_dir="$1"
  if [[ -z "${TARGET_NAME}" ]]; then
    # 交互式终端优先走菜单
    if is_tty; then
      local hosts=()
      mapfile -t hosts < <(list_hosts "${repo_dir}")
      if [[ ${#hosts[@]} -eq 0 ]]; then
        error "未找到可用的 hosts 目录。"
      fi
      local default_index=1
      local i=1
      local h
      for h in "${hosts[@]}"; do
        if [[ "${h}" == "nixos" ]]; then
          default_index="${i}"
          break
        fi
        i=$((i + 1))
      done
      local pick
      pick="$(menu_prompt "选择主机" "${default_index}" "${hosts[@]}")"
      TARGET_NAME="${hosts[$((pick - 1))]}"
    else
      # 非交互式则默认使用 nixos
      TARGET_NAME="nixos"
    fi
  fi
}

# 校验主机名合法性。
validate_host() {
  local repo_dir="$1"
  if [[ -z "${TARGET_NAME}" ]]; then
    error "未指定主机名称。"
  fi
  # 确保 hosts/<name> 存在
  if [[ ! -d "${repo_dir}/hosts/${TARGET_NAME}" ]]; then
    error "主机不存在：hosts/${TARGET_NAME}"
  fi
}

# 检测主机 profile 类型（server/desktop/unknown）。
detect_host_profile_kind() {
  local repo_dir="$1"
  local host_file="${repo_dir}/hosts/${TARGET_NAME}/default.nix"
  HOST_PROFILE_KIND="unknown"
  if [[ ! -f "${host_file}" ]]; then
    return 0
  fi
  if grep -qE '\.\./profiles/server\.nix' "${host_file}"; then
    HOST_PROFILE_KIND="server"
  elif grep -qE '\.\./profiles/desktop\.nix' "${host_file}"; then
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

# 检测每用户 TUN 配置是否完整。
detect_per_user_tun() {
  local repo_dir="$1"
  local host_file=""
  local line=""

  # 优先通过 flake 求值读取最终配置（包含 local.nix 覆盖，准确性最高）。
  if command -v nix >/dev/null 2>&1; then
    local nix_config="experimental-features = nix-command flakes"
    if [[ -n "${NIX_CONFIG:-}" ]]; then
      nix_config="${NIX_CONFIG}"$'\n'"${nix_config}"
    fi
    local eval_value=""
    eval_value="$(env NIX_CONFIG="${nix_config}" \
      nix eval --raw "${repo_dir}#nixosConfigurations.${TARGET_NAME}.config.mcb.perUserTun.enable" 2>/dev/null || true)"
    case "${eval_value}" in
      true) return 0 ;;
      false) return 1 ;;
    esac
  fi

  # 回退：文本扫描（兼容无 nix 命令的极简环境）
  for host_file in \
    "${repo_dir}/hosts/${TARGET_NAME}/local.nix" \
    "${repo_dir}/hosts/${TARGET_NAME}/default.nix"
  do
    [[ -f "${host_file}" ]] || continue

    if grep -qE 'mcb\.perUserTun\.enable[[:space:]]*=[[:space:]]*true' "${host_file}" 2>/dev/null; then
      return 0
    fi

    local in_block=0
    while IFS= read -r line; do
      line="${line%%#*}"
      if [[ "${line}" == *"perUserTun"* && "${line}" == *"{"* ]]; then
        in_block=1
      fi
      if [[ ${in_block} -eq 1 && "${line}" == *"enable"* && "${line}" == *"true"* ]]; then
        return 0
      fi
      if [[ ${in_block} -eq 1 && "${line}" == *"}"* ]]; then
        in_block=0
      fi
    done < "${host_file}"
  done

  return 1
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
    files+=("${TMP_DIR}/hosts/${TARGET_NAME}/local.nix")
    files+=("${TMP_DIR}/hosts/${TARGET_NAME}/default.nix")
  fi
  if [[ -n "${ETC_DIR}" && -n "${TARGET_NAME}" ]]; then
    files+=("${ETC_DIR}/hosts/${TARGET_NAME}/local.nix")
    files+=("${ETC_DIR}/hosts/${TARGET_NAME}/default.nix")
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
  printf '%s' "mcbnixos"
}

# 列出仓库中已存在的 Home Manager 用户目录。
list_existing_home_users() {
  local repo_dir="$1"
  local users_dir="${repo_dir}/home/users"
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
