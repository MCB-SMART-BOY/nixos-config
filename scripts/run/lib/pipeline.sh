# run.sh 部署管线函数

# 备份 /etc/nixos 到时间戳目录。
backup_etc() {
  # 备份目录按时间戳命名，便于回滚
  local timestamp
  timestamp="$(date +%Y%m%d-%H%M%S)"
  local backup_dir="${ETC_DIR}.backup-${timestamp}"
  log "备份 ${ETC_DIR} -> ${backup_dir}"
  as_root mkdir -p "${backup_dir}"
  if command -v rsync >/dev/null 2>&1; then
    as_root rsync -a "${ETC_DIR}/" "${backup_dir}/"
  else
    as_root cp -a "${ETC_DIR}/." "${backup_dir}/"
  fi
  success "备份完成"
}

# 准备 /etc/nixos 目录。
prepare_etc_dir() {
  # 当目录已存在时，根据策略决定是否备份/覆盖
  if [[ -d "${ETC_DIR}" && -n "$(ls -A "${ETC_DIR}" 2>/dev/null)" ]]; then
    case "${OVERWRITE_MODE}" in
      backup)
        backup_etc
        ;;
      overwrite)
        note "将覆盖 ${ETC_DIR}（未启用备份）"
        ;;
      ask)
        if is_tty; then
          while true; do
            read -r -p "检测到 ${ETC_DIR} 已存在，选择 [b]备份并覆盖/[o]直接覆盖/[q]退出（默认 b）： " answer
            case "${answer}" in
              b|B|"")
                backup_etc
                OVERWRITE_MODE="backup"
                break
                ;;
              o|O)
                OVERWRITE_MODE="overwrite"
                break
                ;;
              q|Q)
                error "已退出"
                ;;
              *)
                echo "无效选择，请重试。"
                ;;
            esac
          done
        else
          backup_etc
          OVERWRITE_MODE="backup"
        fi
        ;;
      *)
        error "不支持的覆盖策略：${OVERWRITE_MODE}"
        ;;
    esac
  fi
}

# 清理 /etc/nixos，保留硬件配置文件。
clean_etc_dir_keep_hardware() {
  if [[ -z "${ETC_DIR}" || "${ETC_DIR}" == "/" ]]; then
    error "ETC_DIR 无效，拒绝清理：${ETC_DIR}"
  fi
  if [[ ! -d "${ETC_DIR}" ]]; then
    return 0
  fi

  local preserve_dir
  preserve_dir="$(mktemp -d)"

  if [[ -f "${ETC_DIR}/hardware-configuration.nix" ]]; then
    as_root cp -a "${ETC_DIR}/hardware-configuration.nix" "${preserve_dir}/"
  fi

  if [[ -d "${ETC_DIR}/hosts" ]]; then
    while IFS= read -r -d '' file; do
      local rel="${file#"${ETC_DIR}"/}"
      as_root mkdir -p "${preserve_dir}/$(dirname "${rel}")"
      as_root cp -a "${file}" "${preserve_dir}/${rel}"
    done < <(find "${ETC_DIR}/hosts" -maxdepth 2 -name hardware-configuration.nix -print0 2>/dev/null)
  fi

  as_root find "${ETC_DIR}" -mindepth 1 -maxdepth 1 -exec rm -rf {} +

  if [[ -f "${preserve_dir}/hardware-configuration.nix" ]]; then
    as_root cp -a "${preserve_dir}/hardware-configuration.nix" "${ETC_DIR}/"
  fi

  if [[ -d "${preserve_dir}/hosts" ]]; then
    as_root mkdir -p "${ETC_DIR}/hosts"
    as_root cp -a "${preserve_dir}/hosts/." "${ETC_DIR}/hosts/"
  fi

  rm -rf "${preserve_dir}"
}

# 检测默认网卡名称。
detect_default_iface() {
  # 读取默认路由对应的网卡
  if command -v ip >/dev/null 2>&1; then
    ip route show default 2>/dev/null | awk 'NR==1 {print $5; exit}'
  fi
}

