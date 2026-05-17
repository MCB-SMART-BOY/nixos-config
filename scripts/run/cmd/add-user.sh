# run.sh add-user 命令：增量添加用户到已有系统。

add_user_usage() {
  cat <<'EOF'
用法:
  run.sh add-user <username> [选项]

说明:
  向已有 NixOS 系统增量添加一个用户。自动生成 Home Manager 模板、
  更新 local.nix，并可选立即重建。

选项:
  --host, -H <name>      目标主机（默认自动检测）
  --admin, -a            设为管理员（加入 wheel 组）
  --copy-from <user>     从已有用户复制 config/assets/scripts
  --no-rebuild           仅生成文件，不重建系统
  --dry-run              预览变更，不实际写入

示例:
  run.sh add-user alice --host desktop --admin
  run.sh add-user bob --copy-from admin
  run.sh add-user ops --no-rebuild
  run.sh add-user eve --admin --dry-run
EOF
}

add_user_flow() {
  local username=""
  local host=""
  local is_admin=false
  local copy_from=""
  local do_rebuild=true
  local dry_run=false

  # 解析参数
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --host|-H)
        host="$2"; shift 2 ;;
      --admin|-a)
        is_admin=true; shift ;;
      --copy-from)
        copy_from="$2"; shift 2 ;;
      --no-rebuild)
        do_rebuild=false; shift ;;
      --dry-run)
        dry_run=true; shift ;;
      --help|-h)
        add_user_usage; return 0 ;;
      -*)
        warn "未知选项：$1"
        add_user_usage
        return 2
        ;;
      *)
        if [[ -z "${username}" ]]; then
          username="$1"; shift
        else
          warn "多余的参数：$1"
          add_user_usage
          return 2
        fi
        ;;
    esac
  done

  # 校验用户名
  if [[ -z "${username}" ]]; then
    error "请提供用户名。用法：run.sh add-user <username>"
  fi

  if [[ ! "${username}" =~ ^[a-z_][a-z0-9_-]*$ ]]; then
    error "用户名不合法：${username}（仅允许小写字母、数字、下划线、连字符，以字母或下划线开头）"
  fi

  # 自动检测主机
  if [[ -z "${host}" ]]; then
    host="$(detect_etc_host)"
    log "自动检测主机：${host}"
  fi

  section "添加用户：${username}"

  # 查找仓库目录
  local repo_dir
  repo_dir="$(detect_local_repo_dir || true)"
  if [[ -z "${repo_dir}" ]]; then
    error "未检测到本地仓库；请在仓库目录中运行此命令。"
  fi

  local user_dir="${repo_dir}/users/${username}"
  local repo_root="${repo_dir}"

  # 检查主机目录
  if [[ ! -d "${repo_root}" ]]; then
    error "主机目录不存在：${repo_root}"
  fi

  # 检查是否已存在
  if [[ -d "${user_dir}" ]]; then
    warn "Home Manager 目录已存在：users/${username}（将跳过模板生成）"
  else
    log "将生成 Home Manager 模板：users/${username}"
  fi

  # 预览模式
  if [[ "${dry_run}" == "true" ]]; then
    note "=== [dry-run] 预览 ==="
    note "用户名：${username}"
    note "主机：${host}"
    note "管理员：${is_admin}"
    [[ -n "${copy_from}" ]] && note "复制模板自：${copy_from}"
    [[ ! -d "${user_dir}" ]] && note "将创建：users/${username}/"
    note "将更新：host/local.nix（追加用户）"
    [[ "${do_rebuild}" == "true" ]] && note "将执行：nixos-rebuild switch"
    note "=== [dry-run] 结束 ==="
    return 0
  fi

  # 设置 TARGET_USERS 供 ensure_user_home_entries 使用
  TARGET_USERS=("${username}")
  TARGET_NAME="${host}"
  HOST_PROFILE_KIND="$(detect_host_profile_kind "${repo_dir}")"

  # 如果指定了 copy-from，设置 COPY_TEMPLATE
  if [[ -n "${copy_from}" ]]; then
    if [[ -d "${repo_dir}/users/${copy_from}" ]]; then
      export RUN_SH_COPY_USER_TEMPLATE=true
      note "将从 users/${copy_from} 复制模板内容"
    else
      warn "模板用户不存在：${copy_from}，将使用默认模板。"
    fi
  fi

  # 生成 Home Manager 模板
  ensure_user_home_entries "${repo_dir}"

  # 增量追加到 local.nix
  append_user_to_local_override "${repo_dir}" "${host}" "${username}" "${is_admin}"

  # 打印后续步骤
  echo ""
  section "后续步骤"
  note "1. 编辑 users/${username}/packages.nix — 添加该用户的软件"
  note "2. 编辑 users/${username}/git.nix — 设置 Git 身份"
  note "3. 可选：从其他用户复制 config/ 目录到 users/${username}/config/"

  if [[ "${do_rebuild}" == "true" ]]; then
    echo ""
    confirm_continue "立即重建系统以创建用户 ${username}？"
    MODE="switch"
    if ! rebuild_system; then
      error "系统重建失败，请检查日志"
    fi
    success "用户 ${username} 已创建并部署"
  else
    note "跳过重建。准备就绪后运行：run.sh switch"
  fi
}
