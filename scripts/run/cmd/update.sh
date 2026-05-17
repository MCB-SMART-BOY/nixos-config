# run.sh update 命令：更新 flake 输入并可选重建。

update_usage() {
  cat <<'EOF'
用法:
  run.sh update [选项]

说明:
  更新 flake.lock 中的上游输入（nixpkgs、home-manager 等），
  可选择是否随后重建系统。

选项:
  --host, -H <name>    目标主机（用于后续重建，默认自动检测）
  --rebuild, -r        更新后立即重建系统
  --no-rebuild         仅更新 flake.lock，不重建（默认）
  --commit, -c         更新后提交 flake.lock（需要 git）

示例:
  run.sh update                       # 仅更新 flake.lock
  run.sh update --rebuild             # 更新并重建
  run.sh update --rebuild --host desktop
  run.sh update --commit              # 更新并提交（CI 友好）
EOF
}

update_flow() {
  local host=""
  local do_rebuild=false
  local do_commit=false

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --host|-H)
        host="$2"; shift 2 ;;
      --rebuild|-r)
        do_rebuild=true; shift ;;
      --no-rebuild)
        do_rebuild=false; shift ;;
      --commit|-c)
        do_commit=true; shift ;;
      --help|-h)
        update_usage; return 0 ;;
      *)
        warn "未知选项：$1"
        update_usage
        return 2
        ;;
    esac
  done

  section "更新 flake 输入"

  # 检查是否在仓库目录中
  local repo_dir
  repo_dir="$(detect_local_repo_dir || true)"
  if [[ -z "${repo_dir}" ]]; then
    error "未检测到本地仓库；请在仓库目录中运行此命令。"
  fi

  log "更新 flake.lock..."
  if ! nix flake update --flake "${repo_dir}"; then
    error "flake update 失败"
  fi
  success "flake.lock 已更新"

  if [[ "${do_commit}" == "true" ]]; then
    if git -C "${repo_dir}" diff --quiet flake.lock; then
      note "flake.lock 无变化，跳过提交。"
    else
      git -C "${repo_dir}" add flake.lock
      git -C "${repo_dir}" commit -m "chore: update flake.lock"
      success "已提交 flake.lock"
    fi
  fi

  if [[ "${do_rebuild}" == "true" ]]; then
    if [[ -z "${host}" ]]; then
      host="$(detect_etc_host)"
      log "自动检测主机：${host}"
    fi
    MODE="switch"
    TARGET_NAME="${host}"
    REBUILD_UPGRADE=true
    section "重建系统"
    if ! rebuild_system; then
      error "系统重建失败，请检查日志"
    fi
    success "更新并重建完成"
  fi
}