TEMP_DNS_BACKEND=""
TEMP_DNS_BACKUP=""
TEMP_DNS_IFACE=""

# 临时启用 DNS 以修复网络。
temp_dns_enable() {
  local servers=("223.5.5.5" "223.6.6.6")
  local iface=""

  if [[ "${ROOTLESS}" == "true" ]]; then
    warn "rootless 模式无法临时设置 DNS，跳过。"
    return 1
  fi

  # 优先通过 systemd-resolved 临时设置 DNS
  if command -v resolvectl >/dev/null 2>&1 && command -v systemctl >/dev/null 2>&1; then
    if systemctl is-active --quiet systemd-resolved; then
      iface="$(detect_default_iface)"

      if [[ -n "${iface}" ]]; then
        log "临时 DNS（resolvectl ${iface}）：${servers[*]}"
        as_root resolvectl dns "${iface}" "${servers[@]}"
        as_root resolvectl domain "${iface}" "~."
        TEMP_DNS_BACKEND="resolvectl"
        TEMP_DNS_IFACE="${iface}"
        DNS_ENABLED=true
        return 0
      fi
    fi
  fi

  # 兜底方案：直接写 /etc/resolv.conf
  if [[ -f /etc/resolv.conf ]]; then
    TEMP_DNS_BACKUP="$(mktemp)"
    as_root cp -a /etc/resolv.conf "${TEMP_DNS_BACKUP}"
    as_root rm -f /etc/resolv.conf
    printf 'nameserver %s\n' "${servers[@]}" | as_root tee /etc/resolv.conf >/dev/null
    log "临时 DNS（/etc/resolv.conf）：${servers[*]}"
    TEMP_DNS_BACKEND="resolv.conf"
    DNS_ENABLED=true
    return 0
  fi

  error "无法设置临时 DNS（无 resolvectl 且缺少 /etc/resolv.conf）。"
}

# 恢复系统 DNS 设置。
temp_dns_disable() {
  if [[ "${TEMP_DNS_BACKEND}" == "resolvectl" ]]; then
    if [[ -n "${TEMP_DNS_IFACE}" ]]; then
      log "恢复 DNS（resolvectl ${TEMP_DNS_IFACE}）"
      as_root resolvectl revert "${TEMP_DNS_IFACE}" || true
      as_root resolvectl flush-caches >/dev/null 2>&1 || true
    fi
  elif [[ "${TEMP_DNS_BACKEND}" == "resolv.conf" ]]; then
    if [[ -n "${TEMP_DNS_BACKUP}" && -f "${TEMP_DNS_BACKUP}" ]]; then
      log "恢复 /etc/resolv.conf"
      as_root cp -a "${TEMP_DNS_BACKUP}" /etc/resolv.conf || true
      rm -f "${TEMP_DNS_BACKUP}"
    fi
  fi
}

# 检测本地仓库目录（优先当前目录，其次脚本所在目录）。
detect_local_repo_dir() {
  local candidates=(
    "$(pwd)"
    "${SCRIPT_DIR}"
  )
  local dir
  for dir in "${candidates[@]}"; do
    if [[ -f "${dir}/flake.nix" && -d "${dir}/hosts" && -d "${dir}/modules" && -d "${dir}/home" ]]; then
      printf '%s' "${dir}"
      return 0
    fi
  done
  return 1
}

# 未使用本地仓库时，要求固定远端来源版本（除非显式允许跟随远端 HEAD）。
require_remote_source_pin() {
  if [[ "${ALLOW_REMOTE_HEAD}" == "true" ]]; then
    warn "当前将跟随远端分支最新提交（存在供应链风险）。"
    return 0
  fi
  if [[ -z "${SOURCE_REF}" ]]; then
    error "未检测到本地仓库，且未选择远端固定版本；请在向导中选择固定版本或明确选择远端最新版本。"
  fi
}

