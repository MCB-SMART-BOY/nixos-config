# run.sh clean 命令：清理旧备份目录。

clean_usage() {
  cat <<'EOF'
用法:
  run.sh clean [选项]

说明:
  删除 /etc/nixos 的旧备份目录（/etc/nixos.backup-*）。

选项:
  --keep, -k <n>      保留最近 n 个备份（默认 3）
  --dry-run           显示将要删除的目录，不实际删除

示例:
  run.sh clean                    # 保留最近 3 个备份
  run.sh clean --keep 5           # 保留最近 5 个
  run.sh clean --dry-run          # 预览
EOF
}

clean_flow() {
  local keep=3
  local dry_run=false

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --keep|-k)
        keep="$2"; shift 2 ;;
      --dry-run)
        dry_run=true; shift ;;
      --help|-h)
        clean_usage; return 0 ;;
      *)
        warn "未知选项：$1"
        clean_usage
        return 2
        ;;
    esac
  done

  section "清理旧备份"

  local backup_pattern="${ETC_DIR}.backup-"*
  local backups=()
  local dir

  # shellcheck disable=SC2012
  for dir in $(ls -1dt ${backup_pattern} 2>/dev/null || true); do
    if [[ -d "${dir}" ]]; then
      backups+=("${dir}")
    fi
  done

  if [[ ${#backups[@]} -eq 0 ]]; then
    note "没有找到备份目录。"
    return 0
  fi

  note "共找到 ${#backups[@]} 个备份目录"

  if [[ ${#backups[@]} -le ${keep} ]]; then
    note "备份数 (${#backups[@]}) 不超过保留数 (${keep})，无需清理。"
    return 0
  fi

  local to_delete=("${backups[@]:${keep}}")
  local count=${#to_delete[@]}

  for dir in "${to_delete[@]}"; do
    if [[ "${dry_run}" == "true" ]]; then
      log "[dry-run] 将删除：${dir}"
    else
      log "删除：${dir}"
      as_root rm -rf "${dir}"
    fi
  done

  if [[ "${dry_run}" == "true" ]]; then
    note "[dry-run] 将删除 ${count} 个旧备份，保留 ${keep} 个最新。"
  else
    success "已删除 ${count} 个旧备份，保留 ${keep} 个最新。"
  fi
}
