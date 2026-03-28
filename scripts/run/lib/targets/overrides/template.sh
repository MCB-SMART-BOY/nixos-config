# run.sh 用户模板与 local 覆盖写入

# 为缺失用户自动生成 Home Manager 入口模板，并补齐默认用户配置。
ensure_user_home_entries() {
  local repo_dir="$1"
  local profile_import="../../profiles/full.nix"
  local extra_imports=(
    "./git.nix"
    "./packages.nix"
  )
  local include_user_files=true
  if [[ "${HOST_PROFILE_KIND}" == "server" ]]; then
    profile_import="../../profiles/minimal.nix"
    include_user_files=false
  fi
  local default_user=""
  local template_user=""
  local template_dir=""
  default_user="$(resolve_default_user)"
  if [[ -n "${default_user}" && -d "${repo_dir}/home/users/${default_user}" ]]; then
    template_user="${default_user}"
    template_dir="${repo_dir}/home/users/${default_user}"
  elif [[ -d "${repo_dir}/home/users/mcbnixos" ]]; then
    template_user="mcbnixos"
    template_dir="${repo_dir}/home/users/mcbnixos"
  fi
  if [[ -n "${template_dir}" ]]; then
    note "新用户模板来源：home/users/${template_user}"
  fi
  local copy_template_content="false"
  if [[ "${RUN_SH_COPY_USER_TEMPLATE:-false}" == "true" ]]; then
    copy_template_content="true"
    note "将复制模板用户目录内容（RUN_SH_COPY_USER_TEMPLATE=true）"
  else
    note "默认仅生成最小用户模板（不复制 config/assets/scripts）；如需复制可设置 RUN_SH_COPY_USER_TEMPLATE=true"
  fi

  local user=""
  for user in "${TARGET_USERS[@]}"; do
    local user_dir="${repo_dir}/home/users/${user}"
    local user_file="${user_dir}/default.nix"
    local create_default=false
    if [[ ! -f "${user_file}" ]]; then
      create_default=true
    fi

    mkdir -p "${user_dir}"
    if [[ "${create_default}" == "true" && -n "${template_dir}" && "${user_dir}" != "${template_dir}" ]]; then
      if [[ "${include_user_files}" == "true" && "${copy_template_content}" == "true" ]]; then
        local item=""
        for item in config assets scripts; do
          if [[ -e "${template_dir}/${item}" && ! -e "${user_dir}/${item}" ]]; then
            cp -a "${template_dir}/${item}" "${user_dir}/${item}"
          fi
        done
        local template_file=""
        for template_file in files.nix scripts.nix; do
          if [[ -f "${template_dir}/${template_file}" && ! -f "${user_dir}/${template_file}" ]]; then
            cp -a "${template_dir}/${template_file}" "${user_dir}/${template_file}"
          fi
        done
      fi
    fi
    if [[ ! -f "${user_dir}/git.nix" ]]; then
      cat > "${user_dir}/git.nix" <<'EOF_GIT'
# 默认 Git 身份（请按需修改）
{ config, ... }:

{
  programs.git.settings.user = {
    name = config.home.username;
    # email = "you@example.com";
  };
}
EOF_GIT
    fi
    if [[ ! -f "${user_dir}/packages.nix" ]]; then
      if [[ -n "${template_dir}" && -f "${template_dir}/packages.nix" && "${user_dir}" != "${template_dir}" ]]; then
        cp -a "${template_dir}/packages.nix" "${user_dir}/packages.nix"
      elif [[ "${HOST_PROFILE_KIND}" == "server" ]]; then
        cat > "${user_dir}/packages.nix" <<'EOF_PKGS_SERVER'
# 用户个人软件入口（服务器最小模板）
{ pkgs, ... }:

{
  home.packages = with pkgs; [
    # tmux
    # htop
    # rsync
  ];
}
EOF_PKGS_SERVER
      else
        cat > "${user_dir}/packages.nix" <<'EOF_PKGS'
# 用户个人软件入口（按需启用，不影响其他用户可见性）
{ pkgs, ... }:

{
  mcb.desktopEntries = {
    enableZed = false;
    enableYesPlayMusic = false;
  };

  # 逐个声明该用户的软件（仅此用户可见）
  home.packages = with pkgs; [
    # firefox
    # helix
    # (callPackage ../../../pkgs/zed { })            # 同时把 enableZed 改为 true
    # (callPackage ../../../pkgs/yesplaymusic { })   # 同时把 enableYesPlayMusic 改为 true
  ];
}
EOF_PKGS
      fi
    fi
    if [[ ! -f "${user_dir}/local.nix.example" ]]; then
      cat > "${user_dir}/local.nix.example" <<'EOF_LOCAL'
# 用户私有覆盖示例（按需复制为 local.nix）
{ ... }:

{
  # 仅当前用户生效的个性化开关示例：
  # home.packages = with pkgs; [ localsend ];
}
EOF_LOCAL
    fi

    if [[ "${create_default}" != "true" ]]; then
      continue
    fi

    local import_lines="    ${profile_import}"
    local extra_import=""
    for extra_import in "${extra_imports[@]}"; do
      if [[ -f "${user_dir}/${extra_import}" ]]; then
        import_lines+=$'\n'"    ${extra_import}"
      fi
    done
    if [[ "${include_user_files}" == "true" ]]; then
      if [[ -f "${user_dir}/files.nix" ]]; then
        import_lines+=$'\n'"    ./files.nix"
      fi
      if [[ -f "${user_dir}/scripts.nix" ]]; then
        import_lines+=$'\n'"    ./scripts.nix"
      fi
    fi
    cat > "${user_file}" <<EOF_USER
{ lib, ... }:

let
  user = "${user}";
in
{
  imports = [
${import_lines}
  ] ++ lib.optional (builtins.pathExists ./local.nix) ./local.nix;

  home.username = user;
  home.homeDirectory = "/home/\${user}";
  home.stateVersion = "25.11";

  programs.home-manager.enable = true;
  xdg.enable = true;
}
EOF_USER
    CREATED_HOME_USERS+=("${user}")
    warn "已为新用户自动生成 Home Manager 入口：home/users/${user}/default.nix"
  done
}