# 使用本地仓库作为部署源，避免依赖远端浮动分支。
prepare_local_source() {
  local tmp_dir="$1"
  local source_dir="$2"
  log "使用本地仓库：${source_dir}"
  rm -rf "${tmp_dir}"
  mkdir -p "${tmp_dir}"
  if command -v rsync >/dev/null 2>&1; then
    rsync -a --exclude '.git/' "${source_dir}/" "${tmp_dir}/"
  else
    (cd "${source_dir}" && tar --exclude=.git -cf - .) | tar -C "${tmp_dir}" -xf -
  fi
  if git -C "${source_dir}" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    SOURCE_COMMIT="$(git -C "${source_dir}" rev-parse HEAD 2>/dev/null || true)"
  fi
  if [[ -n "${SOURCE_COMMIT}" ]]; then
    note "本地源提交：${SOURCE_COMMIT}"
  fi
}

# 带超时保护执行 git 命令（避免镜像无响应时长时间卡住）。
run_git_with_timeout() {
  local timeout_sec="$1"
  shift
  if command -v timeout >/dev/null 2>&1; then
    timeout --foreground "${timeout_sec}" "$@"
  else
    "$@"
  fi
}

# 克隆配置仓库。
clone_repo() {
  local tmp_dir="$1"
  local url="$2"
  local clone_timeout="${GIT_CLONE_TIMEOUT_SEC}"

  if [[ -n "${SOURCE_REF}" ]]; then
    log "拉取仓库：${url}（固定 ref: ${SOURCE_REF}，超时 ${clone_timeout}s）"
    # 固定 ref 模式：完整克隆后切到指定提交/标签，避免跟随浮动分支。
    if run_git_with_timeout "${clone_timeout}" \
      env GIT_TERMINAL_PROMPT=0 git -c http.lowSpeedLimit=1024 -c http.lowSpeedTime=20 clone "${url}" "${tmp_dir}"; then
      if env GIT_TERMINAL_PROMPT=0 git -C "${tmp_dir}" checkout --detach "${SOURCE_REF}" >/dev/null 2>&1; then
        SOURCE_COMMIT="$(git -C "${tmp_dir}" rev-parse HEAD 2>/dev/null || true)"
        success "仓库拉取完成（${SOURCE_COMMIT}）"
        return 0
      fi
      warn "已拉取仓库，但 checkout 失败：${url}（ref: ${SOURCE_REF}）"
      return 1
    fi
    local rc=$?
    if [[ ${rc} -eq 124 ]]; then
      warn "仓库拉取超时：${url}（${clone_timeout}s）"
    fi
    warn "仓库拉取或 checkout 失败：${url}（ref: ${SOURCE_REF}）"
    return 1
  fi

  log "拉取仓库：${url}（${BRANCH}，超时 ${clone_timeout}s）"
  # 仅在显式允许时使用远端分支 HEAD。
  if run_git_with_timeout "${clone_timeout}" \
    env GIT_TERMINAL_PROMPT=0 git -c http.lowSpeedLimit=1024 -c http.lowSpeedTime=20 clone --depth 1 --branch "${BRANCH}" "${url}" "${tmp_dir}"; then
    SOURCE_COMMIT="$(git -C "${tmp_dir}" rev-parse HEAD 2>/dev/null || true)"
    success "仓库拉取完成（${SOURCE_COMMIT}）"
    return 0
  fi
  local rc=$?
  if [[ ${rc} -eq 124 ]]; then
    warn "仓库拉取超时：${url}（${clone_timeout}s）"
  fi
  warn "仓库拉取失败：${url}"
  return 1
}

# 尝试多个镜像地址克隆。
clone_repo_any() {
  local tmp_dir="$1"
  local url
  local index=0
  local total="${#REPO_URLS[@]}"
  SOURCE_COMMIT=""
  # 依次尝试多个镜像
  for url in "${REPO_URLS[@]}"; do
    index=$((index + 1))
    note "尝试镜像 (${index}/${total})：${url}"
    rm -rf "${tmp_dir}"
    mkdir -p "${tmp_dir}"
    if clone_repo "${tmp_dir}" "${url}"; then
      return 0
    fi
  done
  return 1
}

