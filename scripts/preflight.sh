#!/usr/bin/env bash
# Preflight checks for this NixOS config repository.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./lib.sh
source "${SCRIPT_DIR}/lib.sh"

HOST_FILE="${REPO_ROOT}/host.nix"
FLAKE_FILE="${REPO_ROOT}/flake.nix"
HOME_FILE="${REPO_ROOT}/home/home.nix"
HARDWARE_FILE="${REPO_ROOT}/hardware-configuration.nix"
ETC_HARDWARE="/etc/nixos/hardware-configuration.nix"
NIRI_CONFIG="${REPO_ROOT}/home/config/niri/config.kdl"
PACKAGES_FILE="${REPO_ROOT}/home/modules/packages.nix"
I18N_FILE="${REPO_ROOT}/modules/i18n.nix"
DESKTOP_FILE="${REPO_ROOT}/home/modules/desktop.nix"
WALLPAPER_SCRIPT="${REPO_ROOT}/home/scripts/wallpaper-random"
WALLPAPER_DIR="${REPO_ROOT}/home/assets/wallpapers"
FASTFETCH_CONFIG="${REPO_ROOT}/home/config/fastfetch/config.jsonc"
BTOP_CONFIG="${REPO_ROOT}/home/config/btop/btop.conf"
BTOP_THEME="${REPO_ROOT}/home/config/btop/themes/noctalia.theme"
WAYBAR_UPDATES_SCRIPT="${REPO_ROOT}/home/scripts/waybar-flake-updates"
WAYBAR_PROXY_SCRIPT="${REPO_ROOT}/home/scripts/waybar-proxy-status"
WAYBAR_NET_SCRIPT="${REPO_ROOT}/home/scripts/waybar-net-speed"

NO_NETWORK=false
NO_PORTS=false
STRICT=false

failures=0
warnings=0

usage() {
  cat <<'EOF_USAGE'
Usage: preflight.sh [options]

Options:
  --no-network   Skip network reachability checks
  --no-ports     Skip local port listening checks
  --strict       Treat warnings as errors
  -h, --help     Show this help
EOF_USAGE
}

warn_msg() {
  warnings=$((warnings + 1))
  warn "$*"
}

fail_msg() {
  failures=$((failures + 1))
  msg FAIL "$*"
}

ok_msg() {
  ok "$*"
}

check_file() {
  local path="$1"
  local label="$2"
  if [[ -f "${path}" ]]; then
    ok_msg "${label} found: ${path}"
  else
    fail_msg "${label} missing: ${path}"
  fi
}

check_optional_file() {
  local path="$1"
  local label="$2"
  if [[ -f "${path}" ]]; then
    ok_msg "${label} found: ${path}"
  else
    warn_msg "${label} missing: ${path}"
  fi
}

check_executable() {
  local path="$1"
  local label="$2"
  if [[ -x "${path}" ]]; then
    ok_msg "${label} executable: ${path}"
  else
    warn_msg "${label} missing or not executable: ${path}"
  fi
}

check_cmd() {
  local cmd="$1"
  local label="$2"
  if command -v "${cmd}" >/dev/null 2>&1; then
    ok_msg "${label} available: ${cmd}"
  else
    warn_msg "${label} missing: ${cmd}"
  fi
}

check_package_reference() {
  local pkg="$1"
  if grep -q "\b${pkg}\b" "${PACKAGES_FILE}"; then
    ok_msg "Package referenced in packages.nix: ${pkg}"
  else
    warn_msg "Package not found in packages.nix: ${pkg}"
  fi
}

parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --no-network)
        NO_NETWORK=true
        ;;
      --no-ports)
        NO_PORTS=true
        ;;
      --strict)
        STRICT=true
        ;;
      -h|--help)
        usage
        exit 0
        ;;
      *)
        fail_msg "Unknown option: $1"
        usage
        exit 1
        ;;
    esac
    shift
  done
}

