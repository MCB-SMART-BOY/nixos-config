# run.sh GPU 覆盖配置

# 交互式配置 GPU。
configure_gpu() {
  if ! is_tty; then
    reset_gpu_override
    return 0
  fi

  local pick
  pick="$(menu_prompt "GPU 配置方式" 1 "沿用主机配置" "选择 GPU 模式" "返回")"
  case "${pick}" in
    1)
      reset_gpu_override
      return 0
      ;;
    2)
      GPU_OVERRIDE=true
      ;;
    3)
      WIZARD_ACTION="back"
      return 0
      ;;
  esac

  pick="$(menu_prompt "选择 GPU 模式" 2 "核显 (igpu)" "混合 (hybrid)" "独显 (dgpu)" "返回")"
  case "${pick}" in
    1) GPU_MODE="igpu" ;;
    2) GPU_MODE="hybrid" ;;
    3) GPU_MODE="dgpu" ;;
    4)
      WIZARD_ACTION="back"
      return 0
      ;;
  esac

  if [[ "${GPU_MODE}" == "igpu" || "${GPU_MODE}" == "hybrid" ]]; then
    pick="$(menu_prompt "核显厂商" 1 "Intel" "AMD" "返回")"
    case "${pick}" in
      1) GPU_IGPU_VENDOR="intel" ;;
      2) GPU_IGPU_VENDOR="amd" ;;
      3)
        WIZARD_ACTION="back"
        return 0
        ;;
    esac
  fi

  if [[ "${GPU_MODE}" == "hybrid" ]]; then
    pick="$(menu_prompt "PRIME 模式" 1 "offload（推荐，Wayland）" "sync（偏向 X11）" "reverseSync（偏向 X11）" "返回")"
    case "${pick}" in
      1) GPU_PRIME_MODE="offload" ;;
      2) GPU_PRIME_MODE="sync" ;;
      3) GPU_PRIME_MODE="reverseSync" ;;
      4)
        WIZARD_ACTION="back"
        return 0
        ;;
    esac

    # iGPU busId（从检测结果中选择）
    if [[ "${GPU_IGPU_VENDOR}" == "intel" ]]; then
      local intel_candidates=()
      mapfile -t intel_candidates < <(bus_candidates_for_vendor intel)
      if [[ ${#intel_candidates[@]} -eq 0 ]]; then
        pick="$(menu_prompt "未检测到 Intel iGPU busId" 1 "沿用主机配置" "返回")"
        case "${pick}" in
          1)
            reset_gpu_override
            return 0
            ;;
          2)
            WIZARD_ACTION="back"
            return 0
            ;;
        esac
      fi
      local intel_options=("${intel_candidates[@]}" "返回")
      pick="$(menu_prompt "选择 Intel iGPU busId" 1 "${intel_options[@]}")"
      if (( pick == ${#intel_options[@]} )); then
        WIZARD_ACTION="back"
        return 0
      fi
      GPU_INTEL_BUS="${intel_options[$((pick - 1))]}"
    else
      local amd_candidates=()
      mapfile -t amd_candidates < <(bus_candidates_for_vendor amd)
      if [[ ${#amd_candidates[@]} -eq 0 ]]; then
        pick="$(menu_prompt "未检测到 AMD iGPU busId" 1 "沿用主机配置" "返回")"
        case "${pick}" in
          1)
            reset_gpu_override
            return 0
            ;;
          2)
            WIZARD_ACTION="back"
            return 0
            ;;
        esac
      fi
      local amd_options=("${amd_candidates[@]}" "返回")
      pick="$(menu_prompt "选择 AMD iGPU busId" 1 "${amd_options[@]}")"
      if (( pick == ${#amd_options[@]} )); then
        WIZARD_ACTION="back"
        return 0
      fi
      GPU_AMD_BUS="${amd_options[$((pick - 1))]}"
    fi

    # NVIDIA busId（从检测结果中选择）
    local nvidia_candidates=()
    mapfile -t nvidia_candidates < <(bus_candidates_for_vendor nvidia)
    if [[ ${#nvidia_candidates[@]} -eq 0 ]]; then
      pick="$(menu_prompt "未检测到 NVIDIA dGPU busId" 1 "沿用主机配置" "返回")"
      case "${pick}" in
        1)
          reset_gpu_override
          return 0
          ;;
        2)
          WIZARD_ACTION="back"
          return 0
          ;;
      esac
    fi
    local nvidia_options=("${nvidia_candidates[@]}" "返回")
    pick="$(menu_prompt "选择 NVIDIA dGPU busId" 1 "${nvidia_options[@]}")"
    if (( pick == ${#nvidia_options[@]} )); then
      WIZARD_ACTION="back"
      return 0
    fi
    GPU_NVIDIA_BUS="${nvidia_options[$((pick - 1))]}"
  fi

  if [[ "${GPU_MODE}" == "hybrid" || "${GPU_MODE}" == "dgpu" ]]; then
    pick="$(menu_prompt "NVIDIA 使用开源内核模块？" 1 "是（open=true）" "否（open=false）" "返回")"
    case "${pick}" in
      1) GPU_NVIDIA_OPEN="true" ;;
      2) GPU_NVIDIA_OPEN="false" ;;
      3)
        WIZARD_ACTION="back"
        return 0
        ;;
    esac
  fi

  if [[ "${GPU_MODE}" == "hybrid" ]]; then
    pick="$(menu_prompt "生成 GPU specialisation（igpu/hybrid/dgpu）以便切换？" 1 "是" "否" "返回")"
    case "${pick}" in
      1)
        GPU_SPECIALISATIONS_ENABLED=true
        GPU_SPECIALISATIONS_SET=true
        ;;
      2)
        GPU_SPECIALISATIONS_ENABLED=false
        GPU_SPECIALISATIONS_SET=true
        ;;
      3)
        WIZARD_ACTION="back"
        return 0
        ;;
    esac
    if [[ "${GPU_SPECIALISATIONS_ENABLED}" == "true" ]]; then
      GPU_SPECIALISATION_MODES=("igpu" "hybrid" "dgpu")
    fi
  fi
}
