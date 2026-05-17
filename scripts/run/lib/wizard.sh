# run.sh 向导摘要与流程

# 交互式选择部署模式。
prompt_deploy_mode() {
  if [[ "${DEPLOY_MODE_SET}" == "true" || ! -t 0 || ! -t 1 ]]; then
    return 0
  fi
  local pick
  pick="$(menu_prompt "选择部署模式" 1 "新增/调整用户并部署（可修改用户/权限）" "仅更新当前配置（网络仓库最新，不改用户/权限）")"
  case "${pick}" in
    1)
      set_deploy_mode "manage-users"
      ;;
    2)
      set_deploy_mode "update-existing"
      ;;
  esac
}

# 交互式选择覆盖策略。
prompt_overwrite_mode() {
  if [[ "${OVERWRITE_MODE_SET}" == "true" ]]; then
    return 0
  fi
  if ! is_tty; then
    OVERWRITE_MODE="backup"
    OVERWRITE_MODE_SET=true
    return 0
  fi
  local pick
  pick="$(menu_prompt "选择覆盖策略（/etc/nixos 已存在时）" 1 "先备份再覆盖（推荐）" "直接覆盖（不备份）" "执行时再询问")"
  case "${pick}" in
    1) OVERWRITE_MODE="backup" ;;
    2) OVERWRITE_MODE="overwrite" ;;
    3) OVERWRITE_MODE="ask" ;;
  esac
  OVERWRITE_MODE_SET=true
}

# 交互式选择是否在重建时升级上游依赖。
prompt_rebuild_upgrade() {
  if ! is_tty; then
    REBUILD_UPGRADE=false
    return 0
  fi
  REBUILD_UPGRADE="$(ask_bool "重建时升级上游依赖？" "false")"
}

# 交互式选择源代码来源与版本策略。
prompt_source_strategy() {
  if [[ "${SOURCE_CHOICE_SET}" == "true" ]]; then
    return 0
  fi

  local local_repo=""
  local_repo="$(detect_local_repo_dir || true)"

  if ! is_tty; then
    if [[ "${DEPLOY_MODE}" == "update-existing" ]]; then
      FORCE_REMOTE_SOURCE=true
      ALLOW_REMOTE_HEAD=true
      SOURCE_REF=""
    else
      if [[ -n "${local_repo}" ]]; then
        FORCE_REMOTE_SOURCE=false
        ALLOW_REMOTE_HEAD=false
        SOURCE_REF=""
      else
        FORCE_REMOTE_SOURCE=true
        ALLOW_REMOTE_HEAD=false
      fi
    fi
    SOURCE_CHOICE_SET=true
    return 0
  fi

  local options=()
  local default_index=1
  if [[ -n "${local_repo}" ]]; then
    options+=("使用本地仓库（推荐）: ${local_repo}")
  fi
  options+=("使用网络仓库固定版本（输入 commit/tag）")
  options+=("使用网络仓库最新版本（HEAD）")

  if [[ "${DEPLOY_MODE}" == "update-existing" ]]; then
    default_index=${#options[@]}
  fi

  local pick
  pick="$(menu_prompt "选择配置来源" "${default_index}" "${options[@]}")"

  if [[ -n "${local_repo}" && "${pick}" == "1" ]]; then
    FORCE_REMOTE_SOURCE=false
    ALLOW_REMOTE_HEAD=false
    SOURCE_REF=""
  else
    local remote_pick="${pick}"
    if [[ -n "${local_repo}" ]]; then
      remote_pick=$((pick - 1))
    fi
    case "${remote_pick}" in
      1)
        FORCE_REMOTE_SOURCE=true
        ALLOW_REMOTE_HEAD=false
        while true; do
          read -r -p "请输入远端固定版本（commit/tag）： " SOURCE_REF
          if [[ -n "${SOURCE_REF}" ]]; then
            break
          fi
          echo "版本不能为空，请重试。"
        done
        ;;
      2)
        FORCE_REMOTE_SOURCE=true
        ALLOW_REMOTE_HEAD=true
        SOURCE_REF=""
        ;;
    esac
  fi

  SOURCE_CHOICE_SET=true
}