check_preflight() {
  msg INFO "==> Basic files"
  check_file "${FLAKE_FILE}" "flake.nix"
  check_file "${HOST_FILE}" "host.nix"
  check_file "${HOME_FILE}" "home/home.nix"

  msg INFO "==> Hardware configuration"
  if [[ -f "${HARDWARE_FILE}" ]]; then
    ok_msg "Repo hardware config present"
  elif [[ -f "${ETC_HARDWARE}" ]]; then
    warn_msg "Repo hardware config missing; will sync from ${ETC_HARDWARE} during install"
  else
    fail_msg "hardware-configuration.nix missing in repo and /etc/nixos"
  fi

  msg INFO "==> Flake target"
  if grep -q "nixosConfigurations\.nixos" "${FLAKE_FILE}"; then
    ok_msg "flake target nixos exists"
  else
    warn_msg "flake target nixos not found; check flake.nix"
  fi

  msg INFO "==> Host variables"
  if [[ -f "${HOST_FILE}" ]]; then
    local user proxy tun
    user="$(get_host_var "user" "${HOST_FILE}")"
    proxy="$(get_host_var "proxyUrl" "${HOST_FILE}")"
    tun="$(get_host_var "tunInterface" "${HOST_FILE}")"

    if [[ -n "${user}" ]]; then
      ok_msg "vars.user = ${user}"
      if [[ -n "${USER:-}" && "${USER}" != "${user}" ]]; then
        warn_msg "Current user (${USER}) differs from vars.user (${user})"
      fi
    else
      warn_msg "vars.user not detected"
    fi

    if [[ -n "${proxy}" ]]; then
      ok_msg "vars.proxyUrl = ${proxy}"
    else
      warn_msg "vars.proxyUrl is empty (proxy disabled)"
    fi

    if [[ -n "${tun}" ]]; then
      ok_msg "vars.tunInterface = ${tun}"
    else
      warn_msg "vars.tunInterface is empty"
    fi
  fi

  msg INFO "==> Required commands"
  check_cmd sudo "sudo"
  check_cmd nixos-rebuild "nixos-rebuild"
  check_cmd nix "nix"
  check_cmd git "git (install_from_github.sh)"
  check_cmd rsync "rsync (faster sync)"
  check_cmd curl "curl (network checks)"
  check_cmd ip "ip (TUN check)"
  check_cmd ss "ss (port check)"

  msg INFO "==> Desktop config references"
  if [[ -f "${DESKTOP_FILE}" ]]; then
    grep -q "programs.waybar.enable = true;" "${DESKTOP_FILE}" && ok_msg "waybar enabled" || warn_msg "waybar not enabled"
    grep -q "programs.fuzzel.enable = true;" "${DESKTOP_FILE}" && ok_msg "fuzzel enabled" || warn_msg "fuzzel not enabled"
    grep -q "programs.swaylock.enable = true;" "${DESKTOP_FILE}" && ok_msg "swaylock enabled" || warn_msg "swaylock not enabled"
  fi

  if [[ -f "${NIRI_CONFIG}" ]]; then
    if grep -q '^[[:space:]]*output "' "${NIRI_CONFIG}"; then
      warn_msg "Niri output is hardcoded; ensure output name matches target hardware"
    else
      ok_msg "Niri output is not hardcoded"
    fi
  fi

  msg INFO "==> Desktop helpers"
  check_executable "${WALLPAPER_SCRIPT}" "wallpaper-random"
  check_executable "${WAYBAR_UPDATES_SCRIPT}" "waybar-flake-updates"
  check_executable "${WAYBAR_PROXY_SCRIPT}" "waybar-proxy-status"
  check_executable "${WAYBAR_NET_SCRIPT}" "waybar-net-speed"

  msg INFO "==> CLI theming files"
  check_optional_file "${FASTFETCH_CONFIG}" "fastfetch config"
  check_optional_file "${BTOP_CONFIG}" "btop config"
  check_optional_file "${BTOP_THEME}" "btop theme"

  if [[ -d "${WALLPAPER_DIR}" ]]; then
    if find "${WALLPAPER_DIR}" -maxdepth 1 -type f | grep -q .; then
      ok_msg "wallpapers present"
    else
      warn_msg "wallpaper directory empty"
    fi
  else
    warn_msg "wallpaper directory missing"
  fi

  msg INFO "==> Package coverage for autostart"
  if [[ -f "${PACKAGES_FILE}" ]]; then
    check_package_reference "mako"
    check_package_reference "swaybg"
    check_package_reference "swayidle"
    check_package_reference "brightnessctl"
    check_package_reference "wl-clipboard"
    check_package_reference "grim"
    check_package_reference "slurp"
    check_package_reference "swappy"
    check_package_reference "networkmanagerapplet"
    check_package_reference "pavucontrol"
  fi

  msg INFO "==> CLI theming packages"
  if [[ -f "${PACKAGES_FILE}" ]]; then
    check_package_reference "fastfetch"
    check_package_reference "btop"
  fi

  msg INFO "==> Input method"
  if [[ -f "${I18N_FILE}" ]] && grep -q "fcitx5" "${I18N_FILE}"; then
    ok_msg "fcitx5 configured in modules/i18n.nix"
  else
    warn_msg "fcitx5 not configured in modules/i18n.nix"
  fi

  if [[ "${NO_PORTS}" == false ]]; then
    msg INFO "==> Local port checks"
    if [[ -f "${HOST_FILE}" ]]; then
      local proxy url_hostport url_port
      proxy="$(get_host_var "proxyUrl" "${HOST_FILE}")"
      if [[ -n "${proxy}" ]]; then
        url_hostport="${proxy#*://}"
        url_hostport="${url_hostport%%/*}"
        if [[ "${url_hostport}" == *:* ]]; then
          url_port="${url_hostport##*:}"
          if [[ "${url_port}" =~ ^[0-9]+$ ]]; then
            if command -v ss >/dev/null 2>&1; then
              if ss -lnt | awk '{print $4}' | grep -Eq "[:.]${url_port}$"; then
                ok_msg "proxy port is listening: ${url_port}"
              else
                warn_msg "proxy port not listening: ${url_port}"
              fi
            else
              warn_msg "ss not available; cannot verify proxy port"
            fi
          else
            warn_msg "proxy URL does not include a valid port"
          fi
        else
          warn_msg "proxy URL does not include a port"
        fi
      fi
    fi
  fi

  if [[ "${NO_NETWORK}" == false ]]; then
    msg INFO "==> Network reachability"
    if command -v curl >/dev/null 2>&1; then
      if curl -fsSL --max-time 5 https://cache.nixos.org/nix-cache-info >/dev/null 2>&1; then
        ok_msg "cache.nixos.org reachable"
      else
        warn_msg "cache.nixos.org unreachable"
      fi

      if curl -fsSL --max-time 5 https://mirrors.ustc.edu.cn/nix-channels/store/nix-cache-info >/dev/null 2>&1; then
        ok_msg "USTC mirror reachable"
      else
        warn_msg "USTC mirror unreachable"
      fi
    else
      warn_msg "curl missing; skip network reachability checks"
    fi
  fi

  if [[ "${STRICT}" == true && "${warnings}" -gt 0 ]]; then
    failures=$((failures + warnings))
    warnings=0
  fi

  msg INFO "==> Summary"
  if [[ "${failures}" -eq 0 ]]; then
    ok_msg "Preflight completed with ${warnings} warning(s)"
  else
    fail_msg "Preflight failed with ${failures} error(s) and ${warnings} warning(s)"
  fi
}

parse_args "$@"
check_preflight

if [[ "${failures}" -ne 0 ]]; then
  exit 1
fi
