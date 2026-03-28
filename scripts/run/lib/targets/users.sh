# run.sh 用户/管理员选择函数

# 交互式配置每用户 TUN。
configure_per_user_tun() {
  if [[ "${PER_USER_TUN_ENABLED}" != "true" ]]; then
    return 0
  fi

  if is_tty; then
    # 让用户选择配置方式
    local pick
    pick="$(menu_prompt "TUN 配置方式" 1 "沿用主机配置" "使用默认接口/端口 (tun0/tun1 + 1053..)" "使用常见接口名 (Meta/Mihomo/clash0)" "返回")"
    case "${pick}" in
      4)
        WIZARD_ACTION="back"
        return 0
        ;;
      1)
        reset_tun_maps
        return 0
        ;;
      2)
        # 自动分配 tun0/tun1 + 1053/1054 ...
        reset_tun_maps
        local idx=0
        local user=""
        for user in "${TARGET_USERS[@]}"; do
          USER_TUN["${user}"]="tun${idx}"
          USER_DNS["${user}"]=$((1053 + idx))
          idx=$((idx + 1))
        done
        return 0
        ;;
      3)
        reset_tun_maps
        local idx=0
        local user=""
        local common_ifaces=("Meta" "Mihomo" "clash0" "tun0" "tun1" "tun2")
        for user in "${TARGET_USERS[@]}"; do
          local iface="tun${idx}"
          if (( idx < ${#common_ifaces[@]} )); then
            iface="${common_ifaces[$idx]}"
          fi
          USER_TUN["${user}"]="${iface}"
          USER_DNS["${user}"]=$((1053 + idx))
          idx=$((idx + 1))
        done
        return 0
        ;;
    esac
  else
    reset_tun_maps
    return 0
  fi
}

user_in_list() {
  local needle="$1"
  shift
  local item=""
  for item in "$@"; do
    if [[ "${item}" == "${needle}" ]]; then
      return 0
    fi
  done
  return 1
}

# 向 TARGET_USERS 添加用户（去重）。
add_target_user() {
  local user="$1"
  if ! user_in_list "${user}" "${TARGET_USERS[@]}"; then
    TARGET_USERS+=("${user}")
  fi
}

# 从 TARGET_USERS 移除用户。
remove_target_user() {
  local user="$1"
  local kept=()
  local item=""
  for item in "${TARGET_USERS[@]}"; do
    if [[ "${item}" != "${user}" ]]; then
      kept+=("${item}")
    fi
  done
  TARGET_USERS=("${kept[@]}")
}

# 切换 TARGET_USERS 中用户选中状态。
toggle_target_user() {
  local user="$1"
  if user_in_list "${user}" "${TARGET_USERS[@]}"; then
    remove_target_user "${user}"
  else
    add_target_user "${user}"
  fi
}

# 向 TARGET_ADMIN_USERS 添加用户（去重）。
add_admin_user() {
  local user="$1"
  if ! user_in_list "${user}" "${TARGET_ADMIN_USERS[@]}"; then
    TARGET_ADMIN_USERS+=("${user}")
  fi
}

# 从 TARGET_ADMIN_USERS 移除用户。
remove_admin_user() {
  local user="$1"
  local kept=()
  local item=""
  for item in "${TARGET_ADMIN_USERS[@]}"; do
    if [[ "${item}" != "${user}" ]]; then
      kept+=("${item}")
    fi
  done
  TARGET_ADMIN_USERS=("${kept[@]}")
}

# 切换 TARGET_ADMIN_USERS 中用户选中状态。
toggle_admin_user() {
  local user="$1"
  if user_in_list "${user}" "${TARGET_ADMIN_USERS[@]}"; then
    remove_admin_user "${user}"
  else
    add_admin_user "${user}"
  fi
}

# 从已存在用户中勾选目标用户。
select_existing_users_menu() {
  local users=("$@")
  local pick
  while true; do
    local options=()
    local user=""
    for user in "${users[@]}"; do
      if user_in_list "${user}" "${TARGET_USERS[@]}"; then
        options+=("[x] ${user}")
      else
        options+=("[ ] ${user}")
      fi
    done
    options+=("完成")
    options+=("返回")

    pick="$(menu_prompt "勾选已有用户（可重复切换）" 1 "${options[@]}")"
    if (( pick >= 1 && pick <= ${#users[@]} )); then
      toggle_target_user "${users[$((pick - 1))]}"
      continue
    fi
    if (( pick == ${#users[@]} + 1 )); then
      return 0
    fi
    return 1
  done
}

# 从已选用户中勾选管理员。
select_admin_users_menu() {
  local pick
  while true; do
    local options=()
    local user=""
    for user in "${TARGET_USERS[@]}"; do
      if user_in_list "${user}" "${TARGET_ADMIN_USERS[@]}"; then
        options+=("[x] ${user}")
      else
        options+=("[ ] ${user}")
      fi
    done
    options+=("完成")
    options+=("返回")

    pick="$(menu_prompt "勾选管理员用户（可重复切换）" 1 "${options[@]}")"
    if (( pick >= 1 && pick <= ${#TARGET_USERS[@]} )); then
      toggle_admin_user "${TARGET_USERS[$((pick - 1))]}"
      continue
    fi
    if (( pick == ${#TARGET_USERS[@]} + 1 )); then
      return 0
    fi
    return 1
  done
}

# 交互式输入用户列表。
prompt_users() {
  local default_user=""
  default_user="$(resolve_default_user)"

  if ! is_tty; then
    if [[ ${#TARGET_USERS[@]} -eq 0 ]]; then
      TARGET_USERS=("${default_user}")
    fi
    return 0
  fi

  if [[ ${#TARGET_USERS[@]} -eq 0 ]]; then
    TARGET_USERS=("${default_user}")
  fi

  while true; do
    local current_users="未选择"
    if [[ ${#TARGET_USERS[@]} -gt 0 ]]; then
      current_users="${TARGET_USERS[*]}"
    fi

    local pick
    pick="$(menu_prompt "选择用户（当前：${current_users}）" 1 "仅使用默认用户 (${default_user})" "从已有 Home 用户中选择" "新增用户（手写用户名）" "清空已选用户" "完成" "返回" "退出")"
    case "${pick}" in
      1)
        TARGET_USERS=("${default_user}")
        ;;
      2)
        local existing_users=()
        mapfile -t existing_users < <(list_existing_home_users "${TMP_DIR}" | sort -u)
        if [[ ${#existing_users[@]} -eq 0 ]]; then
          warn "未发现可选的已有 Home 用户目录。"
          continue
        fi
        select_existing_users_menu "${existing_users[@]}" || true
        ;;
      3)
        local input=""
        read -r -p "输入新增用户名（留空取消）： " input
        if [[ -z "${input}" ]]; then
          continue
        fi
        if [[ ! "${input}" =~ ^[a-z_][a-z0-9_-]*$ ]]; then
          warn "用户名不合法：${input}"
          continue
        fi
        add_target_user "${input}"
        ;;
      4)
        TARGET_USERS=()
        ;;
      5)
        if [[ ${#TARGET_USERS[@]} -eq 0 ]]; then
          warn "请至少选择一个用户。"
          continue
        fi
        return 0
        ;;
      6)
        WIZARD_ACTION="back"
        return 0
        ;;
      7)
        error "已退出"
        ;;
    esac
  done
}

# 交互式输入管理员用户列表（wheel）。
prompt_admin_users() {
  local default_admin="${TARGET_USERS[0]}"
  if [[ ${#TARGET_USERS[@]} -eq 0 ]]; then
    error "用户列表为空，无法选择管理员。"
  fi

  if ! is_tty; then
    if [[ ${#TARGET_ADMIN_USERS[@]} -eq 0 ]]; then
      TARGET_ADMIN_USERS=("${default_admin}")
    fi
    return 0
  fi

  if [[ ${#TARGET_ADMIN_USERS[@]} -eq 0 ]]; then
    TARGET_ADMIN_USERS=("${default_admin}")
  fi

  while true; do
    local current_admins="未选择"
    if [[ ${#TARGET_ADMIN_USERS[@]} -gt 0 ]]; then
      current_admins="${TARGET_ADMIN_USERS[*]}"
    fi

    local pick
    pick="$(menu_prompt "管理员权限（wheel，当前：${current_admins}）" 1 "仅主用户 (${default_admin})" "所有用户" "自定义勾选管理员" "清空管理员" "完成" "返回" "退出")"
    case "${pick}" in
      1)
        TARGET_ADMIN_USERS=("${default_admin}")
        ;;
      2)
        TARGET_ADMIN_USERS=("${TARGET_USERS[@]}")
        ;;
      3)
        select_admin_users_menu || true
        ;;
      4)
        TARGET_ADMIN_USERS=()
        ;;
      5)
        if [[ ${#TARGET_ADMIN_USERS[@]} -eq 0 ]]; then
          warn "至少需要一个管理员用户。"
          continue
        fi
        return 0
        ;;
      6)
        WIZARD_ACTION="back"
        return 0
        ;;
      7)
        error "已退出"
        ;;
    esac
  done
}

# 用户列表去重并保持顺序。
dedupe_users() {
  local user
  local -A seen=()
  local unique=()
  for user in "${TARGET_USERS[@]}"; do
    if [[ -z "${seen[${user}]+x}" ]]; then
      unique+=("${user}")
      seen["${user}"]=1
    fi
  done
  TARGET_USERS=("${unique[@]}")
}

# 管理员列表去重并保持顺序。
dedupe_admin_users() {
  local user
  local -A seen=()
  local unique=()
  for user in "${TARGET_ADMIN_USERS[@]}"; do
    if [[ -z "${seen[${user}]+x}" ]]; then
      unique+=("${user}")
      seen["${user}"]=1
    fi
  done
  TARGET_ADMIN_USERS=("${unique[@]}")
}

# 校验用户列表与格式。
validate_users() {
  local user
  for user in "${TARGET_USERS[@]}"; do
    # 只允许 linux 用户名格式
    if [[ ! "${user}" =~ ^[a-z_][a-z0-9_-]*$ ]]; then
      error "用户名不合法：${user}"
    fi
  done
}

# 校验管理员列表：格式合法且必须是用户子集。
validate_admin_users() {
  local user
  if [[ ${#TARGET_ADMIN_USERS[@]} -eq 0 ]]; then
    TARGET_ADMIN_USERS=("${TARGET_USERS[0]}")
  fi
  for user in "${TARGET_ADMIN_USERS[@]}"; do
    if [[ ! "${user}" =~ ^[a-z_][a-z0-9_-]*$ ]]; then
      error "管理员用户名不合法：${user}"
    fi
    if [[ ! " ${TARGET_USERS[*]} " =~ [[:space:]]${user}[[:space:]] ]]; then
      error "管理员用户必须包含在用户列表中：${user}"
    fi
  done
}
