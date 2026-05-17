# run.sh switch 命令：快速重建系统（跳过向导/源码克隆/同步）。
# 适用于日常"改了配置 → 重建"场景，是 deploy 之外最高频的操作。

switch_usage() {
  cat <<'EOF'
用法:
  run.sh switch [选项]

说明:
  直接对 /etc/nixos 执行 nixos-rebuild，跳过向导、源码克隆和同步。
  适合已经部署过、只改了配置文件后快速重建的场景。

选项:
  --host, -H <name>    目标主机（默认自动检测 /etc/nixos 中的主机名）
  --test, -t           使用 nixos-rebuild test（试跑不落地）
  --build, -b          使用 nixos-rebuild build（仅构建不切换）
  --upgrade, -u        重建时升级上游依赖

示例:
  run.sh switch                     # 快速重建当前主机
  run.sh switch --host desktop      # 指定主机重建
  run.sh switch --test              # 试跑不落地
  run.sh switch --upgrade           # 升级依赖并重建
EOF
}

# 从 /etc/nixos 自动检测目标主机名。
detect_etc_host() {
  local etc="${ETC_DIR:-/etc/nixos}"
  if [[ -f "${etc}/flake.nix" ]]; then
    # 单主机架构：固定为 host
    if [[ -d "${etc}" ]]; then
      printf '%s' "host"
      return 0
    fi
  fi
  # 回退：使用当前主机名
  hostname 2>/dev/null || printf '%s' "nixos"
}

switch_flow() {
  local host=""
  local rebuild_mode="switch"
  local upgrade_flag=""

  # 解析参数
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --host|-H)
        host="$2"; shift 2 ;;
      --test|-t)
        rebuild_mode="test"; shift ;;
      --build|-b)
        rebuild_mode="build"; shift ;;
      --upgrade|-u)
        REBUILD_UPGRADE=true; shift ;;
      --help|-h)
        switch_usage; return 0 ;;
      *)
        warn "未知选项：$1"
        switch_usage
        return 2
        ;;
    esac
  done

  # 自动检测主机
  if [[ -z "${host}" ]]; then
    host="$(detect_etc_host)"
    log "自动检测主机：${host}"
  fi

  # 设置 MODE 和 TARGET_NAME 供 rebuild_system 使用
  MODE="${rebuild_mode}"
  TARGET_NAME="${host}"

  section "快速重建"
  note "主机：${TARGET_NAME}，模式：${MODE}"

  # 确认 /etc/nixos 存在
  if [[ ! -d "${ETC_DIR}" || ! -f "${ETC_DIR}/flake.nix" ]]; then
    error "${ETC_DIR} 不存在或不包含 flake.nix；请先运行 run.sh deploy 完成首次部署。"
  fi

  # 直接重建，不经过源码准备/同步/向导
  if ! rebuild_system; then
    if [[ "${DNS_ENABLED}" == false ]]; then
      log "尝试临时切换阿里云 DNS 后重试重建"
      if temp_dns_enable; then
        rebuild_system || error "系统重建失败，请检查日志"
      else
        warn "临时 DNS 设置失败，将继续使用当前 DNS 再重试一次。"
        rebuild_system || error "系统重建失败，请检查日志"
      fi
    else
      error "系统重建失败，请检查日志"
    fi
  fi

  success "重建完成"
}
