# run.sh check 命令：运行 flake check（快捷方式）。

check_usage() {
  cat <<'EOF'
用法:
  run.sh check [选项]

说明:
  对当前仓库运行 nix flake check，包含 Nix 求值检查和脚本语法检查。

选项:
  --no-build        跳过构建（仅求值检查，更快）

示例:
  run.sh check                 # 完整检查
  run.sh check --no-build      # 仅求值检查
EOF
}

check_flow() {
  local extra_args=()

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --no-build)
        extra_args+=("--no-build"); shift ;;
      --help|-h)
        check_usage; return 0 ;;
      *)
        warn "未知选项：$1"
        check_usage
        return 2
        ;;
    esac
  done

  section "运行 flake check"

  local repo_dir
  repo_dir="$(detect_local_repo_dir || true)"
  if [[ -z "${repo_dir}" ]]; then
    error "未检测到本地仓库；请在仓库目录中运行此命令。"
  fi

  log "检查仓库：${repo_dir}"
  if nix flake check "${extra_args[@]}" --flake "${repo_dir}"; then
    success "所有检查通过"
  else
    error "检查未通过，请查看上方输出"
  fi
}
