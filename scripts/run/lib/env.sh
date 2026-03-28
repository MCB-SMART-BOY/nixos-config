# run.sh 环境检测 / 脚本自检函数

# 检测是否存在硬件配置文件。
has_any_hardware_config() {
  local etc_dir="$1"
  if [[ -f "${etc_dir}/hardware-configuration.nix" ]]; then
    return 0
  fi
  if [[ -n "${TARGET_NAME}" && -f "${etc_dir}/hosts/${TARGET_NAME}/hardware-configuration.nix" ]]; then
    return 0
  fi
  if [[ -d "${etc_dir}/hosts" ]]; then
    if find "${etc_dir}/hosts" -maxdepth 2 -name hardware-configuration.nix -print -quit 2>/dev/null | grep -q .; then
      return 0
    fi
  fi
  return 1
}

should_require_hardware_config() {
  # rootless + build 仅做构建/评估，不强制要求目标目录存在硬件文件
  if [[ "${ROOTLESS}" == "true" && "${MODE}" == "build" ]]; then
    return 1
  fi
  return 0
}

# 选定主机后检查硬件配置是否存在。
ensure_host_hardware_config() {
  if ! should_require_hardware_config; then
    return 0
  fi
  if [[ -f "${ETC_DIR}/hardware-configuration.nix" ]]; then
    return 0
  fi
  if [[ -n "${TARGET_NAME}" && -f "${ETC_DIR}/hosts/${TARGET_NAME}/hardware-configuration.nix" ]]; then
    return 0
  fi
  error "缺少硬件配置：${ETC_DIR}/hardware-configuration.nix 或 ${ETC_DIR}/hosts/${TARGET_NAME}/hardware-configuration.nix；请先运行 nixos-generate-config。"
}

# 检查环境依赖与权限。
check_env() {
  log "检查环境..."

  # root 直接运行；普通用户依赖 sudo（若不可用则进入 rootless）
  if [[ "${EUID}" -eq 0 ]]; then
    warn "检测到 root，将跳过 sudo。"
    SUDO=""
  else
    if ! command -v sudo >/dev/null 2>&1; then
      warn "未找到 sudo，进入 rootless 模式。"
      SUDO=""
      ROOTLESS=true
    fi
  fi

  if ! command -v git >/dev/null 2>&1; then
    error "未找到 git。"
  fi

  # 确保当前环境是 NixOS
  if ! command -v nixos-rebuild >/dev/null 2>&1; then
    error "未找到 nixos-rebuild。"
  fi

  # sudo 可用性检查（避免容器 no_new_privileges）
  if [[ -n "${SUDO}" ]]; then
    local sudo_check_file=""
    sudo_check_file="$(mktemp)"
    if ! sudo -n true 2>"${sudo_check_file}"; then
      if grep -qi "no new privileges" "${sudo_check_file}" 2>/dev/null; then
        warn "sudo 无法提权（no new privileges），进入 rootless 模式。"
        SUDO=""
        ROOTLESS=true
      else
        warn "sudo 需要交互输入密码，将在需要时提示。"
      fi
    fi
    rm -f "${sudo_check_file}" 2>/dev/null || true
  fi

  # rootless 模式下校验写入路径与 rebuild 模式
  if [[ "${ROOTLESS}" == "true" ]]; then
    if [[ ! -w "${ETC_DIR}" ]]; then
      if is_tty; then
        local alt_dir="${HOME}/.nixos"
        read -r -p "无权限写入 ${ETC_DIR}，改用 ${alt_dir}？ [Y/n] " ans
        case "${ans}" in
          n|N)
            error "无法写入 ${ETC_DIR}，请使用 root 运行或修改权限。"
            ;;
          *)
            ETC_DIR="${alt_dir}"
            ;;
        esac
      else
        ETC_DIR="${HOME}/.nixos"
      fi
      log "rootless 模式使用目录：${ETC_DIR}"
    fi

    if [[ "${MODE}" == "switch" || "${MODE}" == "test" ]]; then
      warn "rootless 模式无法切换系统，将自动改为 build。"
      MODE="build"
    fi
  fi

  # 仅在可切换系统场景强制要求硬件配置；rootless+build 可依赖 host fallback 做评估/构建
  if should_require_hardware_config; then
    if ! has_any_hardware_config "${ETC_DIR}"; then
      error "缺少硬件配置：${ETC_DIR}/hardware-configuration.nix 或 ${ETC_DIR}/hosts/<hostname>/hardware-configuration.nix；请先运行 nixos-generate-config。"
    fi
  else
    note "rootless + build 模式：跳过硬件配置强制检查（仅构建/评估）。"
  fi
}

# 检测脚本的 shebang shell。
script_shebang_shell() {
  # 只允许 bash/sh 作为 shebang，避免执行失败
  local line="$1"
  case "${line}" in
    *"/bash"*|*"env bash"*|*"/sh"*|*"env sh"*) return 0 ;;
    *) return 1 ;;
  esac
}

