# run.sh deploy 命令链路

deploy_flow() {
  banner
  prompt_deploy_mode
  validate_mode_conflicts
  prompt_overwrite_mode
  prompt_rebuild_upgrade
  prompt_source_strategy

  if [[ -n "${SOURCE_REF}" && "${ALLOW_REMOTE_HEAD}" == "true" ]]; then
    warn "检测到来源策略冲突，将优先使用固定版本。"
    ALLOW_REMOTE_HEAD=false
  fi
  section "环境检查"
  check_env
  progress_step "环境检查"

  TMP_DIR="$(mktemp -d)"

  cleanup() {
    local status=$?
    temp_dns_disable
    if [[ -n "${TMP_DIR}" ]]; then
      rm -rf "${TMP_DIR}"
    fi
    exit "${status}"
  }
  trap cleanup EXIT

  # 按模式准备源代码：默认优先本地；仅更新模式强制走远端。
  section "准备源代码"
  while true; do
    if prepare_source_repo "${TMP_DIR}"; then
      break
    fi

    if ! is_tty; then
      error "仓库拉取失败，请检查网络或更换来源策略"
    fi

    local retry_pick
    retry_pick="$(menu_prompt "准备源代码失败，下一步" 1 "重试当前来源" "重新选择来源策略" "退出")"
    case "${retry_pick}" in
      1)
        continue
        ;;
      2)
        SOURCE_CHOICE_SET=false
        prompt_source_strategy
        continue
        ;;
      3)
        error "已退出"
        ;;
    esac
  done
  progress_step "准备源代码"

  section "脚本自检"
  self_check_scripts "${TMP_DIR}"
  progress_step "脚本自检"

  # 交互式向导：选择主机/用户/TUN
  wizard_flow
  if [[ "${DEPLOY_MODE}" == "update-existing" ]]; then
    preserve_existing_local_override "${TMP_DIR}"
  else
    ensure_user_home_entries "${TMP_DIR}"
    if [[ ${#CREATED_HOME_USERS[@]} -gt 0 ]]; then
      warn "已自动创建用户 Home Manager 模板：${CREATED_HOME_USERS[*]}"
    fi
    write_local_override "${TMP_DIR}"
  fi
  ensure_host_hardware_config
  progress_step "收集配置"
  confirm_continue "确认以上配置并继续同步？"
  section "同步与构建"
  prepare_etc_dir
  progress_step "准备覆盖策略"

  sync_repo_to_etc "${TMP_DIR}"
  progress_step "同步配置"
  confirm_continue "配置已同步，继续重建系统？"
  if ! rebuild_system; then
    if [[ "${DNS_ENABLED}" == false ]]; then
      log "尝试临时切换阿里云 DNS 后重试重建"
      if ! temp_dns_enable; then
        warn "临时 DNS 设置失败，将继续使用当前 DNS 重试重建。"
      fi
      if ! rebuild_system; then
        error "系统重建失败，请检查日志"
      fi
    else
      error "系统重建失败，请检查日志"
    fi
  fi
  progress_step "系统重建"
}
