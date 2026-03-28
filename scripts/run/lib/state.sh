# run.sh 状态管理 / GPU bus 工具函数

# 清空每用户 TUN 临时配置。
reset_tun_maps() {
  USER_TUN=()
  USER_DNS=()
}

# 清空管理员用户临时配置。
reset_admin_users() {
  TARGET_ADMIN_USERS=()
}

# 清空服务器软件覆盖配置。
reset_server_overrides() {
  SERVER_OVERRIDES_ENABLED=false
  SERVER_ENABLE_NETWORK_CLI=""
  SERVER_ENABLE_NETWORK_GUI=""
  SERVER_ENABLE_SHELL_TOOLS=""
  SERVER_ENABLE_WAYLAND_TOOLS=""
  SERVER_ENABLE_SYSTEM_TOOLS=""
  SERVER_ENABLE_GEEK_TOOLS=""
  SERVER_ENABLE_GAMING=""
  SERVER_ENABLE_INSECURE_TOOLS=""
  SERVER_ENABLE_DOCKER=""
  SERVER_ENABLE_LIBVIRTD=""
}

# 清空 GPU 临时配置。
reset_gpu_override() {
  GPU_OVERRIDE=false
  GPU_MODE=""
  GPU_IGPU_VENDOR=""
  GPU_PRIME_MODE=""
  GPU_INTEL_BUS=""
  GPU_AMD_BUS=""
  GPU_NVIDIA_BUS=""
  GPU_NVIDIA_OPEN=""
  GPU_SPECIALISATIONS_ENABLED=false
  GPU_SPECIALISATION_MODES=()
  GPU_SPECIALISATIONS_SET=false
}

# 规整 PCI busId（0000:00:02.0 -> PCI:0:2:0）。
strip_leading_zeros() {
  local value="$1"
  value="$(printf '%s' "${value}" | sed -E 's/^0+//')"
  printf '%s' "${value:-0}"
}

normalize_pci_bus_id() {
  local addr="$1"
  local raw="${addr#0000:}"
  local bus="${raw%%:*}"
  local rest="${raw#*:}"
  local dev="${rest%%.*}"
  local func="${rest#*.}"
  bus="$(strip_leading_zeros "${bus}")"
  dev="$(strip_leading_zeros "${dev}")"
  func="$(strip_leading_zeros "${func}")"
  printf 'PCI:%s:%s:%s' "${bus}" "${dev}" "${func}"
}

detect_bus_ids_from_lspci() {
  local vendor="$1"
  if ! command -v lspci >/dev/null 2>&1; then
    return 0
  fi
  local line=""
  while IFS= read -r line; do
    case "${vendor}" in
      intel) [[ "${line}" == *"Intel"* ]] || continue ;;
      amd)
        [[ "${line}" == *"AMD"* || "${line}" == *"Advanced Micro Devices"* ]] || continue
        ;;
      nvidia) [[ "${line}" == *"NVIDIA"* ]] || continue ;;
      *) return 0 ;;
    esac
    local addr="${line%% *}"
    if [[ "${addr}" == *":"*"."* ]]; then
      normalize_pci_bus_id "${addr}"
    fi
  done < <(lspci -D -d ::03xx 2>/dev/null || true)
}

detect_bus_id_from_lspci() {
  local vendor="$1"
  local first=""
  first="$(detect_bus_ids_from_lspci "${vendor}" | head -n1 || true)"
  if [[ -n "${first}" ]]; then
    printf '%s' "${first}"
  fi
}

extract_bus_id_from_file() {
  local file="$1"
  local key="$2"
  local line=""
  line="$(grep -E "${key}[[:space:]]*=[[:space:]]*\"[^\"]+\"" "${file}" 2>/dev/null | head -n1 || true)"
  if [[ -n "${line}" ]]; then
    printf '%s' "${line}" | sed -E 's/.*"([^"]+)".*/\1/'
  fi
}

resolve_bus_id_default() {
  local vendor="$1"
  local detected=""
  detected="$(detect_bus_id_from_lspci "${vendor}" || true)"
  if [[ -n "${detected}" ]]; then
    printf '%s' "${detected}"
    return 0
  fi

  local key=""
  case "${vendor}" in
    intel) key="intelBusId" ;;
    amd) key="amdgpuBusId" ;;
    nvidia) key="nvidiaBusId" ;;
    *) return 0 ;;
  esac

  local files=()
  if [[ -n "${ETC_DIR}" && -n "${TARGET_NAME}" ]]; then
    files+=("${ETC_DIR}/hosts/${TARGET_NAME}/local.nix")
    files+=("${ETC_DIR}/hosts/${TARGET_NAME}/default.nix")
  fi
  if [[ -n "${TMP_DIR}" && -n "${TARGET_NAME}" ]]; then
    files+=("${TMP_DIR}/hosts/${TARGET_NAME}/local.nix")
    files+=("${TMP_DIR}/hosts/${TARGET_NAME}/default.nix")
  fi

  local file=""
  for file in "${files[@]}"; do
    if [[ -f "${file}" ]]; then
      local value=""
      value="$(extract_bus_id_from_file "${file}" "${key}")"
      if [[ -n "${value}" ]]; then
        printf '%s' "${value}"
        return 0
      fi
    fi
  done
}

bus_candidates_for_vendor() {
  local vendor="$1"
  local -A seen=()
  local result=()
  local value=""

  while IFS= read -r value; do
    [[ -n "${value}" ]] || continue
    if [[ -z "${seen[${value}]+x}" ]]; then
      result+=("${value}")
      seen["${value}"]=1
    fi
  done < <(detect_bus_ids_from_lspci "${vendor}" || true)

  local fallback=""
  fallback="$(resolve_bus_id_default "${vendor}" || true)"
  if [[ -n "${fallback}" && -z "${seen[${fallback}]+x}" ]]; then
    result=("${fallback}" "${result[@]}")
  fi

  if [[ ${#result[@]} -gt 0 ]]; then
    printf '%s\n' "${result[@]}"
  fi
}
