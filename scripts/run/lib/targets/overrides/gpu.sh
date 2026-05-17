# run.sh GPU 配置提示。
# GPU 驱动与 busId 由 nixos-generate-config 写入 hardware-configuration.nix。
# 本项目不再维护 GPU 抽象层，此步骤仅做提示。

configure_gpu() {
  if ! is_tty; then
    reset_gpu_override
    return 0
  fi

  local pick
  pick="$(menu_prompt "GPU 配置" 1 "跳过（使用 nixos-generate-config 生成的 hardware-configuration.nix）" "返回")"
  case "${pick}" in
    1)
      reset_gpu_override
      note "GPU 驱动与 busId 请通过 'sudo nixos-generate-config' 写入 hardware-configuration.nix。"
      return 0
      ;;
    2)
      WIZARD_ACTION="back"
      return 0
      ;;
  esac
}