# 校验部署模式与运行时状态是否冲突。
validate_mode_conflicts() {
  if [[ "${DEPLOY_MODE}" == "update-existing" ]]; then
    if [[ ${#TARGET_USERS[@]} -gt 0 ]]; then
      error "仅更新模式不允许修改用户列表；该模式会保留现有用户与权限。"
    fi
  fi
}

# 打印部署摘要与提示。
print_summary() {
  section "部署概要"
  if [[ "${DEPLOY_MODE}" == "update-existing" ]]; then
    printf '%s部署模式：%s%s\n' "${COLOR_BOLD}" "仅更新当前配置（保留用户/权限）" "${COLOR_RESET}"
  else
    printf '%s部署模式：%s%s\n' "${COLOR_BOLD}" "新增/调整用户并部署" "${COLOR_RESET}"
  fi
  printf '%s主机：%s%s\n' "${COLOR_BOLD}" "${TARGET_NAME}" "${COLOR_RESET}"
  if [[ "${DEPLOY_MODE}" == "update-existing" ]]; then
    if [[ -n "${SOURCE_REF}" ]]; then
      printf '%s源策略：%s%s\n' "${COLOR_BOLD}" "网络仓库固定版本 (${SOURCE_REF})" "${COLOR_RESET}"
    else
      printf '%s源策略：%s%s\n' "${COLOR_BOLD}" "网络仓库最新 HEAD" "${COLOR_RESET}"
    fi
    printf '%s用户/权限：%s%s\n' "${COLOR_BOLD}" "保持当前主机 local.nix" "${COLOR_RESET}"
  else
    printf '%s用户：%s%s\n' "${COLOR_BOLD}" "${TARGET_USERS[*]}" "${COLOR_RESET}"
    printf '%s管理员：%s%s\n' "${COLOR_BOLD}" "${TARGET_ADMIN_USERS[*]}" "${COLOR_RESET}"
  fi
  if [[ -n "${SOURCE_COMMIT}" ]]; then
    printf '%s源提交：%s%s\n' "${COLOR_BOLD}" "${SOURCE_COMMIT}" "${COLOR_RESET}"
  fi
  printf '%s覆盖策略：%s%s\n' "${COLOR_BOLD}" "${OVERWRITE_MODE}" "${COLOR_RESET}"
  if [[ "${REBUILD_UPGRADE}" == "true" ]]; then
    printf '%s依赖升级：%s%s\n' "${COLOR_BOLD}" "启用" "${COLOR_RESET}"
  else
    printf '%s依赖升级：%s%s\n' "${COLOR_BOLD}" "关闭" "${COLOR_RESET}"
  fi
  if [[ "${DEPLOY_MODE}" == "update-existing" ]]; then
    return 0
  fi

  if [[ "${GPU_OVERRIDE}" == "true" ]]; then
    printf '%sGPU：%s%s（硬件配置请写入 hardware-configuration.nix）\n' "${COLOR_BOLD}" "${GPU_MODE}" "${COLOR_RESET}"
  else
    printf '%sGPU：%s%s\n' "${COLOR_BOLD}" "使用 hardware-configuration.nix" "${COLOR_RESET}"
  fi

  if [[ "${SERVER_OVERRIDES_ENABLED}" == "true" ]]; then
    printf '%s服务器软件覆盖：%s%s\n' "${COLOR_BOLD}" "已启用" "${COLOR_RESET}"
    printf '  - enableNetworkCli=%s\n' "${SERVER_ENABLE_NETWORK_CLI}"
    printf '  - enableNetworkGui=%s\n' "${SERVER_ENABLE_NETWORK_GUI}"
    printf '  - enableShellTools=%s\n' "${SERVER_ENABLE_SHELL_TOOLS}"
    printf '  - enableWaylandTools=%s\n' "${SERVER_ENABLE_WAYLAND_TOOLS}"
    printf '  - enableSystemTools=%s\n' "${SERVER_ENABLE_SYSTEM_TOOLS}"
    printf '  - enableGeekTools=%s\n' "${SERVER_ENABLE_GEEK_TOOLS}"
    printf '  - enableGaming=%s\n' "${SERVER_ENABLE_GAMING}"
    printf '  - enableInsecureTools=%s\n' "${SERVER_ENABLE_INSECURE_TOOLS}"
    printf '  - docker.enable=%s\n' "${SERVER_ENABLE_DOCKER}"
    printf '  - libvirtd.enable=%s\n' "${SERVER_ENABLE_LIBVIRTD}"
  fi
}

# 交互式向导主流程。
wizard_flow() {
  local step=1
  WIZARD_ACTION=""

  if [[ "${DEPLOY_MODE}" == "update-existing" ]]; then
    while true; do
      case "${step}" in
        1)
          select_host "${TMP_DIR}"
          validate_host "${TMP_DIR}"
          detect_host_profile_kind "${TMP_DIR}"
          step=2
          ;;
        2)
          print_summary
          if is_tty; then
            wizard_back_or_quit "确认仅更新当前配置并继续？"
            case "${WIZARD_ACTION}" in
              back)
                TARGET_NAME=""
                step=1
                ;;
              continue)
                return 0
                ;;
              *)
                return 0
                ;;
            esac
          else
            return 0
          fi
          ;;
      esac
    done
  fi

  while true; do
    case "${step}" in
      1)
        select_host "${TMP_DIR}"
        validate_host "${TMP_DIR}"
        detect_host_profile_kind "${TMP_DIR}"
        step=2
        ;;
      2)
        WIZARD_ACTION=""
        prompt_users
        if [[ "${WIZARD_ACTION}" == "back" ]]; then
          TARGET_USERS=()
          reset_admin_users
          reset_gpu_override
          reset_server_overrides
          TARGET_NAME=""
          step=1
          continue
        fi
        dedupe_users
        validate_users
        reset_admin_users
        reset_gpu_override
        reset_server_overrides
        step=3
        ;;
      3)
        WIZARD_ACTION=""
        prompt_admin_users
        if [[ "${WIZARD_ACTION}" == "back" ]]; then
          reset_admin_users
          step=2
          continue
        fi
        dedupe_admin_users
        validate_admin_users
        step=4
        ;;
      4)
        if [[ "${HOST_PROFILE_KIND}" == "server" ]]; then
          reset_gpu_override
          step=5
          continue
        fi
        WIZARD_ACTION=""
        configure_gpu
        if [[ "${WIZARD_ACTION}" == "back" ]]; then
          reset_gpu_override
          step=3
          continue
        fi
        step=5
        ;;
      5)
        if [[ "${HOST_PROFILE_KIND}" != "server" ]]; then
          reset_server_overrides
          step=6
          continue
        fi
        WIZARD_ACTION=""
        configure_server_overrides
        if [[ "${WIZARD_ACTION}" == "back" ]]; then
          reset_server_overrides
          step=4
          continue
        fi
        step=6
        ;;
      6)
        print_summary
        if is_tty; then
          wizard_back_or_quit "确认以上配置"
          case "${WIZARD_ACTION}" in
            back)
              if [[ "${HOST_PROFILE_KIND}" == "server" ]]; then
                step=5
              else
                step=4
              fi
              ;;
            continue)
              return 0
              ;;
            *)
              return 0
              ;;
          esac
        else
          return 0
        fi
        ;;
    esac
  done
}
