# run.sh 向导摘要与流程

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

  if [[ "${PER_USER_TUN_ENABLED}" == "true" ]]; then
    if [[ ${#USER_TUN[@]} -gt 0 ]]; then
      printf '%sPer-user TUN：%s%s\n' "${COLOR_BOLD}" "已启用" "${COLOR_RESET}"
      local user
      for user in "${TARGET_USERS[@]}"; do
        printf '  - %s -> %s (DNS %s)\n' "${user}" "${USER_TUN[${user}]}" "${USER_DNS[${user}]}"
      done
    else
      printf '%sPer-user TUN：%s%s\n' "${COLOR_BOLD}" "已启用（沿用主机配置）" "${COLOR_RESET}"
    fi
  else
    printf '%sPer-user TUN：%s%s\n' "${COLOR_BOLD}" "未启用" "${COLOR_RESET}"
  fi

  if [[ "${GPU_OVERRIDE}" == "true" ]]; then
    printf '%sGPU：%s%s\n' "${COLOR_BOLD}" "${GPU_MODE}" "${COLOR_RESET}"
    if [[ -n "${GPU_IGPU_VENDOR}" ]]; then
      printf '  - iGPU 厂商：%s\n' "${GPU_IGPU_VENDOR}"
    fi
    if [[ -n "${GPU_PRIME_MODE}" ]]; then
      printf '  - PRIME：%s\n' "${GPU_PRIME_MODE}"
    fi
    if [[ -n "${GPU_INTEL_BUS}" ]]; then
      printf '  - Intel busId：%s\n' "${GPU_INTEL_BUS}"
    fi
    if [[ -n "${GPU_AMD_BUS}" ]]; then
      printf '  - AMD busId：%s\n' "${GPU_AMD_BUS}"
    fi
    if [[ -n "${GPU_NVIDIA_BUS}" ]]; then
      printf '  - NVIDIA busId：%s\n' "${GPU_NVIDIA_BUS}"
    fi
    if [[ -n "${GPU_NVIDIA_OPEN}" ]]; then
      printf '  - NVIDIA open：%s\n' "${GPU_NVIDIA_OPEN}"
    fi
    if [[ "${GPU_SPECIALISATIONS_ENABLED}" == "true" ]]; then
      printf '  - specialisation：启用 (%s)\n' "${GPU_SPECIALISATION_MODES[*]}"
    fi
  else
    printf '%sGPU：%s%s\n' "${COLOR_BOLD}" "沿用主机配置" "${COLOR_RESET}"
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
        # 选择主机
        select_host "${TMP_DIR}"
        validate_host "${TMP_DIR}"
        detect_host_profile_kind "${TMP_DIR}"
        step=2
        ;;
      2)
        # 选择用户列表
        WIZARD_ACTION=""
        prompt_users
        if [[ "${WIZARD_ACTION}" == "back" ]]; then
          TARGET_USERS=()
          reset_admin_users
          reset_tun_maps
          reset_gpu_override
          reset_server_overrides
          TARGET_NAME=""
          step=1
          continue
        fi
        dedupe_users
        validate_users
        reset_admin_users
        reset_tun_maps
        reset_gpu_override
        reset_server_overrides
        step=3
        ;;
      3)
        # 选择管理员用户（wheel）
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
        # 检测并配置 per-user TUN
        if detect_per_user_tun "${TMP_DIR}"; then
          PER_USER_TUN_ENABLED=true
        else
          PER_USER_TUN_ENABLED=false
        fi
        WIZARD_ACTION=""
        if [[ "${PER_USER_TUN_ENABLED}" == "true" ]]; then
          configure_per_user_tun
          if [[ "${WIZARD_ACTION}" == "back" ]]; then
            reset_tun_maps
            step=3
            continue
          fi
        else
          reset_tun_maps
        fi
        step=5
        ;;
      5)
        # 配置 GPU 覆盖（可选，server 主机默认跳过）
        if [[ "${HOST_PROFILE_KIND}" == "server" ]]; then
          reset_gpu_override
          step=6
          continue
        fi
        WIZARD_ACTION=""
        configure_gpu
        if [[ "${WIZARD_ACTION}" == "back" ]]; then
          reset_gpu_override
          step=4
          continue
        fi
        step=6
        ;;
      6)
        # 服务器软件/虚拟化配置（仅 server profile）
        if [[ "${HOST_PROFILE_KIND}" != "server" ]]; then
          reset_server_overrides
          step=7
          continue
        fi
        WIZARD_ACTION=""
        configure_server_overrides
        if [[ "${WIZARD_ACTION}" == "back" ]]; then
          reset_server_overrides
          step=5
          continue
        fi
        step=7
        ;;
      7)
        # 最终确认
        print_summary
        if is_tty; then
          wizard_back_or_quit "确认以上配置"
          case "${WIZARD_ACTION}" in
            back)
              if [[ "${HOST_PROFILE_KIND}" == "server" ]]; then
                step=6
              else
                step=5
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