# 自检脚本的 shebang 兼容性。
self_check_scripts() {
  # 遍历仓库脚本，确保语法与 shebang 合法
  local repo_dir="$1"
  local user_scripts_dir="${repo_dir}/home/users"
  local pkg_scripts_dir="${repo_dir}/pkgs"
  local run_scripts_dir="${repo_dir}/scripts/run"
  local scripts=()

  # 收集 home/users/*/scripts 下的脚本（可执行脚本）
  if [[ -d "${user_scripts_dir}" ]]; then
    mapfile -d '' -t scripts < <(
      find "${user_scripts_dir}" -type f -path "*/scripts/*" -print0 2>/dev/null
    )
  fi

  # 收集 pkgs/*/scripts/*.sh 下的脚本（可执行脚本）
  if [[ -d "${pkg_scripts_dir}" ]]; then
    local pkg_scripts=()
    mapfile -d '' -t pkg_scripts < <(
      find "${pkg_scripts_dir}" -type f -path "*/scripts/*.sh" -print0 2>/dev/null
    )
    if [[ ${#pkg_scripts[@]} -gt 0 ]]; then
      scripts+=("${pkg_scripts[@]}")
    fi
  fi

  # 收集 run.sh 分层脚本（source 片段）
  if [[ -d "${run_scripts_dir}" ]]; then
    local run_lib_scripts=()
    local run_cmd_scripts=()
    mapfile -d '' -t run_lib_scripts < <(
      find "${run_scripts_dir}/lib" -type f -name '*.sh' -print0 2>/dev/null
    )
    mapfile -d '' -t run_cmd_scripts < <(
      find "${run_scripts_dir}/cmd" -type f -name '*.sh' -print0 2>/dev/null
    )
    if [[ ${#run_lib_scripts[@]} -gt 0 ]]; then
      scripts+=("${run_lib_scripts[@]}")
    fi
    if [[ ${#run_cmd_scripts[@]} -gt 0 ]]; then
      scripts+=("${run_cmd_scripts[@]}")
    fi
  fi

  # 也检查本脚本本身
  if [[ -f "${repo_dir}/run.sh" ]]; then
    scripts+=("${repo_dir}/run.sh")
  fi

  if [[ ${#scripts[@]} -eq 0 ]]; then
    warn "未找到可自检脚本（home/users/*/scripts、pkgs/*/scripts/*.sh、scripts/run/*.sh 或 run.sh）"
    return 0
  fi

  log "脚本自检..."

  local errors=0
  local warnings=0
  local file=""
  local rel=""
  local shellcheck_available=false
  local syntax_check_output=""
  local shellcheck_output=""
  syntax_check_output="$(mktemp)"

  if command -v shellcheck >/dev/null 2>&1; then
    shellcheck_available=true
    shellcheck_output="$(mktemp)"
  else
    warn "未检测到 shellcheck，跳过 Lint 检查"
  fi

  for file in "${scripts[@]}"; do
    rel="${file#"${repo_dir}"/}"
    local shebang=""
    local require_exec=true
    local require_shebang=true

    # run.sh 分层脚本是 source 片段，不要求 +x / shebang
    if [[ "${rel}" == scripts/run/lib/* || "${rel}" == scripts/run/cmd/* ]]; then
      require_exec=false
      require_shebang=false
    fi

    if [[ ! -s "${file}" ]]; then
      warn "脚本为空：${rel}"
      warnings=$((warnings + 1))
      continue
    fi

    if [[ "${require_exec}" == "true" && ! -x "${file}" ]]; then
      warn "脚本缺少可执行权限：${rel}"
      errors=$((errors + 1))
    fi

    if LC_ALL=C grep -q $'\r' "${file}"; then
      warn "检测到 CRLF：${rel}"
      errors=$((errors + 1))
    fi

    if [[ "${require_shebang}" == "true" ]]; then
      shebang="$(head -n1 "${file}" 2>/dev/null || true)"
      if [[ "${shebang}" != "#!"* ]]; then
        warn "缺少 shebang：${rel}"
        errors=$((errors + 1))
        continue
      fi

      if ! script_shebang_shell "${shebang}"; then
        warn "非 bash/sh 脚本，跳过语法检查：${rel}"
        warnings=$((warnings + 1))
        continue
      fi
    fi

    if ! bash -n "${file}" 2>"${syntax_check_output}"; then
      warn "语法检查失败：${rel}"
      sed 's/^/  /' "${syntax_check_output}" >&2 || true
      errors=$((errors + 1))
      continue
    fi

    if [[ "${shellcheck_available}" == "true" ]]; then
      if ! shellcheck -x -s bash -e SC1090,SC1091,SC2034,SC2154,SC2329 "${file}" >"${shellcheck_output}" 2>&1; then
        warn "shellcheck 警告：${rel}"
        sed 's/^/  /' "${shellcheck_output}" >&2 || true
        warnings=$((warnings + 1))
      fi
    fi
  done

  rm -f "${syntax_check_output}" 2>/dev/null || true
  if [[ -n "${shellcheck_output}" ]]; then
    rm -f "${shellcheck_output}" 2>/dev/null || true
  fi

  if (( errors > 0 )); then
    error "脚本自检失败：${errors} 个错误（请修复后再继续）"
  fi

  success "脚本自检完成（${warnings} 个警告）"
}
