# run.sh 服务器覆盖配置

# 交互式配置服务器软件/虚拟化覆盖项。
configure_server_overrides() {
  if [[ "${HOST_PROFILE_KIND}" != "server" ]]; then
    reset_server_overrides
    return 0
  fi

  if ! is_tty; then
    reset_server_overrides
    return 0
  fi

  local pick
  pick="$(menu_prompt "服务器软件配置" 1 "沿用主机配置" "运维服务器预设（CLI + Geek + Docker）" "自定义开关" "返回")"
  case "${pick}" in
    1)
      reset_server_overrides
      return 0
      ;;
    2)
      SERVER_OVERRIDES_ENABLED=true
      SERVER_ENABLE_NETWORK_CLI="true"
      SERVER_ENABLE_NETWORK_GUI="false"
      SERVER_ENABLE_SHELL_TOOLS="true"
      SERVER_ENABLE_WAYLAND_TOOLS="false"
      SERVER_ENABLE_SYSTEM_TOOLS="true"
      SERVER_ENABLE_GEEK_TOOLS="true"
      SERVER_ENABLE_GAMING="false"
      SERVER_ENABLE_INSECURE_TOOLS="false"
      SERVER_ENABLE_DOCKER="true"
      SERVER_ENABLE_LIBVIRTD="false"
      return 0
      ;;
    3)
      SERVER_OVERRIDES_ENABLED=true
      SERVER_ENABLE_NETWORK_CLI="$(ask_bool "启用网络/代理 CLI（mcb.packages.enableNetworkCli）？" "true")"
      SERVER_ENABLE_NETWORK_GUI="$(ask_bool "启用网络图形工具（mcb.packages.enableNetworkGui）？" "false")"
      SERVER_ENABLE_SHELL_TOOLS="$(ask_bool "启用命令行工具组（mcb.packages.enableShellTools）？" "true")"
      SERVER_ENABLE_WAYLAND_TOOLS="$(ask_bool "启用 Wayland 工具组（mcb.packages.enableWaylandTools）？" "false")"
      SERVER_ENABLE_SYSTEM_TOOLS="$(ask_bool "启用系统工具组（mcb.packages.enableSystemTools）？" "true")"
      SERVER_ENABLE_GEEK_TOOLS="$(ask_bool "启用调试/诊断工具（mcb.packages.enableGeekTools）？" "false")"
      SERVER_ENABLE_GAMING="$(ask_bool "启用游戏工具组（mcb.packages.enableGaming）？" "false")"
      SERVER_ENABLE_INSECURE_TOOLS="$(ask_bool "启用不安全软件组（mcb.packages.enableInsecureTools）？" "false")"
      SERVER_ENABLE_DOCKER="$(ask_bool "启用 Docker（mcb.virtualisation.docker.enable）？" "false")"
      SERVER_ENABLE_LIBVIRTD="$(ask_bool "启用 Libvirt/KVM（mcb.virtualisation.libvirtd.enable）？" "false")"
      return 0
      ;;
    4)
      WIZARD_ACTION="back"
      return 0
      ;;
  esac
}