# 仅更新模式下保留当前主机 local.nix，避免覆盖现有用户/权限。
preserve_existing_local_override() {
  local repo_dir="$1"
  if [[ "${DEPLOY_MODE}" != "update-existing" ]]; then
    return 0
  fi
  if [[ -z "${TARGET_NAME}" ]]; then
    return 0
  fi
  local src="${ETC_DIR}/hosts/${TARGET_NAME}/local.nix"
  local dst="${repo_dir}/hosts/${TARGET_NAME}/local.nix"
  if [[ -f "${src}" ]]; then
    mkdir -p "$(dirname "${dst}")"
    if cp -a "${src}" "${dst}"; then
      note "仅更新模式：已保留现有 hosts/${TARGET_NAME}/local.nix"
    else
      warn "仅更新模式：复制现有 local.nix 失败，将继续使用仓库版本。"
    fi
  else
    note "仅更新模式：未发现现有 hosts/${TARGET_NAME}/local.nix，将按仓库默认配置更新。"
  fi
}

# 写入 hosts/<host>/local.nix 覆盖项。
write_local_override() {
  local repo_dir="$1"
  local host_dir="${repo_dir}/hosts/${TARGET_NAME}"
  local file="${host_dir}/local.nix"

  if [[ ${#TARGET_USERS[@]} -eq 0 ]]; then
    return 0
  fi

  # 只在需要时生成 local.nix（不会覆盖已有文件）
  if [[ ! -d "${host_dir}" ]]; then
    error "主机目录不存在：${host_dir}"
  fi

  local primary="${TARGET_USERS[0]}"
  local list=""
  local admin_list=""
  local user=""
  if [[ ${#TARGET_ADMIN_USERS[@]} -eq 0 ]]; then
    TARGET_ADMIN_USERS=("${primary}")
  fi

  # 生成用户列表字符串
  for user in "${TARGET_USERS[@]}"; do
    list="${list} \"${user}\""
  done
  for user in "${TARGET_ADMIN_USERS[@]}"; do
    admin_list="${admin_list} \"${user}\""
  done

  {
    # local.nix 会覆盖 mcb.user/mcb.users 等配置
    echo "{ lib, ... }:"
    echo ""
    echo "{"
    echo "  mcb.user = lib.mkForce \"${primary}\";"
    echo "  mcb.users = lib.mkForce [${list} ];"
    echo "  mcb.adminUsers = lib.mkForce [${admin_list} ];"

    if [[ "${PER_USER_TUN_ENABLED}" == "true" && ${#USER_TUN[@]} -gt 0 ]]; then
      echo "  mcb.perUserTun.interfaces = lib.mkForce {"
      for user in "${TARGET_USERS[@]}"; do
        echo "    ${user} = \"${USER_TUN[${user}]}\";"
      done
      echo "  };"
      echo "  mcb.perUserTun.dnsPorts = lib.mkForce {"
      for user in "${TARGET_USERS[@]}"; do
        echo "    ${user} = ${USER_DNS[${user}]};"
      done
      echo "  };"
    fi

    if [[ "${GPU_OVERRIDE}" == "true" ]]; then
      echo "  mcb.hardware.gpu.mode = lib.mkForce \"${GPU_MODE}\";"
      if [[ -n "${GPU_IGPU_VENDOR}" ]]; then
        echo "  mcb.hardware.gpu.igpuVendor = lib.mkForce \"${GPU_IGPU_VENDOR}\";"
      fi
      if [[ -n "${GPU_NVIDIA_OPEN}" ]]; then
        echo "  mcb.hardware.gpu.nvidia.open = lib.mkForce ${GPU_NVIDIA_OPEN};"
      fi
      if [[ -n "${GPU_PRIME_MODE}" || -n "${GPU_INTEL_BUS}" || -n "${GPU_AMD_BUS}" || -n "${GPU_NVIDIA_BUS}" ]]; then
        echo "  mcb.hardware.gpu.prime = lib.mkForce {"
        if [[ -n "${GPU_PRIME_MODE}" ]]; then
          echo "    mode = \"${GPU_PRIME_MODE}\";"
        fi
        if [[ -n "${GPU_INTEL_BUS}" ]]; then
          echo "    intelBusId = \"${GPU_INTEL_BUS}\";"
        fi
        if [[ -n "${GPU_AMD_BUS}" ]]; then
          echo "    amdgpuBusId = \"${GPU_AMD_BUS}\";"
        fi
        if [[ -n "${GPU_NVIDIA_BUS}" ]]; then
          echo "    nvidiaBusId = \"${GPU_NVIDIA_BUS}\";"
        fi
        echo "  };"
      fi
      if [[ "${GPU_SPECIALISATIONS_SET}" == "true" ]]; then
        echo "  mcb.hardware.gpu.specialisations.enable = lib.mkForce ${GPU_SPECIALISATIONS_ENABLED};"
        if [[ "${GPU_SPECIALISATIONS_ENABLED}" == "true" && ${#GPU_SPECIALISATION_MODES[@]} -gt 0 ]]; then
          local mode_list=""
          local mode
          for mode in "${GPU_SPECIALISATION_MODES[@]}"; do
            mode_list+=" \"${mode}\""
          done
          echo "  mcb.hardware.gpu.specialisations.modes = lib.mkForce [${mode_list} ];"
        fi
      fi
    fi

    if [[ "${SERVER_OVERRIDES_ENABLED}" == "true" ]]; then
      echo "  mcb.packages.enableNetworkCli = lib.mkForce ${SERVER_ENABLE_NETWORK_CLI};"
      echo "  mcb.packages.enableNetworkGui = lib.mkForce ${SERVER_ENABLE_NETWORK_GUI};"
      echo "  mcb.packages.enableShellTools = lib.mkForce ${SERVER_ENABLE_SHELL_TOOLS};"
      echo "  mcb.packages.enableWaylandTools = lib.mkForce ${SERVER_ENABLE_WAYLAND_TOOLS};"
      echo "  mcb.packages.enableSystemTools = lib.mkForce ${SERVER_ENABLE_SYSTEM_TOOLS};"
      echo "  mcb.packages.enableGeekTools = lib.mkForce ${SERVER_ENABLE_GEEK_TOOLS};"
      echo "  mcb.packages.enableGaming = lib.mkForce ${SERVER_ENABLE_GAMING};"
      echo "  mcb.packages.enableInsecureTools = lib.mkForce ${SERVER_ENABLE_INSECURE_TOOLS};"
      echo "  mcb.virtualisation.docker.enable = lib.mkForce ${SERVER_ENABLE_DOCKER};"
      echo "  mcb.virtualisation.libvirtd.enable = lib.mkForce ${SERVER_ENABLE_LIBVIRTD};"
    fi

    echo "}"
  } > "${file}"
}