# 远端仓库拉取：失败后可选启用临时 DNS 再重试一次。
clone_repo_any_with_dns_retry() {
  local tmp_dir="$1"
  if clone_repo_any "${tmp_dir}"; then
    return 0
  fi

  log "尝试临时切换阿里云 DNS 后重试"
  if ! temp_dns_enable; then
    warn "临时 DNS 设置失败，将继续使用当前 DNS 再重试一次。"
  fi

  rm -rf "${tmp_dir}"
  mkdir -p "${tmp_dir}"
  clone_repo_any "${tmp_dir}"
}

# 准备源代码（本地或远端），失败时返回非 0。
prepare_source_repo() {
  local tmp_dir="$1"

  if [[ "${FORCE_REMOTE_SOURCE}" == "true" ]]; then
    require_remote_source_pin
    clone_repo_any_with_dns_retry "${tmp_dir}"
    return $?
  fi

  local source_dir=""
  source_dir="$(detect_local_repo_dir || true)"
  if [[ -n "${source_dir}" ]]; then
    prepare_local_source "${tmp_dir}" "${source_dir}"
    return 0
  fi

  require_remote_source_pin
  clone_repo_any_with_dns_retry "${tmp_dir}"
}

# 同步仓库到 /etc/nixos。
sync_repo_to_etc() {
  local repo_dir="$1"
  local delete_flags=()
  if [[ "${OVERWRITE_MODE}" == "overwrite" || "${OVERWRITE_MODE}" == "backup" ]]; then
    delete_flags=(--delete)
  fi
  log "同步到 ${ETC_DIR}"
  as_root mkdir -p "${ETC_DIR}"

  # 同步时排除 .git 与硬件配置，避免覆盖本机硬件配置
  if command -v rsync >/dev/null 2>&1; then
    as_root rsync -a \
      "${delete_flags[@]}" \
      --exclude '.git/' \
      --exclude 'hardware-configuration.nix' \
      --exclude 'hosts/*/hardware-configuration.nix' \
      "${repo_dir}/" "${ETC_DIR}/"
  else
    if [[ ${#delete_flags[@]} -gt 0 ]]; then
      clean_etc_dir_keep_hardware
    fi
    (cd "${repo_dir}" && tar --exclude=.git --exclude=hardware-configuration.nix --exclude=hosts/*/hardware-configuration.nix -cf - .) | as_root tar -C "${ETC_DIR}" -xf -
  fi

  success "配置同步完成"
}

# 执行 nixos-rebuild（switch/test/build）。
rebuild_system() {
  log "重建系统（${MODE}），目标：${TARGET_NAME}"
  local nix_config="experimental-features = nix-command flakes"
  # 默认不带 --upgrade，显式请求时再升级上游依赖
  local rebuild_args=("${MODE}" "--show-trace")
  if [[ "${REBUILD_UPGRADE}" == "true" ]]; then
    rebuild_args+=("--upgrade")
  fi
  # 合并外部 NIX_CONFIG（如用户自定义缓存）
  if [[ -n "${NIX_CONFIG:-}" ]]; then
    nix_config="${NIX_CONFIG}"$'\n'"${nix_config}"
  fi
  if [[ -n "${SUDO}" ]]; then
    if sudo -E env NIX_CONFIG="${nix_config}" nixos-rebuild "${rebuild_args[@]}" --flake "${ETC_DIR}#${TARGET_NAME}"; then
      success "系统重建完成"
      return 0
    fi
  else
    if env NIX_CONFIG="${nix_config}" nixos-rebuild "${rebuild_args[@]}" --flake "${ETC_DIR}#${TARGET_NAME}"; then
      success "系统重建完成"
      return 0
    fi
  fi
  warn "系统重建失败"
  return 1
}
