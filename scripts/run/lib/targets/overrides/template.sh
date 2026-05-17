# run.sh 用户模板与 local 覆盖写入

# 为缺失用户自动生成 Home Manager 入口模板，并补齐默认用户配置。
ensure_user_home_entries() {
  local repo_dir="$1"
  # 从 admin 复制的共享模块（不含 git.nix，每个用户有自包含的 git.nix）
  local shared_module_files=(
    "./base.nix"
    "./programs.nix"
    "./desktop.nix"
    "./shell.nix"

  )
  local extra_imports=(
    "./git.nix"
    "./packages.nix"
  )
  local include_user_files=true
  if [[ "${HOST_PROFILE_KIND}" == "server" ]]; then
    shared_module_files=(
      "./base.nix"
      "./shell.nix"

    )
    include_user_files=false
  fi
  local default_user=""
  local template_user=""
  local template_dir=""
  default_user="$(resolve_default_user)"
  if [[ -n "${default_user}" && -d "${repo_dir}/users/${default_user}" ]]; then
    template_user="${default_user}"
    template_dir="${repo_dir}/users/${default_user}"
  elif [[ -d "${repo_dir}/users/admin" ]]; then
    template_user="admin"
    template_dir="${repo_dir}/users/admin"
  fi
  if [[ -n "${template_dir}" ]]; then
    note "新用户模板来源：users/${template_user}"
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
    local user_dir="${repo_dir}/users/${user}"
    local user_file="${user_dir}/default.nix"
    local create_default=false
    if [[ ! -f "${user_file}" ]]; then
      create_default=true
    fi

    mkdir -p "${user_dir}"
    if [[ "${create_default}" == "true" && -n "${template_dir}" && "${user_dir}" != "${template_dir}" ]]; then
      # 始终复制共享模块文件（base.nix 等）从模板用户到新用户
      local _smf=""
      for _smf in "${shared_module_files[@]}"; do
        if [[ -f "${template_dir}/${_smf}" && ! -f "${user_dir}/${_smf}" ]]; then
          cp -a "${template_dir}/${_smf}" "${user_dir}/${_smf}"
        fi
      done

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
# 当前用户的 Git 配置（自包含：选项定义 + 程序配置 + 身份覆盖）。
# 默认值在 home/git.nix 中定义；此处按需覆盖。

{ config, lib, ... }:

{
  options.mcb.git = {
    userName = lib.mkOption {
      type = lib.types.str;
      default = "your-name";
      description = "Git user.name for commits.";
    };
    userEmail = lib.mkOption {
      type = lib.types.str;
      default = "you@example.com";
      description = "Git user.email for commits.";
    };
  };

  config = {
    mcb.git.userName = lib.mkDefault "your-name";
    mcb.git.userEmail = lib.mkDefault "you@example.com";

    programs.git = {
      enable = true;
      lfs.enable = true;
      settings = {
        user = {
          name = cfg.userName;
          email = cfg.userEmail;
        };
        core = {
          editor = "hx";
          pager = "delta";
        };
        interactive.diffFilter = "delta --color-only";
        delta = {
          navigate = true;
          "side-by-side" = true;
        };
      };
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

    local import_lines=""
    local smf
    for smf in "${shared_module_files[@]}"; do
      if [[ -z "${import_lines}" ]]; then
        import_lines="    ${smf}"
      else
        import_lines+=$'\n'"    ${smf}"
      fi
    done
    local extra_import=""
    for extra_import in "${extra_imports[@]}"; do
      if [[ -f "${user_dir}/${extra_import}" ]]; then
        import_lines+=$'\n'"    ${extra_import}"
      fi
    done
      # 始终复制共享模块文件（base.nix 等）从模板用户到新用户
      local _smf=""
      for _smf in "${shared_module_files[@]}"; do
        if [[ -f "${template_dir}/${_smf}" && ! -f "${user_dir}/${_smf}" ]]; then
          cp -a "${template_dir}/${_smf}" "${user_dir}/${_smf}"
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
    warn "已为新用户自动生成 Home Manager 入口：users/${user}/default.nix"
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
  local src="${ETC_DIR}/local.nix"
  local dst="${repo_dir}/local.nix"
  if [[ -f "${src}" ]]; then
    mkdir -p "$(dirname "${dst}")"
    if cp -a "${src}" "${dst}"; then
      note "仅更新模式：已保留现有 host/local.nix"
    else
      warn "仅更新模式：复制现有 local.nix 失败，将继续使用仓库版本。"
    fi
  else
    note "仅更新模式：未发现现有 host/local.nix，将按仓库默认配置更新。"
  fi
}

# 增量追加用户到 local.nix（用于 add-user 命令，不动已有配置）。
# 参数: repo_dir host_name username is_admin
append_user_to_local_override() {
  local repo_dir="$1"
  local host_name="$2"
  local username="$3"
  local is_admin="${4:-false}"
  local repo_root="${repo_dir}"
  local file="${repo_root}/local.nix"

  if [[ ! -d "${repo_root}" ]]; then
    error "主机目录不存在：${repo_root}"
  fi

  # 如果 local.nix 不存在，生成全新文件
  if [[ ! -f "${file}" ]]; then
    log "local.nix 不存在，生成新文件"
    TARGET_NAME="${host_name}"
    TARGET_USERS=("${username}")
    if [[ "${is_admin}" == "true" ]]; then
      TARGET_ADMIN_USERS=("${username}")
    fi
    write_local_override "${repo_dir}"
    return 0
  fi

  # local.nix 存在：读取现有用户列表，追加新用户
  log "在现有 local.nix 中追加用户：${username}"

  local tmp_file
  tmp_file="$(mktemp)"

  local in_users=false
  local in_admin=false
  local user_found=false
  local admin_found=false

  while IFS= read -r line; do
    # 检测 mcb.users 行
    if [[ "${line}" =~ mcb\.users.*=.*\[ ]]; then
      in_users=true
      # 检查是否已包含该用户
      if [[ "${line}" == *"\"${username}\""* ]]; then
        user_found=true
      fi
    fi

    # 在 mcb.users 列表闭括号前追加
    if [[ "${in_users}" == "true" && "${line}" == *"];"* ]]; then
      if [[ "${user_found}" != "true" ]]; then
        # 在 ] 前插入新用户名
        printf '%s\n' "${line/];/  \"${username}\"\n  ];}" >> "${tmp_file}"
      else
        printf '%s\n' "${line}" >> "${tmp_file}"
        note "用户 ${username} 已在 mcb.users 中，跳过追加。"
      fi
      in_users=false
      continue
    fi

    # 检测 mcb.adminUsers 行
    if [[ "${line}" =~ mcb\.adminUsers.*=.*\[ ]]; then
      in_admin=true
      if [[ "${line}" == *"\"${username}\""* ]]; then
        admin_found=true
      fi
    fi

    # 在 mcb.adminUsers 闭括号前追加（仅当 --admin）
    if [[ "${in_admin}" == "true" && "${line}" == *"];"* ]]; then
      if [[ "${is_admin}" == "true" && "${admin_found}" != "true" ]]; then
        printf '%s\n' "${line/];/  \"${username}\"\n  ];}" >> "${tmp_file}"
      else
        printf '%s\n' "${line}" >> "${tmp_file}"
      fi
      in_admin=false
      continue
    fi

    printf '%s\n' "${line}" >> "${tmp_file}"
  done < "${file}"

  mv "${tmp_file}" "${file}"
  success "已将 ${username} 追加到 host/local.nix"
}

# 写入 host/local.nix 覆盖项（全文重写模式，用于初次部署）。
write_local_override() {
  local repo_dir="$1"
  local repo_root="${repo_dir}"
  local file="${repo_root}/local.nix"

  if [[ ${#TARGET_USERS[@]} -eq 0 ]]; then
    return 0
  fi

  # 只在需要时生成 local.nix（不会覆盖已有文件）
  if [[ ! -d "${repo_root}" ]]; then
    error "主机目录不存在：${repo_root}"
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
